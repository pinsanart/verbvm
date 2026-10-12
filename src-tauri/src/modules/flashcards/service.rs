use std::collections::BTreeMap;
use std::sync::Arc;

use super::error::{FlashcardsError, FlashcardsResult};
use super::model::{Direction, FieldValue, Flashcard, FlashcardDraft, TypeCount};
use super::repository::{FlashcardRepository, SqliteFlashcardRepository};
use crate::dependencies::db::connection::SqliteHandle;

/// A draft to publish and the field values that become its flashcard.
#[derive(Debug, Clone)]
pub struct PublishedDraft {
    pub draft_uuid: String,
    pub fields: Vec<FieldValue>,
}

/// Public API of the flashcards module. Other modules depend on this trait only.
///
/// Flashcards and drafts are identified by uuid outside the backend. Deck and type ids are
/// foreign keys held by the callers that resolved them.
pub trait FlashcardApi: Send + Sync {
    /// Conflicts when the deck, the type or a field definition does not exist.
    fn create_flashcard(
        &self,
        deck_id: i64,
        flashcard_type_id: i64,
        fields: Vec<FieldValue>,
    ) -> FlashcardsResult<Flashcard>;
    fn get_flashcard(&self, uuid: &str) -> FlashcardsResult<Flashcard>;
    fn list_flashcards(&self, deck_id: i64) -> FlashcardsResult<Vec<Flashcard>>;
    fn delete_flashcard(&self, uuid: &str) -> FlashcardsResult<()>;
    /// How many flashcards of each type the deck has. Types with none are left out.
    fn count_by_type(&self, deck_id: i64) -> FlashcardsResult<Vec<TypeCount>>;

    /// Adds an empty draft at the end of the list of its deck and type.
    fn create_draft(&self, deck_id: i64, flashcard_type_id: i64) -> FlashcardsResult<FlashcardDraft>;
    /// The drafts of a deck and type, in the order they are edited.
    fn list_drafts(&self, deck_id: i64, flashcard_type_id: i64) -> FlashcardsResult<Vec<FlashcardDraft>>;
    /// Replaces the typed values of a draft.
    fn update_draft(&self, uuid: &str, values: BTreeMap<String, String>) -> FlashcardsResult<FlashcardDraft>;
    /// Copies a draft and puts the copy right after it.
    fn duplicate_draft(&self, uuid: &str) -> FlashcardsResult<FlashcardDraft>;
    /// Moves a draft one place. Moving the first draft up or the last one down changes nothing.
    fn move_draft(&self, uuid: &str, direction: Direction) -> FlashcardsResult<()>;
    fn delete_draft(&self, uuid: &str) -> FlashcardsResult<()>;

    /// Turns drafts into flashcards of one deck and type and deletes the drafts, all or nothing.
    fn publish_drafts(
        &self,
        deck_id: i64,
        flashcard_type_id: i64,
        drafts: Vec<PublishedDraft>,
    ) -> FlashcardsResult<Vec<Flashcard>>;
}

pub struct FlashcardService {
    repository: Arc<dyn FlashcardRepository>,
}

impl FlashcardService {
    pub fn new(repository: Arc<dyn FlashcardRepository>) -> Self {
        Self { repository }
    }

    /// Service backed by the SQLite repository.
    pub fn sqlite(db: Arc<SqliteHandle>) -> Self {
        Self::new(Arc::new(SqliteFlashcardRepository::new(db)))
    }

    fn draft(&self, uuid: &str) -> FlashcardsResult<FlashcardDraft> {
        self.repository.find_draft(uuid)?.ok_or(FlashcardsError::NotFound("draft"))
    }
}

impl FlashcardApi for FlashcardService {
    fn create_flashcard(
        &self,
        deck_id: i64,
        flashcard_type_id: i64,
        fields: Vec<FieldValue>,
    ) -> FlashcardsResult<Flashcard> {
        let flashcard = Flashcard::new(deck_id, flashcard_type_id, fields)?;
        Ok(self.repository.insert_flashcard(&flashcard)?)
    }

