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
    /// The flashcards linked by a stack, with their values, oldest first.
    fn find_flashcards_by_stack(&self, stack_uuid: &str) -> Result<Vec<Flashcard>, DbError>;
    /// Links a flashcard to the stack with this uuid (creating the stack when it is new), or
    /// unlinks it with `None`. Returns `false` when the flashcard does not exist.
    fn set_flashcard_stack(&self, uuid: &str, stack_uuid: Option<&str>) -> Result<bool, DbError>;
    /// Deletes every flashcard of a stack and the stack. Returns how many cards were deleted.
    fn delete_flashcards_in_stack(&self, stack_uuid: &str) -> Result<usize, DbError>;
    /// A stack with fewer than two flashcards is not a stack: unlinks the one that is left and
    /// deletes the stack.
    fn dissolve_stack_if_small(&self, stack_uuid: &str) -> Result<(), DbError>;
    fn count_by_type(&self, deck_id: i64) -> Result<Vec<TypeCount>, DbError>;
    /// Replaces the text of a saved flashcard: the given fields are written and the others are
    /// cleared (a field that still has media stays, with empty text). The flashcard keeps its id,
    /// so its review history is untouched. Returns `false` when it does not exist.
    fn update_flashcard_fields(&self, uuid: &str, fields: &[FieldValue]) -> Result<bool, DbError>;
    /// Attaches media to a field of a saved flashcard. Attaching the same file twice changes
    /// nothing. Returns `false` when the flashcard does not exist.
    fn add_flashcard_media(&self, uuid: &str, media: &FieldMedia) -> Result<bool, DbError>;
    /// Detaches the file with this hash from a field. Returns `false` when it was not attached.
    fn remove_flashcard_media(
        &self,
        uuid: &str,
        field_definition_id: i64,
        sha256: &str,
    ) -> Result<bool, DbError>;

    /// Stores an unsaved draft at the end of its deck's list.
    fn insert_draft(&self, draft: &FlashcardDraft) -> Result<FlashcardDraft, DbError>;
    /// Stores an unsaved draft right after the draft `source_uuid`, moving the later ones down.
    /// Returns `None` when the source does not exist.
    fn insert_draft_after(
        &self,
        source_uuid: &str,
        draft: &FlashcardDraft,
    ) -> Result<Option<FlashcardDraft>, DbError> {
        self.insert_draft_copy(source_uuid, source_uuid, draft)
    }
    /// Stores an unsaved draft right after the draft `after_uuid`, as a copy of `copy_of_uuid`
    /// (same type, and its media is copied), moving the later ones down. Returns `None` when
    /// either does not exist.
    fn insert_draft_copy(
        &self,
        copy_of_uuid: &str,
        after_uuid: &str,
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
    /// The drafts of a stack, by position.
    fn find_drafts_by_stack(&self, stack_uuid: &str) -> Result<Vec<FlashcardDraft>, DbError>;
    /// Puts a deck's drafts in this order (positions 0, 1, 2...). Drafts not listed keep their
    /// relative order after the listed ones.
    fn reorder_drafts(&self, deck_id: i64, ordered_uuids: &[String]) -> Result<(), DbError>;
    /// Sets or clears the stack of a draft. Returns `false` when the draft does not exist.
    fn set_draft_stack(&self, uuid: &str, stack_uuid: Option<&str>) -> Result<bool, DbError>;
    /// Deletes every draft of a stack. Returns how many were deleted.
    fn delete_drafts_in_stack(&self, stack_uuid: &str) -> Result<usize, DbError>;
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
/// The same plus the uuid of the flashcard's stack, for reading (not for `RETURNING`).
const CARD_SELECT: &str = "id, uuid, deck_id, flashcard_type_id, created_at, updated_at,
     (SELECT s.uuid FROM stacks s WHERE s.id = flashcards.stack_id)";
const DRAFT_COLUMNS: &str =
    "id, uuid, deck_id, flashcard_type_id, position, values_json, stack_uuid, updated_at";

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
        row.get(6)?,
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
        row.get(6)?,
        utc_column(row, 7)?,
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

    if let Some(stack_uuid) = flashcard.stack_uuid() {
        link_to_stack(tx, id, stack_uuid)?;
    }

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

    Ok(Flashcard::restore(
        id,
        uuid,
        deck_id,
        type_id,
        created,
        updated,
        fields,
        flashcard.media().to_vec(),
        flashcard.stack_uuid().map(str::to_owned),
    ))
}

