use chrono::{DateTime, Duration, Utc};

use super::error::{ReviewsError, ReviewsResult};

/// How well the card was recalled, from worst to best.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl Rating {
    pub fn as_i64(self) -> i64 {
        self as i64
    }
}

impl TryFrom<i64> for Rating {
    type Error = ReviewsError;

    fn try_from(value: i64) -> ReviewsResult<Self> {
        match value {
            1 => Ok(Rating::Again),
            2 => Ok(Rating::Hard),
            3 => Ok(Rating::Good),
            4 => Ok(Rating::Easy),
            _ => Err(ReviewsError::Invalid { field: "rating", reason: "must be between 1 and 4" }),
        }
    }
}

/// What kind of card a review was made on, for the daily limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewPhase {
    /// The first review of a card that had never been reviewed.
    New,
    /// A card still being learned: in its first wave, or relearning after Again.
    Learning,
    /// A card that had been recalled and came back to be reviewed.
    Review,
}

impl ReviewPhase {
    /// The value stored in the database.
    pub fn as_str(self) -> &'static str {
        match self {
            ReviewPhase::New => "NEW",
            ReviewPhase::Learning => "LEARNING",
            ReviewPhase::Review => "REVIEW",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "NEW" => Some(ReviewPhase::New),
            "LEARNING" => Some(ReviewPhase::Learning),
            "REVIEW" => Some(ReviewPhase::Review),
            _ => None,
        }
    }
}

/// How much reviewing was done in a stretch of time, by phase. The daily limits are measured
/// against it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReviewActivity {
    /// Cards seen for the first time.
    pub new_started: i64,
    /// Answers given to cards that had been recalled before.
    pub reviews_done: i64,
}

const SECONDS_PER_DAY: f64 = 86_400.0;

/// SM-2 scheduling state of one flashcard.
///
/// The SM-2 rule lives in [`Sm2State::review`]:
///
/// | Rating | Repetitions | Ease factor | Next interval |
/// |---|---|---|---|
/// | Again | reset to 0 | -0.20 | 10 minutes |
/// | Hard | +1 | -0.15 | previous x 1.2, at least 1 day |
/// | Good | +1 | unchanged | 1 day, then 6 days, then previous x ease |
/// | Easy | +1 | +0.15 | the Good interval x 1.3 |
///
/// The ease factor starts at 2.5 and never drops below 1.3. The interval uses the ease factor
/// from before the rating is applied.
///
/// **First wave.** A card that has never been recalled (new) must be reviewed twice before it
/// leaves the learning queue, unless it is answered Easy. Hard and Good on the first review only
/// move it to the second step and bring it back in 10 minutes, with no repetition counted. Again
/// starts the wave over. The second Hard or Good graduates it by the table above, and Easy
/// graduates it at once from either step. A card that lapses later (Again after it graduated)
/// does not go through the first wave again.
#[derive(Debug, Clone, PartialEq)]
pub struct Sm2State {
    flashcard_id: i64,
    due_at: DateTime<Utc>,
    interval_days: f64,
    ease_factor: f64,
    repetitions: i64,
    last_reviewed_at: Option<DateTime<Utc>>,
    /// Reviews already passed in the first wave, or `None` once the card has graduated.
    learning_step: Option<i64>,
}

impl Sm2State {
    pub const INITIAL_EASE: f64 = 2.5;
    pub const MIN_EASE: f64 = 1.3;
    const AGAIN_INTERVAL_DAYS: f64 = 10.0 / 1_440.0;
    /// How long a card waits between the reviews of its first wave.
    const FIRST_WAVE_INTERVAL_DAYS: f64 = 10.0 / 1_440.0;
    /// How many reviews a new card needs, if it is not answered Easy.
    const FIRST_WAVE_REVIEWS: i64 = 2;

    /// State of a flashcard that has never been reviewed: due right away.
    pub fn initial(flashcard_id: i64, now: DateTime<Utc>) -> Self {
        Self {
            flashcard_id,
            due_at: now,
            interval_days: 0.0,
            ease_factor: Self::INITIAL_EASE,
            repetitions: 0,
            last_reviewed_at: None,
            learning_step: Some(0),
        }
    }

    /// Rebuilds a state from stored data.
    pub(super) fn restore(
        flashcard_id: i64,
        due_at: DateTime<Utc>,
        interval_days: f64,
        ease_factor: f64,
        repetitions: i64,
        last_reviewed_at: Option<DateTime<Utc>>,
        learning_step: Option<i64>,
    ) -> Self {
        Self { flashcard_id, due_at, interval_days, ease_factor, repetitions, last_reviewed_at, learning_step }
    }

