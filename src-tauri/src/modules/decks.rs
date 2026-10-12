//! Decks: named collections of flashcards in one language variant.
//!
//! The module owns the `decks` table. A deck is identified outside the backend by its unique
//! name. The language variant is held as a foreign key, and resolving it is up to callers that
//! know the languages module (see the orchestrators).

mod error;
mod model;
mod repository;
mod service;

pub use error::{DecksError, DecksResult};
pub use model::Deck;
pub use repository::{DeckRepository, SqliteDeckRepository};
pub use service::{DeckApi, DeckService};
