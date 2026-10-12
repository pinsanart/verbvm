use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use rusqlite::types::Type;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row, Transaction};

use super::model::{FieldMedia, FieldValue, Flashcard, FlashcardDraft, MediaAttachment, TypeCount};
use crate::dependencies::db::sql_time::{optional_utc_column, utc_column};
use crate::dependencies::media_store::MediaKind;
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

    /// Stores an unsaved draft at the end of its deck's list.
    fn insert_draft(&self, draft: &FlashcardDraft) -> Result<FlashcardDraft, DbError>;
    /// Stores an unsaved draft right after the draft `source_uuid`, moving the later ones down.
    /// Returns `None` when the source does not exist.
    fn insert_draft_after(
        &self,
        source_uuid: &str,
        draft: &FlashcardDraft,
    ) -> Result<Option<FlashcardDraft>, DbError>;
    fn find_draft(&self, uuid: &str) -> Result<Option<FlashcardDraft>, DbError>;
    /// The drafts of a deck, of every type, by position.
    fn find_drafts(&self, deck_id: i64) -> Result<Vec<FlashcardDraft>, DbError>;
    fn update_draft_values(
        &self,
        uuid: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<bool, DbError>;
    /// Exchanges the positions of two drafts. Returns `false` if either does not exist.
    fn swap_draft_positions(&self, uuid_a: &str, uuid_b: &str) -> Result<bool, DbError>;
    fn delete_draft(&self, uuid: &str) -> Result<bool, DbError>;
    /// Attaches media to a field of a draft, after the media already there. Attaching the same
    /// file to the same field again changes nothing. Returns `false` when the draft does not exist.
    fn add_draft_media(
        &self,
        draft_uuid: &str,
        field_key: &str,
        media: &MediaAttachment,
    ) -> Result<bool, DbError>;
    /// Detaches the file with this hash from a field of a draft. Returns `false` when it was not
    /// attached.
    fn remove_draft_media(
        &self,
        draft_uuid: &str,
        field_key: &str,
        sha256: &str,
    ) -> Result<bool, DbError>;

    /// Stores the flashcards (with their field values and media) and deletes the drafts they came
    /// from, all or nothing.
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

const MEDIA_COLUMNS: &str = "kind, mime_type, sha256, file_name, size_bytes, position";

fn placeholders(count: usize) -> String {
    vec!["?"; count].join(", ")
}

/// Reads an attachment from the columns of `MEDIA_COLUMNS`, starting at `offset`.
fn media_from_row(row: &Row, offset: usize) -> rusqlite::Result<MediaAttachment> {
    let kind: String = row.get(offset)?;
    let kind = MediaKind::parse(&kind)
        .ok_or_else(|| conversion_error(offset, format!("invalid media kind: {kind}")))?;
    Ok(MediaAttachment::restore(
        kind,
        row.get(offset + 1)?,
        row.get(offset + 2)?,
        row.get(offset + 3)?,
        row.get(offset + 4)?,
        row.get(offset + 5)?,
    ))
}

type DraftMedia = BTreeMap<String, Vec<MediaAttachment>>;

fn load_draft_media(conn: &Connection, draft_ids: &[i64]) -> rusqlite::Result<HashMap<i64, DraftMedia>> {
    let mut by_draft: HashMap<i64, DraftMedia> = HashMap::new();
    if draft_ids.is_empty() {
        return Ok(by_draft);
    }
    let mut stmt = conn.prepare(&format!(
        "SELECT draft_id, field_key, {MEDIA_COLUMNS} FROM flashcard_draft_media
         WHERE draft_id IN ({}) ORDER BY position, id",
        placeholders(draft_ids.len())
    ))?;
    let rows = stmt.query_map(params_from_iter(draft_ids), |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, media_from_row(row, 2)?))
    })?;
    for row in rows {
        let (draft_id, field_key, media) = row?;
        by_draft.entry(draft_id).or_default().entry(field_key).or_default().push(media);
    }
    Ok(by_draft)
}

