//! Use cases that coordinate more than one module.
//!
//! An orchestrator holds the public APIs of the modules it needs and nothing else. It resolves
//! the natural keys the view uses (deck name, type slug, ISO and BCP 47 codes) into the ids the
//! modules use, so no database id crosses the boundary to the view.

mod deck_overview;
mod error;
mod flashcard_editor;

pub use deck_overview::{DeckOverview, DeckOverviewOrchestrator};
pub use error::{OrchestratorError, OrchestratorResult};
pub use flashcard_editor::{
    DraftIssue, EditorForm, FlashcardEditorOrchestrator, PublishOutcome,
};

#[cfg(test)]
mod tests;
