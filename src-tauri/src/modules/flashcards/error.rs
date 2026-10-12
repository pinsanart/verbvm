use rusqlite::ErrorCode;
use thiserror::Error;

use crate::dependencies::db::error::DbError;

pub type FlashcardsResult<T> = Result<T, FlashcardsError>;

#[derive(Debug, Error)]
pub enum FlashcardsError {
    #[error("{0} not found")]
    NotFound(&'static str),

    #[error("invalid {field}: {reason}")]
    Invalid { field: &'static str, reason: &'static str },

    /// A constraint rejected the write, for example a flashcard for a deck that does not exist.
    #[error("conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Db(DbError),
}

impl From<DbError> for FlashcardsError {
    fn from(err: DbError) -> Self {
        match &err {
            DbError::Sqlite(rusqlite::Error::SqliteFailure(failure, msg))
                if failure.code == ErrorCode::ConstraintViolation =>
            {
                FlashcardsError::Conflict(msg.clone().unwrap_or_else(|| failure.to_string()))
            }
            _ => FlashcardsError::Db(err),
        }
    }
}
