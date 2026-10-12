use chrono::{DateTime, Utc};

use super::error::{DecksError, DecksResult};

const MAX_NAME_LEN: usize = 100;

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
    fn failed_revise_leaves_the_deck_untouched() {
        let mut deck = Deck::new("A".into(), 1, Some("d".into())).unwrap();
        assert!(deck.revise("".into(), None).is_err());
        assert_eq!((deck.name(), deck.description()), ("A", Some("d")));
    }
}
