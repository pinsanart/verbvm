use std::collections::BTreeMap;
use std::sync::Arc;

use super::error::{FlashcardsError, FlashcardsResult};
use super::model::{
    Direction, FieldMedia, FieldValue, Flashcard, FlashcardDraft, MediaAttachment, TypeCount,
};
use super::repository::{FlashcardRepository, SqliteFlashcardRepository};
use crate::dependencies::db::connection::SqliteHandle;

/// A draft to publish and the field values that become its flashcard.
#[derive(Debug, Clone)]
pub struct PublishedDraft {
    pub draft_uuid: String,
    pub flashcard_type_id: i64,
    pub fields: Vec<FieldValue>,
    pub media: Vec<FieldMedia>,
    /// The drafts published together with the same value become flashcards linked by one stack.
    pub stack_uuid: Option<String>,
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
        media: Vec<FieldMedia>,
    ) -> FlashcardsResult<Flashcard>;
    fn get_flashcard(&self, uuid: &str) -> FlashcardsResult<Flashcard>;
    fn list_flashcards(&self, deck_id: i64) -> FlashcardsResult<Vec<Flashcard>>;
    fn delete_flashcard(&self, uuid: &str) -> FlashcardsResult<()>;
    /// How many flashcards of each type the deck has. Types with none are left out.
    fn count_by_type(&self, deck_id: i64) -> FlashcardsResult<Vec<TypeCount>>;

    /// Replaces the text of a saved flashcard (see `FlashcardRepository::update_flashcard_fields`).
    /// Its review history is untouched.
    fn update_flashcard_fields(&self, uuid: &str, fields: Vec<FieldValue>) -> FlashcardsResult<Flashcard>;
    /// Attaches a stored file to a field of a saved flashcard.
    fn attach_flashcard_media(&self, uuid: &str, media: FieldMedia) -> FlashcardsResult<Flashcard>;
    /// Detaches the file with this hash from a field of a saved flashcard.
    fn remove_flashcard_media(
        &self,
        uuid: &str,
        field_definition_id: i64,
        sha256: &str,
    ) -> FlashcardsResult<Flashcard>;

    /// Adds an empty draft of a type at the end of the deck's list.
    fn create_draft(&self, deck_id: i64, flashcard_type_id: i64) -> FlashcardsResult<FlashcardDraft>;
    /// Adds an empty draft that is part of the stack with this uuid. A stack is a set of drafts
    /// that the editor shows as one and that are linked when published.
    fn create_draft_in_stack(
        &self,
        deck_id: i64,
        flashcard_type_id: i64,
        stack_uuid: &str,
    ) -> FlashcardsResult<FlashcardDraft>;
    fn get_draft(&self, uuid: &str) -> FlashcardsResult<FlashcardDraft>;
    /// The drafts of a deck, of every type, in the order they are edited. The drafts of a stack
    /// come together, where its first draft is.
    fn list_drafts(&self, deck_id: i64) -> FlashcardsResult<Vec<FlashcardDraft>>;
    /// The drafts of a stack, in order.
    fn list_stack_drafts(&self, stack_uuid: &str) -> FlashcardsResult<Vec<FlashcardDraft>>;
    /// Takes a draft out of its stack. A stack left with fewer than two drafts is dissolved.
    fn unstack_draft(&self, uuid: &str) -> FlashcardsResult<FlashcardDraft>;
    /// Deletes every draft of a stack.
    fn delete_stack_drafts(&self, stack_uuid: &str) -> FlashcardsResult<()>;
    /// The flashcards linked by a stack, oldest first.
    fn list_stack_flashcards(&self, stack_uuid: &str) -> FlashcardsResult<Vec<Flashcard>>;
    /// Takes a flashcard out of its stack. A stack left with fewer than two flashcards is
    /// dissolved.
    fn unstack_flashcard(&self, uuid: &str) -> FlashcardsResult<Flashcard>;
    /// Deletes every flashcard of a stack, with their review history.
    fn delete_stack_flashcards(&self, stack_uuid: &str) -> FlashcardsResult<()>;
    /// Replaces the typed values of a draft.
    fn update_draft(&self, uuid: &str, values: BTreeMap<String, String>) -> FlashcardsResult<FlashcardDraft>;
    /// Copies a draft and puts the copy right after it. A draft in a stack copies the whole stack
    /// (the copy is a new stack) and returns the copy of that draft.
    fn duplicate_draft(&self, uuid: &str) -> FlashcardsResult<FlashcardDraft>;
    /// Moves a draft one place. A draft in a stack moves the whole stack past the neighbor draft
    /// or stack. Moving the first one up or the last one down changes nothing.
    fn move_draft(&self, uuid: &str, direction: Direction) -> FlashcardsResult<()>;
    /// Deletes only this draft. A stack left with fewer than two drafts is dissolved.
    fn delete_draft(&self, uuid: &str) -> FlashcardsResult<()>;
    /// Attaches a stored file to a field of a draft. The same file attached twice counts once.
    fn attach_draft_media(
        &self,
        uuid: &str,
        field_key: &str,
        media: MediaAttachment,
    ) -> FlashcardsResult<FlashcardDraft>;
    /// Detaches the file with this hash from a field of a draft.
    fn remove_draft_media(&self, uuid: &str, field_key: &str, sha256: &str) -> FlashcardsResult<FlashcardDraft>;

    /// Turns drafts into flashcards of one deck, each with its own type, and deletes the drafts,
    /// all or nothing.
    fn publish_drafts(&self, deck_id: i64, drafts: Vec<PublishedDraft>) -> FlashcardsResult<Vec<Flashcard>>;
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

    /// A deck's drafts as units: each draft on its own, and the drafts of a stack together, at the
    /// place of the first one.
    fn units(&self, deck_id: i64) -> FlashcardsResult<Vec<Vec<FlashcardDraft>>> {
        let mut units: Vec<Vec<FlashcardDraft>> = Vec::new();
        for draft in self.repository.find_drafts(deck_id)? {
            let existing = draft
                .stack_uuid()
                .and_then(|stack| units.iter().position(|u| u[0].stack_uuid() == Some(stack)));
            match existing {
                Some(index) => units[index].push(draft),
                None => units.push(vec![draft]),
            }
        }
        Ok(units)
    }

    /// A stack with fewer than two drafts is not a stack: its draft becomes a regular one.
    fn dissolve_if_small(&self, stack_uuid: &str) -> FlashcardsResult<()> {
        let members = self.repository.find_drafts_by_stack(stack_uuid)?;
        if members.len() < 2 {
            for member in members {
                self.repository.set_draft_stack(member.uuid(), None)?;
            }
        }
        Ok(())
    }
}

