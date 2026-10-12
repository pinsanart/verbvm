//! Reading UTC timestamp columns from rows. See [`crate::dependencies::clock`] for the format.

use chrono::{DateTime, Utc};
use rusqlite::types::Type;
use rusqlite::Row;

use crate::dependencies::clock::parse_utc;

fn invalid(idx: usize, text: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        idx,
        Type::Text,
        format!("invalid UTC timestamp: {text}").into(),
    )
}

/// Reads a `NOT NULL` timestamp column.
pub fn utc_column(row: &Row, idx: usize) -> rusqlite::Result<DateTime<Utc>> {
    let text: String = row.get(idx)?;
    parse_utc(&text).ok_or_else(|| invalid(idx, &text))
}

/// Reads a nullable timestamp column.
pub fn optional_utc_column(row: &Row, idx: usize) -> rusqlite::Result<Option<DateTime<Utc>>> {
    let text: Option<String> = row.get(idx)?;
    text.map(|t| parse_utc(&t).ok_or_else(|| invalid(idx, &t))).transpose()
}
