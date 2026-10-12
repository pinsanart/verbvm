use rusqlite::ErrorCode;
use thiserror::Error;

use crate::dependencies::db::error::DbError;

pub type CardTypesResult<T> = Result<T, CardTypesError>;

#[derive(Debug, Error)]
pub enum CardTypesError {
    #[error("{0} not found")]
    NotFound(&'static str),

    #[error("invalid {field}: {reason}")]
    Invalid { field: &'static str, reason: &'static str },

    /// A constraint rejected the write, for example a slug or a field key that already exists.
    #[error("conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Db(DbError),
}

impl From<DbError> for CardTypesError {
    fn from(err: DbError) -> Self {
        match &err {
            DbError::Sqlite(rusqlite::Error::SqliteFailure(failure, msg))
                if failure.code == ErrorCode::ConstraintViolation =>
            {
                CardTypesError::Conflict(msg.clone().unwrap_or_else(|| failure.to_string()))
            }
            _ => CardTypesError::Db(err),
        }
    }
}