impl FlashcardApi for FlashcardService {
    fn create_flashcard(
        &self,
        deck_id: i64,
        flashcard_type_id: i64,
        fields: Vec<FieldValue>,
        media: Vec<FieldMedia>,
    ) -> FlashcardsResult<Flashcard> {
        let flashcard = Flashcard::new(deck_id, flashcard_type_id, fields, media)?;
        Ok(self.repository.insert_flashcard(&flashcard)?)
    }

    fn get_flashcard(&self, uuid: &str) -> FlashcardsResult<Flashcard> {
        self.repository.find_flashcard(uuid)?.ok_or(FlashcardsError::NotFound("flashcard"))
    }

    fn list_flashcards(&self, deck_id: i64) -> FlashcardsResult<Vec<Flashcard>> {
        Ok(self.repository.find_flashcards_by_deck(deck_id)?)
    }

    fn list_stack_flashcards(&self, stack_uuid: &str) -> FlashcardsResult<Vec<Flashcard>> {
        Ok(self.repository.find_flashcards_by_stack(stack_uuid)?)
    }

    fn unstack_flashcard(&self, uuid: &str) -> FlashcardsResult<Flashcard> {
        if !self.repository.set_flashcard_stack(uuid, None)? {
            return Err(FlashcardsError::NotFound("flashcard"));
        }
        self.get_flashcard(uuid)
    }

