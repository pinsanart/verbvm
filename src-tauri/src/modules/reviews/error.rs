use rusqlite::ErrorCode;
use thiserror::Error;

use crate::dependencies::db::error::DbError;

pub type ReviewsResult<T> = Result<T, ReviewsError>;

#[derive(Debug, Error)]
pub enum ReviewsError {
    #[error("invalid {field}: {reason}")]
    Invalid { field: &'static str, reason: &'static str },

    /// A constraint rejected the write, for example a review of a flashcard that does not exist.
    #[error("conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Db(DbError),
}

impl From<DbError> for ReviewsError {
    fn from(err: DbError) -> Self {
        match &err {
            DbError::Sqlite(rusqlite::Error::SqliteFailure(failure, msg))
                if failure.code == ErrorCode::ConstraintViolation =>
            {
                ReviewsError::Conflict(msg.clone().unwrap_or_else(|| failure.to_string()))
            }
            _ => ReviewsError::Db(err),
        }
    }
}
