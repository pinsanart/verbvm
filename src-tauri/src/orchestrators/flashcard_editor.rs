use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::error::{OrchestratorError, OrchestratorResult};
use super::pronunciation_text;
use crate::dependencies::media_store::{MediaStorage, StoredMedia};
use crate::dependencies::pronunciation::PronunciationSource;
use crate::dependencies::sanitizer::HtmlSanitizer;
use crate::dependencies::speech::SpeechSynthesizer;
use crate::modules::card_types::{CardTypeApi, FieldDefinition, FlashcardType};
use crate::modules::decks::{Deck, DeckApi};
use crate::modules::flashcards::{
    Direction, FieldMedia, FieldValue, Flashcard, FlashcardApi, FlashcardDraft, MediaAttachment, PublishedDraft,
};
use crate::modules::languages::{Language, LanguageApi, LanguageVariant};

/// A flashcard type with the fields it has in a deck's language.
#[derive(Debug, Clone)]
pub struct TypeForm {
    pub flashcard_type: FlashcardType,
    /// The type's fields for the deck's language, front before back.
    pub fields: Vec<FieldDefinition>,
}

/// What the editor needs to show a deck: its language and accent, and the form of every
/// flashcard type, since the deck has one list of drafts that mixes types.
#[derive(Debug, Clone)]
pub struct EditorForm {
    pub deck: Deck,
    pub language: Language,
    pub variant: LanguageVariant,
    pub types: Vec<TypeForm>,
}

/// What a generation produced, with the problems that did not stop it (for example the words that
/// had no pronunciation, or one of the two things a stack generates failing).
#[derive(Debug, Clone)]
pub struct Generated<T> {
    pub value: T,
    pub warnings: Vec<String>,
}

mod stacks;

/// A saved flashcard in the shape the editor works with: its values and media by field key, the
/// same as a draft, so one form edits both.
#[derive(Debug, Clone)]
pub struct SavedCard {
    pub uuid: String,
    pub type_slug: String,
    pub values: BTreeMap<String, String>,
    pub media: BTreeMap<String, Vec<MediaAttachment>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// The stack that links this card to the others made with it, if any.
    pub stack_uuid: Option<String>,
}

/// A problem that stopped a draft from being published.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftIssue {
    pub draft_uuid: String,
    /// The field the problem is about, if it is about one.
    pub field_key: Option<String>,
    pub message: String,
}

/// Result of publishing the drafts of a deck. When `issues` is not empty nothing was saved and
/// `published` is 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishOutcome {
    pub published: usize,
    pub issues: Vec<DraftIssue>,
}

/// Edits the drafts of a deck and publishes them as flashcards.
///
/// A deck has one list of drafts, in the order they were created, and each draft has a type.
/// Rules enforced here, because they need the card type, the language and the deck together:
/// only fields of the draft's type (for the deck's language) can be filled, rich fields are
/// sanitized before they are stored, media can only be attached to fields that allow it, and
/// required fields must have text or media for the drafts to be published.
pub struct FlashcardEditorOrchestrator {
    decks: Arc<dyn DeckApi>,
    languages: Arc<dyn LanguageApi>,
    card_types: Arc<dyn CardTypeApi>,
    flashcards: Arc<dyn FlashcardApi>,
    sanitizer: Arc<dyn HtmlSanitizer>,
    media: Arc<dyn MediaStorage>,
    speech: Arc<dyn SpeechSynthesizer>,
    pronunciation: Arc<dyn PronunciationSource>,
}

/// The most media files one field can hold.
const MAX_MEDIA_PER_FIELD: usize = 10;

struct TypeContext {
    flashcard_type: FlashcardType,
    type_id: i64,
    fields: Vec<FieldDefinition>,
}

struct Context {
    deck: Deck,
    deck_id: i64,
    language: Language,
    variant: LanguageVariant,
    types: Vec<TypeContext>,
}

impl Context {
    fn type_by_slug(&self, slug: &str) -> OrchestratorResult<&TypeContext> {
        self.types
            .iter()
            .find(|t| t.flashcard_type.slug() == slug)
            .ok_or_else(|| OrchestratorError::NotFound("flashcard type not found".into()))
    }

