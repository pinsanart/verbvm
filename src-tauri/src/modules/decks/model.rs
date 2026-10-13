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
    day_starts_at_minute: i64,
}

impl DeckSettings {
    /// The largest daily limit. Zero is allowed and means none that day.
    pub const MAX_PER_DAY: i64 = 9_999;
    pub const DEFAULT_NEW_CARDS_PER_DAY: i64 = 20;
    pub const DEFAULT_MAX_REVIEWS_PER_DAY: i64 = 200;
    /// The study day starts at 06:00 UTC, which is 03:00 in São Paulo.
    pub const DEFAULT_DAY_STARTS_AT_MINUTE: i64 = 6 * 60;
    /// The last minute of a UTC day, 23:59.
    pub const MAX_DAY_STARTS_AT_MINUTE: i64 = 24 * 60 - 1;

    /// Both limits must be between 0 and 9999.
    pub fn new(new_cards_per_day: i64, max_reviews_per_day: i64, autoplay_audio: bool) -> DecksResult<Self> {
        let in_range = |value: i64| (0..=Self::MAX_PER_DAY).contains(&value);
        if !in_range(new_cards_per_day) {
            return Err(DecksError::Invalid { field: "new_cards_per_day", reason: "must be between 0 and 9999" });
        }
        if !in_range(max_reviews_per_day) {
            return Err(DecksError::Invalid { field: "max_reviews_per_day", reason: "must be between 0 and 9999" });
        }
        Ok(Self {
            new_cards_per_day,
            max_reviews_per_day,
            autoplay_audio,
            day_starts_at_minute: Self::DEFAULT_DAY_STARTS_AT_MINUTE,
        })
    }

    /// The same settings with the study day starting at this many minutes after UTC midnight
    /// (0 to 1439).
    pub fn with_day_starts_at(self, minute: i64) -> DecksResult<Self> {
        if !(0..=Self::MAX_DAY_STARTS_AT_MINUTE).contains(&minute) {
            return Err(DecksError::Invalid { field: "day_starts_at_minute", reason: "must be between 0 and 1439" });
        }
        Ok(Self { day_starts_at_minute: minute, ..self })
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

    /// When the study day starts, in minutes after UTC midnight. From then on, the review cards due
    /// that day are all offered, and the daily limits count from it.
    pub fn day_starts_at_minute(&self) -> i64 {
        self.day_starts_at_minute
    }
}

impl Default for DeckSettings {
    fn default() -> Self {
        Self {
            new_cards_per_day: Self::DEFAULT_NEW_CARDS_PER_DAY,
            max_reviews_per_day: Self::DEFAULT_MAX_REVIEWS_PER_DAY,
            autoplay_audio: true,
            day_starts_at_minute: Self::DEFAULT_DAY_STARTS_AT_MINUTE,
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
    fn the_study_day_starts_at_six_utc_by_default_and_the_minute_is_checked() {
        let settings = DeckSettings::default();
        assert_eq!(settings.day_starts_at_minute(), 360, "03:00 in São Paulo");
        assert_eq!(DeckSettings::new(1, 1, true).unwrap().day_starts_at_minute(), 360);
        assert_eq!(settings.with_day_starts_at(0).unwrap().day_starts_at_minute(), 0);
        assert_eq!(settings.with_day_starts_at(1_439).unwrap().day_starts_at_minute(), 1_439);
        assert!(settings.with_day_starts_at(-1).is_err());
        assert!(settings.with_day_starts_at(1_440).is_err());
        assert_eq!(settings.with_day_starts_at(90).unwrap().new_cards_per_day(), 20, "the rest is kept");
    }

    #[test]
    fn failed_revise_leaves_the_deck_untouched() {
        let mut deck = Deck::new("A".into(), 1, Some("d".into())).unwrap();
        assert!(deck.revise("".into(), None).is_err());
        assert_eq!((deck.name(), deck.description()), ("A", Some("d")));
    }
}