    fn get_flashcard(&self, uuid: &str) -> FlashcardsResult<Flashcard> {
        self.repository.find_flashcard(uuid)?.ok_or(FlashcardsError::NotFound("flashcard"))
    }

    fn list_flashcards(&self, deck_id: i64) -> FlashcardsResult<Vec<Flashcard>> {
        Ok(self.repository.find_flashcards_by_deck(deck_id)?)
    }

    fn delete_flashcard(&self, uuid: &str) -> FlashcardsResult<()> {
        if self.repository.delete_flashcard(uuid)? {
            Ok(())
        } else {
            Err(FlashcardsError::NotFound("flashcard"))
        }
    }

    fn count_by_type(&self, deck_id: i64) -> FlashcardsResult<Vec<TypeCount>> {
        Ok(self.repository.count_by_type(deck_id)?)
    }

    fn create_draft(&self, deck_id: i64, flashcard_type_id: i64) -> FlashcardsResult<FlashcardDraft> {
        let draft = FlashcardDraft::new(deck_id, flashcard_type_id);
        Ok(self.repository.insert_draft(&draft)?)
    }

    fn list_drafts(&self, deck_id: i64, flashcard_type_id: i64) -> FlashcardsResult<Vec<FlashcardDraft>> {
        Ok(self.repository.find_drafts(deck_id, flashcard_type_id)?)
    }

    fn update_draft(&self, uuid: &str, values: BTreeMap<String, String>) -> FlashcardsResult<FlashcardDraft> {
        let mut draft = self.draft(uuid)?;
        draft.set_values(values)?;
        if !self.repository.update_draft_values(uuid, draft.values())? {
            return Err(FlashcardsError::NotFound("draft"));
        }
        self.draft(uuid)
    }

    fn duplicate_draft(&self, uuid: &str) -> FlashcardsResult<FlashcardDraft> {
        let copy = self.draft(uuid)?.duplicate();
        self.repository
            .insert_draft_after(uuid, &copy)?
            .ok_or(FlashcardsError::NotFound("draft"))
    }

    fn move_draft(&self, uuid: &str, direction: Direction) -> FlashcardsResult<()> {
        let draft = self.draft(uuid)?;
        let drafts = self.repository.find_drafts(draft.deck_id(), draft.flashcard_type_id())?;
        let index = drafts
            .iter()
            .position(|d| d.uuid() == uuid)
            .ok_or(FlashcardsError::NotFound("draft"))?;
        let neighbor = match direction {
            Direction::Up => index.checked_sub(1),
            Direction::Down => Some(index + 1).filter(|i| *i < drafts.len()),
        };
        if let Some(neighbor) = neighbor {
            self.repository.swap_draft_positions(uuid, drafts[neighbor].uuid())?;
        }
        Ok(())
    }

    fn delete_draft(&self, uuid: &str) -> FlashcardsResult<()> {
        if self.repository.delete_draft(uuid)? {
            Ok(())
        } else {
            Err(FlashcardsError::NotFound("draft"))
        }
    }

    fn publish_drafts(
        &self,
        deck_id: i64,
        flashcard_type_id: i64,
        drafts: Vec<PublishedDraft>,
    ) -> FlashcardsResult<Vec<Flashcard>> {
        let mut flashcards = Vec::with_capacity(drafts.len());
        let mut draft_uuids = Vec::with_capacity(drafts.len());
        for draft in drafts {
            flashcards.push(Flashcard::new(deck_id, flashcard_type_id, draft.fields)?);
            draft_uuids.push(draft.draft_uuid);
        }
        Ok(self.repository.publish(&flashcards, &draft_uuids)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // From the migration seed: variant 1 is en-US, type 1 is vocabulary, and field definitions
    // 1, 2 and 3 are its phrase, word_or_expression and meaning.
    const VOCABULARY: i64 = 1;

    struct Fixture {
        service: FlashcardService,
        db: Arc<SqliteHandle>,
        deck_id: i64,
        path: std::path::PathBuf,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{suffix}", self.path.display()));
            }
        }
    }

