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

/// How a field is filled in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Plain text.
    Plain,
    /// HTML, edited in the rich editor.
    Rich,
    /// Audio only, with no text of its own. It is uploaded or generated from the text of another
    /// field (its source key).
    Audio,
}

impl FieldKind {
    /// The value stored in the database.
    pub fn as_str(self) -> &'static str {
        match self {
            FieldKind::Plain => "PLAIN",
            FieldKind::Rich => "RICH",
            FieldKind::Audio => "AUDIO",
        }
    }

    pub fn parse(value: &str) -> CardTypesResult<Self> {
        match value {
            "PLAIN" => Ok(FieldKind::Plain),
            "RICH" => Ok(FieldKind::Rich),
            "AUDIO" => Ok(FieldKind::Audio),
            _ => Err(CardTypesError::Invalid { field: "kind", reason: "must be PLAIN, RICH or AUDIO" }),
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
/// A rich field holds HTML, a plain field holds text and an audio field holds only audio, spoken
/// from the text of the field named by `source_key`. A field that allows media can have images
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
    kind: FieldKind,
    source_key: Option<String>,
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
        kind: FieldKind,
        source_key: Option<String>,
        media_allowed: bool,
    ) -> CardTypesResult<Self> {
        let source_key = source_key.map(|k| validate_identifier("source_key", k)).transpose()?;
        match (kind, &source_key) {
            (FieldKind::Audio, None) => {
                return Err(CardTypesError::Invalid {
                    field: "source_key",
                    reason: "an audio field needs the key of the field it is spoken from",
                })
            }
            (FieldKind::Audio, Some(_)) if !media_allowed => {
                return Err(CardTypesError::Invalid {
                    field: "media_allowed",
                    reason: "an audio field must allow media",
                })
            }
            (FieldKind::Plain | FieldKind::Rich, Some(_)) => {
                return Err(CardTypesError::Invalid {
                    field: "source_key",
                    reason: "only an audio field has a source key",
                })
            }
            _ => {}
        }
        Ok(Self {
            id: None,
            flashcard_type_id,
            language_id,
            side,
            key: validate_identifier("key", key)?,
            label: validate_text("label", label)?,
            position,
            is_required,
            kind,
            source_key,
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
        kind: FieldKind,
        source_key: Option<String>,
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
            kind,
            source_key,
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

    pub fn kind(&self) -> FieldKind {
        self.kind
    }

    /// The field holds HTML.
    pub fn is_rich(&self) -> bool {
        self.kind == FieldKind::Rich
    }

    /// The field holds audio only.
    pub fn is_audio(&self) -> bool {
        self.kind == FieldKind::Audio
    }

    /// For an audio field, the key of the field its audio is spoken from.
    pub fn source_key(&self) -> Option<&str> {
        self.source_key.as_deref()
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
            FieldDefinition::new(1, None, Side::Front, key.into(), "Label".into(), 0, true, FieldKind::Plain, None, false)
        };
        assert!(field("word_or_expression").is_ok());
        assert!(field("Word").is_err());
        assert!(field("").is_err());
    }

    #[test]
    fn an_audio_field_needs_a_source_and_media_and_others_have_no_source() {
        let build = |kind, source: Option<&str>, media| {
            FieldDefinition::new(1, None, Side::Front, "audio".into(), "Audio".into(), 0, true, kind, source.map(String::from), media)
        };
        let audio = build(FieldKind::Audio, Some("phrase"), true).unwrap();
        assert!(audio.is_audio() && !audio.is_rich());
        assert_eq!(audio.source_key(), Some("phrase"));
        assert!(build(FieldKind::Audio, None, true).is_err());
        assert!(build(FieldKind::Audio, Some("phrase"), false).is_err());
        assert!(build(FieldKind::Plain, Some("phrase"), true).is_err());
        assert!(build(FieldKind::Rich, None, true).is_ok());
    }
}
