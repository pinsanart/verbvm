use std::sync::Arc;

use rusqlite::{params, OptionalExtension, Row};

use super::model::Deck;
use crate::dependencies::db::sql_time::utc_column;
use crate::dependencies::db::{connection::SqliteHandle, error::DbError};

/// Storage port for decks.
pub trait DeckRepository: Send + Sync {
    /// Stores an unsaved deck and returns it with its id and creation time.
    fn insert(&self, deck: &Deck) -> Result<Deck, DbError>;
    fn find(&self, id: i64) -> Result<Option<Deck>, DbError>;
    fn find_by_name(&self, name: &str) -> Result<Option<Deck>, DbError>;
    fn find_all(&self) -> Result<Vec<Deck>, DbError>;
    /// Writes the name, the description and the language variant. Returns `false` when the deck
    /// is unsaved or gone.
    fn update(&self, deck: &Deck) -> Result<bool, DbError>;
    /// Cascades to the deck's flashcards and drafts.
    fn delete(&self, id: i64) -> Result<bool, DbError>;
}

const COLUMNS: &str = "id, name, language_variant_id, description, created_at";

fn from_row(row: &Row) -> rusqlite::Result<Deck> {
    Ok(Deck::restore(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, utc_column(row, 4)?))
}

pub struct SqliteDeckRepository {
    db: Arc<SqliteHandle>,
}

impl SqliteDeckRepository {
    pub fn new(db: Arc<SqliteHandle>) -> Self {
        Self { db }
    }
}

impl DeckRepository for SqliteDeckRepository {
    fn insert(&self, deck: &Deck) -> Result<Deck, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!(
                    "INSERT INTO decks (name, language_variant_id, description)
                     VALUES (?1, ?2, ?3) RETURNING {COLUMNS}"
                ),
                params![deck.name(), deck.language_variant_id(), deck.description()],
                from_row,
            )
        })
    }

    fn find(&self, id: i64) -> Result<Option<Deck>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(&format!("SELECT {COLUMNS} FROM decks WHERE id = ?1"), [id], from_row)
                .optional()
        })
    }

    fn find_by_name(&self, name: &str) -> Result<Option<Deck>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(&format!("SELECT {COLUMNS} FROM decks WHERE name = ?1"), [name], from_row)
                .optional()
        })
    }

    fn find_all(&self) -> Result<Vec<Deck>, DbError> {
        self.db.with_connection(|conn| {
            conn.prepare(&format!("SELECT {COLUMNS} FROM decks ORDER BY name"))?
                .query_map([], from_row)?
                .collect()
        })
    }

    fn update(&self, deck: &Deck) -> Result<bool, DbError> {
        let Some(id) = deck.id() else { return Ok(false) };
        self.db.with_connection(|conn| {
            let rows = conn.execute(
                "UPDATE decks SET name = ?1, language_variant_id = ?2, description = ?3 WHERE id = ?4",
                params![deck.name(), deck.language_variant_id(), deck.description(), id],
            )?;
            Ok(rows > 0)
        })
    }

    fn delete(&self, id: i64) -> Result<bool, DbError> {
        self.db
            .with_connection(|conn| Ok(conn.execute("DELETE FROM decks WHERE id = ?1", [id])? > 0))
    }
}
