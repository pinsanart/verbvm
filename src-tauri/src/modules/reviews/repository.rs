use std::sync::Arc;

use chrono::{DateTime, Utc};
use rusqlite::types::Type;
use rusqlite::{params, params_from_iter, OptionalExtension, Row};

use super::model::{CardStats, Rating, ReviewLog, Sm2State};
use crate::dependencies::clock::{format_utc, parse_utc};
use crate::dependencies::db::{connection::SqliteHandle, error::DbError};

/// Storage port for the module's three tables (`sm2_states`, `review_logs`, `card_stats`).
///
/// They share one trait because a review writes all three and must do so atomically.
pub trait ReviewRepository: Send + Sync {
    fn find_state(&self, flashcard_id: i64) -> Result<Option<Sm2State>, DbError>;
    fn find_states(&self, flashcard_ids: &[i64]) -> Result<Vec<Sm2State>, DbError>;
    /// States due at or before `now`, oldest first.
    fn find_due(&self, now: DateTime<Utc>, limit: u32) -> Result<Vec<Sm2State>, DbError>;
    fn find_stats(&self, flashcard_id: i64) -> Result<Option<CardStats>, DbError>;
    /// The flashcard's review history, oldest first.
    fn history(&self, flashcard_id: i64) -> Result<Vec<ReviewLog>, DbError>;
    /// Upserts the state and the stats and appends the log, in one transaction.
    /// Returns the log with its id.
    fn save_review(
        &self,
        state: &Sm2State,
        log: &ReviewLog,
        stats: &CardStats,
    ) -> Result<ReviewLog, DbError>;
}

const STATE_COLUMNS: &str =
    "flashcard_id, due_at, interval_days, ease_factor, repetitions, last_reviewed_at";
const LOG_COLUMNS: &str =
    "id, flashcard_id, reviewed_at, rating, elapsed_days, scheduled_days, duration_ms";
const STATS_COLUMNS: &str =
    "flashcard_id, review_count, correct_count, lapses, total_duration_ms, last_rating";

fn conversion_error(idx: usize, ty: Type, message: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(idx, ty, message.into())
}

fn utc(row: &Row, idx: usize) -> rusqlite::Result<DateTime<Utc>> {
    let text: String = row.get(idx)?;
    parse_utc(&text)
        .ok_or_else(|| conversion_error(idx, Type::Text, format!("invalid UTC timestamp: {text}")))
}

fn optional_utc(row: &Row, idx: usize) -> rusqlite::Result<Option<DateTime<Utc>>> {
    let text: Option<String> = row.get(idx)?;
    text.map(|t| {
        parse_utc(&t)
            .ok_or_else(|| conversion_error(idx, Type::Text, format!("invalid UTC timestamp: {t}")))
    })
    .transpose()
}

fn rating(row: &Row, idx: usize) -> rusqlite::Result<Rating> {
    let value: i64 = row.get(idx)?;
    Rating::try_from(value)
        .map_err(|_| conversion_error(idx, Type::Integer, format!("invalid rating: {value}")))
}

