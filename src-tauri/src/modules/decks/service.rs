use std::sync::Arc;

use super::error::{DecksError, DecksResult};
use super::model::Deck;
use super::repository::{DeckRepository, SqliteDeckRepository};
use crate::dependencies::db::connection::SqliteHandle;

/// Public API of the decks module. Other modules depend on this trait only.
///
/// Decks are identified by name outside the backend. `get_deck` takes an id and exists for
/// modules that hold a foreign key.
pub trait DeckApi: Send + Sync {
    /// Conflicts when the name is already taken.
    fn create_deck(
        &self,
        name: String,
        language_variant_id: i64,
        description: Option<String>,
    ) -> DecksResult<Deck>;
    fn get_deck(&self, id: i64) -> DecksResult<Deck>;
    fn get_deck_by_name(&self, name: &str) -> DecksResult<Deck>;
    /// All decks, ordered by name.
    fn list_decks(&self) -> DecksResult<Vec<Deck>>;
    /// Renames the deck and replaces its description. Conflicts when the new name is taken.
    fn update_deck(
        &self,
        name: &str,
        new_name: String,
        description: Option<String>,
    ) -> DecksResult<Deck>;
    /// Deletes the deck with its flashcards and drafts.
    fn delete_deck(&self, name: &str) -> DecksResult<()>;
}

pub struct DeckService {
    decks: Arc<dyn DeckRepository>,
}

impl DeckService {
    pub fn new(decks: Arc<dyn DeckRepository>) -> Self {
        Self { decks }
    }

    /// Service backed by the SQLite repository.
    pub fn sqlite(db: Arc<SqliteHandle>) -> Self {
        Self::new(Arc::new(SqliteDeckRepository::new(db)))
    }
}

impl DeckApi for DeckService {
    fn create_deck(
        &self,
        name: String,
        language_variant_id: i64,
        description: Option<String>,
    ) -> DecksResult<Deck> {
        let deck = Deck::new(name, language_variant_id, description)?;
        Ok(self.decks.insert(&deck)?)
    }

    fn get_deck(&self, id: i64) -> DecksResult<Deck> {
        self.decks.find(id)?.ok_or(DecksError::NotFound("deck"))
    }

    fn get_deck_by_name(&self, name: &str) -> DecksResult<Deck> {
        self.decks.find_by_name(name)?.ok_or(DecksError::NotFound("deck"))
    }

    fn list_decks(&self) -> DecksResult<Vec<Deck>> {
        Ok(self.decks.find_all()?)
    }

    fn update_deck(
        &self,
        name: &str,
        new_name: String,
        description: Option<String>,
    ) -> DecksResult<Deck> {
        let mut deck = self.get_deck_by_name(name)?;
        deck.revise(new_name, description)?;
        if !self.decks.update(&deck)? {
            return Err(DecksError::NotFound("deck"));
        }
        Ok(deck)
    }

    fn delete_deck(&self, name: &str) -> DecksResult<()> {
        let deck = self.get_deck_by_name(name)?;
        let id = deck.id().ok_or(DecksError::NotFound("deck"))?;
        if self.decks.delete(id)? {
            Ok(())
        } else {
            Err(DecksError::NotFound("deck"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> (DeckService, Arc<SqliteHandle>, std::path::PathBuf) {
        let path = std::env::temp_dir()
            .join(format!("verbvm-decks-{}-{:?}.db", std::process::id(), std::thread::current().id()));
        let _ = std::fs::remove_file(&path);
        let db = Arc::new(SqliteHandle::new(&path).unwrap());
        (DeckService::sqlite(db.clone()), db, path)
    }

    // Variant id 1 (en-US) comes from the migration seed.
    fn deck(svc: &DeckService, name: &str) -> DecksResult<Deck> {
        svc.create_deck(name.into(), 1, Some("About travel".into()))
    }

    #[test]
    fn create_and_find_by_name() {
        let (svc, _db, path) = service();
        let created = deck(&svc, "Travel").unwrap();
        assert!(created.id().is_some());
        assert!(created.created_at().is_some());
        let found = svc.get_deck_by_name("Travel").unwrap();
        assert_eq!(found, created);
        assert_eq!(svc.get_deck(created.id().unwrap()).unwrap(), created);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn duplicate_names_conflict_and_invalid_names_are_rejected() {
        let (svc, _db, path) = service();
        deck(&svc, "Travel").unwrap();
        assert!(matches!(deck(&svc, "Travel"), Err(DecksError::Conflict(_))));
        assert!(matches!(deck(&svc, "  "), Err(DecksError::Invalid { .. })));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn list_is_ordered_by_name_and_update_renames() {
        let (svc, _db, path) = service();
        deck(&svc, "Zoo").unwrap();
        deck(&svc, "Animals").unwrap();
        let names: Vec<_> = svc.list_decks().unwrap().iter().map(|d| d.name().to_owned()).collect();
        assert_eq!(names, vec!["Animals", "Zoo"]);

        let renamed = svc.update_deck("Zoo", "Wildlife".into(), None).unwrap();
        assert_eq!((renamed.name(), renamed.description()), ("Wildlife", None));
        assert!(matches!(svc.get_deck_by_name("Zoo"), Err(DecksError::NotFound(_))));
        assert!(matches!(
            svc.update_deck("Wildlife", "Animals".into(), None),
            Err(DecksError::Conflict(_))
        ));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn delete_removes_the_deck_and_reports_missing_ones() {
        let (svc, _db, path) = service();
        deck(&svc, "Travel").unwrap();
        svc.delete_deck("Travel").unwrap();
        assert!(matches!(svc.delete_deck("Travel"), Err(DecksError::NotFound(_))));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_language_variant_with_decks_cannot_be_deleted() {
        let (svc, db, path) = service();
        deck(&svc, "Travel").unwrap();
        let result = db.with_connection(|conn| conn.execute("DELETE FROM language_variants WHERE id = 1", []));
        assert!(result.is_err());
        let _ = std::fs::remove_file(path);
    }
}
