use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dependencies::media_store::MediaKind;

use super::error::{FlashcardsError, FlashcardsResult};

/// Longest content a single field can hold, in characters.
pub const MAX_CONTENT_LEN: usize = 50_000;

fn validate_content(content: &str) -> FlashcardsResult<()> {
    if content.chars().count() > MAX_CONTENT_LEN {
        return Err(FlashcardsError::Invalid { field: "content", reason: "is too long" });
    }
    Ok(())
}

/// The content of one field of a flashcard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValue {
    field_definition_id: i64,
    content: String,
}

impl FieldValue {
    pub fn new(field_definition_id: i64, content: String) -> FlashcardsResult<Self> {
        validate_content(&content)?;
        Ok(Self { field_definition_id, content })
    }

    pub fn field_definition_id(&self) -> i64 {
        self.field_definition_id
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

/// An image or audio file attached to a field. The file itself is in the media store under
/// `file_name`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaAttachment {
    kind: MediaKind,
    mime_type: String,
    sha256: String,
    file_name: String,
    size_bytes: i64,
    position: i64,
}

impl MediaAttachment {
    /// An attachment for a stored file. `sha256` is the 64 hex characters of the content hash.
    pub fn new(
        kind: MediaKind,
        mime_type: String,
        sha256: String,
        file_name: String,
        size_bytes: i64,
    ) -> FlashcardsResult<Self> {
        if sha256.len() != 64 || !sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(FlashcardsError::Invalid { field: "sha256", reason: "must be 64 hex characters" });
        }
        if file_name.is_empty() || size_bytes < 0 {
            return Err(FlashcardsError::Invalid { field: "media", reason: "is incomplete" });
        }
        Ok(Self { kind, mime_type, sha256, file_name, size_bytes, position: 0 })
    }

    pub(super) fn restore(
        kind: MediaKind,
        mime_type: String,
        sha256: String,
        file_name: String,
        size_bytes: i64,
        position: i64,
    ) -> Self {
        Self { kind, mime_type, sha256, file_name, size_bytes, position }
    }

    pub fn kind(&self) -> MediaKind {
        self.kind
    }

    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn size_bytes(&self) -> i64 {
        self.size_bytes
    }

    pub fn position(&self) -> i64 {
        self.position
    }
}

/// A media attachment of a saved flashcard, with the field definition it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldMedia {
    pub field_definition_id: i64,
    pub attachment: MediaAttachment,
}

/// A saved flashcard of a deck, with one value per field it has content for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flashcard {
    id: Option<i64>,
    uuid: String,
    deck_id: i64,
    flashcard_type_id: i64,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    fields: Vec<FieldValue>,
    media: Vec<FieldMedia>,
}

impl Flashcard {
    /// An unsaved flashcard with a fresh uuid. A field can appear only once.
    pub fn new(
        deck_id: i64,
        flashcard_type_id: i64,
        fields: Vec<FieldValue>,
        media: Vec<FieldMedia>,
    ) -> FlashcardsResult<Self> {
        let mut seen = HashSet::new();
        if !fields.iter().all(|f| seen.insert(f.field_definition_id())) {
            return Err(FlashcardsError::Invalid { field: "fields", reason: "a field is repeated" });
        }
        Ok(Self {
            id: None,
            uuid: Uuid::new_v4().to_string(),
            deck_id,
            flashcard_type_id,
            created_at: None,
            updated_at: None,
            fields,
            media,
        })
    }

    pub(super) fn restore(
        id: i64,
        uuid: String,
        deck_id: i64,
        flashcard_type_id: i64,
        created_at: DateTime<Utc>,
        updated_at: Option<DateTime<Utc>>,
        fields: Vec<FieldValue>,
        media: Vec<FieldMedia>,
    ) -> Self {
        Self {
            id: Some(id),
            uuid,
            deck_id,
            flashcard_type_id,
            created_at: Some(created_at),
            updated_at,
            fields,
            media,
        }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn deck_id(&self) -> i64 {
        self.deck_id
    }

    pub fn flashcard_type_id(&self) -> i64 {
        self.flashcard_type_id
    }

    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }

    pub fn fields(&self) -> &[FieldValue] {
        &self.fields
    }

    pub fn media(&self) -> &[FieldMedia] {
        &self.media
    }
}

