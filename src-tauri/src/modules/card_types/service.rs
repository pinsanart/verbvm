use std::sync::Arc;

use super::error::{CardTypesError, CardTypesResult};
use super::model::{FieldDefinition, FlashcardType, Side};
use super::repository::{
    FieldDefinitionRepository, FlashcardTypeRepository, SqliteFieldDefinitionRepository,
    SqliteFlashcardTypeRepository,
};
use crate::dependencies::db::connection::SqliteHandle;

/// Public API of the card types module. Other modules depend on this trait only.
///
/// Types are identified by slug outside the backend. `get_type` takes an id and exists for
/// modules that hold a foreign key.
pub trait CardTypeApi: Send + Sync {
    fn list_types(&self) -> CardTypesResult<Vec<FlashcardType>>;
    fn get_type(&self, id: i64) -> CardTypesResult<FlashcardType>;
    fn get_type_by_slug(&self, slug: &str) -> CardTypesResult<FlashcardType>;
    /// Conflicts when the slug is already taken.
    fn create_type(
        &self,
        slug: String,
        name: String,
        description: Option<String>,
    ) -> CardTypesResult<FlashcardType>;

    /// The fields a flashcard of this type has in a deck of the given language, front before
    /// back and by position. A field limited to the language replaces the language-agnostic
    /// field with the same key.
    fn fields_for(
        &self,
        type_slug: &str,
        language_id: Option<i64>,
    ) -> CardTypesResult<Vec<FieldDefinition>>;

    /// Adds a field to a type, for every language (`language_id` `None`) or for one. Conflicts
    /// when the type already has a field with that key for the same scope.
    #[allow(clippy::too_many_arguments)]
    fn add_field_definition(
        &self,
        type_slug: &str,
        language_id: Option<i64>,
        side: Side,
        key: String,
        label: String,
        position: i64,
        is_required: bool,
        is_rich: bool,
        media_allowed: bool,
    ) -> CardTypesResult<FieldDefinition>;
}

pub struct CardTypeService {
    types: Arc<dyn FlashcardTypeRepository>,
    definitions: Arc<dyn FieldDefinitionRepository>,
}

impl CardTypeService {
    pub fn new(
        types: Arc<dyn FlashcardTypeRepository>,
        definitions: Arc<dyn FieldDefinitionRepository>,
    ) -> Self {
        Self { types, definitions }
    }

    /// Service backed by the SQLite repositories.
    pub fn sqlite(db: Arc<SqliteHandle>) -> Self {
        Self::new(
            Arc::new(SqliteFlashcardTypeRepository::new(db.clone())),
            Arc::new(SqliteFieldDefinitionRepository::new(db)),
        )
    }
}

impl CardTypeApi for CardTypeService {
    fn list_types(&self) -> CardTypesResult<Vec<FlashcardType>> {
        Ok(self.types.find_all()?)
    }

    fn get_type(&self, id: i64) -> CardTypesResult<FlashcardType> {
        self.types.find(id)?.ok_or(CardTypesError::NotFound("flashcard type"))
    }

    fn get_type_by_slug(&self, slug: &str) -> CardTypesResult<FlashcardType> {
        self.types.find_by_slug(slug)?.ok_or(CardTypesError::NotFound("flashcard type"))
    }

    fn create_type(
        &self,
        slug: String,
        name: String,
        description: Option<String>,
    ) -> CardTypesResult<FlashcardType> {
        let flashcard_type = FlashcardType::new(slug, name, description)?;
        Ok(self.types.insert(&flashcard_type)?)
    }

    fn fields_for(
        &self,
        type_slug: &str,
        language_id: Option<i64>,
    ) -> CardTypesResult<Vec<FieldDefinition>> {
        let flashcard_type = self.get_type_by_slug(type_slug)?;
        let type_id = flashcard_type.id().ok_or(CardTypesError::NotFound("flashcard type"))?;
        let found = self.definitions.find_for_type(type_id, language_id)?;

        // A language-specific definition replaces the agnostic one with the same key, keeping
        // the position of the one it replaces.
        let mut fields: Vec<FieldDefinition> = Vec::with_capacity(found.len());
        for definition in found {
            match fields.iter().position(|f| f.key() == definition.key()) {
                Some(index) if definition.language_id().is_some() => fields[index] = definition,
                Some(_) => {}
                None => fields.push(definition),
            }
        }
        Ok(fields)
    }