    fn type_by_id(&self, id: i64) -> OrchestratorResult<&TypeContext> {
        self.types
            .iter()
            .find(|t| t.type_id == id)
            .ok_or_else(|| OrchestratorError::NotFound("flashcard type not found".into()))
    }
}

fn saved_id(id: Option<i64>, what: &str) -> OrchestratorResult<i64> {
    id.ok_or_else(|| OrchestratorError::Internal(format!("{what} was not saved")))
}

impl FlashcardEditorOrchestrator {
    pub fn new(
        decks: Arc<dyn DeckApi>,
        languages: Arc<dyn LanguageApi>,
        card_types: Arc<dyn CardTypeApi>,
        flashcards: Arc<dyn FlashcardApi>,
        sanitizer: Arc<dyn HtmlSanitizer>,
        media: Arc<dyn MediaStorage>,
        speech: Arc<dyn SpeechSynthesizer>,
        pronunciation: Arc<dyn PronunciationSource>,
    ) -> Self {
        Self { decks, languages, card_types, flashcards, sanitizer, media, speech, pronunciation }
    }

    pub fn form(&self, deck_name: &str) -> OrchestratorResult<EditorForm> {
        let ctx = self.context(deck_name)?;
        Ok(EditorForm {
            deck: ctx.deck,
            language: ctx.language,
            variant: ctx.variant,
            types: ctx
                .types
                .into_iter()
                .map(|t| TypeForm { flashcard_type: t.flashcard_type, fields: t.fields })
                .collect(),
        })
    }