    fn delete_stack_flashcards(&self, stack_uuid: &str) -> FlashcardsResult<()> {
        if self.repository.delete_flashcards_in_stack(stack_uuid)? == 0 {
            return Err(FlashcardsError::NotFound("stack"));
        }
        Ok(())
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

    fn update_flashcard_fields(&self, uuid: &str, fields: Vec<FieldValue>) -> FlashcardsResult<Flashcard> {
        let current = self.get_flashcard(uuid)?;
        // Building the entity checks that no field is repeated.
        Flashcard::new(current.deck_id(), current.flashcard_type_id(), fields.clone(), vec![])?;
        if !self.repository.update_flashcard_fields(uuid, &fields)? {
            return Err(FlashcardsError::NotFound("flashcard"));
        }
        self.get_flashcard(uuid)
    }

    fn attach_flashcard_media(&self, uuid: &str, media: FieldMedia) -> FlashcardsResult<Flashcard> {
        if !self.repository.add_flashcard_media(uuid, &media)? {
            return Err(FlashcardsError::NotFound("flashcard"));
        }
        self.get_flashcard(uuid)
    }

    fn remove_flashcard_media(
        &self,
        uuid: &str,
        field_definition_id: i64,
        sha256: &str,
    ) -> FlashcardsResult<Flashcard> {
        self.get_flashcard(uuid)?;
        if !self.repository.remove_flashcard_media(uuid, field_definition_id, sha256)? {
            return Err(FlashcardsError::NotFound("media"));
        }
        self.get_flashcard(uuid)
    }

    fn create_draft(&self, deck_id: i64, flashcard_type_id: i64) -> FlashcardsResult<FlashcardDraft> {
        let draft = FlashcardDraft::new(deck_id, flashcard_type_id);
        Ok(self.repository.insert_draft(&draft)?)
    }

    fn create_draft_in_stack(
        &self,
        deck_id: i64,
        flashcard_type_id: i64,
        stack_uuid: &str,
    ) -> FlashcardsResult<FlashcardDraft> {
        let draft = FlashcardDraft::new(deck_id, flashcard_type_id).in_stack(Some(stack_uuid.to_owned()));
        Ok(self.repository.insert_draft(&draft)?)
    }

    fn get_draft(&self, uuid: &str) -> FlashcardsResult<FlashcardDraft> {
        self.draft(uuid)
    }

    fn list_drafts(&self, deck_id: i64) -> FlashcardsResult<Vec<FlashcardDraft>> {
        Ok(self.units(deck_id)?.into_iter().flatten().collect())
    }

    fn list_stack_drafts(&self, stack_uuid: &str) -> FlashcardsResult<Vec<FlashcardDraft>> {
        Ok(self.repository.find_drafts_by_stack(stack_uuid)?)
    }

    fn unstack_draft(&self, uuid: &str) -> FlashcardsResult<FlashcardDraft> {
        let draft = self.draft(uuid)?;
        if let Some(stack) = draft.stack_uuid() {
            self.repository.set_draft_stack(uuid, None)?;
            self.dissolve_if_small(stack)?;
        }
        self.draft(uuid)
    }

    fn delete_stack_drafts(&self, stack_uuid: &str) -> FlashcardsResult<()> {
        if self.repository.delete_drafts_in_stack(stack_uuid)? == 0 {
            return Err(FlashcardsError::NotFound("stack"));
        }
        Ok(())
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
        let draft = self.draft(uuid)?;
        let Some(stack) = draft.stack_uuid() else {
            let copy = draft.duplicate();
            return self
                .repository
                .insert_draft_after(uuid, &copy)?
                .ok_or(FlashcardsError::NotFound("draft"));
        };

        // A stack is copied whole, as a new stack, right after its last draft.
        let members = self.repository.find_drafts_by_stack(stack)?;
        let new_stack = uuid::Uuid::new_v4().to_string();
        let mut after = members.last().map_or(uuid.to_owned(), |m| m.uuid().to_owned());
        let mut copy_of_requested = None;
        for member in &members {
            let copy = member.duplicate().in_stack(Some(new_stack.clone()));
            let saved = self
                .repository
                .insert_draft_copy(member.uuid(), &after, &copy)?
                .ok_or(FlashcardsError::NotFound("draft"))?;
            after = saved.uuid().to_owned();
            if member.uuid() == uuid {
                copy_of_requested = Some(saved);
            }
        }
        copy_of_requested.ok_or(FlashcardsError::NotFound("draft"))
    }

    fn move_draft(&self, uuid: &str, direction: Direction) -> FlashcardsResult<()> {
        let draft = self.draft(uuid)?;
        let mut units = self.units(draft.deck_id())?;
        let index = units
            .iter()
            .position(|unit| unit.iter().any(|d| d.uuid() == uuid))
            .ok_or(FlashcardsError::NotFound("draft"))?;
        let neighbor = match direction {
            Direction::Up => index.checked_sub(1),
            Direction::Down => Some(index + 1).filter(|i| *i < units.len()),
        };
        if let Some(neighbor) = neighbor {
            units.swap(index, neighbor);
            let order: Vec<String> = units.iter().flatten().map(|d| d.uuid().to_owned()).collect();
            self.repository.reorder_drafts(draft.deck_id(), &order)?;
        }
        Ok(())
    }

    fn delete_draft(&self, uuid: &str) -> FlashcardsResult<()> {
        let draft = self.draft(uuid)?;
        if !self.repository.delete_draft(uuid)? {
            return Err(FlashcardsError::NotFound("draft"));
        }
        if let Some(stack) = draft.stack_uuid() {
            self.dissolve_if_small(stack)?;
        }
        Ok(())
    }

    fn attach_draft_media(
        &self,
        uuid: &str,
        field_key: &str,
        media: MediaAttachment,
    ) -> FlashcardsResult<FlashcardDraft> {
        if !self.repository.add_draft_media(uuid, field_key, &media)? {
            return Err(FlashcardsError::NotFound("draft"));
        }
        self.draft(uuid)
    }

    fn remove_draft_media(&self, uuid: &str, field_key: &str, sha256: &str) -> FlashcardsResult<FlashcardDraft> {
        self.draft(uuid)?;
        if !self.repository.remove_draft_media(uuid, field_key, sha256)? {
            return Err(FlashcardsError::NotFound("media"));
        }
        self.draft(uuid)
    }

    fn publish_drafts(&self, deck_id: i64, drafts: Vec<PublishedDraft>) -> FlashcardsResult<Vec<Flashcard>> {
        let mut flashcards = Vec::with_capacity(drafts.len());
        let mut draft_uuids = Vec::with_capacity(drafts.len());
        for draft in drafts {
            flashcards.push(
                Flashcard::new(deck_id, draft.flashcard_type_id, draft.fields, draft.media)?
                    .in_stack(draft.stack_uuid),
            );
            draft_uuids.push(draft.draft_uuid);
        }
        Ok(self.repository.publish(&flashcards, &draft_uuids)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependencies::media_store::MediaKind;

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
        let created = f.service.create_flashcard(f.deck_id, VOCABULARY, fields.clone(), vec![]).unwrap();
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
            f.service.create_flashcard(99, VOCABULARY, vec![], vec![]),
            Err(FlashcardsError::Conflict(_))
        ));
        let bad_field = vec![FieldValue::new(999, "x".into()).unwrap()];
        assert!(matches!(
            f.service.create_flashcard(f.deck_id, VOCABULARY, bad_field, vec![]),
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
        let listed = f.service.list_drafts(f.deck_id).unwrap();
        assert_eq!(uuids(&listed), vec![a.uuid(), b.uuid(), c.uuid()]);

        let updated = f.service.update_draft(b.uuid(), values(&[("phrase", "hello")])).unwrap();
        assert_eq!(updated.values().get("phrase").map(String::as_str), Some("hello"));
        assert_eq!(f.service.list_drafts(f.deck_id).unwrap()[1].values().len(), 1);
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
        let listed = f.service.list_drafts(f.deck_id).unwrap();
        assert_eq!(uuids(&listed), vec![a.uuid(), copy.uuid(), b.uuid()]);
    }

    #[test]
    fn moving_swaps_neighbours_and_stops_at_the_ends() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let c = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let order = |f: &Fixture| uuids(&f.service.list_drafts(f.deck_id).unwrap());

        f.service.move_draft(c.uuid(), Direction::Up).unwrap();
        assert_eq!(order(&f), vec![a.uuid(), c.uuid(), b.uuid()]);
        f.service.move_draft(a.uuid(), Direction::Up).unwrap();
        assert_eq!(order(&f), vec![a.uuid(), c.uuid(), b.uuid()], "first draft cannot go up");
        f.service.move_draft(b.uuid(), Direction::Down).unwrap();
        assert_eq!(order(&f), vec![a.uuid(), c.uuid(), b.uuid()], "last draft cannot go down");
        f.service.move_draft(a.uuid(), Direction::Down).unwrap();
        assert_eq!(order(&f), vec![c.uuid(), a.uuid(), b.uuid()]);
    }

    // From the migration seed: type 2 is phonetic, with field definitions 4 (phrase) and 5 (ipa_phonetics).
    const PHONETIC: i64 = 2;

    #[test]
    fn drafts_of_different_types_share_one_ordered_list() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, PHONETIC).unwrap();
        let c = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let order = |f: &Fixture| uuids(&f.service.list_drafts(f.deck_id).unwrap());
        assert_eq!(order(&f), vec![a.uuid(), b.uuid(), c.uuid()]);

