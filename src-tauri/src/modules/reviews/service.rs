use std::sync::Arc;

use super::error::ReviewsResult;
use super::model::{CardStats, Rating, ReviewLog, Sm2State};
use super::repository::{ReviewRepository, SqliteReviewRepository};
use crate::dependencies::clock::Clock;
use crate::dependencies::db::connection::SqliteHandle;

/// Public API of the reviews module. Other modules depend on this trait only.
pub trait ReviewApi: Send + Sync {
    /// Records a review made now: updates the SM-2 state and the stats and appends to the
    /// history, all or nothing. The first review of a flashcard creates its state.
    /// Conflicts when the flashcard does not exist.
    fn record_review(
        &self,
        flashcard_id: i64,
        rating: Rating,
        duration_ms: Option<i64>,
    ) -> ReviewsResult<Sm2State>;

    /// Flashcards due now or earlier, oldest first. Never-reviewed flashcards have no state
    /// and are not included.
    fn due(&self, limit: u32) -> ReviewsResult<Vec<Sm2State>>;

    /// The states of the given flashcards. Flashcards never reviewed are left out.
    fn states_for(&self, flashcard_ids: &[i64]) -> ReviewsResult<Vec<Sm2State>>;

    /// The counters of a flashcard, all zero if it was never reviewed.
    fn stats(&self, flashcard_id: i64) -> ReviewsResult<CardStats>;

    /// The flashcard's review history, oldest first.
    fn history(&self, flashcard_id: i64) -> ReviewsResult<Vec<ReviewLog>>;
}

pub struct ReviewService {
    repository: Arc<dyn ReviewRepository>,
    clock: Arc<dyn Clock>,
}

impl ReviewService {
    pub fn new(repository: Arc<dyn ReviewRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { repository, clock }
    }

    /// Service backed by the SQLite repository.
    pub fn sqlite(db: Arc<SqliteHandle>, clock: Arc<dyn Clock>) -> Self {
        Self::new(Arc::new(SqliteReviewRepository::new(db)), clock)
    }
}

impl ReviewApi for ReviewService {
    fn record_review(
        &self,
        flashcard_id: i64,
        rating: Rating,
        duration_ms: Option<i64>,
    ) -> ReviewsResult<Sm2State> {
        let now = self.clock.now();
        let mut state = self
            .repository
            .find_state(flashcard_id)?
            .unwrap_or_else(|| Sm2State::initial(flashcard_id, now));
        let mut stats = self
            .repository
            .find_stats(flashcard_id)?
            .unwrap_or_else(|| CardStats::empty(flashcard_id));

        let elapsed_days = state.elapsed_days(now);
        let had_successful_repetition = state.repetitions() > 0;

        state.review(rating, now);
        let log = ReviewLog::new(
            flashcard_id,
            now,
            rating,
            elapsed_days,
            state.interval_days(),
            duration_ms,
        )?;
        stats.record(rating, duration_ms, had_successful_repetition);

        self.repository.save_review(&state, &log, &stats)?;
        Ok(state)
    }

    fn due(&self, limit: u32) -> ReviewsResult<Vec<Sm2State>> {
        Ok(self.repository.find_due(self.clock.now(), limit)?)
    }

    fn states_for(&self, flashcard_ids: &[i64]) -> ReviewsResult<Vec<Sm2State>> {
        Ok(self.repository.find_states(flashcard_ids)?)
    }

    fn stats(&self, flashcard_id: i64) -> ReviewsResult<CardStats> {
        Ok(self
            .repository
            .find_stats(flashcard_id)?
            .unwrap_or_else(|| CardStats::empty(flashcard_id)))
    }

    fn history(&self, flashcard_id: i64) -> ReviewsResult<Vec<ReviewLog>> {
        Ok(self.repository.history(flashcard_id)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependencies::clock::{parse_utc, FixedClock};
    use crate::modules::reviews::ReviewsError;

    struct Fixture {
        service: ReviewService,
        repository: SqliteReviewRepository,
        clock: Arc<FixedClock>,
        db: Arc<SqliteHandle>,
        path: std::path::PathBuf,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{suffix}", self.path.display()));
            }
        }
    }

