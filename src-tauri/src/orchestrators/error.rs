use thiserror::Error;

use crate::dependencies::media_store::MediaError;
use crate::dependencies::pronunciation::PronunciationError;
use crate::dependencies::speech::SpeechError;
use crate::modules::card_types::CardTypesError;
use crate::modules::decks::DecksError;
use crate::modules::flashcards::FlashcardsError;
use crate::modules::languages::LanguagesError;
use crate::modules::reviews::ReviewsError;

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

    /// A service outside the app, such as text to speech, could not be reached.
    #[error("{0}")]
    Unavailable(String),

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

impl From<PronunciationError> for OrchestratorError {
    fn from(err: PronunciationError) -> Self {
        let message = err.to_string();
        match err {
            PronunciationError::NotInstalled | PronunciationError::Unsupported(_) => {
                OrchestratorError::Unavailable(message)
            }
            PronunciationError::NothingToImport => OrchestratorError::Invalid(message),
            PronunciationError::Db(_) | PronunciationError::Io(_) => OrchestratorError::Internal(message),
        }
    }
}

impl From<SpeechError> for OrchestratorError {
    fn from(err: SpeechError) -> Self {
        let message = err.to_string();
        match err {
            SpeechError::EmptyText | SpeechError::TextTooLong { .. } => OrchestratorError::Invalid(message),
            SpeechError::Unavailable(_) => OrchestratorError::Unavailable(message),
        }
    }
}

impl From<ReviewsError> for OrchestratorError {
    fn from(err: ReviewsError) -> Self {
        let message = err.to_string();
        match err {
            ReviewsError::Invalid { .. } => OrchestratorError::Invalid(message),
            ReviewsError::Conflict(_) => OrchestratorError::Conflict(message),
            ReviewsError::Db(_) => OrchestratorError::Internal(message),
        }
    }
}
