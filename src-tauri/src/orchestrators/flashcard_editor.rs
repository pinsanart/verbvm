use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::error::{OrchestratorError, OrchestratorResult};
use crate::dependencies::media_store::MediaStorage;
use crate::dependencies::sanitizer::HtmlSanitizer;
use crate::modules::card_types::{CardTypeApi, FieldDefinition, FlashcardType};
use crate::modules::decks::{Deck, DeckApi};
use crate::modules::flashcards::{
    Direction, FieldMedia, FieldValue, FlashcardApi, FlashcardDraft, MediaAttachment, PublishedDraft,
};
use crate::modules::languages::{Language, LanguageApi, LanguageVariant};

/// What the editor needs to show the form of a flashcard type in a deck.
#[derive(Debug, Clone)]
pub struct EditorForm {
    pub deck: Deck,
    pub language: Language,
    pub variant: LanguageVariant,
    pub flashcard_type: FlashcardType,
    /// The type's fields for the deck's language, front before back.
    pub fields: Vec<FieldDefinition>,
}

/// A problem that stopped a draft from being published.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftIssue {
    pub draft_uuid: String,
    /// The field the problem is about, if it is about one.
    pub field_key: Option<String>,
    pub message: String,
}

/// Result of publishing the drafts of a deck and type. When `issues` is not empty nothing was
/// saved and `published` is 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishOutcome {
    pub published: usize,
    pub issues: Vec<DraftIssue>,
}

/// Edits the drafts of a deck and type and publishes them as flashcards.
///
/// Rules enforced here, because they need the card type, the language and the deck together:
/// only fields of the type (for the deck's language) can be filled, rich fields are sanitized
/// before they are stored, media can only be attached to fields that allow it, and required
/// fields must have text or media to be published.
pub struct FlashcardEditorOrchestrator {
    decks: Arc<dyn DeckApi>,
    languages: Arc<dyn LanguageApi>,
    card_types: Arc<dyn CardTypeApi>,
    flashcards: Arc<dyn FlashcardApi>,
    sanitizer: Arc<dyn HtmlSanitizer>,
    media: Arc<dyn MediaStorage>,
}

/// The most media files one field can hold.
const MAX_MEDIA_PER_FIELD: usize = 10;

struct Context {
    deck: Deck,
    deck_id: i64,
    language: Language,
    variant: LanguageVariant,
    flashcard_type: FlashcardType,
    type_id: i64,
    fields: Vec<FieldDefinition>,
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
    ) -> Self {
        Self { decks, languages, card_types, flashcards, sanitizer, media }
    }

    pub fn form(&self, deck_name: &str, type_slug: &str) -> OrchestratorResult<EditorForm> {
        let ctx = self.context(deck_name, type_slug)?;
        Ok(EditorForm {
            deck: ctx.deck,
            language: ctx.language,
            variant: ctx.variant,
            flashcard_type: ctx.flashcard_type,
            fields: ctx.fields,
        })
    }

    pub fn drafts(&self, deck_name: &str, type_slug: &str) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let ctx = self.context(deck_name, type_slug)?;
        Ok(self.flashcards.list_drafts(ctx.deck_id, ctx.type_id)?)
    }

    /// Adds an empty draft and returns the updated list.
    pub fn add_draft(&self, deck_name: &str, type_slug: &str) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let ctx = self.context(deck_name, type_slug)?;
        self.flashcards.create_draft(ctx.deck_id, ctx.type_id)?;
        Ok(self.flashcards.list_drafts(ctx.deck_id, ctx.type_id)?)
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
        let mut cleaned = BTreeMap::new();
        for (key, content) in values {
            let definition = ctx.fields.iter().find(|f| f.key() == key).ok_or_else(|| {
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
        let definition = ctx.fields.iter().find(|f| f.key() == field_key).ok_or_else(|| {
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

        let stored = self.media.import(source)?;
        let attachment = MediaAttachment::new(
            stored.kind,
            stored.mime_type,
            stored.sha256,
            stored.file_name,
            i64::try_from(stored.size_bytes).unwrap_or(i64::MAX),
        )?;
        Ok(self.flashcards.attach_draft_media(draft_uuid, field_key, attachment)?)
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

    /// Where a stored media file is on disk, for the view to preview.
    pub fn media_path(&self, file_name: &str) -> Option<PathBuf> {
        self.media.path_of(file_name)
    }

    /// Copies a draft right after itself and returns the updated list.
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

    /// Publishes every draft of a deck and type. All of them are checked first. If any has a
    /// problem nothing is saved and the problems are returned.
    pub fn publish_drafts(&self, deck_name: &str, type_slug: &str) -> OrchestratorResult<PublishOutcome> {
        let ctx = self.context(deck_name, type_slug)?;
        let drafts = self.flashcards.list_drafts(ctx.deck_id, ctx.type_id)?;

        let mut issues = Vec::new();
        let mut ready = Vec::with_capacity(drafts.len());
        for draft in &drafts {
            let mut fields = Vec::new();
            let mut media = Vec::new();
            for definition in &ctx.fields {
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
            ready.push(PublishedDraft { draft_uuid: draft.uuid().to_owned(), fields, media });
        }

        if !issues.is_empty() || ready.is_empty() {
            return Ok(PublishOutcome { published: 0, issues });
        }
        let published = self.flashcards.publish_drafts(ctx.deck_id, ctx.type_id, ready)?;
        Ok(PublishOutcome { published: published.len(), issues })
    }

    fn list_like(&self, draft: &FlashcardDraft) -> OrchestratorResult<Vec<FlashcardDraft>> {
        Ok(self.flashcards.list_drafts(draft.deck_id(), draft.flashcard_type_id())?)
    }

    fn context(&self, deck_name: &str, type_slug: &str) -> OrchestratorResult<Context> {
        let deck = self.decks.get_deck_by_name(deck_name)?;
        let flashcard_type = self.card_types.get_type_by_slug(type_slug)?;
        self.build(deck, flashcard_type)
    }

    fn context_of(&self, draft: &FlashcardDraft) -> OrchestratorResult<Context> {
        let deck = self.decks.get_deck(draft.deck_id())?;
        let flashcard_type = self.card_types.get_type(draft.flashcard_type_id())?;
        self.build(deck, flashcard_type)
    }

    fn build(&self, deck: Deck, flashcard_type: FlashcardType) -> OrchestratorResult<Context> {
        let variant = self.languages.get_variant(deck.language_variant_id())?;
        let language = self.languages.get_language(variant.language_id())?;
        let fields = self.card_types.fields_for(flashcard_type.slug(), language.id())?;
        Ok(Context {
            deck_id: saved_id(deck.id(), "deck")?,
            type_id: saved_id(flashcard_type.id(), "flashcard type")?,
            deck,
            language,
            variant,
            flashcard_type,
            fields,
        })
    }
}
