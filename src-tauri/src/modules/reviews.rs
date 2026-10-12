//! Flashcard review: SM-2 scheduling, review history and per-card statistics.
//!
//! The module owns the `sm2_states`, `review_logs` and `card_stats` tables. A flashcard that has
//! no `sm2_states` row has never been reviewed. The row is created on its first review, so
//! creating a flashcard never depends on this module.
//!
//! Layers: [`model`] holds the entities and the SM-2 rule, `repository` the storage port and its
//! SQLite implementation, and `service` the public [`ReviewApi`]. Other code depends on
//! [`ReviewApi`] only. Time comes from an injected
//! [`Clock`](crate::dependencies::clock::Clock), always in UTC.

mod error;
mod model;
mod repository;
mod service;

pub use error::{ReviewsError, ReviewsResult};
pub use model::{CardStats, Rating, ReviewActivity, ReviewLog, ReviewPhase, Sm2State};
pub use repository::{ReviewRepository, SqliteReviewRepository};
pub use service::{ReviewApi, ReviewService};