    fn fixture() -> Fixture {
        let path = std::env::temp_dir().join(format!(
            "verbvm-flashcards-{}-{:?}.db",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_file(&path);
        let db = Arc::new(SqliteHandle::new(&path).unwrap());
        db.with_connection(|conn| {
            conn.execute_batch("INSERT INTO decks (name, language_variant_id) VALUES ('Deck', 1);")
        })
        .unwrap();
        Fixture { service: FlashcardService::sqlite(db.clone()), db, deck_id: 1, path }
    }

    fn values(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn uuids(drafts: &[FlashcardDraft]) -> Vec<String> {
        drafts.iter().map(|d| d.uuid().to_owned()).collect()
    }

    fn count(f: &Fixture, table: &str) -> i64 {
        f.db.with_connection(|conn| conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0)))
            .unwrap()
    }

    #[test]
    fn flashcards_round_trip_with_their_fields() {
        let f = fixture();
        let fields = vec![
            FieldValue::new(1, "<b>It's raining</b>".into()).unwrap(),
            FieldValue::new(2, "raining cats and dogs".into()).unwrap(),
        ];
        let created = f.service.create_flashcard(f.deck_id, VOCABULARY, fields.clone()).unwrap();
        assert!(created.id().is_some() && created.created_at().is_some());

        let found = f.service.get_flashcard(created.uuid()).unwrap();
        assert_eq!(found.fields(), fields.as_slice());
        assert_eq!(f.service.list_flashcards(f.deck_id).unwrap().len(), 1);
        assert_eq!(f.service.list_flashcards(f.deck_id).unwrap()[0].fields().len(), 2);

        let counts = f.service.count_by_type(f.deck_id).unwrap();
        assert_eq!(counts, vec![TypeCount { flashcard_type_id: VOCABULARY, count: 1 }]);

        f.service.delete_flashcard(created.uuid()).unwrap();
        assert_eq!(count(&f, "flashcard_fields"), 0);
        assert!(matches!(f.service.get_flashcard(created.uuid()), Err(FlashcardsError::NotFound(_))));
    }

    #[test]
    fn a_flashcard_for_a_missing_deck_or_field_conflicts() {
        let f = fixture();
        assert!(matches!(
            f.service.create_flashcard(99, VOCABULARY, vec![]),
            Err(FlashcardsError::Conflict(_))
        ));
        let bad_field = vec![FieldValue::new(999, "x".into()).unwrap()];
        assert!(matches!(
            f.service.create_flashcard(f.deck_id, VOCABULARY, bad_field),
            Err(FlashcardsError::Conflict(_))
        ));
        assert_eq!(count(&f, "flashcards"), 0, "the failed insert is rolled back");
    }

    #[test]
    fn drafts_keep_their_order_and_values() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let c = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let listed = f.service.list_drafts(f.deck_id, VOCABULARY).unwrap();
        assert_eq!(uuids(&listed), vec![a.uuid(), b.uuid(), c.uuid()]);

        let updated = f.service.update_draft(b.uuid(), values(&[("phrase", "hello")])).unwrap();
        assert_eq!(updated.values().get("phrase").map(String::as_str), Some("hello"));
        assert_eq!(f.service.list_drafts(f.deck_id, VOCABULARY).unwrap()[1].values().len(), 1);
        assert!(matches!(
            f.service.update_draft("nope", values(&[])),
            Err(FlashcardsError::NotFound(_))
        ));
    }

    #[test]
    fn duplicate_goes_right_after_the_source() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        f.service.update_draft(a.uuid(), values(&[("phrase", "one")])).unwrap();