    pub fn flashcard_id(&self) -> i64 {
        self.flashcard_id
    }

    pub fn due_at(&self) -> DateTime<Utc> {
        self.due_at
    }

    pub fn interval_days(&self) -> f64 {
        self.interval_days
    }

    pub fn ease_factor(&self) -> f64 {
        self.ease_factor
    }

    pub fn repetitions(&self) -> i64 {
        self.repetitions
    }

    pub fn last_reviewed_at(&self) -> Option<DateTime<Utc>> {
        self.last_reviewed_at
    }

    /// Reviews passed in the first wave, or `None` once the card has graduated.
    pub fn learning_step(&self) -> Option<i64> {
        self.learning_step
    }

    /// Days since the previous review, or `None` if there was none.
    pub fn elapsed_days(&self, now: DateTime<Utc>) -> Option<f64> {
        self.last_reviewed_at
            .map(|last| (now - last).num_seconds() as f64 / SECONDS_PER_DAY)
    }

    /// Applies a review made at `now` and schedules the next one.
    pub fn review(&mut self, rating: Rating, now: DateTime<Utc>) {
        let ease_before = self.ease_factor;

        if let Some(step) = self.learning_step {
            match rating {
                Rating::Again => self.learning_step = Some(0),
                Rating::Hard | Rating::Good if step + 1 < Self::FIRST_WAVE_REVIEWS => {
                    // Passed, but a new card needs one more review before it graduates.
                    self.learning_step = Some(step + 1);
                    self.interval_days = Self::FIRST_WAVE_INTERVAL_DAYS;
                    if rating == Rating::Hard {
                        self.ease_factor = (ease_before - 0.15).max(Self::MIN_EASE);
                    }
                    self.last_reviewed_at = Some(now);
                    self.due_at = now + Duration::seconds((self.interval_days * SECONDS_PER_DAY).round() as i64);
                    return;
                }
                // The last step, or Easy: the card graduates by the normal rule below.
                _ => self.learning_step = None,
            }
        }

        let good_interval = match self.repetitions {
            0 => 1.0,
            1 => 6.0,
            _ => self.interval_days * ease_before,
        };

        match rating {
            Rating::Again => {
                self.repetitions = 0;
                self.interval_days = Self::AGAIN_INTERVAL_DAYS;
                self.ease_factor = (ease_before - 0.20).max(Self::MIN_EASE);
            }
            Rating::Hard => {
                self.repetitions += 1;
                self.interval_days = (self.interval_days * 1.2).max(1.0);
                self.ease_factor = (ease_before - 0.15).max(Self::MIN_EASE);
            }
            Rating::Good => {
                self.repetitions += 1;
                self.interval_days = good_interval;
            }
            Rating::Easy => {
                self.repetitions += 1;
                self.interval_days = good_interval * 1.3;
                self.ease_factor = ease_before + 0.15;
            }
        }

        self.last_reviewed_at = Some(now);
        self.due_at = now + Duration::seconds((self.interval_days * SECONDS_PER_DAY).round() as i64);
    }
}

/// One entry of the append-only review history.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewLog {
    id: Option<i64>,
    flashcard_id: i64,
    reviewed_at: DateTime<Utc>,
    rating: Rating,
    elapsed_days: Option<f64>,
    scheduled_days: f64,
    duration_ms: Option<i64>,
    phase: ReviewPhase,
}

impl ReviewLog {
    /// An unsaved log entry. `duration_ms`, when given, must not be negative.
    pub fn new(
        flashcard_id: i64,
        reviewed_at: DateTime<Utc>,
        rating: Rating,
        elapsed_days: Option<f64>,
        scheduled_days: f64,
        duration_ms: Option<i64>,
        phase: ReviewPhase,
    ) -> ReviewsResult<Self> {
        if duration_ms.is_some_and(|d| d < 0) {
            return Err(ReviewsError::Invalid { field: "duration_ms", reason: "must not be negative" });
        }
        Ok(Self {
            id: None,
            flashcard_id,
            reviewed_at,
            rating,
            elapsed_days,
            scheduled_days,
            duration_ms,
            phase,
        })
    }

