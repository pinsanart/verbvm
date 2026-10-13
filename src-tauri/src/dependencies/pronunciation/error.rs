use thiserror::Error;

#[derive(Debug, Error)]
pub enum PronunciationError {
    #[error("the pronunciation dictionary is not installed: import it in Settings")]
    NotInstalled,

    #[error("there is no pronunciation data for {0}")]
    Unsupported(String),

    #[error("no dictionary files were found in that folder")]
    NothingToImport,

    #[error("pronunciation dictionary error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("could not read or write the pronunciation dictionary: {0}")]
    Io(#[from] std::io::Error),
}
