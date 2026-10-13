//! Pronunciation and stacks, which are part of the flashcard editor.
//!
//! A stack is a set of drafts (or of saved flashcards) that the view shows as one: the vocabulary,
//! phonetic and orthographic cards of the same phrase. The cards stay independent: each has its
//! own type, fields and review history, and a stack is only the link between them.

use super::*;

/// The cards a new stack is made of, in the order they are shown.
const STACK_TYPES: [&str; 3] = ["vocabulary", "phonetic", "orthographic"];

/// The field the cards of a stack share, and the one their generated fields are made from.
const SHARED_KEY: &str = "phrase";

impl FlashcardEditorOrchestrator {
    // ---- pronunciation ----

    /// Fills an IPA field of a draft with the pronunciation of the words of the field it is made
    /// from (one line per word, from the pronunciation dictionary). The text replaces what the
    /// field had. The text used is the one stored with the draft, so the caller saves pending
    /// edits first. The warnings name the words that had no pronunciation.
    pub fn generate_pronunciation(
        &self,
        draft_uuid: &str,
        field_key: &str,
    ) -> OrchestratorResult<Generated<FlashcardDraft>> {
        let draft = self.flashcards.get_draft(draft_uuid)?;
        let ctx = self.context_of(&draft)?;
        let fields = &ctx.type_by_id(draft.flashcard_type_id())?.fields;
        let definition = Self::field_by_key(fields, field_key)?;
        let source = Self::ipa_source(fields, definition)?;
        let raw = draft.values().get(source.key()).map(String::as_str).unwrap_or_default();
        let text = self.spoken_text(source, raw)?;
        let (html, warnings) = self.pronunciation_html(&text, ctx.variant.bcp47_code())?;

        let mut values = draft.values().clone();
        values.insert(field_key.to_owned(), html);
        Ok(Generated { value: self.flashcards.update_draft(draft_uuid, values)?, warnings })
    }

    /// Like [`generate_pronunciation`](Self::generate_pronunciation) for a saved flashcard, which
    /// keeps its other fields.
    pub fn generate_saved_pronunciation(
        &self,
        card_uuid: &str,
        field_key: &str,
    ) -> OrchestratorResult<Generated<SavedCard>> {
        let card = self.flashcards.get_flashcard(card_uuid)?;
        let ctx = self.context_of_deck(card.deck_id())?;
        let fields = &ctx.type_by_id(card.flashcard_type_id())?.fields;
        let definition = Self::field_by_key(fields, field_key)?;
        let source = Self::ipa_source(fields, definition)?;
        let source_id = saved_id(source.id(), "field definition")?;
        let raw = card
            .fields()
            .iter()
            .find(|f| f.field_definition_id() == source_id)
            .map(|f| f.content())
            .unwrap_or_default();
        let text = self.spoken_text(source, raw)?;
        let (html, warnings) = self.pronunciation_html(&text, ctx.variant.bcp47_code())?;

        let target_id = saved_id(definition.id(), "field definition")?;
        let mut values: Vec<FieldValue> = card
            .fields()
            .iter()
            .filter(|f| f.field_definition_id() != target_id)
            .cloned()
            .collect();
        values.push(FieldValue::new(target_id, html)?);
        let updated = self.flashcards.update_flashcard_fields(card_uuid, values)?;
        Ok(Generated { value: Self::saved_card(&ctx, &updated)?, warnings })
    }

    // ---- stacks of drafts ----

