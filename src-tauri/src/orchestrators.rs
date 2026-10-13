//! Use cases that coordinate more than one module.
//!
//! An orchestrator holds the public APIs of the modules it needs and nothing else. It resolves
//! the natural keys the view uses (deck name, type slug, ISO and BCP 47 codes) into the ids the
//! modules use, so no database id crosses the boundary to the view.

mod anki_import;
mod deck_overview;
mod error;
mod flashcard_editor;
mod pronunciation_text;
mod review_session;

pub use anki_import::{AnkiImportOrchestrator, AnkiImportReport, DeckImport};
pub use deck_overview::{DeckOverview, DeckOverviewOrchestrator};
pub use error::{OrchestratorError, OrchestratorResult};
pub use review_session::{
    CardState, DeckReview, Intervals, ReviewCardView, ReviewCounts, ReviewField, ReviewNext, ReviewSessionOrchestrator,
    TypeReview,
};
pub use flashcard_editor::{
    DraftIssue, EditorForm, FlashcardEditorOrchestrator, Generated, PublishOutcome, SavedCard, TypeForm,
};

#[cfg(test)]
mod fake_pronunciation;
#[cfg(test)]
mod fake_speech;
#[cfg(test)]
mod review_session_tests;
#[cfg(test)]
mod tests;