/// An unsaved flashcard being edited. `values` maps a field key to what was typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlashcardDraft {
    id: Option<i64>,
    uuid: String,
    deck_id: i64,
    flashcard_type_id: i64,
    position: i64,
    values: BTreeMap<String, String>,
    /// Attached media by field key.
    media: BTreeMap<String, Vec<MediaAttachment>>,
    updated_at: Option<DateTime<Utc>>,
}

impl FlashcardDraft {
    /// An empty, unsaved draft with a fresh uuid.
    pub fn new(deck_id: i64, flashcard_type_id: i64) -> Self {
        Self {
            id: None,
            uuid: Uuid::new_v4().to_string(),
            deck_id,
            flashcard_type_id,
            position: 0,
            values: BTreeMap::new(),
            media: BTreeMap::new(),
            updated_at: None,
        }
    }

    pub(super) fn restore(
        id: i64,
        uuid: String,
        deck_id: i64,
        flashcard_type_id: i64,
        position: i64,
        values: BTreeMap<String, String>,
        media: BTreeMap<String, Vec<MediaAttachment>>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            uuid,
            deck_id,
            flashcard_type_id,
            position,
            values,
            media,
            updated_at: Some(updated_at),
        }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn deck_id(&self) -> i64 {
        self.deck_id
    }

    pub fn flashcard_type_id(&self) -> i64 {
        self.flashcard_type_id
    }

    pub fn position(&self) -> i64 {
        self.position
    }

    pub fn values(&self) -> &BTreeMap<String, String> {
        &self.values
    }

    pub fn media(&self) -> &BTreeMap<String, Vec<MediaAttachment>> {
        &self.media
    }

    pub(super) fn with_media(mut self, media: BTreeMap<String, Vec<MediaAttachment>>) -> Self {
        self.media = media;
        self
    }

    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }

    /// Replaces all the values. Nothing changes if one of them is invalid.
    pub fn set_values(&mut self, values: BTreeMap<String, String>) -> FlashcardsResult<()> {
        for (key, content) in &values {
            if key.is_empty() || key.len() > 50 {
                return Err(FlashcardsError::Invalid { field: "key", reason: "must be 1-50 characters" });
            }
            validate_content(content)?;
        }
        self.values = values;
        Ok(())
    }

    /// An unsaved copy with a fresh uuid and the same values.
    pub fn duplicate(&self) -> Self {
        Self {
            id: None,
            uuid: Uuid::new_v4().to_string(),
            deck_id: self.deck_id,
            flashcard_type_id: self.flashcard_type_id,
            position: self.position + 1,
            values: self.values.clone(),
            media: self.media.clone(),
            updated_at: None,
        }
    }
}

/// Which way a draft moves in its list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

/// How many flashcards of a type a deck has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeCount {
    pub flashcard_type_id: i64,
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flashcard_cannot_repeat_a_field() {
        let value = |id| FieldValue::new(id, "x".into()).unwrap();
        assert!(Flashcard::new(1, 1, vec![value(1), value(1)], vec![]).is_err());
        let card = Flashcard::new(1, 1, vec![value(1), value(2)], vec![]).unwrap();
        assert_eq!(card.fields().len(), 2);
        assert!(card.id().is_none());
        assert_eq!(card.uuid().len(), 36);
    }

    #[test]
    fn content_length_is_bounded() {
        assert!(FieldValue::new(1, "x".repeat(MAX_CONTENT_LEN)).is_ok());
        assert!(FieldValue::new(1, "x".repeat(MAX_CONTENT_LEN + 1)).is_err());
    }

    #[test]
    fn set_values_validates_and_duplicate_gets_a_new_uuid() {
        let mut draft = FlashcardDraft::new(1, 1);
        let mut values = BTreeMap::new();
        values.insert("phrase".to_string(), "hello".to_string());
        draft.set_values(values.clone()).unwrap();

        let mut bad = BTreeMap::new();
        bad.insert(String::new(), "x".to_string());
        assert!(draft.set_values(bad).is_err());
        assert_eq!(draft.values(), &values, "a failed set_values changes nothing");

        let copy = draft.duplicate();
        assert_ne!(copy.uuid(), draft.uuid());
        assert_eq!(copy.values(), draft.values());
    }
}