    /// Adds a stack of drafts (vocabulary, phonetic and orthographic) at the end of the deck's
    /// list and returns the updated list.
    pub fn add_stack(&self, deck_name: &str) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let ctx = self.context(deck_name)?;
        let stack_uuid = uuid::Uuid::new_v4().to_string();
        for slug in STACK_TYPES {
            let type_context = ctx.type_by_slug(slug)?;
            self.flashcards.create_draft_in_stack(ctx.deck_id, type_context.type_id, &stack_uuid)?;
        }
        Ok(self.flashcards.list_drafts(ctx.deck_id)?)
    }

    /// Adds a draft of a type the stack does not have yet, with the shared phrase copied in.
    pub fn add_stack_member(&self, stack_uuid: &str, type_slug: &str) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let members = self.flashcards.list_stack_drafts(stack_uuid)?;
        let first = members.first().ok_or_else(|| OrchestratorError::NotFound("stack not found".into()))?;
        let ctx = self.context_of(first)?;
        let type_context = ctx.type_by_slug(type_slug)?;
        if members.iter().any(|m| m.flashcard_type_id() == type_context.type_id) {
            return Err(OrchestratorError::Invalid("the stack already has a card of this type".into()));
        }
        let draft = self.flashcards.create_draft_in_stack(ctx.deck_id, type_context.type_id, stack_uuid)?;

        // The phrase of the first card that has one, as plain text.
        let phrase = members.iter().find_map(|member| {
            let raw = member.values().get(SHARED_KEY).filter(|v| !v.trim().is_empty())?;
            let rich = ctx
                .type_by_id(member.flashcard_type_id())
                .ok()?
                .fields
                .iter()
                .any(|f| f.key() == SHARED_KEY && f.is_rich());
            Some(if rich { self.sanitizer.strip_tags(raw) } else { raw.clone() })
        });
        if let Some(phrase) = phrase {
            if type_context.fields.iter().any(|f| f.key() == SHARED_KEY) {
                self.update_draft(draft.uuid(), BTreeMap::from([(SHARED_KEY.to_owned(), phrase)]))?;
            }
        }
        Ok(self.flashcards.list_drafts(ctx.deck_id)?)
    }

    /// Takes a draft out of its stack, where it becomes a regular draft, and returns the list.
    pub fn unstack(&self, draft_uuid: &str) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let draft = self.flashcards.get_draft(draft_uuid)?;
        self.flashcards.unstack_draft(draft_uuid)?;
        self.list_like(&draft)
    }

    /// Deletes every draft of a stack and returns the list.
    pub fn delete_stack(&self, stack_uuid: &str) -> OrchestratorResult<Vec<FlashcardDraft>> {
        let members = self.flashcards.list_stack_drafts(stack_uuid)?;
        let first = members.first().ok_or_else(|| OrchestratorError::NotFound("stack not found".into()))?;
        self.flashcards.delete_stack_drafts(stack_uuid)?;
        self.list_like(first)
    }

    /// Generates, for every card of a stack, what can be generated: the audio of an audio field
    /// and the pronunciation of an IPA field, each from the phrase of its own card. What works is
    /// applied and what does not is reported in the warnings, so one failure (offline, no
    /// dictionary) does not lose the other. The text used is the one stored with the drafts.
    pub async fn generate_stack(&self, stack_uuid: &str) -> OrchestratorResult<Generated<Vec<FlashcardDraft>>> {
        let members = self.flashcards.list_stack_drafts(stack_uuid)?;
        let first = members.first().ok_or_else(|| OrchestratorError::NotFound("stack not found".into()))?;
        let ctx = self.context_of(first)?;
        let mut warnings = Vec::new();
        for member in &members {
            let type_context = ctx.type_by_id(member.flashcard_type_id())?;
            let type_name = type_context.flashcard_type.name().to_owned();
            for definition in type_context.fields.iter().filter(|f| f.is_ipa() || f.is_audio()) {
                let outcome = if definition.is_ipa() {
                    self.generate_pronunciation(member.uuid(), definition.key()).map(|g| g.warnings)
                } else {
                    self.generate_media(member.uuid(), definition.key()).await.map(|_| Vec::new())
                };
                match outcome {
                    Ok(found) => warnings.extend(found.into_iter().map(|w| format!("{type_name}: {w}"))),
                    Err(error) => warnings.push(format!("{type_name}, {}: {error}", definition.label())),
                }
            }
        }
        Ok(Generated { value: self.flashcards.list_drafts(ctx.deck_id)?, warnings })
    }

    // ---- stacks of saved flashcards ----

    /// Replaces the text of several saved flashcards at once (the cards of a stack edited
    /// together). Every card is checked first, with the rules of `update_saved`, and nothing is
    /// stored unless all of them pass.
    pub fn update_saved_many(
        &self,
        updates: Vec<(String, BTreeMap<String, String>)>,
    ) -> OrchestratorResult<Vec<SavedCard>> {
        let prepared = updates
            .iter()
            .map(|(uuid, values)| self.prepare_saved_update(uuid, values))
            .collect::<OrchestratorResult<Vec<_>>>()?;
        let mut saved = Vec::with_capacity(prepared.len());
        for (ctx, card, fields) in prepared {
            let updated = self.flashcards.update_flashcard_fields(card.uuid(), fields)?;
            saved.push(Self::saved_card(&ctx, &updated)?);
        }
        Ok(saved)
    }

    /// Takes a saved flashcard out of its stack. The card is kept, and a stack left with fewer
    /// than two cards is dissolved.
    pub fn unstack_saved(&self, card_uuid: &str) -> OrchestratorResult<SavedCard> {
        let card = self.flashcards.unstack_flashcard(card_uuid)?;
        let ctx = self.context_of_deck(card.deck_id())?;
        Self::saved_card(&ctx, &card)
    }

    /// Deletes every flashcard of a stack, with their review history.
    pub fn delete_saved_stack(&self, stack_uuid: &str) -> OrchestratorResult<()> {
        Ok(self.flashcards.delete_stack_flashcards(stack_uuid)?)
    }

    /// Like [`generate_stack`](Self::generate_stack) for the saved flashcards of a stack. The text
    /// used is the saved text, so the caller saves pending edits first.
    pub async fn generate_saved_stack(&self, stack_uuid: &str) -> OrchestratorResult<Generated<Vec<SavedCard>>> {
        let members = self.flashcards.list_stack_flashcards(stack_uuid)?;
        let first = members.first().ok_or_else(|| OrchestratorError::NotFound("stack not found".into()))?;
        let ctx = self.context_of_deck(first.deck_id())?;
        let mut warnings = Vec::new();
        for member in &members {
            let type_context = ctx.type_by_id(member.flashcard_type_id())?;
            let type_name = type_context.flashcard_type.name().to_owned();
            for definition in type_context.fields.iter().filter(|f| f.is_ipa() || f.is_audio()) {
                let outcome = if definition.is_ipa() {
                    self.generate_saved_pronunciation(member.uuid(), definition.key()).map(|g| g.warnings)
                } else {
                    self.generate_saved_media(member.uuid(), definition.key()).await.map(|_| Vec::new())
                };
                match outcome {
                    Ok(found) => warnings.extend(found.into_iter().map(|w| format!("{type_name}: {w}"))),
                    Err(error) => warnings.push(format!("{type_name}, {}: {error}", definition.label())),
                }
            }
        }
        let mut cards = Vec::with_capacity(members.len());
        for member in &members {
            cards.push(Self::saved_card(&ctx, &self.flashcards.get_flashcard(member.uuid())?)?);
        }
        Ok(Generated { value: cards, warnings })
    }
}