    pub(super) fn restore(
        id: i64,
        flashcard_id: i64,
        reviewed_at: DateTime<Utc>,
        rating: Rating,
        elapsed_days: Option<f64>,
        scheduled_days: f64,
        duration_ms: Option<i64>,
        phase: ReviewPhase,
    ) -> Self {
        Self {
            id: Some(id),
            flashcard_id,
            reviewed_at,
            rating,
            elapsed_days,
            scheduled_days,
            duration_ms,
            phase,
        }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn flashcard_id(&self) -> i64 {
        self.flashcard_id
    }

    pub fn reviewed_at(&self) -> DateTime<Utc> {
        self.reviewed_at
    }

    pub fn rating(&self) -> Rating {
        self.rating
    }

    pub fn elapsed_days(&self) -> Option<f64> {
        self.elapsed_days
    }

    pub fn scheduled_days(&self) -> f64 {
        self.scheduled_days
    }

    pub fn duration_ms(&self) -> Option<i64> {
        self.duration_ms
    }

    pub fn phase(&self) -> ReviewPhase {
        self.phase
    }
}

/// Counters for one flashcard, independent of the scheduling algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardStats {
    flashcard_id: i64,
    review_count: i64,
    correct_count: i64,
    lapses: i64,
    total_duration_ms: i64,
    last_rating: Option<Rating>,
}

impl CardStats {
    /// Counters of a flashcard that has never been reviewed.
    pub fn empty(flashcard_id: i64) -> Self {
        Self {
            flashcard_id,
            review_count: 0,
            correct_count: 0,
            lapses: 0,
            total_duration_ms: 0,
            last_rating: None,
        }
    }

    pub(super) fn restore(
        flashcard_id: i64,
        review_count: i64,
        correct_count: i64,
        lapses: i64,
        total_duration_ms: i64,
        last_rating: Option<Rating>,
    ) -> Self {
        Self { flashcard_id, review_count, correct_count, lapses, total_duration_ms, last_rating }
    }

    pub fn flashcard_id(&self) -> i64 {
        self.flashcard_id
    }

    pub fn review_count(&self) -> i64 {
        self.review_count
    }

    /// Reviews rated anything but Again.
    pub fn correct_count(&self) -> i64 {
        self.correct_count
    }

    /// Times the card was forgotten after it had been recalled at least once.
    pub fn lapses(&self) -> i64 {
        self.lapses
    }

    pub fn total_duration_ms(&self) -> i64 {
        self.total_duration_ms
    }

    pub fn last_rating(&self) -> Option<Rating> {
        self.last_rating
    }

