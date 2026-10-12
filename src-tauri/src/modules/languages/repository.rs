use std::sync::Arc;

use rusqlite::{params, OptionalExtension, Row};

use super::model::{Language, LanguageVariant};
use crate::dependencies::db::{connection::SqliteHandle, error::DbError};

/// Storage port for languages. `Send + Sync` so services can live in Tauri state.
pub trait LanguageRepository: Send + Sync {
    /// Stores an unsaved language and returns it with its id.
    fn insert(&self, language: &Language) -> Result<Language, DbError>;
    fn find(&self, id: i64) -> Result<Option<Language>, DbError>;
    fn find_by_iso_639_3(&self, iso_639_3: &str) -> Result<Option<Language>, DbError>;
    fn find_all(&self) -> Result<Vec<Language>, DbError>;
    /// Writes every editable column. Returns `false` when the language is unsaved or gone.
    fn update(&self, language: &Language) -> Result<bool, DbError>;
    /// Cascades to the language's variants.
    fn delete(&self, id: i64) -> Result<bool, DbError>;
}

/// Storage port for language variants.
pub trait LanguageVariantRepository: Send + Sync {
    fn insert(&self, variant: &LanguageVariant) -> Result<LanguageVariant, DbError>;
    fn find(&self, id: i64) -> Result<Option<LanguageVariant>, DbError>;
    fn find_by_bcp47(&self, bcp47_code: &str) -> Result<Option<LanguageVariant>, DbError>;
    fn find_by_language(&self, language_id: i64) -> Result<Vec<LanguageVariant>, DbError>;
    fn update(&self, variant: &LanguageVariant) -> Result<bool, DbError>;
    fn delete(&self, id: i64) -> Result<bool, DbError>;
}

const LANGUAGE_COLUMNS: &str = "id, iso_639_1, iso_639_3, name, default_variant_id";
const VARIANT_COLUMNS: &str = "id, language_id, bcp47_code, variant_name, country_code";

fn language_from_row(row: &Row) -> rusqlite::Result<Language> {
    Ok(Language::restore(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
}

fn variant_from_row(row: &Row) -> rusqlite::Result<LanguageVariant> {
    Ok(LanguageVariant::restore(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
}

pub struct SqliteLanguageRepository {
    db: Arc<SqliteHandle>,
}

impl SqliteLanguageRepository {
    pub fn new(db: Arc<SqliteHandle>) -> Self {
        Self { db }
    }
}

impl LanguageRepository for SqliteLanguageRepository {
    fn insert(&self, language: &Language) -> Result<Language, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!(
                    "INSERT INTO languages (iso_639_1, iso_639_3, name, default_variant_id)
                     VALUES (?1, ?2, ?3, ?4) RETURNING {LANGUAGE_COLUMNS}"
                ),
                params![
                    language.iso_639_1(),
                    language.iso_639_3(),
                    language.name(),
                    language.default_variant_id()
                ],
                language_from_row,
            )
        })
    }

    fn find(&self, id: i64) -> Result<Option<Language>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!("SELECT {LANGUAGE_COLUMNS} FROM languages WHERE id = ?1"),
                [id],
                language_from_row,
            )
            .optional()
        })
    }

    fn find_by_iso_639_3(&self, iso_639_3: &str) -> Result<Option<Language>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!("SELECT {LANGUAGE_COLUMNS} FROM languages WHERE iso_639_3 = ?1"),
                [iso_639_3],
                language_from_row,
            )
            .optional()
        })
    }

    fn find_all(&self) -> Result<Vec<Language>, DbError> {
        self.db.with_connection(|conn| {
            conn.prepare(&format!("SELECT {LANGUAGE_COLUMNS} FROM languages ORDER BY name"))?
                .query_map([], language_from_row)?
                .collect()
        })
    }

    fn update(&self, language: &Language) -> Result<bool, DbError> {
        let Some(id) = language.id() else { return Ok(false) };
        self.db.with_connection(|conn| {
            let rows = conn.execute(
                "UPDATE languages
                 SET iso_639_1 = ?1, iso_639_3 = ?2, name = ?3, default_variant_id = ?4
                 WHERE id = ?5",
                params![
                    language.iso_639_1(),
                    language.iso_639_3(),
                    language.name(),
                    language.default_variant_id(),
                    id
                ],
            )?;
            Ok(rows > 0)
        })
    }

    fn delete(&self, id: i64) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            Ok(conn.execute("DELETE FROM languages WHERE id = ?1", [id])? > 0)
        })
    }
}

pub struct SqliteLanguageVariantRepository {
    db: Arc<SqliteHandle>,
}

impl SqliteLanguageVariantRepository {
    pub fn new(db: Arc<SqliteHandle>) -> Self {
        Self { db }
    }
}

impl LanguageVariantRepository for SqliteLanguageVariantRepository {
    fn insert(&self, variant: &LanguageVariant) -> Result<LanguageVariant, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!(
                    "INSERT INTO language_variants (language_id, bcp47_code, variant_name, country_code)
                     VALUES (?1, ?2, ?3, ?4) RETURNING {VARIANT_COLUMNS}"
                ),
                params![
                    variant.language_id(),
                    variant.bcp47_code(),
                    variant.variant_name(),
                    variant.country_code()
                ],
                variant_from_row,
            )
        })
    }

    fn find(&self, id: i64) -> Result<Option<LanguageVariant>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!("SELECT {VARIANT_COLUMNS} FROM language_variants WHERE id = ?1"),
                [id],
                variant_from_row,
            )
            .optional()
        })
    }

    fn find_by_bcp47(&self, bcp47_code: &str) -> Result<Option<LanguageVariant>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!("SELECT {VARIANT_COLUMNS} FROM language_variants WHERE bcp47_code = ?1"),
                [bcp47_code],
                variant_from_row,
            )
            .optional()
        })
    }

    fn find_by_language(&self, language_id: i64) -> Result<Vec<LanguageVariant>, DbError> {
        self.db.with_connection(|conn| {
            conn.prepare(&format!(
                "SELECT {VARIANT_COLUMNS} FROM language_variants
                 WHERE language_id = ?1 ORDER BY variant_name"
            ))?
            .query_map([language_id], variant_from_row)?
            .collect()
        })
    }

    fn update(&self, variant: &LanguageVariant) -> Result<bool, DbError> {
        let Some(id) = variant.id() else { return Ok(false) };
        self.db.with_connection(|conn| {
            let rows = conn.execute(
                "UPDATE language_variants
                 SET bcp47_code = ?1, variant_name = ?2, country_code = ?3 WHERE id = ?4",
                params![variant.bcp47_code(), variant.variant_name(), variant.country_code(), id],
            )?;
            Ok(rows > 0)
        })
    }

    /// Fails while a deck uses the variant. Clears `languages.default_variant_id` if it pointed here.
    fn delete(&self, id: i64) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            Ok(conn.execute("DELETE FROM language_variants WHERE id = ?1", [id])? > 0)
        })
    }
}