fn state_from_row(row: &Row) -> rusqlite::Result<Sm2State> {
    Ok(Sm2State::restore(
        row.get(0)?,
        utc(row, 1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        optional_utc(row, 5)?,
    ))
}

fn log_from_row(row: &Row) -> rusqlite::Result<ReviewLog> {
    Ok(ReviewLog::restore(
        row.get(0)?,
        row.get(1)?,
        utc(row, 2)?,
        rating(row, 3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
    ))
}

fn stats_from_row(row: &Row) -> rusqlite::Result<CardStats> {
    let last_rating: Option<i64> = row.get(5)?;
    let last_rating = last_rating
        .map(|v| {
            Rating::try_from(v)
                .map_err(|_| conversion_error(5, Type::Integer, format!("invalid rating: {v}")))
        })
        .transpose()?;
    Ok(CardStats::restore(
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        last_rating,
    ))
}

pub struct SqliteReviewRepository {
    db: Arc<SqliteHandle>,
}

impl SqliteReviewRepository {
    pub fn new(db: Arc<SqliteHandle>) -> Self {
        Self { db }
    }
}

impl ReviewRepository for SqliteReviewRepository {
    fn find_state(&self, flashcard_id: i64) -> Result<Option<Sm2State>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!("SELECT {STATE_COLUMNS} FROM sm2_states WHERE flashcard_id = ?1"),
                [flashcard_id],
                state_from_row,
            )
            .optional()
        })
    }

    fn find_states(&self, flashcard_ids: &[i64]) -> Result<Vec<Sm2State>, DbError> {
        if flashcard_ids.is_empty() {
            return Ok(Vec::new());
        }
        self.db.with_connection(|conn| {
            let placeholders = vec!["?"; flashcard_ids.len()].join(", ");
            conn.prepare(&format!(
                "SELECT {STATE_COLUMNS} FROM sm2_states
                 WHERE flashcard_id IN ({placeholders}) ORDER BY due_at, flashcard_id"
            ))?
            .query_map(params_from_iter(flashcard_ids), state_from_row)?
            .collect()
        })
    }

    fn find_due(&self, now: DateTime<Utc>, limit: u32) -> Result<Vec<Sm2State>, DbError> {
        self.db.with_connection(|conn| {
            conn.prepare(&format!(
                "SELECT {STATE_COLUMNS} FROM sm2_states
                 WHERE due_at <= ?1 ORDER BY due_at, flashcard_id LIMIT ?2"
            ))?
            .query_map(params![format_utc(now), limit], state_from_row)?
            .collect()
        })
    }

    fn find_stats(&self, flashcard_id: i64) -> Result<Option<CardStats>, DbError> {
        self.db.with_connection(|conn| {
            conn.query_row(
                &format!("SELECT {STATS_COLUMNS} FROM card_stats WHERE flashcard_id = ?1"),
                [flashcard_id],
                stats_from_row,
            )
            .optional()
        })
    }

    fn history(&self, flashcard_id: i64) -> Result<Vec<ReviewLog>, DbError> {
        self.db.with_connection(|conn| {
            conn.prepare(&format!(
                "SELECT {LOG_COLUMNS} FROM review_logs
                 WHERE flashcard_id = ?1 ORDER BY reviewed_at, id"
            ))?
            .query_map([flashcard_id], log_from_row)?
            .collect()
        })
    }

    fn save_review(
        &self,
        state: &Sm2State,
        log: &ReviewLog,
        stats: &CardStats,
    ) -> Result<ReviewLog, DbError> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;

            tx.execute(
                "INSERT INTO sm2_states (flashcard_id, due_at, interval_days, ease_factor, repetitions, last_reviewed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(flashcard_id) DO UPDATE SET
                     due_at = excluded.due_at,
                     interval_days = excluded.interval_days,
                     ease_factor = excluded.ease_factor,
                     repetitions = excluded.repetitions,
                     last_reviewed_at = excluded.last_reviewed_at",
                params![
                    state.flashcard_id(),
                    format_utc(state.due_at()),
                    state.interval_days(),
                    state.ease_factor(),
                    state.repetitions(),
                    state.last_reviewed_at().map(format_utc),
                ],
            )?;

            tx.execute(
                "INSERT INTO card_stats (flashcard_id, review_count, correct_count, lapses, total_duration_ms, last_rating)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(flashcard_id) DO UPDATE SET
                     review_count = excluded.review_count,
                     correct_count = excluded.correct_count,
                     lapses = excluded.lapses,
                     total_duration_ms = excluded.total_duration_ms,
                     last_rating = excluded.last_rating",
                params![
                    stats.flashcard_id(),
                    stats.review_count(),
                    stats.correct_count(),
                    stats.lapses(),
                    stats.total_duration_ms(),
                    stats.last_rating().map(Rating::as_i64),
                ],
            )?;

            let saved = tx.query_row(
                &format!(
                    "INSERT INTO review_logs (flashcard_id, reviewed_at, rating, elapsed_days, scheduled_days, duration_ms)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING {LOG_COLUMNS}"
                ),
                params![
                    log.flashcard_id(),
                    format_utc(log.reviewed_at()),
                    log.rating().as_i64(),
                    log.elapsed_days(),
                    log.scheduled_days(),
                    log.duration_ms(),
                ],
                log_from_row,
            )?;

            tx.commit()?;
            Ok(saved)
        })
    }
}