    /// Counts a review. `had_successful_repetition` tells whether the card was recalled before,
    /// which is what makes an Again a lapse instead of a failed first attempt.
    pub fn record(&mut self, rating: Rating, duration_ms: Option<i64>, had_successful_repetition: bool) {
        self.review_count += 1;
        if rating == Rating::Again {
            if had_successful_repetition {
                self.lapses += 1;
            }
        } else {
            self.correct_count += 1;
        }
        self.total_duration_ms += duration_ms.unwrap_or(0);
        self.last_rating = Some(rating);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependencies::clock::parse_utc;

    fn at(text: &str) -> DateTime<Utc> {
        parse_utc(text).unwrap()
    }

    fn fresh() -> Sm2State {
        Sm2State::initial(1, at("2026-10-12 08:00:00"))
    }

    #[test]
    fn rating_converts_from_integers_one_to_four_only() {
        assert_eq!(Rating::try_from(3).unwrap(), Rating::Good);
        assert!(Rating::try_from(0).is_err());
        assert!(Rating::try_from(5).is_err());
    }

    #[test]
    fn good_ratings_grow_one_then_six_then_by_ease() {
        let mut s = fresh();
        let now = at("2026-10-12 08:00:00");
        s.review(Rating::Good, now);
        s.review(Rating::Good, now);
        assert_eq!(s.interval_days(), 1.0);
        assert_eq!(s.due_at(), at("2026-10-13 08:00:00"));
        s.review(Rating::Good, now);
        assert_eq!(s.interval_days(), 6.0);
        s.review(Rating::Good, now);
        assert_eq!(s.interval_days(), 15.0);
        assert_eq!(s.repetitions(), 3);
        assert_eq!(s.ease_factor(), 2.5);
    }

    #[test]
    fn again_resets_repetitions_lowers_ease_and_is_due_in_ten_minutes() {
        let mut s = fresh();
        let now = at("2026-10-12 08:00:00");
        s.review(Rating::Good, now);
        s.review(Rating::Good, now);
        s.review(Rating::Good, now);
        s.review(Rating::Again, now);
        assert_eq!(s.repetitions(), 0);
        assert!((s.ease_factor() - 2.3).abs() < 1e-9);
        assert_eq!(s.due_at(), at("2026-10-12 08:10:00"));
        s.review(Rating::Good, now);
        assert_eq!(s.interval_days(), 1.0, "recovery restarts the 1, 6, ... sequence");
        assert_eq!(s.learning_step(), None, "a lapsed card does not go through the first wave again");
    }

    #[test]
    fn ease_never_drops_below_the_floor() {
        let mut s = fresh();
        let now = at("2026-10-12 08:00:00");
        for _ in 0..20 {
            s.review(Rating::Again, now);
        }
        assert_eq!(s.ease_factor(), Sm2State::MIN_EASE);
    }

    #[test]
    fn hard_and_easy_adjust_interval_and_ease() {
        let now = at("2026-10-12 08:00:00");

        let mut hard = fresh();
        hard.review(Rating::Hard, now);
        hard.review(Rating::Hard, now);
        assert_eq!(hard.interval_days(), 1.0, "at least one day");
        assert!((hard.ease_factor() - 2.2).abs() < 1e-9);
        hard.review(Rating::Good, now);
        hard.review(Rating::Good, now);
        let before = hard.interval_days();
        hard.review(Rating::Hard, now);
        assert!((hard.interval_days() - before * 1.2).abs() < 1e-9);

        let mut easy = fresh();
        easy.review(Rating::Easy, now);
        assert!((easy.interval_days() - 1.3).abs() < 1e-9);
        assert!((easy.ease_factor() - 2.65).abs() < 1e-9);
    }

    #[test]
    fn a_new_card_needs_two_reviews_unless_it_is_answered_easy() {
        let now = at("2026-10-12 08:00:00");

        for first in [Rating::Hard, Rating::Good] {
            let mut s = fresh();
            s.review(first, now);
            assert_eq!(s.repetitions(), 0, "{first:?}: not graduated yet");
            assert_eq!(s.learning_step(), Some(1));
            assert_eq!(s.due_at(), at("2026-10-12 08:10:00"), "{first:?}: it comes back in 10 minutes");

            s.review(Rating::Good, now);
            assert_eq!(s.learning_step(), None);
            assert_eq!(s.repetitions(), 1);
            assert_eq!(s.interval_days(), 1.0, "{first:?}: graduates to one day");
            assert_eq!(s.due_at(), at("2026-10-13 08:00:00"));
        }

        let mut easy = fresh();
        easy.review(Rating::Easy, now);
        assert_eq!(easy.learning_step(), None);
        assert!((easy.interval_days() - 1.3).abs() < 1e-9, "Easy graduates on the first review");

        let mut easy_second = fresh();
        easy_second.review(Rating::Good, now);
        easy_second.review(Rating::Easy, now);
        assert_eq!((easy_second.learning_step(), easy_second.repetitions()), (None, 1));
        assert!((easy_second.interval_days() - 1.3).abs() < 1e-9);
    }

    #[test]
    fn again_starts_the_first_wave_over() {
        let now = at("2026-10-12 08:00:00");
        let mut s = fresh();
        s.review(Rating::Good, now);
        s.review(Rating::Again, now);
        assert_eq!(s.learning_step(), Some(0));
        assert_eq!(s.due_at(), at("2026-10-12 08:10:00"));
        s.review(Rating::Good, now);
        assert_eq!(s.learning_step(), Some(1), "two reviews are needed again");
        s.review(Rating::Good, now);
        assert_eq!(s.learning_step(), None);
    }

    #[test]
    fn elapsed_days_counts_from_the_last_review() {
        let mut s = fresh();
        assert_eq!(s.elapsed_days(at("2026-10-12 08:00:00")), None);
        s.review(Rating::Good, at("2026-10-12 08:00:00"));
        assert_eq!(s.elapsed_days(at("2026-10-12 20:00:00")), Some(0.5));
    }

    #[test]
    fn stats_count_correct_answers_and_lapses() {
        let mut stats = CardStats::empty(1);
        stats.record(Rating::Again, Some(1_000), false);
        assert_eq!((stats.review_count(), stats.correct_count(), stats.lapses()), (1, 0, 0));
        stats.record(Rating::Good, Some(2_000), false);
        stats.record(Rating::Again, None, true);
        assert_eq!((stats.review_count(), stats.correct_count(), stats.lapses()), (3, 1, 1));
        assert_eq!(stats.total_duration_ms(), 3_000);
        assert_eq!(stats.last_rating(), Some(Rating::Again));
    }

    #[test]
    fn phase_round_trips_through_its_stored_value() {
        for phase in [ReviewPhase::New, ReviewPhase::Learning, ReviewPhase::Review] {
            assert_eq!(ReviewPhase::parse(phase.as_str()), Some(phase));
        }
        assert_eq!(ReviewPhase::parse("OLD"), None);
    }

    #[test]
    fn review_log_rejects_negative_duration() {
        let now = at("2026-10-12 08:00:00");
        assert!(ReviewLog::new(1, now, Rating::Good, None, 1.0, Some(-1), ReviewPhase::New).is_err());
        assert!(ReviewLog::new(1, now, Rating::Good, None, 1.0, Some(0), ReviewPhase::New).is_ok());
    }
}
