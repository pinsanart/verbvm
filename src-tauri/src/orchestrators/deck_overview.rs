use std::sync::Arc;

use super::error::{OrchestratorError, OrchestratorResult};
use crate::modules::card_types::{CardTypeApi, FlashcardType};
use crate::modules::decks::{Deck, DeckApi, DeckSettings};
use crate::modules::flashcards::FlashcardApi;
use crate::modules::languages::{Language, LanguageApi, LanguageVariant};

/// A deck with its language, its accent (the language variant) and its card counts.
#[derive(Debug, Clone)]
pub struct DeckOverview {
    pub deck: Deck,
    pub language: Language,
    pub variant: LanguageVariant,
    /// How many flashcards the deck has of each type, including types it has none of.
    pub counts: Vec<(FlashcardType, i64)>,
    /// The study options of each type of the deck, including the types that use the defaults.
    pub settings: Vec<(FlashcardType, DeckSettings)>,
}

impl DeckOverview {
    pub fn total_cards(&self) -> i64 {
        self.counts.iter().map(|(_, count)| count).sum()
    }
}

/// Lists, creates and deletes decks, resolving languages, variants and card counts.
pub struct DeckOverviewOrchestrator {
    decks: Arc<dyn DeckApi>,
    languages: Arc<dyn LanguageApi>,
    card_types: Arc<dyn CardTypeApi>,
    flashcards: Arc<dyn FlashcardApi>,
}

impl DeckOverviewOrchestrator {
    pub fn new(
        decks: Arc<dyn DeckApi>,
        languages: Arc<dyn LanguageApi>,
        card_types: Arc<dyn CardTypeApi>,
        flashcards: Arc<dyn FlashcardApi>,
    ) -> Self {
        Self { decks, languages, card_types, flashcards }
    }

    /// Every deck, ordered by name.
    pub fn list(&self) -> OrchestratorResult<Vec<DeckOverview>> {
        let types = self.card_types.list_types()?;
        self.decks
            .list_decks()?
            .into_iter()
            .map(|deck| self.overview(deck, &types))
            .collect()
    }

    pub fn get(&self, deck_name: &str) -> OrchestratorResult<DeckOverview> {
        let deck = self.decks.get_deck_by_name(deck_name)?;
        self.overview(deck, &self.card_types.list_types()?)
    }

    /// Creates a deck in a language. `variant_bcp47` picks the accent. Without it the language's
    /// default variant is used. The variant must belong to the language.
    pub fn create(
        &self,
        name: String,
        language_iso_639_3: &str,
        variant_bcp47: Option<&str>,
        description: Option<String>,
    ) -> OrchestratorResult<DeckOverview> {
        let language = self.languages.get_language_by_iso(language_iso_639_3)?;
        let variant = match variant_bcp47 {
            Some(code) => self.languages.get_variant_by_bcp47(code)?,
            None => {
                let id = language.default_variant_id().ok_or_else(|| {
                    OrchestratorError::Invalid(format!("{} has no variants", language.name()))
                })?;
                self.languages.get_variant(id)?
            }
        };
        if Some(variant.language_id()) != language.id() {
            return Err(OrchestratorError::Invalid(format!(
                "{} is not a variant of {}",
                variant.bcp47_code(),
                language.name()
            )));
        }
        let variant_id = variant.id().ok_or_else(|| OrchestratorError::Internal("variant was not saved".into()))?;
        let deck = self.decks.create_deck(name, variant_id, description)?;
        self.overview(deck, &self.card_types.list_types()?)
    }

    /// Renames a deck, replaces its description and sets the study options of the types given
    /// (`(type slug, settings)`; the other types keep theirs). The language and accent do not
    /// change, since the fields of its flashcards depend on the language. Conflicts when the new
    /// name is taken. Nothing changes if a type is unknown.
    pub fn update(
        &self,
        deck_name: &str,
        new_name: String,
        description: Option<String>,
        type_settings: &[(String, DeckSettings)],
    ) -> OrchestratorResult<DeckOverview> {
        let type_ids = type_settings
            .iter()
            .map(|(slug, settings)| Ok((self.type_id(slug)?, *settings)))
            .collect::<OrchestratorResult<Vec<_>>>()?;
        let deck = self.decks.update_deck(deck_name, new_name, description)?;
        let deck_id = deck.id().ok_or_else(|| OrchestratorError::Internal("deck was not saved".into()))?;
        for (type_id, settings) in type_ids {
            self.decks.set_type_settings(deck_id, type_id, settings)?;
        }
        self.overview(deck, &self.card_types.list_types()?)
    }

    fn type_id(&self, slug: &str) -> OrchestratorResult<i64> {
        self.card_types
            .get_type_by_slug(slug)?
            .id()
            .ok_or_else(|| OrchestratorError::Internal("flashcard type was not saved".into()))
    }

    /// Deletes the deck with its flashcards and drafts.
    pub fn delete(&self, deck_name: &str) -> OrchestratorResult<()> {
        Ok(self.decks.delete_deck(deck_name)?)
    }

    fn overview(&self, deck: Deck, types: &[FlashcardType]) -> OrchestratorResult<DeckOverview> {
        let variant = self.languages.get_variant(deck.language_variant_id())?;
        let language = self.languages.get_language(variant.language_id())?;
        let deck_id = deck.id().ok_or_else(|| OrchestratorError::Internal("deck was not saved".into()))?;
        let counted = self.flashcards.count_by_type(deck_id)?;
        let counts = types
            .iter()
            .map(|t| {
                let count = counted
                    .iter()
                    .find(|c| Some(c.flashcard_type_id) == t.id())
                    .map_or(0, |c| c.count);
                (t.clone(), count)
            })
            .collect();
        let stored = self.decks.list_type_settings(deck_id)?;
        let settings = types
            .iter()
            .map(|t| {
                let own = stored
                    .iter()
                    .find(|(id, _)| Some(*id) == t.id())
                    .map_or_else(DeckSettings::default, |(_, settings)| *settings);
                (t.clone(), own)
            })
            .collect();
        Ok(DeckOverview { deck, language, variant, counts, settings })
    }
}