    /// The deck's drafts, of every type, in order.
    pub fn drafts(&self, deck_name: &str) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let deck = self.decks.get_deck_by_name(deck_name)?;
        Ok(self.flashcards.list_drafts(saved_id(deck.id(), "deck")?)?)
    }

    /// Adds an empty draft of a type at the end of the deck's list and returns just that draft, for
    /// callers that make many (the list of a big deck is not cheap to build each time).
    pub fn new_draft(&self, deck_name: &str, type_slug: &str) -> OrchestratorResult<FlashcardDraft> {
        let ctx = self.context(deck_name)?;
        let flashcard_type = ctx.type_by_slug(type_slug)?;
        Ok(self.flashcards.create_draft(ctx.deck_id, flashcard_type.type_id)?)
    }

    /// Adds an empty draft of a type at the end of the deck's list and returns the updated list.
    pub fn add_draft(&self, deck_name: &str, type_slug: &str) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let ctx = self.context(deck_name)?;
        let flashcard_type = ctx.type_by_slug(type_slug)?;
        self.flashcards.create_draft(ctx.deck_id, flashcard_type.type_id)?;
        Ok(self.flashcards.list_drafts(ctx.deck_id)?)
    }

    /// Replaces the values of a draft. Keys must be fields of the draft's type for its deck's
    /// language, and rich values are sanitized.
    pub fn update_draft(
        &self,
        draft_uuid: &str,
        values: BTreeMap<String, String>,
    ) -> OrchestratorResult<FlashcardDraft> {
        let draft = self.flashcards.get_draft(draft_uuid)?;
        let ctx = self.context_of(&draft)?;
        let fields = &ctx.type_by_id(draft.flashcard_type_id())?.fields;
        let mut cleaned = BTreeMap::new();
        for (key, content) in values {
            let definition = fields.iter().find(|f| f.key() == key).ok_or_else(|| {
                OrchestratorError::Invalid(format!("'{key}' is not a field of this flashcard type"))
            })?;
            let content = if definition.is_rich() { self.sanitizer.sanitize_html(&content) } else { content };
            cleaned.insert(key, content);
        }
        Ok(self.flashcards.update_draft(draft_uuid, cleaned)?)
    }

    /// Copies a file into the media store and attaches it to a field of a draft. The field must
    /// allow media and hold at most 10 files. Only images and audio are accepted, judged by the
    /// content of the file.
    pub fn attach_media(
        &self,
        draft_uuid: &str,
        field_key: &str,
        source: &Path,
    ) -> OrchestratorResult<FlashcardDraft> {
        let draft = self.flashcards.get_draft(draft_uuid)?;
        let ctx = self.context_of(&draft)?;
        let fields = &ctx.type_by_id(draft.flashcard_type_id())?.fields;
        let definition = fields.iter().find(|f| f.key() == field_key).ok_or_else(|| {
            OrchestratorError::Invalid(format!("'{field_key}' is not a field of this flashcard type"))
        })?;
        if !definition.media_allowed() {
            return Err(OrchestratorError::Invalid(format!("{} does not accept media", definition.label())));
        }
        let attached = draft.media().get(field_key).map_or(0, Vec::len);
        if attached >= MAX_MEDIA_PER_FIELD {
            return Err(OrchestratorError::Invalid(format!(
                "{} already has {MAX_MEDIA_PER_FIELD} files",
                definition.label()
            )));
        }

        let attachment = Self::attachment_of(self.media.import(source)?)?;
        Ok(self.flashcards.attach_draft_media(draft_uuid, field_key, attachment)?)
    }

    /// Speaks the text of the field an audio field is generated from, stores the audio and
    /// attaches it to the audio field of a draft. The same limits as for an uploaded file apply.
    /// The text is the one stored with the draft, so the caller saves pending edits first.
    pub async fn generate_media(
        &self,
        draft_uuid: &str,
        field_key: &str,
    ) -> OrchestratorResult<FlashcardDraft> {
        // Everything that reads the database happens before the request, so nothing is held
        // while waiting for the network.
        let (text, language_tag) = {
            let draft = self.flashcards.get_draft(draft_uuid)?;
            let ctx = self.context_of(&draft)?;
            let fields = &ctx.type_by_id(draft.flashcard_type_id())?.fields;
            let definition = Self::field_by_key(fields, field_key)?;
            let source = Self::speech_source(fields, definition)?;
            let attached = draft.media().get(field_key).map_or(0, Vec::len);
            Self::check_media_room(definition, attached)?;
            let raw = draft.values().get(source.key()).map(String::as_str).unwrap_or_default();
            (self.spoken_text(source, raw)?, ctx.variant.bcp47_code().to_owned())
        };

        let audio = self.speech.synthesize(&text, &language_tag).await?;
        let attachment = Self::attachment_of(self.media.import_bytes(&audio.bytes)?)?;
        let keep = attachment.sha256().to_owned();
        let draft = self.flashcards.attach_draft_media(draft_uuid, field_key, attachment)?;
        // Generating makes the audio of the phrase as it is now, so the audio of an older phrase
        // is replaced.
        let mut draft = draft;
        let stale: Vec<String> = draft
            .media()
            .get(field_key)
            .into_iter()
            .flatten()
            .filter(|m| m.sha256() != keep)
            .map(|m| m.sha256().to_owned())
            .collect();
        for sha256 in stale {
            draft = self.flashcards.remove_draft_media(draft_uuid, field_key, &sha256)?;
        }
        Ok(draft)
    }

    /// Detaches a file (by its hash) from a field of a draft. The file stays in the media store.
    pub fn remove_media(
        &self,
        draft_uuid: &str,
        field_key: &str,
        sha256: &str,
    ) -> OrchestratorResult<FlashcardDraft> {
        Ok(self.flashcards.remove_draft_media(draft_uuid, field_key, sha256)?)
    }

    /// The slug of each flashcard type, by the id drafts and flashcards hold. For building the
    /// view's types, which are identified by slug.
    pub fn type_slugs(&self) -> OrchestratorResult<std::collections::HashMap<i64, String>> {
        Ok(self
            .card_types
            .list_types()?
            .into_iter()
            .filter_map(|t| t.id().map(|id| (id, t.slug().to_owned())))
            .collect())
    }

    /// The saved flashcards of a deck, newest first.
    pub fn saved_cards(&self, deck_name: &str) -> OrchestratorResult<Vec<SavedCard>> {
        let ctx = self.context(deck_name)?;
        let mut cards = self.flashcards.list_flashcards(ctx.deck_id)?;
        cards.sort_by(|a, b| b.created_at().cmp(&a.created_at()).then(b.id().cmp(&a.id())));
        cards.iter().map(|card| Self::saved_card(&ctx, card)).collect()
    }

    /// Replaces the text of a saved flashcard. The same rules as a draft apply: keys must be
    /// fields of its type for the deck's language, rich values are sanitized, and required fields
    /// need text or media. Its review history is kept.
    pub fn update_saved(
        &self,
        card_uuid: &str,
        values: BTreeMap<String, String>,
    ) -> OrchestratorResult<SavedCard> {
        let (ctx, _, fields) = self.prepare_saved_update(card_uuid, &values)?;
        let updated = self.flashcards.update_flashcard_fields(card_uuid, fields)?;
        Self::saved_card(&ctx, &updated)
    }

    /// Checks the new text of a saved flashcard against the rules of its type and returns what
    /// would be stored, without storing it.
    fn prepare_saved_update(
        &self,
        card_uuid: &str,
        values: &BTreeMap<String, String>,
    ) -> OrchestratorResult<(Context, Flashcard, Vec<FieldValue>)> {
        let card = self.flashcards.get_flashcard(card_uuid)?;
        let ctx = self.context_of_deck(card.deck_id())?;
        let type_context = ctx.type_by_id(card.flashcard_type_id())?;

        let mut fields = Vec::new();
        for (key, raw) in values {
            let definition = type_context.fields.iter().find(|f| f.key() == key).ok_or_else(|| {
                OrchestratorError::Invalid(format!("'{key}' is not a field of this flashcard type"))
            })?;
            let content = if definition.is_rich() {
                self.sanitizer.sanitize_html(raw)
            } else {
                raw.trim().to_owned()
            };
            let has_content = if definition.is_rich() {
                !self.sanitizer.strip_tags(&content).trim().is_empty()
            } else {
                !content.is_empty()
            };
            if has_content {
                fields.push(FieldValue::new(saved_id(definition.id(), "field definition")?, content)?);
            }
        }
        for definition in type_context.fields.iter().filter(|f| f.is_required()) {
            let id = saved_id(definition.id(), "field definition")?;
            let has_text = fields.iter().any(|f| f.field_definition_id() == id);
            let has_media = card.media().iter().any(|m| m.field_definition_id == id);
            if !has_text && !has_media {
                return Err(OrchestratorError::Invalid(format!("{} is required", definition.label())));
            }
        }
        Ok((ctx, card, fields))
    }

    /// Copies a file into the media store and attaches it to a field of a saved flashcard, under
    /// the same rules as for a draft.
    pub fn attach_saved_media(
        &self,
        card_uuid: &str,
        field_key: &str,
        source: &Path,
    ) -> OrchestratorResult<SavedCard> {
        let card = self.flashcards.get_flashcard(card_uuid)?;
        let ctx = self.context_of_deck(card.deck_id())?;
        let definition = ctx
            .type_by_id(card.flashcard_type_id())?
            .fields
            .iter()
            .find(|f| f.key() == field_key)
            .ok_or_else(|| {
                OrchestratorError::Invalid(format!("'{field_key}' is not a field of this flashcard type"))
            })?;
        if !definition.media_allowed() {
            return Err(OrchestratorError::Invalid(format!("{} does not accept media", definition.label())));
        }
        let definition_id = saved_id(definition.id(), "field definition")?;
        let attached = card.media().iter().filter(|m| m.field_definition_id == definition_id).count();
        if attached >= MAX_MEDIA_PER_FIELD {
            return Err(OrchestratorError::Invalid(format!(
                "{} already has {MAX_MEDIA_PER_FIELD} files",
                definition.label()
            )));
        }

        let attachment = Self::attachment_of(self.media.import(source)?)?;
        let updated = self
            .flashcards
            .attach_flashcard_media(card_uuid, FieldMedia { field_definition_id: definition_id, attachment })?;
        Self::saved_card(&ctx, &updated)
    }

    /// Like [`generate_media`](Self::generate_media), for a saved flashcard: the text comes from
    /// the saved value of the source field.
    pub async fn generate_saved_media(
        &self,
        card_uuid: &str,
        field_key: &str,
    ) -> OrchestratorResult<SavedCard> {
        let (text, language_tag, definition_id) = {
            let card = self.flashcards.get_flashcard(card_uuid)?;
            let ctx = self.context_of_deck(card.deck_id())?;
            let fields = &ctx.type_by_id(card.flashcard_type_id())?.fields;
            let definition = Self::field_by_key(fields, field_key)?;
            let source = Self::speech_source(fields, definition)?;
            let definition_id = saved_id(definition.id(), "field definition")?;
            let source_id = saved_id(source.id(), "field definition")?;
            let attached = card.media().iter().filter(|m| m.field_definition_id == definition_id).count();
            Self::check_media_room(definition, attached)?;
            let raw = card
                .fields()
                .iter()
                .find(|f| f.field_definition_id() == source_id)
                .map(|f| f.content())
                .unwrap_or_default();
            (self.spoken_text(source, raw)?, ctx.variant.bcp47_code().to_owned(), definition_id)
        };

        let audio = self.speech.synthesize(&text, &language_tag).await?;
        let attachment = Self::attachment_of(self.media.import_bytes(&audio.bytes)?)?;
        let keep = attachment.sha256().to_owned();
        let mut updated = self
            .flashcards
            .attach_flashcard_media(card_uuid, FieldMedia { field_definition_id: definition_id, attachment })?;
        // Generating makes the audio of the phrase as it is now, so older audio is replaced.
        let stale: Vec<String> = updated
            .media()
            .iter()
            .filter(|m| m.field_definition_id == definition_id && m.attachment.sha256() != keep)
            .map(|m| m.attachment.sha256().to_owned())
            .collect();
        for sha256 in stale {
            updated = self.flashcards.remove_flashcard_media(card_uuid, definition_id, &sha256)?;
        }
        let ctx = self.context_of_deck(updated.deck_id())?;
        Self::saved_card(&ctx, &updated)
    }

    fn attachment_of(stored: StoredMedia) -> OrchestratorResult<MediaAttachment> {
        Ok(MediaAttachment::new(
            stored.kind,
            stored.mime_type,
            stored.sha256,
            stored.file_name,
            i64::try_from(stored.size_bytes).unwrap_or(i64::MAX),
        )?)
    }

    fn field_by_key<'a>(fields: &'a [FieldDefinition], key: &str) -> OrchestratorResult<&'a FieldDefinition> {
        fields.iter().find(|f| f.key() == key).ok_or_else(|| {
            OrchestratorError::Invalid(format!("'{key}' is not a field of this flashcard type"))
        })
    }

    fn check_media_room(definition: &FieldDefinition, attached: usize) -> OrchestratorResult<()> {
        if attached >= MAX_MEDIA_PER_FIELD {
            return Err(OrchestratorError::Invalid(format!(
                "{} already has {MAX_MEDIA_PER_FIELD} files",
                definition.label()
            )));
        }
        Ok(())
    }

    /// The field an audio field is spoken from. Only audio fields can be generated.
    fn speech_source<'a>(
        fields: &'a [FieldDefinition],
        definition: &FieldDefinition,
    ) -> OrchestratorResult<&'a FieldDefinition> {
        let source_key = definition.source_key().filter(|_| definition.is_audio()).ok_or_else(|| {
            OrchestratorError::Invalid(format!("{} cannot be generated", definition.label()))
        })?;
        Self::field_by_key(fields, source_key)
    }

    /// The field an IPA field is generated from. Only IPA fields can be generated this way.
    fn ipa_source<'a>(
        fields: &'a [FieldDefinition],
        definition: &FieldDefinition,
    ) -> OrchestratorResult<&'a FieldDefinition> {
        let source_key = definition.source_key().filter(|_| definition.is_ipa()).ok_or_else(|| {
            OrchestratorError::Invalid(format!("{} cannot be generated", definition.label()))
        })?;
        Self::field_by_key(fields, source_key)
    }

    /// The pronunciation of the words of `text` in the deck's language, as the HTML of an IPA
    /// field, and a warning naming the words that had none.
    fn pronunciation_html(&self, text: &str, language_tag: &str) -> OrchestratorResult<(String, Vec<String>)> {
        let result = pronunciation_text::transcribe(text, language_tag, self.pronunciation.as_ref())?;
        if result.html.is_empty() {
            return Err(OrchestratorError::Invalid(format!(
                "no pronunciation was found for the words of the phrase ({})",
                result.missing.join(", ")
            )));
        }
        let warnings = if result.missing.is_empty() {
            Vec::new()
        } else {
            vec![format!("No pronunciation found for: {}", result.missing.join(", "))]
        };
        Ok((self.sanitizer.sanitize_html(&result.html), warnings))
    }

    /// The plain text of a source field's value, ready to be spoken.
    fn spoken_text(&self, source: &FieldDefinition, raw: &str) -> OrchestratorResult<String> {
        let text = if source.is_rich() { self.sanitizer.strip_tags(raw) } else { raw.to_owned() };
        let text = text.trim();
        if text.is_empty() {
            return Err(OrchestratorError::Invalid(format!("{} is empty, so there is nothing to speak", source.label())));
        }
        Ok(text.to_owned())
    }

    /// Detaches a file (by its hash) from a field of a saved flashcard, unless the field would be
    /// left empty while required. The file stays in the media store.
    pub fn remove_saved_media(
        &self,
        card_uuid: &str,
        field_key: &str,
        sha256: &str,
    ) -> OrchestratorResult<SavedCard> {
        let card = self.flashcards.get_flashcard(card_uuid)?;
        let ctx = self.context_of_deck(card.deck_id())?;
        let definition = ctx
            .type_by_id(card.flashcard_type_id())?
            .fields
            .iter()
            .find(|f| f.key() == field_key)
            .ok_or_else(|| {
                OrchestratorError::Invalid(format!("'{field_key}' is not a field of this flashcard type"))
            })?;
        let definition_id = saved_id(definition.id(), "field definition")?;
        let updated = self.flashcards.remove_flashcard_media(card_uuid, definition_id, sha256)?;
        Self::saved_card(&ctx, &updated)
    }

    /// Deletes a saved flashcard with its fields, media links and review history.
    pub fn delete_saved(&self, card_uuid: &str) -> OrchestratorResult<()> {
        Ok(self.flashcards.delete_flashcard(card_uuid)?)
    }

    fn saved_card(ctx: &Context, card: &Flashcard) -> OrchestratorResult<SavedCard> {
        let type_context = ctx.type_by_id(card.flashcard_type_id())?;
        let key_of = |definition_id: i64| {
            type_context.fields.iter().find(|f| f.id() == Some(definition_id)).map(|f| f.key().to_owned())
        };
        let mut values = BTreeMap::new();
        for value in card.fields() {
            if let Some(key) = key_of(value.field_definition_id()) {
                values.insert(key, value.content().to_owned());
            }
        }
        let mut media: BTreeMap<String, Vec<MediaAttachment>> = BTreeMap::new();
        for item in card.media() {
            if let Some(key) = key_of(item.field_definition_id) {
                media.entry(key).or_default().push(item.attachment.clone());
            }
        }
        Ok(SavedCard {
            uuid: card.uuid().to_owned(),
            type_slug: type_context.flashcard_type.slug().to_owned(),
            values,
            media,
            created_at: card.created_at(),
            updated_at: card.updated_at(),
            stack_uuid: card.stack_uuid().map(str::to_owned),
        })
    }

    /// Where a stored media file is on disk, for the view to preview.
    pub fn media_path(&self, file_name: &str) -> Option<PathBuf> {
        self.media.path_of(file_name)
    }

    /// Copies a draft right after itself and returns the deck's updated list.
    pub fn duplicate_draft(&self, draft_uuid: &str) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let draft = self.flashcards.get_draft(draft_uuid)?;
        self.flashcards.duplicate_draft(draft_uuid)?;
        self.list_like(&draft)
    }

    pub fn move_draft(&self, draft_uuid: &str, direction: Direction) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let draft = self.flashcards.get_draft(draft_uuid)?;
        self.flashcards.move_draft(draft_uuid, direction)?;
        self.list_like(&draft)
    }

    pub fn delete_draft(&self, draft_uuid: &str) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let draft = self.flashcards.get_draft(draft_uuid)?;
        self.flashcards.delete_draft(draft_uuid)?;
        self.list_like(&draft)
    }

    /// Publishes every draft of a deck. All of them are checked first, each against the fields of
    /// its own type. If any has a problem nothing is saved and the problems are returned.
    pub fn publish_drafts(&self, deck_name: &str) -> OrchestratorResult<PublishOutcome> {
        let ctx = self.context(deck_name)?;
        let drafts = self.flashcards.list_drafts(ctx.deck_id)?;

        let mut issues = Vec::new();
        let mut ready = Vec::with_capacity(drafts.len());
        for draft in &drafts {
            let type_context = ctx.type_by_id(draft.flashcard_type_id())?;
            let mut fields = Vec::new();
            let mut media = Vec::new();
            for definition in &type_context.fields {
                let attached = if definition.media_allowed() {
                    draft.media().get(definition.key()).map(Vec::as_slice).unwrap_or(&[])
                } else {
                    &[]
                };
                let definition_id = saved_id(definition.id(), "field definition")?;
                media.extend(attached.iter().map(|attachment| FieldMedia {
                    field_definition_id: definition_id,
                    attachment: attachment.clone(),
                }));

                let raw = draft.values().get(definition.key()).map(String::as_str).unwrap_or("");
                let content = if definition.is_rich() { self.sanitizer.sanitize_html(raw) } else { raw.trim().to_owned() };
                let has_content = if definition.is_rich() {
                    !self.sanitizer.strip_tags(&content).trim().is_empty()
                } else {
                    !content.is_empty()
                };

                if !has_content && attached.is_empty() {
                    if definition.is_required() {
                        issues.push(DraftIssue {
                            draft_uuid: draft.uuid().to_owned(),
                            field_key: Some(definition.key().to_owned()),
                            message: format!("{} is required", definition.label()),
                        });
                    }
                    continue;
                }
                if !has_content {
                    continue; // media is the only content of this field
                }
                match FieldValue::new(definition_id, content) {
                    Ok(value) => fields.push(value),
                    Err(err) => issues.push(DraftIssue {
                        draft_uuid: draft.uuid().to_owned(),
                        field_key: Some(definition.key().to_owned()),
                        message: err.to_string(),
                    }),
                }
            }
            ready.push(PublishedDraft {
                draft_uuid: draft.uuid().to_owned(),
                flashcard_type_id: type_context.type_id,
                fields,
                media,
                stack_uuid: draft.stack_uuid().map(str::to_owned),
            });
        }

        if !issues.is_empty() || ready.is_empty() {
            return Ok(PublishOutcome { published: 0, issues });
        }
        let published = self.flashcards.publish_drafts(ctx.deck_id, ready)?;
        Ok(PublishOutcome { published: published.len(), issues })
    }

    fn list_like(&self, draft: &FlashcardDraft) -> OrchestratorResult<Vec<FlashcardDraft>> {
        Ok(self.flashcards.list_drafts(draft.deck_id())?)
    }

    fn context(&self, deck_name: &str) -> OrchestratorResult<Context> {
        let deck = self.decks.get_deck_by_name(deck_name)?;
        self.build(deck)
    }

    fn context_of(&self, draft: &FlashcardDraft) -> OrchestratorResult<Context> {
        self.context_of_deck(draft.deck_id())
    }

    fn context_of_deck(&self, deck_id: i64) -> OrchestratorResult<Context> {
        let deck = self.decks.get_deck(deck_id)?;
        self.build(deck)
    }

    fn build(&self, deck: Deck) -> OrchestratorResult<Context> {
        let variant = self.languages.get_variant(deck.language_variant_id())?;
        let language = self.languages.get_language(variant.language_id())?;
        let mut types = Vec::new();
        for flashcard_type in self.card_types.list_types()? {
            let fields = self.card_types.fields_for(flashcard_type.slug(), language.id())?;
            types.push(TypeContext {
                type_id: saved_id(flashcard_type.id(), "flashcard type")?,
                flashcard_type,
                fields,
            });
        }
        Ok(Context {
            deck_id: saved_id(deck.id(), "deck")?,
            deck,
            language,
            variant,
            types,
        })
    }
}
