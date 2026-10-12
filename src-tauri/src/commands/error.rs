use serde::Serialize;

use crate::modules::languages::LanguagesError;

/// Error returned to the frontend: a stable `kind` to branch on and a message to show.
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub kind: &'static str,
    pub message: String,
}

impl CommandError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self { kind: "internal", message: message.into() }
    }
}

impl From<LanguagesError> for CommandError {
    fn from(err: LanguagesError) -> Self {
        let kind = match &err {
            LanguagesError::NotFound(_) => "not_found",
            LanguagesError::Invalid { .. } => "invalid",
            LanguagesError::Conflict(_) => "conflict",
            LanguagesError::Db(_) => "internal",
        };
        Self { kind, message: err.to_string() }
    }
}