    fn add_field_definition(
        &self,
        type_slug: &str,
        language_id: Option<i64>,
        side: Side,
        key: String,
        label: String,
        position: i64,
        is_required: bool,
        is_rich: bool,
        media_allowed: bool,
    ) -> CardTypesResult<FieldDefinition> {
        let flashcard_type = self.get_type_by_slug(type_slug)?;
        let type_id = flashcard_type.id().ok_or(CardTypesError::NotFound("flashcard type"))?;
        let definition = FieldDefinition::new(
            type_id,
            language_id,
            side,
            key,
            label,
            position,
            is_required,
            is_rich,
            media_allowed,
        )?;
        Ok(self.definitions.insert(&definition)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> (CardTypeService, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "verbvm-card-types-{}-{:?}.db",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_file(&path);
        let db = Arc::new(SqliteHandle::new(&path).unwrap());
        (CardTypeService::sqlite(db), path)
    }

    fn keys(fields: &[FieldDefinition]) -> Vec<(&str, &str)> {
        fields.iter().map(|f| (f.side().as_str(), f.key())).collect()
    }

    #[test]
    fn migration_seeds_the_three_types() {
        let (svc, path) = service();
        let slugs: Vec<_> = svc.list_types().unwrap().iter().map(|t| t.slug().to_owned()).collect();
        assert_eq!(slugs, vec!["vocabulary", "phonetic", "orthographic"]);
        assert!(matches!(svc.get_type_by_slug("nope"), Err(CardTypesError::NotFound(_))));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn vocabulary_fields_come_front_first_with_the_seeded_flags() {
        let (svc, path) = service();
        let fields = svc.fields_for("vocabulary", None).unwrap();
        assert_eq!(
            keys(&fields),
            vec![("FRONT", "phrase"), ("BACK", "word_or_expression"), ("BACK", "meaning")]
        );
        let phrase = &fields[0];
        assert!(phrase.is_rich() && phrase.media_allowed() && phrase.is_required());
        assert!(!fields[1].is_rich() && !fields[1].media_allowed());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn language_specific_fields_are_added_only_for_that_language() {
        let (svc, path) = service();
        // Japanese is seeded as language id 7 (en, pt, es, fr, it, de, ja, ...).
        let japanese = 7;
        svc.add_field_definition("vocabulary", Some(japanese), Side::Back, "romaji".into(), "Romaji".into(), 2, false, false, false)
            .unwrap();

        let japanese_fields = svc.fields_for("vocabulary", Some(japanese)).unwrap();
        assert_eq!(japanese_fields.len(), 4);
        assert_eq!(japanese_fields[3].key(), "romaji");
        assert_eq!(svc.fields_for("vocabulary", Some(1)).unwrap().len(), 3);
        assert_eq!(svc.fields_for("vocabulary", None).unwrap().len(), 3);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_language_field_replaces_the_agnostic_one_with_the_same_key() {
        let (svc, path) = service();
        svc.add_field_definition("vocabulary", Some(7), Side::Back, "meaning".into(), "Meaning (English)".into(), 1, true, true, false)
            .unwrap();
        let fields = svc.fields_for("vocabulary", Some(7)).unwrap();
        assert_eq!(fields.len(), 3);
        let meaning = fields.iter().find(|f| f.key() == "meaning").unwrap();
        assert_eq!(meaning.label(), "Meaning (English)");
        assert!(meaning.is_rich());
        // Other languages keep the agnostic definition.
        let other = svc.fields_for("vocabulary", Some(1)).unwrap();
        assert_eq!(other.iter().find(|f| f.key() == "meaning").unwrap().label(), "Meaning");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn duplicate_keys_and_slugs_conflict() {
        let (svc, path) = service();
        assert!(matches!(
            svc.add_field_definition("vocabulary", None, Side::Back, "meaning".into(), "Again".into(), 5, false, false, false),
            Err(CardTypesError::Conflict(_))
        ));
        assert!(matches!(
            svc.create_type("vocabulary".into(), "Vocab".into(), None),
            Err(CardTypesError::Conflict(_))
        ));
        let created = svc.create_type("listening".into(), "Listening".into(), None).unwrap();
        assert!(created.id().is_some());
        let _ = std::fs::remove_file(path);
    }
}
