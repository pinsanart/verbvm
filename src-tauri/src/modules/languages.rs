//! Languages and their regional or script variants.
//!
//! The module owns the `languages` and `language_variants` tables. A language has many variants,
//! each identified by a BCP 47 code, and one of them can be the language's default. The two tables
//! reference each other, so `default_variant_id` is nullable: the first variant created for a
//! language becomes its default.
//!
//! Layers: [`Language`] and [`LanguageVariant`] are entities that enforce their own invariants,
//! the repository traits are the storage port (with SQLite implementations), and
//! [`LanguageService`] implements the public [`LanguageApi`]. Other code depends on
//! [`LanguageApi`] only.

mod error;
mod model;
mod repository;
mod service;

pub use error::{LanguagesError, LanguagesResult};
pub use model::{Language, LanguageVariant};
pub use repository::{
    LanguageRepository, LanguageVariantRepository, SqliteLanguageRepository,
    SqliteLanguageVariantRepository,
};
pub use service::{LanguageApi, LanguageService};
