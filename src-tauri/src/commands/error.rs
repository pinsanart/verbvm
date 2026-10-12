use serde::Serialize;

use crate::modules::languages::LanguagesError;
use crate::orchestrators::OrchestratorError;

/// Error returned to the frontend: a stable `kind` to branch on and a message to show.
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub kind: &'static str,
    pub message: String,
}

impl CommandError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self { kind: "invalid", message: message.into() }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self { kind: "not_found", message: message.into() }
    }

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

impl From<OrchestratorError> for CommandError {
    fn from(err: OrchestratorError) -> Self {
        let kind = match &err {
            OrchestratorError::NotFound(_) => "not_found",
            OrchestratorError::Invalid(_) => "invalid",
            OrchestratorError::Conflict(_) => "conflict",
            OrchestratorError::Unavailable(_) => "unavailable",
            OrchestratorError::Internal(_) => "internal",
        };
        Self { kind, message: err.to_string() }
    }
}
