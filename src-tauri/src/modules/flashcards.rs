//! Flashcards, their field values and the drafts they start as.
//!
//! The module owns the `flashcards`, `flashcard_fields` and `flashcard_drafts` tables. A
//! flashcard is identified outside the backend by an opaque uuid. A draft is an unsaved
//! flashcard being edited: it keeps the typed values by field key, survives restarts and is
//! turned into a flashcard by publishing it.
//!
//! The module stores what it is given. Whether a value belongs to a field of the card's type and
//! language, and whether it is sanitized, is checked by the caller (see the orchestrators).

mod error;
mod model;
mod repository;
mod service;

pub use error::{FlashcardsError, FlashcardsResult};
pub use model::{
    Direction, FieldMedia, FieldValue, Flashcard, FlashcardDraft, MediaAttachment, TypeCount,
};
pub use repository::{FlashcardRepository, SqliteFlashcardRepository};
pub use service::{FlashcardApi, FlashcardService, PublishedDraft};
