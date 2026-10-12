//! Flashcard types and the fields each type is made of.
//!
//! The module owns the `flashcard_types` and `field_definitions` tables. A card's layout is data:
//! a type has field definitions on a side (front or back), and a definition can be limited to one
//! language (for example romaji for Japanese). Outside the backend a type is identified by its
//! slug and a field by its key.

mod error;
mod model;
mod repository;
mod service;

pub use error::{CardTypesError, CardTypesResult};
pub use model::{FieldDefinition, FlashcardType, Side};
pub use repository::{
    FieldDefinitionRepository, FlashcardTypeRepository, SqliteFieldDefinitionRepository,
    SqliteFlashcardTypeRepository,
};
pub use service::{CardTypeApi, CardTypeService};
