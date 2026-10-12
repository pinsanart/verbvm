use thiserror::Error;

use crate::dependencies::media_store::MediaError;
use crate::modules::card_types::CardTypesError;
use crate::modules::decks::DecksError;
use crate::modules::flashcards::FlashcardsError;
use crate::modules::languages::LanguagesError;

pub type OrchestratorResult<T> = Result<T, OrchestratorError>;

/// What went wrong in a use case, in terms the commands can report.
#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Invalid(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    Internal(String),
}

macro_rules! convert_module_error {
    ($error:ty) => {
        impl From<$error> for OrchestratorError {
            fn from(err: $error) -> Self {
                use $error as E;
                let message = err.to_string();
                match err {
                    E::NotFound(_) => OrchestratorError::NotFound(message),
                    E::Invalid { .. } => OrchestratorError::Invalid(message),
                    E::Conflict(_) => OrchestratorError::Conflict(message),
                    E::Db(_) => OrchestratorError::Internal(message),
                }
            }
        }
    };
}

convert_module_error!(LanguagesError);
convert_module_error!(DecksError);
convert_module_error!(CardTypesError);
convert_module_error!(FlashcardsError);

impl From<MediaError> for OrchestratorError {
    fn from(err: MediaError) -> Self {
        let message = err.to_string();
        match err {
            MediaError::NotAFile | MediaError::TooLarge | MediaError::UnsupportedType => {
                OrchestratorError::Invalid(message)
            }
            MediaError::Io(_) => OrchestratorError::Internal(message),
        }
    }
}