        f.service.move_draft(c.uuid(), Direction::Up).unwrap();
        assert_eq!(order(&f), vec![a.uuid(), c.uuid(), b.uuid()], "moving crosses types");
        let copy = f.service.duplicate_draft(b.uuid()).unwrap();
        assert_eq!(copy.flashcard_type_id(), PHONETIC);
        assert_eq!(order(&f), vec![a.uuid(), c.uuid(), b.uuid(), copy.uuid()]);
        f.service.delete_draft(a.uuid()).unwrap();
        assert_eq!(order(&f), vec![c.uuid(), b.uuid(), copy.uuid()]);
    }

    const ORTHOGRAPHIC: i64 = 3;

    fn stack_of_three(f: &Fixture, stack: &str) -> Vec<FlashcardDraft> {
        [VOCABULARY, PHONETIC, ORTHOGRAPHIC]
            .into_iter()
            .map(|t| f.service.create_draft_in_stack(f.deck_id, t, stack).unwrap())
            .collect()
    }

    #[test]
    fn the_drafts_of_a_stack_come_together_where_the_first_one_is() {
        let f = fixture();
        let stack = stack_of_three(&f, "s1");
        let single = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        // Added later, so it is stored after the single draft, but it is listed with its stack.
        let late = f.service.create_draft_in_stack(f.deck_id, VOCABULARY, "s1").unwrap();
        let listed = uuids(&f.service.list_drafts(f.deck_id).unwrap());
        let expected: Vec<String> = stack.iter().chain([&late, &single]).map(|d| d.uuid().to_owned()).collect();
        assert_eq!(listed, expected);
        assert_eq!(f.service.list_stack_drafts("s1").unwrap().len(), 4);
    }

    #[test]
    fn moving_a_draft_of_a_stack_moves_the_whole_stack() {
        let f = fixture();
        let before = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let stack = stack_of_three(&f, "s1");
        let after = f.service.create_draft(f.deck_id, PHONETIC).unwrap();

        f.service.move_draft(stack[1].uuid(), Direction::Up).unwrap();
        let order = uuids(&f.service.list_drafts(f.deck_id).unwrap());
        let mut expected: Vec<String> = stack.iter().map(|d| d.uuid().to_owned()).collect();
        expected.extend([before.uuid().to_owned(), after.uuid().to_owned()]);
        assert_eq!(order, expected, "the stack went past the draft above it");

        f.service.move_draft(stack[0].uuid(), Direction::Up).unwrap();
        assert_eq!(uuids(&f.service.list_drafts(f.deck_id).unwrap()), expected, "already first");

        f.service.move_draft(before.uuid(), Direction::Up).unwrap();
        let order = uuids(&f.service.list_drafts(f.deck_id).unwrap());
        assert_eq!(order[0], before.uuid(), "a single draft moves past a whole stack as one step");
        assert_eq!(order[1..4], expected[0..3], "and the stack is not split");
    }

    #[test]
    fn duplicating_a_stack_copies_all_of_it_as_a_new_stack() {
        let f = fixture();
        let stack = stack_of_three(&f, "s1");
        f.service
            .update_draft(stack[0].uuid(), values(&[("phrase", "hello")]))
            .unwrap();
        let copy = f.service.duplicate_draft(stack[1].uuid()).unwrap();
        assert_eq!(copy.flashcard_type_id(), PHONETIC, "the copy of the draft that was asked for");

        let drafts = f.service.list_drafts(f.deck_id).unwrap();
        assert_eq!(drafts.len(), 6);
        let new_stack = copy.stack_uuid().unwrap().to_owned();
        assert_ne!(new_stack, "s1");
        let copies: Vec<_> = drafts.iter().filter(|d| d.stack_uuid() == Some(new_stack.as_str())).collect();
        assert_eq!(copies.iter().map(|d| d.flashcard_type_id()).collect::<Vec<_>>(), vec![VOCABULARY, PHONETIC, ORTHOGRAPHIC]);
        assert_eq!(copies[0].values()["phrase"], "hello");
        assert_eq!(drafts[3].stack_uuid(), Some(new_stack.as_str()), "right after the original");
    }

    #[test]
    fn deleting_or_taking_out_one_draft_leaves_the_rest_and_dissolves_a_small_stack() {
        let f = fixture();
        let stack = stack_of_three(&f, "s1");

        f.service.delete_draft(stack[2].uuid()).unwrap();
        assert_eq!(f.service.list_stack_drafts("s1").unwrap().len(), 2, "two are still a stack");

        let taken = f.service.unstack_draft(stack[0].uuid()).unwrap();
        assert!(taken.stack_uuid().is_none());
        let left = f.service.get_draft(stack[1].uuid()).unwrap();
        assert!(left.stack_uuid().is_none(), "one draft alone is not a stack");
        assert_eq!(f.service.list_drafts(f.deck_id).unwrap().len(), 2, "nothing was deleted by taking out");
    }

    #[test]
    fn a_whole_stack_of_drafts_can_be_deleted() {
        let f = fixture();
        stack_of_three(&f, "s1");
        let single = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        f.service.delete_stack_drafts("s1").unwrap();
        assert_eq!(uuids(&f.service.list_drafts(f.deck_id).unwrap()), vec![single.uuid().to_owned()]);
        assert!(matches!(f.service.delete_stack_drafts("s1"), Err(FlashcardsError::NotFound(_))));
    }

    fn published_stack(f: &Fixture) -> Vec<Flashcard> {
        let stack = stack_of_three(f, "s1");
        let items = stack
            .iter()
            .zip([1_i64, 4, 6])
            .map(|(draft, field)| PublishedDraft {
                draft_uuid: draft.uuid().into(),
                flashcard_type_id: draft.flashcard_type_id(),
                fields: vec![FieldValue::new(field, "x".into()).unwrap()],
                media: vec![],
                stack_uuid: draft.stack_uuid().map(str::to_owned),
            })
            .collect();
        f.service.publish_drafts(f.deck_id, items).unwrap()
    }

    #[test]
    fn publishing_a_stack_makes_independent_flashcards_linked_by_one_stack() {
        let f = fixture();
        let cards = published_stack(&f);
        assert!(cards.iter().all(|c| c.stack_uuid() == Some("s1")));
        assert_eq!(count(&f, "stacks"), 1);
        assert_eq!(count(&f, "flashcards"), 3);
        assert!(f.service.list_drafts(f.deck_id).unwrap().is_empty());

        let listed = f.service.list_flashcards(f.deck_id).unwrap();
        assert!(listed.iter().all(|c| c.stack_uuid() == Some("s1")), "the link is read back");
        assert_eq!(f.service.list_stack_flashcards("s1").unwrap().len(), 3);
        assert!(f.service.get_flashcard(cards[0].uuid()).unwrap().stack_uuid().is_some());
    }

    #[test]
    fn a_flashcard_can_leave_its_stack_and_a_small_stack_dissolves() {
        let f = fixture();
        let cards = published_stack(&f);

        f.service.delete_flashcard(cards[2].uuid()).unwrap();
        assert_eq!(f.service.list_stack_flashcards("s1").unwrap().len(), 2, "two are still a stack");

        let free = f.service.unstack_flashcard(cards[0].uuid()).unwrap();
        assert!(free.stack_uuid().is_none());
        assert!(f.service.get_flashcard(cards[1].uuid()).unwrap().stack_uuid().is_none());
        assert_eq!(count(&f, "stacks"), 0, "the stack row goes with its last link");
        assert_eq!(count(&f, "flashcards"), 2);
    }

    #[test]
    fn a_whole_stack_of_flashcards_can_be_deleted() {
        let f = fixture();
        published_stack(&f);
        f.service.delete_stack_flashcards("s1").unwrap();
        assert_eq!(count(&f, "flashcards"), 0);
        assert_eq!(count(&f, "stacks"), 0);
        assert!(matches!(f.service.delete_stack_flashcards("s1"), Err(FlashcardsError::NotFound(_))));
    }

    #[test]
    fn publishing_creates_each_flashcard_with_the_type_of_its_draft() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, PHONETIC).unwrap();
        let items = vec![
            PublishedDraft { draft_uuid: a.uuid().into(), flashcard_type_id: VOCABULARY, fields: vec![FieldValue::new(1, "one".into()).unwrap()], media: vec![], stack_uuid: None },
            PublishedDraft { draft_uuid: b.uuid().into(), flashcard_type_id: PHONETIC, fields: vec![FieldValue::new(4, "two".into()).unwrap()], media: vec![], stack_uuid: None },
        ];
        let published = f.service.publish_drafts(f.deck_id, items).unwrap();
        assert_eq!(
            published.iter().map(|c| c.flashcard_type_id()).collect::<Vec<_>>(),
            vec![VOCABULARY, PHONETIC]
        );
        let counts = f.service.count_by_type(f.deck_id).unwrap();
        assert_eq!(counts.len(), 2);
        assert!(f.service.list_drafts(f.deck_id).unwrap().is_empty());
    }

    #[test]
    fn deleting_a_draft_keeps_the_others_in_order() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let c = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        f.service.delete_draft(b.uuid()).unwrap();
        assert_eq!(
            uuids(&f.service.list_drafts(f.deck_id).unwrap()),
            vec![a.uuid(), c.uuid()]
        );
        let d = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        assert_eq!(
            uuids(&f.service.list_drafts(f.deck_id).unwrap()),
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
            PublishedDraft { draft_uuid: a.uuid().into(), flashcard_type_id: VOCABULARY, fields: vec![FieldValue::new(1, "one".into()).unwrap()], media: vec![], stack_uuid: None },
            PublishedDraft { draft_uuid: b.uuid().into(), flashcard_type_id: VOCABULARY, fields: vec![FieldValue::new(1, "two".into()).unwrap()], media: vec![], stack_uuid: None },
        ];
        let published = f.service.publish_drafts(f.deck_id, items).unwrap();
        assert_eq!(published.len(), 2);
        assert_eq!(count(&f, "flashcards"), 2);
        assert!(f.service.list_drafts(f.deck_id).unwrap().is_empty());
    }

    #[test]
    fn a_failing_publish_changes_nothing() {
        let f = fixture();
        let a = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let b = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let items = vec![
            PublishedDraft { draft_uuid: a.uuid().into(), flashcard_type_id: VOCABULARY, fields: vec![FieldValue::new(1, "ok".into()).unwrap()], media: vec![], stack_uuid: None },
            PublishedDraft { draft_uuid: b.uuid().into(), flashcard_type_id: VOCABULARY, fields: vec![FieldValue::new(999, "bad".into()).unwrap()], media: vec![], stack_uuid: None },
        ];
        assert!(f.service.publish_drafts(f.deck_id, items).is_err());
        assert_eq!(count(&f, "flashcards"), 0);
        assert_eq!(f.service.list_drafts(f.deck_id).unwrap().len(), 2);
    }

    #[test]
    fn deleting_a_deck_removes_its_flashcards_and_drafts() {
        let f = fixture();
        f.service.create_flashcard(f.deck_id, VOCABULARY, vec![FieldValue::new(1, "x".into()).unwrap()], vec![]).unwrap();
        f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        f.db.with_connection(|conn| conn.execute("DELETE FROM decks", []).map(|_| ())).unwrap();
        assert_eq!(count(&f, "flashcards"), 0);
        assert_eq!(count(&f, "flashcard_fields"), 0);
        assert_eq!(count(&f, "flashcard_drafts"), 0);
    }

    fn media(fill: char, kind: MediaKind) -> MediaAttachment {
        let sha256: String = std::iter::repeat(fill).take(64).collect();
        let (mime, ext) = match kind {
            MediaKind::Image => ("image/png", "png"),
            MediaKind::Audio => ("audio/mpeg", "mp3"),
            MediaKind::Video => ("video/mp4", "mp4"),
        };
        MediaAttachment::new(kind, mime.into(), sha256.clone(), format!("{sha256}.{ext}"), 12).unwrap()
    }

    #[test]
    fn media_attached_to_a_draft_keeps_its_order_and_ignores_repeats() {
        let f = fixture();
        let draft = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        f.service.attach_draft_media(draft.uuid(), "phrase", media('a', MediaKind::Image)).unwrap();
        f.service.attach_draft_media(draft.uuid(), "phrase", media('b', MediaKind::Audio)).unwrap();
        let updated = f.service.attach_draft_media(draft.uuid(), "phrase", media('a', MediaKind::Image)).unwrap();
        let attached = &updated.media()["phrase"];
        assert_eq!(attached.iter().map(|m| m.kind()).collect::<Vec<_>>(), vec![MediaKind::Image, MediaKind::Audio]);
        assert_eq!(attached[0].position(), 0);
        assert_eq!(attached[1].position(), 1);

        let listed = f.service.list_drafts(f.deck_id).unwrap();
        assert_eq!(listed[0].media()["phrase"].len(), 2, "list_drafts includes the media");

        let after_remove = f.service.remove_draft_media(draft.uuid(), "phrase", &"a".repeat(64)).unwrap();
        assert_eq!(after_remove.media()["phrase"].len(), 1);
        assert!(matches!(
            f.service.remove_draft_media(draft.uuid(), "phrase", &"a".repeat(64)),
            Err(FlashcardsError::NotFound(_))
        ));
        assert!(matches!(
            f.service.attach_draft_media("nope", "phrase", media('c', MediaKind::Image)),
            Err(FlashcardsError::NotFound(_))
        ));
    }

    #[test]
    fn media_attachments_validate_their_hash() {
        assert!(MediaAttachment::new(MediaKind::Image, "image/png".into(), "xyz".into(), "x.png".into(), 1).is_err());
        assert!(MediaAttachment::new(MediaKind::Image, "image/png".into(), "a".repeat(64), "".into(), 1).is_err());
    }

    #[test]
    fn duplicating_a_draft_copies_its_media() {
        let f = fixture();
        let draft = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        f.service.attach_draft_media(draft.uuid(), "phrase", media('a', MediaKind::Image)).unwrap();
        let copy = f.service.duplicate_draft(draft.uuid()).unwrap();
        assert_eq!(copy.media()["phrase"].len(), 1);
        f.service.delete_draft(draft.uuid()).unwrap();
        assert_eq!(f.service.get_draft(copy.uuid()).unwrap().media()["phrase"].len(), 1, "the copy keeps its own rows");
    }

    #[test]
    fn publishing_stores_media_even_when_it_is_the_only_content_of_a_field() {
        let f = fixture();
        let draft = f.service.create_draft(f.deck_id, VOCABULARY).unwrap();
        let item = PublishedDraft {
            draft_uuid: draft.uuid().into(),
            flashcard_type_id: VOCABULARY,
            fields: vec![FieldValue::new(2, "word".into()).unwrap()],
            media: vec![FieldMedia { field_definition_id: 1, attachment: media('a', MediaKind::Image) }],
            stack_uuid: None,
        };
        let published = f.service.publish_drafts(f.deck_id, vec![item]).unwrap();

        let card = f.service.get_flashcard(published[0].uuid()).unwrap();
        assert_eq!(card.media().len(), 1);
        assert_eq!(card.media()[0].field_definition_id, 1);
        assert_eq!(card.media()[0].attachment.kind(), MediaKind::Image);
        let phrase = card.fields().iter().find(|v| v.field_definition_id() == 1).unwrap();
        assert_eq!(phrase.content(), "", "a field with only media has empty text");
        assert_eq!(f.service.list_flashcards(f.deck_id).unwrap()[0].media().len(), 1);
        assert_eq!(count(&f, "field_media"), 1);
        assert_eq!(count(&f, "flashcard_draft_media"), 0, "the draft and its media rows are gone");

        f.service.delete_flashcard(card.uuid()).unwrap();
        assert_eq!(count(&f, "field_media"), 0);
    }

    #[test]
    fn updating_a_saved_flashcard_replaces_its_text_and_keeps_its_identity() {
        let f = fixture();
        let created = f
            .service
            .create_flashcard(
                f.deck_id,
                VOCABULARY,
                vec![FieldValue::new(1, "old".into()).unwrap(), FieldValue::new(2, "word".into()).unwrap()],
                vec![],
            )
            .unwrap();
        let updated = f
            .service
            .update_flashcard_fields(
                created.uuid(),
                vec![FieldValue::new(1, "new".into()).unwrap(), FieldValue::new(3, "meaning".into()).unwrap()],
            )
            .unwrap();
        assert_eq!((updated.id(), updated.uuid()), (created.id(), created.uuid()));
        let texts: Vec<_> = updated.fields().iter().map(|v| (v.field_definition_id(), v.content())).collect();
        assert_eq!(texts, vec![(1, "new"), (3, "meaning")], "field 2 was cleared");
        assert!(updated.updated_at().is_some());
        assert!(matches!(
            f.service.update_flashcard_fields("nope", vec![]),
            Err(FlashcardsError::NotFound(_))
        ));
        assert!(matches!(
            f.service.update_flashcard_fields(
                created.uuid(),
                vec![FieldValue::new(1, "a".into()).unwrap(), FieldValue::new(1, "b".into()).unwrap()]
            ),
            Err(FlashcardsError::Invalid { .. })
        ));
    }

    #[test]
    fn media_of_a_saved_flashcard_can_be_added_and_removed() {
        let f = fixture();
        let card = f
            .service
            .create_flashcard(f.deck_id, VOCABULARY, vec![FieldValue::new(2, "word".into()).unwrap()], vec![])
            .unwrap();
        let image = || FieldMedia { field_definition_id: 1, attachment: media('a', MediaKind::Image) };

        let with = f.service.attach_flashcard_media(card.uuid(), image()).unwrap();
        assert_eq!(with.media().len(), 1);
        assert_eq!(f.service.attach_flashcard_media(card.uuid(), image()).unwrap().media().len(), 1, "the same file counts once");
        assert!(with.fields().iter().any(|v| v.field_definition_id() == 1 && v.content().is_empty()));

        // Clearing the text keeps a field that still has media.
        let cleared = f.service.update_flashcard_fields(card.uuid(), vec![]).unwrap();
        assert_eq!(cleared.media().len(), 1);
        assert_eq!(cleared.fields().len(), 1);

        let without = f.service.remove_flashcard_media(card.uuid(), 1, &"a".repeat(64)).unwrap();
        assert!(without.media().is_empty());
        assert!(without.fields().is_empty(), "a field with nothing left is dropped");
        assert!(matches!(
            f.service.remove_flashcard_media(card.uuid(), 1, &"a".repeat(64)),
            Err(FlashcardsError::NotFound(_))
        ));
        assert_eq!(count(&f, "field_media"), 0);
    }
}
