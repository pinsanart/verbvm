use rusqlite::ErrorCode;
use thiserror::Error;

use crate::dependencies::db::error::DbError;

pub type LanguagesResult<T> = Result<T, LanguagesError>;

#[derive(Debug, Error)]
pub enum LanguagesError {
    #[error("{0} not found")]
    NotFound(&'static str),

    #[error("invalid {field}: {reason}")]
    Invalid { field: &'static str, reason: &'static str },

    /// A UNIQUE or FOREIGN KEY constraint rejected the write.
    #[error("conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Db(DbError),
}

impl From<DbError> for LanguagesError {
    fn from(err: DbError) -> Self {
        match &err {
            DbError::Sqlite(rusqlite::Error::SqliteFailure(failure, msg))
                if failure.code == ErrorCode::ConstraintViolation =>
            {
                LanguagesError::Conflict(msg.clone().unwrap_or_else(|| failure.to_string()))
            }
            _ => LanguagesError::Db(err),
        }
    }
}
