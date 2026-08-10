use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLITE ERROR: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("MIGRATION ERROR: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("IO ERROR: {0}")]
    Io(#[from] std::io::Error),
}