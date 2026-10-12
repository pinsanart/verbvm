use rusqlite::ErrorCode;
use thiserror::Error;

use crate::dependencies::db::error::DbError;

pub type DecksResult<T> = Result<T, DecksError>;

#[derive(Debug, Error)]
pub enum DecksError {
    #[error("{0} not found")]
    NotFound(&'static str),

    #[error("invalid {field}: {reason}")]
    Invalid { field: &'static str, reason: &'static str },

    /// A constraint rejected the write, for example a deck name that is already taken.
    #[error("conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Db(DbError),
}

impl From<DbError> for DecksError {
    fn from(err: DbError) -> Self {
        match &err {
            DbError::Sqlite(rusqlite::Error::SqliteFailure(failure, msg))
                if failure.code == ErrorCode::ConstraintViolation =>
            {
                DecksError::Conflict(msg.clone().unwrap_or_else(|| failure.to_string()))
            }
            _ => DecksError::Db(err),
        }
    }
}