    /// A database with two flashcards (ids 1 and 2). They are seeded with raw SQL because this
    /// module does not own those tables. The languages and card types come from the migration.
    fn fixture() -> Fixture {
        let path = std::env::temp_dir().join(format!(
            "verbvm-reviews-{}-{:?}.db",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_file(&path);
        let db = Arc::new(SqliteHandle::new(&path).unwrap());
        db.with_connection(|conn| {
            conn.execute_batch(
                "INSERT INTO decks (name, language_variant_id) VALUES ('Deck', 1);
                 INSERT INTO flashcards (uuid, deck_id, flashcard_type_id)
                 VALUES ('card-1', 1, 1), ('card-2', 1, 1);",
            )
        })
        .unwrap();
        let clock = Arc::new(FixedClock::at("2026-10-12 08:00:00"));
        Fixture {
            service: ReviewService::sqlite(db.clone(), clock.clone()),
            repository: SqliteReviewRepository::new(db.clone()),
            clock,
            db,
            path,
        }
    }

    #[test]
    fn first_review_creates_state_log_and_stats() {
        let f = fixture();
        let state = f.service.record_review(1, Rating::Good, Some(1_500)).unwrap();
        assert_eq!(state.interval_days(), 1.0);
        assert_eq!(state.due_at(), parse_utc("2026-10-13 08:00:00").unwrap());

        let history = f.service.history(1).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].elapsed_days(), None);
        assert_eq!(history[0].scheduled_days(), 1.0);
        assert_eq!(history[0].duration_ms(), Some(1_500));

        let stats = f.service.stats(1).unwrap();
        assert_eq!((stats.review_count(), stats.correct_count(), stats.total_duration_ms()), (1, 1, 1_500));
        assert_eq!(f.service.states_for(&[1, 2]).unwrap().len(), 1);
    }

    #[test]
    fn second_review_uses_elapsed_days_and_updates_the_same_rows() {
        let f = fixture();
        f.service.record_review(1, Rating::Good, None).unwrap();
        f.clock.set("2026-10-14 20:00:00");
        let state = f.service.record_review(1, Rating::Good, None).unwrap();
        assert_eq!(state.interval_days(), 6.0);

        let history = f.service.history(1).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].elapsed_days(), Some(2.5));
        assert_eq!(f.service.stats(1).unwrap().review_count(), 2);
    }

    #[test]
    fn lapse_is_counted_only_after_a_successful_repetition() {
        let f = fixture();
        f.service.record_review(1, Rating::Again, None).unwrap();
        assert_eq!(f.service.stats(1).unwrap().lapses(), 0);
        f.service.record_review(1, Rating::Good, None).unwrap();
        f.service.record_review(1, Rating::Again, None).unwrap();
        assert_eq!(f.service.stats(1).unwrap().lapses(), 1);
    }

    #[test]
    fn due_lists_reviewed_cards_oldest_first_and_honours_the_limit() {
        let f = fixture();
        f.service.record_review(1, Rating::Good, None).unwrap(); // due 2026-10-13 08:00
        f.clock.set("2026-10-12 09:00:00");
        f.service.record_review(2, Rating::Again, None).unwrap(); // due 2026-10-12 09:10

        assert!(f.service.due(10).unwrap().is_empty());

        f.clock.set("2026-10-13 12:00:00");
        let due = f.service.due(10).unwrap();
        assert_eq!(due.iter().map(|s| s.flashcard_id()).collect::<Vec<_>>(), vec![2, 1]);
        assert_eq!(f.service.due(1).unwrap().len(), 1);
    }

    #[test]
    fn reviewing_a_missing_flashcard_conflicts_and_stores_nothing() {
        let f = fixture();
        assert!(matches!(
            f.service.record_review(99, Rating::Good, None),
            Err(ReviewsError::Conflict(_))
        ));
        assert!(f.service.history(99).unwrap().is_empty());
        assert_eq!(f.service.stats(99).unwrap().review_count(), 0);
    }

    #[test]
    fn rejects_a_negative_duration() {
        let f = fixture();
        assert!(matches!(
            f.service.record_review(1, Rating::Good, Some(-5)),
            Err(ReviewsError::Invalid { .. })
        ));
        assert!(f.service.states_for(&[1]).unwrap().is_empty());
    }

    #[test]
    fn a_failing_log_insert_rolls_back_the_state_and_stats() {
        let f = fixture();
        let now = crate::dependencies::clock::Clock::now(f.clock.as_ref());
        let state = Sm2State::initial(1, now);
        let stats = CardStats::empty(1);
        let log_for_missing_card = ReviewLog::new(99, now, Rating::Good, None, 1.0, None).unwrap();

        assert!(f.repository.save_review(&state, &log_for_missing_card, &stats).is_err());
        assert!(f.repository.find_state(1).unwrap().is_none());
        assert!(f.repository.find_stats(1).unwrap().is_none());
    }

    #[test]
    fn deleting_a_flashcard_removes_its_review_data() {
        let f = fixture();
        f.service.record_review(1, Rating::Good, None).unwrap();
        f.db.with_connection(|conn| conn.execute("DELETE FROM flashcards WHERE id = 1", []).map(|_| ()))
            .unwrap();
        assert!(f.service.history(1).unwrap().is_empty());
        assert!(f.service.states_for(&[1]).unwrap().is_empty());
        assert_eq!(f.service.stats(1).unwrap().review_count(), 0);
    }
}