/// Links a flashcard to the stack with this uuid, creating the stack when it does not exist.
fn link_to_stack(conn: &Connection, flashcard_id: i64, stack_uuid: &str) -> rusqlite::Result<()> {
    conn.execute("INSERT OR IGNORE INTO stacks (uuid) VALUES (?1)", [stack_uuid])?;
    conn.execute(
        "UPDATE flashcards SET stack_id = (SELECT id FROM stacks WHERE uuid = ?1) WHERE id = ?2",
        params![stack_uuid, flashcard_id],
    )?;
    Ok(())
}

/// Unlinks the flashcards of a stack and deletes the stack when fewer than two are left.
fn dissolve_if_small(conn: &Connection, stack_uuid: &str) -> rusqlite::Result<()> {
    let members: i64 = conn.query_row(
        "SELECT COUNT(*) FROM flashcards f JOIN stacks s ON s.id = f.stack_id WHERE s.uuid = ?1",
        [stack_uuid],
        |row| row.get(0),
    )?;
    if members < 2 {
        conn.execute(
            "UPDATE flashcards SET stack_id = NULL
             WHERE stack_id = (SELECT id FROM stacks WHERE uuid = ?1)",
            [stack_uuid],
        )?;
        conn.execute("DELETE FROM stacks WHERE uuid = ?1", [stack_uuid])?;
    }
    Ok(())
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
                    &format!("SELECT {CARD_SELECT} FROM flashcards WHERE uuid = ?1"),
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
                card.stack_uuid().map(str::to_owned),
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
                "SELECT {CARD_SELECT} FROM flashcards WHERE deck_id = ?1 ORDER BY created_at, id"
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
            let stack: Option<String> = conn
                .query_row(
                    "SELECT s.uuid FROM flashcards f JOIN stacks s ON s.id = f.stack_id WHERE f.uuid = ?1",
                    [uuid],
                    |row| row.get(0),
                )
                .optional()?;
            let deleted = conn.execute("DELETE FROM flashcards WHERE uuid = ?1", [uuid])? > 0;
            if let Some(stack) = stack {
                dissolve_if_small(conn, &stack)?;
            }
            Ok(deleted)
        })
    }

    fn find_flashcards_by_stack(&self, stack_uuid: &str) -> Result<Vec<Flashcard>, DbError> {
        let uuids: Vec<String> = self.db.with_connection(|conn| {
            conn.prepare(
                "SELECT f.uuid FROM flashcards f JOIN stacks s ON s.id = f.stack_id
                 WHERE s.uuid = ?1 ORDER BY f.created_at, f.id",
            )?
            .query_map([stack_uuid], |row| row.get(0))?
            .collect()
        })?;
        let mut cards = Vec::with_capacity(uuids.len());
        for uuid in uuids {
            if let Some(card) = self.find_flashcard(&uuid)? {
                cards.push(card);
            }
        }
        Ok(cards)
    }

    fn set_flashcard_stack(&self, uuid: &str, stack_uuid: Option<&str>) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let found: Option<(i64, Option<String>)> = tx
                .query_row(
                    "SELECT f.id, (SELECT s.uuid FROM stacks s WHERE s.id = f.stack_id)
                     FROM flashcards f WHERE f.uuid = ?1",
                    [uuid],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            let Some((id, previous)) = found else { return Ok(false) };
            match stack_uuid {
                Some(stack) => link_to_stack(&tx, id, stack)?,
                None => {
                    tx.execute("UPDATE flashcards SET stack_id = NULL WHERE id = ?1", [id])?;
                }
            }
            if let Some(previous) = previous.filter(|p| Some(p.as_str()) != stack_uuid) {
                dissolve_if_small(&tx, &previous)?;
            }
            tx.commit()?;
            Ok(true)
        })
    }

    fn delete_flashcards_in_stack(&self, stack_uuid: &str) -> Result<usize, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let deleted = tx.execute(
                "DELETE FROM flashcards WHERE stack_id = (SELECT id FROM stacks WHERE uuid = ?1)",
                [stack_uuid],
            )?;
            tx.execute("DELETE FROM stacks WHERE uuid = ?1", [stack_uuid])?;
            tx.commit()?;
            Ok(deleted)
        })
    }

    fn dissolve_stack_if_small(&self, stack_uuid: &str) -> Result<(), DbError> {
        self.db.with_connection(|conn| dissolve_if_small(conn, stack_uuid))
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

    fn update_flashcard_fields(&self, uuid: &str, fields: &[FieldValue]) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let id: Option<i64> = tx
                .query_row("SELECT id FROM flashcards WHERE uuid = ?1", [uuid], |row| row.get(0))
                .optional()?;
            let Some(id) = id else { return Ok(false) };

            let keep: std::collections::HashSet<i64> =
                fields.iter().map(FieldValue::field_definition_id).collect();
            let existing: Vec<(i64, i64, bool)> = tx
                .prepare(
                    "SELECT ff.id, ff.field_definition_id,
                            EXISTS (SELECT 1 FROM field_media fm WHERE fm.flashcard_field_id = ff.id)
                     FROM flashcard_fields ff WHERE ff.flashcard_id = ?1",
                )?
                .query_map([id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .collect::<rusqlite::Result<_>>()?;
            for (field_id, definition_id, has_media) in existing {
                if keep.contains(&definition_id) {
                    continue;
                }
                if has_media {
                    tx.execute("UPDATE flashcard_fields SET content = '' WHERE id = ?1", [field_id])?;
                } else {
                    tx.execute("DELETE FROM flashcard_fields WHERE id = ?1", [field_id])?;
                }
            }
            for field in fields {
                tx.execute(
                    "INSERT INTO flashcard_fields (flashcard_id, field_definition_id, content) VALUES (?1, ?2, ?3)
                     ON CONFLICT(flashcard_id, field_definition_id) DO UPDATE SET content = excluded.content",
                    params![id, field.field_definition_id(), field.content()],
                )?;
            }
            tx.execute("UPDATE flashcards SET updated_at = datetime('now') WHERE id = ?1", [id])?;
            tx.commit()?;
            Ok(true)
        })
    }

    fn add_flashcard_media(&self, uuid: &str, media: &FieldMedia) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let id: Option<i64> = tx
                .query_row("SELECT id FROM flashcards WHERE uuid = ?1", [uuid], |row| row.get(0))
                .optional()?;
            let Some(id) = id else { return Ok(false) };

            tx.execute(
                "INSERT OR IGNORE INTO flashcard_fields (flashcard_id, field_definition_id, content) VALUES (?1, ?2, '')",
                params![id, media.field_definition_id],
            )?;
            let field_id: i64 = tx.query_row(
                "SELECT id FROM flashcard_fields WHERE flashcard_id = ?1 AND field_definition_id = ?2",
                params![id, media.field_definition_id],
                |row| row.get(0),
            )?;
            let m = &media.attachment;
            tx.execute(
                "INSERT INTO field_media (flashcard_field_id, kind, mime_type, sha256, file_name, size_bytes, position)
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6,
                        (SELECT COALESCE(MAX(position) + 1, 0) FROM field_media WHERE flashcard_field_id = ?1)
                 WHERE NOT EXISTS (SELECT 1 FROM field_media WHERE flashcard_field_id = ?1 AND sha256 = ?4)",
                params![field_id, m.kind().as_str(), m.mime_type(), m.sha256(), m.file_name(), m.size_bytes()],
            )?;
            tx.execute("UPDATE flashcards SET updated_at = datetime('now') WHERE id = ?1", [id])?;
            tx.commit()?;
            Ok(true)
        })
    }

    fn remove_flashcard_media(
        &self,
        uuid: &str,
        field_definition_id: i64,
        sha256: &str,
    ) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let field: Option<(i64, i64)> = tx
                .query_row(
                    "SELECT f.id, ff.id FROM flashcards f
                     JOIN flashcard_fields ff ON ff.flashcard_id = f.id
                     WHERE f.uuid = ?1 AND ff.field_definition_id = ?2",
                    params![uuid, field_definition_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            let Some((card_id, field_id)) = field else { return Ok(false) };

            let removed = tx.execute(
                "DELETE FROM field_media WHERE flashcard_field_id = ?1 AND sha256 = ?2",
                params![field_id, sha256],
            )? > 0;
            if removed {
                // A field with no text and no media left has nothing to keep.
                tx.execute(
                    "DELETE FROM flashcard_fields WHERE id = ?1 AND COALESCE(content, '') = ''
                     AND NOT EXISTS (SELECT 1 FROM field_media WHERE flashcard_field_id = ?1)",
                    [field_id],
                )?;
                tx.execute("UPDATE flashcards SET updated_at = datetime('now') WHERE id = ?1", [card_id])?;
            }
            tx.commit()?;
            Ok(removed)
        })
    }

    fn insert_draft(&self, draft: &FlashcardDraft) -> Result<FlashcardDraft, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!(
                    "INSERT INTO flashcard_drafts (uuid, deck_id, flashcard_type_id, position, values_json, stack_uuid)
                     VALUES (?1, ?2, ?3,
                             (SELECT COALESCE(MAX(position) + 1, 0) FROM flashcard_drafts
                              WHERE deck_id = ?2),
                             ?4, ?5)
                     RETURNING {DRAFT_COLUMNS}"
                ),
                params![
                    draft.uuid(),
                    draft.deck_id(),
                    draft.flashcard_type_id(),
                    values_to_json(draft.values())?,
                    draft.stack_uuid()
                ],
                draft_from_row,
            )
        })
    }

    fn insert_draft_copy(
        &self,
        copy_of_uuid: &str,
        after_uuid: &str,
        draft: &FlashcardDraft,
    ) -> Result<Option<FlashcardDraft>, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let source: Option<(i64, i64)> = tx
                .query_row(
                    "SELECT deck_id, flashcard_type_id FROM flashcard_drafts WHERE uuid = ?1",
                    [copy_of_uuid],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            let after: Option<i64> = tx
                .query_row("SELECT position FROM flashcard_drafts WHERE uuid = ?1", [after_uuid], |row| row.get(0))
                .optional()?;
            let (Some((deck_id, type_id)), Some(position)) = (source, after) else { return Ok(None) };

            tx.execute(
                "UPDATE flashcard_drafts SET position = position + 1 WHERE deck_id = ?1 AND position > ?2",
                params![deck_id, position],
            )?;
            let saved = tx.query_row(
                &format!(
                    "INSERT INTO flashcard_drafts (uuid, deck_id, flashcard_type_id, position, values_json, stack_uuid)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING {DRAFT_COLUMNS}"
                ),
                params![
                    draft.uuid(),
                    deck_id,
                    type_id,
                    position + 1,
                    values_to_json(draft.values())?,
                    draft.stack_uuid()
                ],
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
                params![new_id, copy_of_uuid],
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

    fn find_drafts_by_stack(&self, stack_uuid: &str) -> Result<Vec<FlashcardDraft>, DbError> {
        self.db.with_connection(|conn| {
            let drafts: Vec<FlashcardDraft> = conn
                .prepare(&format!(
                    "SELECT {DRAFT_COLUMNS} FROM flashcard_drafts WHERE stack_uuid = ?1 ORDER BY position, id"
                ))?
                .query_map([stack_uuid], draft_from_row)?
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

    fn reorder_drafts(&self, deck_id: i64, ordered_uuids: &[String]) -> Result<(), DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let current: Vec<String> = tx
                .prepare("SELECT uuid FROM flashcard_drafts WHERE deck_id = ?1 ORDER BY position, id")?
                .query_map([deck_id], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            let rest = current.iter().filter(|uuid| !ordered_uuids.contains(uuid));
            for (position, uuid) in ordered_uuids.iter().chain(rest).enumerate() {
                tx.execute(
                    "UPDATE flashcard_drafts SET position = ?1 WHERE uuid = ?2 AND deck_id = ?3",
                    params![position as i64, uuid, deck_id],
                )?;
            }
            tx.commit()?;
            Ok(())
        })
    }

    fn set_draft_stack(&self, uuid: &str, stack_uuid: Option<&str>) -> Result<bool, DbError> {
        self.db.with_connection(|conn| {
            let rows = conn.execute(
                "UPDATE flashcard_drafts SET stack_uuid = ?1 WHERE uuid = ?2",
                params![stack_uuid, uuid],
            )?;
            Ok(rows > 0)
        })
    }

    fn delete_drafts_in_stack(&self, stack_uuid: &str) -> Result<usize, DbError> {
        self.db.with_connection(|conn| {
            Ok(conn.execute("DELETE FROM flashcard_drafts WHERE stack_uuid = ?1", [stack_uuid])?)
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
