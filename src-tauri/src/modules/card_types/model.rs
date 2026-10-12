use super::error::{CardTypesError, CardTypesResult};

const MAX_LEN: usize = 50;

/// Which side of the flashcard a field is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Front,
    Back,
}

impl Side {
    /// The value stored in the database.
    pub fn as_str(self) -> &'static str {
        match self {
            Side::Front => "FRONT",
            Side::Back => "BACK",
        }
    }

    pub fn parse(value: &str) -> CardTypesResult<Self> {
        match value {
            "FRONT" => Ok(Side::Front),
            "BACK" => Ok(Side::Back),
            _ => Err(CardTypesError::Invalid { field: "side", reason: "must be FRONT or BACK" }),
        }
    }
}

/// A kind of flashcard, such as vocabulary. Identified by its slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlashcardType {
    id: Option<i64>,
    slug: String,
    name: String,
    description: Option<String>,
}

impl FlashcardType {
    /// An unsaved type. The slug is 1-50 characters of lowercase letters, digits and `_`.
    pub fn new(slug: String, name: String, description: Option<String>) -> CardTypesResult<Self> {
        Ok(Self {
            id: None,
            slug: validate_identifier("slug", slug)?,
            name: validate_text("name", name)?,
            description: description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
        })
    }

    pub(super) fn restore(id: i64, slug: String, name: String, description: Option<String>) -> Self {
        Self { id: Some(id), slug, name, description }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// One field of a flashcard type. `language_id` is `None` for fields that every language uses.
///
/// A rich field holds HTML, a plain field holds text. A field that allows media can have images
/// or audio attached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDefinition {
    id: Option<i64>,
    flashcard_type_id: i64,
    language_id: Option<i64>,
    side: Side,
    key: String,
    label: String,
    position: i64,
    is_required: bool,
    is_rich: bool,
    media_allowed: bool,
}

impl FieldDefinition {
    /// An unsaved definition. The key is 1-50 characters of lowercase letters, digits and `_`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        flashcard_type_id: i64,
        language_id: Option<i64>,
        side: Side,
        key: String,
        label: String,
        position: i64,
        is_required: bool,
        is_rich: bool,
        media_allowed: bool,
    ) -> CardTypesResult<Self> {
        Ok(Self {
            id: None,
            flashcard_type_id,
            language_id,
            side,
            key: validate_identifier("key", key)?,
            label: validate_text("label", label)?,
            position,
            is_required,
            is_rich,
            media_allowed,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn restore(
        id: i64,
        flashcard_type_id: i64,
        language_id: Option<i64>,
        side: Side,
        key: String,
        label: String,
        position: i64,
        is_required: bool,
        is_rich: bool,
        media_allowed: bool,
    ) -> Self {
        Self {
            id: Some(id),
            flashcard_type_id,
            language_id,
            side,
            key,
            label,
            position,
            is_required,
            is_rich,
            media_allowed,
        }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn flashcard_type_id(&self) -> i64 {
        self.flashcard_type_id
    }

    pub fn language_id(&self) -> Option<i64> {
        self.language_id
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn position(&self) -> i64 {
        self.position
    }

    pub fn is_required(&self) -> bool {
        self.is_required
    }

    pub fn is_rich(&self) -> bool {
        self.is_rich
    }

    pub fn media_allowed(&self) -> bool {
        self.media_allowed
    }
}

fn validate_identifier(field: &'static str, value: String) -> CardTypesResult<String> {
    let valid = !value.is_empty()
        && value.len() <= MAX_LEN
        && value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if valid {
        Ok(value)
    } else {
        Err(CardTypesError::Invalid {
            field,
            reason: "must be 1-50 characters of lowercase letters, digits or '_'",
        })
    }
}

fn validate_text(field: &'static str, value: String) -> CardTypesResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() || value.chars().count() > MAX_LEN {
        return Err(CardTypesError::Invalid { field, reason: "must be 1-50 characters" });
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_round_trips_through_its_stored_value() {
        assert_eq!(Side::parse(Side::Front.as_str()).unwrap(), Side::Front);
        assert_eq!(Side::parse(Side::Back.as_str()).unwrap(), Side::Back);
        assert!(Side::parse("MIDDLE").is_err());
    }

    #[test]
    fn slug_and_key_must_be_lowercase_identifiers() {
        assert!(FlashcardType::new("vocabulary".into(), "Vocabulary".into(), None).is_ok());
        assert!(FlashcardType::new("Vocab".into(), "Vocabulary".into(), None).is_err());
        assert!(FlashcardType::new("a b".into(), "Vocabulary".into(), None).is_err());
        let field = |key: &str| {
            FieldDefinition::new(1, None, Side::Front, key.into(), "Label".into(), 0, true, false, false)
        };
        assert!(field("word_or_expression").is_ok());
        assert!(field("Word").is_err());
        assert!(field("").is_err());
    }
}
