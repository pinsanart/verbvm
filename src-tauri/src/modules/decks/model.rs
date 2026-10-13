use chrono::{DateTime, Utc};

use super::error::{DecksError, DecksResult};

const MAX_NAME_LEN: usize = 100;

/// The options that shape how the cards of one type of a deck are studied. They are kept together so
/// a new option does not change the signature of every function that sets them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeckSettings {
    new_cards_per_day: i64,
    max_reviews_per_day: i64,
    autoplay_audio: bool,
}

impl DeckSettings {
    /// The largest daily limit. Zero is allowed and means none that day.
    pub const MAX_PER_DAY: i64 = 9_999;
    pub const DEFAULT_NEW_CARDS_PER_DAY: i64 = 20;
    pub const DEFAULT_MAX_REVIEWS_PER_DAY: i64 = 200;

    /// Both limits must be between 0 and 9999.
    pub fn new(new_cards_per_day: i64, max_reviews_per_day: i64, autoplay_audio: bool) -> DecksResult<Self> {
        let in_range = |value: i64| (0..=Self::MAX_PER_DAY).contains(&value);
        if !in_range(new_cards_per_day) {
            return Err(DecksError::Invalid { field: "new_cards_per_day", reason: "must be between 0 and 9999" });
        }
        if !in_range(max_reviews_per_day) {
            return Err(DecksError::Invalid { field: "max_reviews_per_day", reason: "must be between 0 and 9999" });
        }
        Ok(Self { new_cards_per_day, max_reviews_per_day, autoplay_audio })
    }

    /// How many cards seen for the first time a day lets in.
    pub fn new_cards_per_day(&self) -> i64 {
        self.new_cards_per_day
    }

    /// How many answers a day lets in for cards that had been recalled before.
    pub fn max_reviews_per_day(&self) -> i64 {
        self.max_reviews_per_day
    }

    /// Whether the audio of a card plays by itself when the card, or its answer, is shown.
    pub fn autoplay_audio(&self) -> bool {
        self.autoplay_audio
    }
}

impl Default for DeckSettings {
    fn default() -> Self {
        Self {
            new_cards_per_day: Self::DEFAULT_NEW_CARDS_PER_DAY,
            max_reviews_per_day: Self::DEFAULT_MAX_REVIEWS_PER_DAY,
            autoplay_audio: true,
        }
    }
}

/// A deck of flashcards. Built through `new` (unsaved, no id) or `restore` (from storage).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    id: Option<i64>,
    name: String,
    language_variant_id: i64,
    description: Option<String>,
    created_at: Option<DateTime<Utc>>,
}

impl Deck {
    /// An unsaved deck. The name is trimmed and must be 1-100 characters. A blank description
    /// becomes `None`.
    pub fn new(
        name: String,
        language_variant_id: i64,
        description: Option<String>,
    ) -> DecksResult<Self> {
        Ok(Self {
            id: None,
            name: validate_name(name)?,
            language_variant_id,
            description: normalize_description(description),
            created_at: None,
        })
    }

    pub(super) fn restore(
        id: i64,
        name: String,
        language_variant_id: i64,
        description: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self { id: Some(id), name, language_variant_id, description, created_at: Some(created_at) }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn language_variant_id(&self) -> i64 {
        self.language_variant_id
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// `None` until the deck is saved.
    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    /// Replaces the name and the description. Nothing changes if the name is invalid.
    pub fn revise(&mut self, name: String, description: Option<String>) -> DecksResult<()> {
        self.name = validate_name(name)?;
        self.description = normalize_description(description);
        Ok(())
    }
}

fn validate_name(name: String) -> DecksResult<String> {
    let name = name.trim().to_string();
    if name.is_empty() || name.chars().count() > MAX_NAME_LEN {
        return Err(DecksError::Invalid { field: "name", reason: "must be 1-100 characters" });
    }
    Ok(name)
}

fn normalize_description(description: Option<String>) -> Option<String> {
    description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_trimmed_and_bounded() {
        assert_eq!(Deck::new("  Travel ".into(), 1, None).unwrap().name(), "Travel");
        assert!(Deck::new("   ".into(), 1, None).is_err());
        assert!(Deck::new("x".repeat(101), 1, None).is_err());
    }

    #[test]
    fn blank_description_becomes_none() {
        assert_eq!(Deck::new("A".into(), 1, Some("  ".into())).unwrap().description(), None);
        assert_eq!(Deck::new("A".into(), 1, Some(" hi ".into())).unwrap().description(), Some("hi"));
    }

    #[test]
    fn settings_default_to_twenty_new_and_two_hundred_reviews_with_audio_on() {
        let settings = DeckSettings::default();
        assert_eq!((settings.new_cards_per_day(), settings.max_reviews_per_day()), (20, 200));
        assert!(settings.autoplay_audio());
    }

    #[test]
    fn the_daily_limits_are_between_zero_and_9999() {
        assert!(DeckSettings::new(0, 0, true).is_ok());
        assert!(DeckSettings::new(9_999, 9_999, true).is_ok());
        assert!(DeckSettings::new(-1, 10, true).is_err());
        assert!(DeckSettings::new(10, 10_000, true).is_err());
    }

    #[test]
    fn failed_revise_leaves_the_deck_untouched() {
        let mut deck = Deck::new("A".into(), 1, Some("d".into())).unwrap();
        assert!(deck.revise("".into(), None).is_err());
        assert_eq!((deck.name(), deck.description()), ("A", Some("d")));
    }
}