fn load_card_media(conn: &Connection, card_ids: &[i64]) -> rusqlite::Result<HashMap<i64, Vec<FieldMedia>>> {
    let mut by_card: HashMap<i64, Vec<FieldMedia>> = HashMap::new();
    if card_ids.is_empty() {
        return Ok(by_card);
    }
    let mut stmt = conn.prepare(&format!(
        "SELECT ff.flashcard_id, ff.field_definition_id,
                fm.kind, fm.mime_type, fm.sha256, fm.file_name, fm.size_bytes, fm.position
         FROM field_media fm JOIN flashcard_fields ff ON ff.id = fm.flashcard_field_id
         WHERE ff.flashcard_id IN ({}) ORDER BY fm.position, fm.id",
        placeholders(card_ids.len())
    ))?;
    let rows = stmt.query_map(params_from_iter(card_ids), |row| {
        Ok((
            row.get::<_, i64>(0)?,
            FieldMedia { field_definition_id: row.get(1)?, attachment: media_from_row(row, 2)? },
        ))
    })?;
    for row in rows {
        let (card_id, media) = row?;
        by_card.entry(card_id).or_default().push(media);
    }
    Ok(by_card)
}

fn card_from_row(row: &Row, fields: Vec<FieldValue>, media: Vec<FieldMedia>) -> rusqlite::Result<Flashcard> {
    Ok(Flashcard::restore(
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        utc_column(row, 4)?,
        optional_utc_column(row, 5)?,
        fields,
        media,
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
        BTreeMap::new(),
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

    let mut fields = flashcard.fields().to_vec();
    for field in &fields {
        tx.execute(
            "INSERT INTO flashcard_fields (flashcard_id, field_definition_id, content) VALUES (?1, ?2, ?3)",
            params![id, field.field_definition_id(), field.content()],
        )?;
    }

    for item in flashcard.media() {
        // Media can be the only content of a field, so the field row is created when it is missing.
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM flashcard_fields WHERE flashcard_id = ?1 AND field_definition_id = ?2",
                params![id, item.field_definition_id],
                |row| row.get(0),
            )
            .optional()?;
        let field_id = match existing {
            Some(field_id) => field_id,
            None => {
                tx.execute(
                    "INSERT INTO flashcard_fields (flashcard_id, field_definition_id, content) VALUES (?1, ?2, '')",
                    params![id, item.field_definition_id],
                )?;
                fields.push(
                    FieldValue::new(item.field_definition_id, String::new())
                        .map_err(|e| conversion_error(0, e.to_string()))?,
                );
                tx.last_insert_rowid()
            }
        };
        let m = &item.attachment;
        tx.execute(
            "INSERT INTO field_media (flashcard_field_id, kind, mime_type, sha256, file_name, size_bytes, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![field_id, m.kind().as_str(), m.mime_type(), m.sha256(), m.file_name(), m.size_bytes(), m.position()],
        )?;
    }

    Ok(Flashcard::restore(id, uuid, deck_id, type_id, created, updated, fields, flashcard.media().to_vec()))
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
                    |row| card_from_row(row, Vec::new(), Vec::new()),
                )
                .optional()?;
            let Some(card) = card else { return Ok(None) };
            let id = card.id().expect("restored flashcards have an id");
            let fields = load_fields(conn, id)?;
            let media = load_card_media(conn, &[id])?.remove(&id).unwrap_or_default();
            Ok(Some(Flashcard::restore(
                id,
                card.uuid().to_owned(),
                card.deck_id(),
                card.flashcard_type_id(),
                card.created_at().expect("restored flashcards have a creation time"),
                card.updated_at(),
                fields,
                media,
            )))
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

            let card_ids: Vec<i64> = conn
                .prepare("SELECT id FROM flashcards WHERE deck_id = ?1")?
                .query_map([deck_id], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            let mut media = load_card_media(conn, &card_ids)?;

            conn.prepare(&format!(
                "SELECT {CARD_COLUMNS} FROM flashcards WHERE deck_id = ?1 ORDER BY created_at, id"
            ))?
            .query_map([deck_id], |row| {
                let id: i64 = row.get(0)?;
                card_from_row(
                    row,
                    by_card.remove(&id).unwrap_or_default(),
                    media.remove(&id).unwrap_or_default(),
                )
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
                              WHERE deck_id = ?2),
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
                "UPDATE flashcard_drafts SET position = position + 1 WHERE deck_id = ?1 AND position > ?2",
                params![deck_id, position],
            )?;
            let saved = tx.query_row(
                &format!(
                    "INSERT INTO flashcard_drafts (uuid, deck_id, flashcard_type_id, position, values_json)
                     VALUES (?1, ?2, ?3, ?4, ?5) RETURNING {DRAFT_COLUMNS}"
                ),
                params![draft.uuid(), deck_id, type_id, position + 1, values_to_json(draft.values())?],
                draft_from_row,
            )?;
            let new_id = saved.id().expect("restored drafts have an id");
            tx.execute(
                &format!(
                    "INSERT INTO flashcard_draft_media (draft_id, field_key, {MEDIA_COLUMNS})
                     SELECT ?1, m.field_key, m.kind, m.mime_type, m.sha256, m.file_name, m.size_bytes, m.position
                     FROM flashcard_draft_media m JOIN flashcard_drafts d ON d.id = m.draft_id
                     WHERE d.uuid = ?2"
                ),
                params![new_id, source_uuid],
            )?;
            let media = load_draft_media(&tx, &[new_id])?.remove(&new_id).unwrap_or_default();
            tx.commit()?;
            Ok(Some(saved.with_media(media)))
        })
    }

    fn find_draft(&self, uuid: &str) -> Result<Option<FlashcardDraft>, DbError> {
        self.db.with_connection(|conn| {
            let draft = conn
                .query_row(
                    &format!("SELECT {DRAFT_COLUMNS} FROM flashcard_drafts WHERE uuid = ?1"),
                    [uuid],
                    draft_from_row,
                )
                .optional()?;
            let Some(draft) = draft else { return Ok(None) };
            let id = draft.id().expect("restored drafts have an id");
            let media = load_draft_media(conn, &[id])?.remove(&id).unwrap_or_default();
            Ok(Some(draft.with_media(media)))
        })
    }

    fn find_drafts(&self, deck_id: i64) -> Result<Vec<FlashcardDraft>, DbError> {
        self.db.with_connection(|conn| {
            let drafts: Vec<FlashcardDraft> = conn
                .prepare(&format!(
                    "SELECT {DRAFT_COLUMNS} FROM flashcard_drafts
                     WHERE deck_id = ?1 ORDER BY position, id"
                ))?
                .query_map([deck_id], draft_from_row)?
                .collect::<rusqlite::Result<_>>()?;
            let ids: Vec<i64> = drafts.iter().filter_map(FlashcardDraft::id).collect();
            let mut media = load_draft_media(conn, &ids)?;
            Ok(drafts
                .into_iter()
                .map(|d| {
                    let m = d.id().and_then(|id| media.remove(&id)).unwrap_or_default();
                    d.with_media(m)
                })
                .collect())
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

    fn add_draft_media(
        &self,
        draft_uuid: &str,
        field_key: &str,
        media: &MediaAttachment,
    ) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            let draft_id: Option<i64> = conn
                .query_row("SELECT id FROM flashcard_drafts WHERE uuid = ?1", [draft_uuid], |row| row.get(0))
                .optional()?;
            let Some(draft_id) = draft_id else { return Ok(false) };
            conn.execute(
                &format!(
                    "INSERT OR IGNORE INTO flashcard_draft_media (draft_id, field_key, {MEDIA_COLUMNS})
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,
                             (SELECT COALESCE(MAX(position) + 1, 0) FROM flashcard_draft_media
                              WHERE draft_id = ?1 AND field_key = ?2))"
                ),
                params![
                    draft_id,
                    field_key,
                    media.kind().as_str(),
                    media.mime_type(),
                    media.sha256(),
                    media.file_name(),
                    media.size_bytes()
                ],
            )?;
            Ok(true)
        })
    }

    fn remove_draft_media(
        &self,
        draft_uuid: &str,
        field_key: &str,
        sha256: &str,
    ) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            let rows = conn.execute(
                "DELETE FROM flashcard_draft_media
                 WHERE draft_id = (SELECT id FROM flashcard_drafts WHERE uuid = ?1)
                   AND field_key = ?2 AND sha256 = ?3",
                params![draft_uuid, field_key, sha256],
            )?;
            Ok(rows > 0)
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