        let copy = f.service.duplicate_draft(a.uuid()).unwrap();
        assert_ne!(copy.uuid(), a.uuid());
        assert_eq!(copy.values().get("phrase").map(String::as_str), Some("one"));
        let listed = f.service.list_drafts(f.deck_id, VOCABULARY).unwrap();
        assert_eq!(uuids(&listed), vec![a.uuid(), copy.uuid(), b.uuid()]);
    }

    #[test]
    fn moving_swaps_neighbours_and_stops_at_the_ends() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let c = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let order = |f: &Fixture| uuids(&f.service.list_drafts(f.deck_id, VOCABULARY).unwrap());

        f.service.move_draft(c.uuid(), Direction::Up).unwrap();
        assert_eq!(order(&f), vec![a.uuid(), c.uuid(), b.uuid()]);
        f.service.move_draft(a.uuid(), Direction::Up).unwrap();
        assert_eq!(order(&f), vec![a.uuid(), c.uuid(), b.uuid()], "first draft cannot go up");
        f.service.move_draft(b.uuid(), Direction::Down).unwrap();
        assert_eq!(order(&f), vec![a.uuid(), c.uuid(), b.uuid()], "last draft cannot go down");
        f.service.move_draft(a.uuid(), Direction::Down).unwrap();
        assert_eq!(order(&f), vec![c.uuid(), a.uuid(), b.uuid()]);
    }

    #[test]
    fn deleting_a_draft_keeps_the_others_in_order() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let c = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        f.service.delete_draft(b.uuid()).unwrap();
        assert_eq!(
            uuids(&f.service.list_drafts(f.deck_id, VOCABULARY).unwrap()),
            vec![a.uuid(), c.uuid()]
        );
        let d = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        assert_eq!(
            uuids(&f.service.list_drafts(f.deck_id, VOCABULARY).unwrap()),
            vec![a.uuid(), c.uuid(), d.uuid()]
        );
        assert!(matches!(f.service.delete_draft(b.uuid()), Err(FlashcardsError::NotFound(_))));
    }

    #[test]
    fn publishing_creates_flashcards_and_removes_the_drafts() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let items = vec![
            PublishedDraft { draft_uuid: a.uuid().into(), fields: vec![FieldValue::new(1, "one".into()).unwrap()] },
            PublishedDraft { draft_uuid: b.uuid().into(), fields: vec![FieldValue::new(1, "two".into()).unwrap()] },
        ];
        let published = f.service.publish_drafts(f.deck_id, VOCABULARY, items).unwrap();
        assert_eq!(published.len(), 2);
        assert_eq!(count(&f, "flashcards"), 2);
        assert!(f.service.list_drafts(f.deck_id, VOCABULARY).unwrap().is_empty());
    }

    #[test]
    fn a_failing_publish_changes_nothing() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let items = vec![
            PublishedDraft { draft_uuid: a.uuid().into(), fields: vec![FieldValue::new(1, "ok".into()).unwrap()] },
            PublishedDraft { draft_uuid: b.uuid().into(), fields: vec![FieldValue::new(999, "bad".into()).unwrap()] },
        ];
        assert!(f.service.publish_drafts(f.deck_id, VOCABULARY, items).is_err());
        assert_eq!(count(&f, "flashcards"), 0);
        assert_eq!(f.service.list_drafts(f.deck_id, VOCABULARY).unwrap().len(), 2);
    }

    #[test]
    fn deleting_a_deck_removes_its_flashcards_and_drafts() {
        let f = fixture();
        f.service.create_flashcard(f.deck_id, VOCABULARY, vec![FieldValue::new(1, "x".into()).unwrap()]).unwrap();
        f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        f.db.with_connection(|conn| conn.execute("DELETE FROM decks", []).map(|_| ())).unwrap();
        assert_eq!(count(&f, "flashcards"), 0);
        assert_eq!(count(&f, "flashcard_fields"), 0);
        assert_eq!(count(&f, "flashcard_drafts"), 0);
    }
}
