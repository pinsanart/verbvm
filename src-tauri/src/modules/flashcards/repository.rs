use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use rusqlite::types::Type;
use rusqlite::{params, OptionalExtension, Row, Transaction};

use super::model::{FieldValue, Flashcard, FlashcardDraft, TypeCount};
use crate::dependencies::db::sql_time::{optional_utc_column, utc_column};
use crate::dependencies::db::{connection::SqliteHandle, error::DbError};

/// Storage port for the module's tables (`flashcards`, `flashcard_fields`, `flashcard_drafts`).
///
/// They share one trait because publishing writes cards, fields and drafts in one transaction.
pub trait FlashcardRepository: Send + Sync {
    /// Stores an unsaved flashcard and its field values.
    fn insert_flashcard(&self, flashcard: &Flashcard) -> Result<Flashcard, DbError>;
    fn find_flashcard(&self, uuid: &str) -> Result<Option<Flashcard>, DbError>;
    /// The deck's flashcards with their values, oldest first.
    fn find_flashcards_by_deck(&self, deck_id: i64) -> Result<Vec<Flashcard>, DbError>;
    /// Cascades to the flashcard's field values.
    fn delete_flashcard(&self, uuid: &str) -> Result<bool, DbError>;
    fn count_by_type(&self, deck_id: i64) -> Result<Vec<TypeCount>, DbError>;

    /// Stores an unsaved draft at the end of its list (deck and type).
    fn insert_draft(&self, draft: &FlashcardDraft) -> Result<FlashcardDraft, DbError>;
    /// Stores an unsaved draft right after the draft `source_uuid`, moving the later ones down.
    /// Returns `None` when the source does not exist.
    fn insert_draft_after(
        &self,
        source_uuid: &str,
        draft: &FlashcardDraft,
    ) -> Result<Option<FlashcardDraft>, DbError>;
    fn find_draft(&self, uuid: &str) -> Result<Option<FlashcardDraft>, DbError>;
    /// The drafts of a deck and type, by position.
    fn find_drafts(&self, deck_id: i64, flashcard_type_id: i64) -> Result<Vec<FlashcardDraft>, DbError>;
    fn update_draft_values(
        &self,
        uuid: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<bool, DbError>;
    /// Exchanges the positions of two drafts. Returns `false` if either does not exist.
    fn swap_draft_positions(&self, uuid_a: &str, uuid_b: &str) -> Result<bool, DbError>;
    fn delete_draft(&self, uuid: &str) -> Result<bool, DbError>;

    /// Stores the flashcards and deletes the drafts they came from, all or nothing.
    fn publish(
        &self,
        flashcards: &[Flashcard],
        draft_uuids: &[String],
    ) -> Result<Vec<Flashcard>, DbError>;
}

const CARD_COLUMNS: &str = "id, uuid, deck_id, flashcard_type_id, created_at, updated_at";
const DRAFT_COLUMNS: &str =
    "id, uuid, deck_id, flashcard_type_id, position, values_json, updated_at";

fn conversion_error(idx: usize, message: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(idx, Type::Text, message.into())
}

fn card_from_row(row: &Row, fields: Vec<FieldValue>) -> rusqlite::Result<Flashcard> {
    Ok(Flashcard::restore(
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        utc_column(row, 4)?,
        optional_utc_column(row, 5)?,
        fields,
    ))
}

fn draft_from_row(row: &Row) -> rusqlite::Result<FlashcardDraft> {
    let json: String = row.get(5)?;
    let values: BTreeMap<String, String> = serde_json::from_str(&json)
        .map_err(|e| conversion_error(5, format!("invalid draft values: {e}")))?;
    Ok(FlashcardDraft::restore(
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        values,
        utc_column(row, 6)?,
    ))
}

fn values_to_json(values: &BTreeMap<String, String>) -> rusqlite::Result<String> {
    serde_json::to_string(values)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
}

fn load_fields(conn: &rusqlite::Connection, flashcard_id: i64) -> rusqlite::Result<Vec<FieldValue>> {
    conn.prepare(
        "SELECT field_definition_id, COALESCE(content, '') FROM flashcard_fields
         WHERE flashcard_id = ?1 ORDER BY id",
    )?
    .query_map([flashcard_id], |row| {
        FieldValue::new(row.get(0)?, row.get(1)?)
            .map_err(|e| conversion_error(1, e.to_string()))
    })?
    .collect()
}

fn insert_flashcard_in(tx: &Transaction, flashcard: &Flashcard) -> rusqlite::Result<Flashcard> {
    let (id, uuid, deck_id, type_id, created, updated) = tx.query_row(
        &format!(
            "INSERT INTO flashcards (uuid, deck_id, flashcard_type_id) VALUES (?1, ?2, ?3)
             RETURNING {CARD_COLUMNS}"
        ),
        params![flashcard.uuid(), flashcard.deck_id(), flashcard.flashcard_type_id()],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                utc_column(row, 4)?,
                optional_utc_column(row, 5)?,
            ))
        },
    )?;
    for field in flashcard.fields() {
        tx.execute(
            "INSERT INTO flashcard_fields (flashcard_id, field_definition_id, content) VALUES (?1, ?2, ?3)",
            params![id, field.field_definition_id(), field.content()],
        )?;
    }
    Ok(Flashcard::restore(id, uuid, deck_id, type_id, created, updated, flashcard.fields().to_vec()))
}

