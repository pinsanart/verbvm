use std::sync::Arc;

use rusqlite::types::Type;
use rusqlite::{params, OptionalExtension, Row};

use super::model::{FieldDefinition, FlashcardType, Side};
use crate::dependencies::db::{connection::SqliteHandle, error::DbError};

/// Storage port for flashcard types.
pub trait FlashcardTypeRepository: Send + Sync {
    fn insert(&self, flashcard_type: &FlashcardType) -> Result<FlashcardType, DbError>;
    fn find(&self, id: i64) -> Result<Option<FlashcardType>, DbError>;
    fn find_by_slug(&self, slug: &str) -> Result<Option<FlashcardType>, DbError>;
    fn find_all(&self) -> Result<Vec<FlashcardType>, DbError>;
    fn update(&self, flashcard_type: &FlashcardType) -> Result<bool, DbError>;
    /// Fails while flashcards, drafts or field definitions use the type.
    fn delete(&self, id: i64) -> Result<bool, DbError>;
}

/// Storage port for field definitions.
pub trait FieldDefinitionRepository: Send + Sync {
    fn insert(&self, definition: &FieldDefinition) -> Result<FieldDefinition, DbError>;
    fn find(&self, id: i64) -> Result<Option<FieldDefinition>, DbError>;
    /// The type's definitions that apply to a language: the language-agnostic ones plus, when
    /// `language_id` is given, the ones limited to it. Ordered front before back, then by
    /// position. A key can appear twice (an agnostic and a language-specific definition).
    fn find_for_type(
        &self,
        flashcard_type_id: i64,
        language_id: Option<i64>,
    ) -> Result<Vec<FieldDefinition>, DbError>;
    fn update(&self, definition: &FieldDefinition) -> Result<bool, DbError>;
    /// Fails while flashcard fields use the definition.
    fn delete(&self, id: i64) -> Result<bool, DbError>;
}

const TYPE_COLUMNS: &str = "id, slug, name, description";
const DEFINITION_COLUMNS: &str =
    "id, flashcard_type_id, language_id, side, key, label, position, is_required, is_rich, media_allowed";

fn type_from_row(row: &Row) -> rusqlite::Result<FlashcardType> {
    Ok(FlashcardType::restore(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
}

fn definition_from_row(row: &Row) -> rusqlite::Result<FieldDefinition> {
    let side: String = row.get(3)?;
    let side = Side::parse(&side).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(3, Type::Text, e.to_string().into())
    })?;
    Ok(FieldDefinition::restore(
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        side,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
        row.get(9)?,
    ))
}

pub struct SqliteFlashcardTypeRepository {
    db: Arc<SqliteHandle>,
}

impl SqliteFlashcardTypeRepository {
    pub fn new(db: Arc<SqliteHandle>) -> Self {
        Self { db }
    }
}

impl FlashcardTypeRepository for SqliteFlashcardTypeRepository {
    fn insert(&self, t: &FlashcardType) -> Result<FlashcardType, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!(
                    "INSERT INTO flashcard_types (slug, name, description) VALUES (?1, ?2, ?3)
                     RETURNING {TYPE_COLUMNS}"
                ),
                params![t.slug(), t.name(), t.description()],
                type_from_row,
            )
        })
    }

    fn find(&self, id: i64) -> Result<Option<FlashcardType>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!("SELECT {TYPE_COLUMNS} FROM flashcard_types WHERE id = ?1"),
                [id],
                type_from_row,
            )
            .optional()
        })
    }

    fn find_by_slug(&self, slug: &str) -> Result<Option<FlashcardType>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!("SELECT {TYPE_COLUMNS} FROM flashcard_types WHERE slug = ?1"),
                [slug],
                type_from_row,
            )
            .optional()
        })
    }

    fn find_all(&self) -> Result<Vec<FlashcardType>, DbError> {
        self.db.with_connection(|conn| {
            conn.prepare(&format!("SELECT {TYPE_COLUMNS} FROM flashcard_types ORDER BY id"))?
                .query_map([], type_from_row)?
                .collect()
        })
    }

    fn update(&self, t: &FlashcardType) -> Result<bool, DbError> {
        let Some(id) = t.id() else { return Ok(false) };
        self.db.with_connection(|conn| {
            let rows = conn.execute(
                "UPDATE flashcard_types SET slug = ?1, name = ?2, description = ?3 WHERE id = ?4",
                params![t.slug(), t.name(), t.description(), id],
            )?;
            Ok(rows > 0)
        })
    }

    fn delete(&self, id: i64) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            Ok(conn.execute("DELETE FROM flashcard_types WHERE id = ?1", [id])? > 0)
        })
    }
}

pub struct SqliteFieldDefinitionRepository {
    db: Arc<SqliteHandle>,
}

impl SqliteFieldDefinitionRepository {
    pub fn new(db: Arc<SqliteHandle>) -> Self {
        Self { db }
    }
}

impl FieldDefinitionRepository for SqliteFieldDefinitionRepository {
    fn insert(&self, d: &FieldDefinition) -> Result<FieldDefinition, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!(
                    "INSERT INTO field_definitions
                         (flashcard_type_id, language_id, side, key, label, position, is_required, is_rich, media_allowed)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) RETURNING {DEFINITION_COLUMNS}"
                ),
                params![
                    d.flashcard_type_id(),
                    d.language_id(),
                    d.side().as_str(),
                    d.key(),
                    d.label(),
                    d.position(),
                    d.is_required(),
                    d.is_rich(),
                    d.media_allowed(),
                ],
                definition_from_row,
            )
        })
    }

    fn find(&self, id: i64) -> Result<Option<FieldDefinition>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!("SELECT {DEFINITION_COLUMNS} FROM field_definitions WHERE id = ?1"),
                [id],
                definition_from_row,
            )
            .optional()
        })
    }

    fn find_for_type(
        &self,
        flashcard_type_id: i64,
        language_id: Option<i64>,
    ) -> Result<Vec<FieldDefinition>, DbError> {
        self.db.with_connection(|conn| {
            conn.prepare(&format!(
                "SELECT {DEFINITION_COLUMNS} FROM field_definitions
                 WHERE flashcard_type_id = ?1 AND (language_id IS NULL OR language_id = ?2)
                 ORDER BY CASE side WHEN 'FRONT' THEN 0 ELSE 1 END, position, id"
            ))?
            .query_map(params![flashcard_type_id, language_id], definition_from_row)?
            .collect()
        })
    }

    fn update(&self, d: &FieldDefinition) -> Result<bool, DbError> {
        let Some(id) = d.id() else { return Ok(false) };
        self.db.with_connection(|conn| {
            let rows = conn.execute(
                "UPDATE field_definitions
                 SET language_id = ?1, side = ?2, key = ?3, label = ?4, position = ?5,
                     is_required = ?6, is_rich = ?7, media_allowed = ?8
                 WHERE id = ?9",
                params![
                    d.language_id(),
                    d.side().as_str(),
                    d.key(),
                    d.label(),
                    d.position(),
                    d.is_required(),
                    d.is_rich(),
                    d.media_allowed(),
                    id
                ],
            )?;
            Ok(rows > 0)
        })
    }

    fn delete(&self, id: i64) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            Ok(conn.execute("DELETE FROM field_definitions WHERE id = ?1", [id])? > 0)
        })
    }
}