pub struct SqliteFlashcardRepository {
    db: Arc<SqliteHandle>,
}

impl SqliteFlashcardRepository {
    pub fn new(db: Arc<SqliteHandle>) -> Self {
        Self { db }
    }
}

impl FlashcardRepository for SqliteFlashcardRepository {
    fn insert_flashcard(&self, flashcard: &Flashcard) -> Result<Flashcard, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let saved = insert_flashcard_in(&tx, flashcard)?;
            tx.commit()?;
            Ok(saved)
        })
    }

    fn find_flashcard(&self, uuid: &str) -> Result<Option<Flashcard>, DbError> {
        self.db.with_connection(|conn| {
            let card = conn
                .query_row(
                    &format!("SELECT {CARD_COLUMNS} FROM flashcards WHERE uuid = ?1"),
                    [uuid],
                    |row| card_from_row(row, Vec::new()),
                )
                .optional()?;
            match card {
                Some(card) => {
                    let id = card.id().expect("restored flashcards have an id");
                    let fields = load_fields(conn, id)?;
                    Ok(Some(Flashcard::restore(
                        id,
                        card.uuid().to_owned(),
                        card.deck_id(),
                        card.flashcard_type_id(),
                        card.created_at().expect("restored flashcards have a creation time"),
                        card.updated_at(),
                        fields,
                    )))
                }
                None => Ok(None),
            }
        })
    }

    fn find_flashcards_by_deck(&self, deck_id: i64) -> Result<Vec<Flashcard>, DbError> {
        self.db.with_connection(|conn| {
            let mut by_card: HashMap<i64, Vec<FieldValue>> = HashMap::new();
            let mut stmt = conn.prepare(
                "SELECT ff.flashcard_id, ff.field_definition_id, COALESCE(ff.content, '')
                 FROM flashcard_fields ff JOIN flashcards f ON f.id = ff.flashcard_id
                 WHERE f.deck_id = ?1 ORDER BY ff.id",
            )?;
            let rows = stmt.query_map([deck_id], |row| {
                let value = FieldValue::new(row.get(1)?, row.get(2)?)
                    .map_err(|e| conversion_error(2, e.to_string()))?;
                Ok((row.get::<_, i64>(0)?, value))
            })?;
            for row in rows {
                let (card_id, value) = row?;
                by_card.entry(card_id).or_default().push(value);
            }

            conn.prepare(&format!(
                "SELECT {CARD_COLUMNS} FROM flashcards WHERE deck_id = ?1 ORDER BY created_at, id"
            ))?
            .query_map([deck_id], |row| {
                let id: i64 = row.get(0)?;
                card_from_row(row, by_card.remove(&id).unwrap_or_default())
            })?
            .collect()
        })
    }

    fn delete_flashcard(&self, uuid: &str) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            Ok(conn.execute("DELETE FROM flashcards WHERE uuid = ?1", [uuid])? > 0)
        })
    }

    fn count_by_type(&self, deck_id: i64) -> Result<Vec<TypeCount>, DbError> {
        self.db.with_connection(|conn| {
            conn.prepare(
                "SELECT flashcard_type_id, COUNT(*) FROM flashcards
                 WHERE deck_id = ?1 GROUP BY flashcard_type_id ORDER BY flashcard_type_id",
            )?
            .query_map([deck_id], |row| {
                Ok(TypeCount { flashcard_type_id: row.get(0)?, count: row.get(1)? })
            })?
            .collect()
        })
    }

    fn insert_draft(&self, draft: &FlashcardDraft) -> Result<FlashcardDraft, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!(
                    "INSERT INTO flashcard_drafts (uuid, deck_id, flashcard_type_id, position, values_json)
                     VALUES (?1, ?2, ?3,
                             (SELECT COALESCE(MAX(position) + 1, 0) FROM flashcard_drafts
                              WHERE deck_id = ?2 AND flashcard_type_id = ?3),
                             ?4)
                     RETURNING {DRAFT_COLUMNS}"
                ),
                params![
                    draft.uuid(),
                    draft.deck_id(),
                    draft.flashcard_type_id(),
                    values_to_json(draft.values())?
                ],
                draft_from_row,
            )
        })
    }

    fn insert_draft_after(
        &self,
        source_uuid: &str,
        draft: &FlashcardDraft,
    ) -> Result<Option<FlashcardDraft>, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let source: Option<(i64, i64, i64)> = tx
                .query_row(
                    "SELECT deck_id, flashcard_type_id, position FROM flashcard_drafts WHERE uuid = ?1",
                    [source_uuid],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()?;
            let Some((deck_id, type_id, position)) = source else { return Ok(None) };

            tx.execute(
                "UPDATE flashcard_drafts SET position = position + 1
                 WHERE deck_id = ?1 AND flashcard_type_id = ?2 AND position > ?3",
                params![deck_id, type_id, position],
            )?;
            let saved = tx.query_row(
                &format!(
                    "INSERT INTO flashcard_drafts (uuid, deck_id, flashcard_type_id, position, values_json)
                     VALUES (?1, ?2, ?3, ?4, ?5) RETURNING {DRAFT_COLUMNS}"
                ),
                params![draft.uuid(), deck_id, type_id, position + 1, values_to_json(draft.values())?],
                draft_from_row,
            )?;
            tx.commit()?;
            Ok(Some(saved))
        })
    }

    fn find_draft(&self, uuid: &str) -> Result<Option<FlashcardDraft>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!("SELECT {DRAFT_COLUMNS} FROM flashcard_drafts WHERE uuid = ?1"),
                [uuid],
                draft_from_row,
            )
            .optional()
        })
    }

    fn find_drafts(&self, deck_id: i64, flashcard_type_id: i64) -> Result<Vec<FlashcardDraft>, DbError> {
        self.db.with_connection(|conn| {
            conn.prepare(&format!(
                "SELECT {DRAFT_COLUMNS} FROM flashcard_drafts
                 WHERE deck_id = ?1 AND flashcard_type_id = ?2 ORDER BY position, id"
            ))?
            .query_map(params![deck_id, flashcard_type_id], draft_from_row)?
            .collect()
        })
    }

    fn update_draft_values(
        &self,
        uuid: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            let rows = conn.execute(
                "UPDATE flashcard_drafts SET values_json = ?1, updated_at = datetime('now') WHERE uuid = ?2",
                params![values_to_json(values)?, uuid],
            )?;
            Ok(rows > 0)
        })
    }

    fn swap_draft_positions(&self, uuid_a: &str, uuid_b: &str) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let position = |uuid: &str| -> rusqlite::Result<Option<i64>> {
                tx.query_row("SELECT position FROM flashcard_drafts WHERE uuid = ?1", [uuid], |r| r.get(0))
                    .optional()
            };
            let (Some(a), Some(b)) = (position(uuid_a)?, position(uuid_b)?) else {
                return Ok(false);
            };
            tx.execute("UPDATE flashcard_drafts SET position = ?1 WHERE uuid = ?2", params![b, uuid_a])?;
            tx.execute("UPDATE flashcard_drafts SET position = ?1 WHERE uuid = ?2", params![a, uuid_b])?;
            tx.commit()?;
            Ok(true)
        })
    }

    fn delete_draft(&self, uuid: &str) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            Ok(conn.execute("DELETE FROM flashcard_drafts WHERE uuid = ?1", [uuid])? > 0)
        })
    }

    fn publish(
        &self,
        flashcards: &[Flashcard],
        draft_uuids: &[String],
    ) -> Result<Vec<Flashcard>, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let mut saved = Vec::with_capacity(flashcards.len());
            for flashcard in flashcards {
                saved.push(insert_flashcard_in(&tx, flashcard)?);
            }
            for uuid in draft_uuids {
                tx.execute("DELETE FROM flashcard_drafts WHERE uuid = ?1", [uuid])?;
            }
            tx.commit()?;
            Ok(saved)
        })
    }
}
