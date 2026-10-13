use std::collections::BTreeMap;
use std::sync::Arc;

use super::*;
use crate::dependencies::clock::{Clock, FixedClock};
use crate::dependencies::db::connection::SqliteHandle;
use crate::dependencies::media_store::LocalMediaStore;
use crate::dependencies::sanitizer::AmmoniaSanitizer;
use crate::modules::card_types::{CardTypeApi, CardTypeService, Side};
use crate::modules::decks::{DeckApi, DeckService, DeckSettings};
use crate::modules::flashcards::{FlashcardApi, FlashcardService};
use crate::modules::languages::{LanguageApi, LanguageService};
use crate::modules::reviews::ReviewService;

const START: &str = "2026-10-12 08:00:00";

struct Fixture {
    overview: DeckOverviewOrchestrator,
    editor: FlashcardEditorOrchestrator,
    review: ReviewSessionOrchestrator,
    clock: Arc<FixedClock>,
    path: std::path::PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.path.display()));
        }
        let _ = std::fs::remove_dir_all(self.path.with_extension("media"));
        let _ = std::fs::remove_dir_all(self.path.with_extension("src"));
    }
}

fn fixture() -> Fixture {
    let path = std::env::temp_dir().join(format!(
        "verbvm-review-session-{}-{:?}.db",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_file(&path);
    let db = Arc::new(SqliteHandle::new(&path).unwrap());
    let clock = Arc::new(FixedClock::at(START));
    let decks: Arc<dyn DeckApi> = Arc::new(DeckService::sqlite(db.clone()));
    let languages: Arc<dyn LanguageApi> = Arc::new(LanguageService::sqlite(db.clone()));
    let card_types: Arc<dyn CardTypeApi> = Arc::new(CardTypeService::sqlite(db.clone()));
    let flashcards: Arc<dyn FlashcardApi> = Arc::new(FlashcardService::sqlite(db.clone()));
    let media = Arc::new(LocalMediaStore::new(path.with_extension("media")));
    let reviews = Arc::new(ReviewService::sqlite(db, clock.clone()));
    Fixture {
        overview: DeckOverviewOrchestrator::new(
            decks.clone(),
            languages.clone(),
            card_types.clone(),
            flashcards.clone(),
        ),
        editor: FlashcardEditorOrchestrator::new(
            decks.clone(),
            languages.clone(),
            card_types.clone(),
            flashcards.clone(),
            Arc::new(AmmoniaSanitizer::new()),
            media.clone(),
            Arc::new(super::fake_speech::FakeSpeech::default()),
            Arc::new(super::fake_pronunciation::FakePronunciation::installed()),
        ),
        review: ReviewSessionOrchestrator::new(
            decks,
            languages,
            card_types,
            flashcards,
            reviews,
            clock.clone(),
            media,
        ),
        clock,
        path,
    }
}

fn values(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

/// Adds `count` flashcards of a type to a deck, filled in and published. Returns nothing: the
/// tests learn the uuids from `next_card`.
fn add_cards(f: &Fixture, deck: &str, type_slug: &str, count: usize) {
    for n in 0..count {
        let drafts = f.editor.add_draft(deck, type_slug).unwrap();
        let draft = drafts.last().unwrap();
        let fill = match type_slug {
            "phonetic" => values(&[("phrase", &format!("phrase {n}")), ("ipa_phonetics", "/x/")]),
            _ => values(&[
                ("phrase", &format!("<b>phrase {n}</b>")),
                ("word_or_expression", &format!("word {n}")),
                ("meaning", &format!("meaning {n}")),
            ]),
        };
        f.editor.update_draft(draft.uuid(), fill).unwrap();
    }
    let outcome = f.editor.publish_drafts(deck).unwrap();
    assert_eq!((outcome.published, outcome.issues.len()), (count, 0));
}

fn counts(f: &Fixture, deck: &str, type_slug: &str) -> ReviewCounts {
    let decks = f.review.overview().unwrap();
    let deck = decks.iter().find(|d| d.deck.name() == deck).unwrap();
    deck.types.iter().find(|t| t.flashcard_type.slug() == type_slug).unwrap().counts
}

fn next(f: &Fixture, deck: &str, type_slug: &str) -> Option<ReviewCardView> {
    f.review.next_card(deck, type_slug).unwrap().card
}

fn counts_of(new: usize, learning: usize, review: usize) -> ReviewCounts {
    ReviewCounts { new, learning, review }
}

#[test]
fn the_overview_lists_only_the_types_a_deck_has_cards_of() {
    let f = fixture();
    f.overview.create("Travel".into(), "fra", None, None).unwrap();
    f.overview.create("Empty".into(), "eng", None, None).unwrap();
    add_cards(&f, "Travel", "vocabulary", 2);
    add_cards(&f, "Travel", "phonetic", 1);

    let overview = f.review.overview().unwrap();
    assert_eq!(overview.len(), 2);

    let travel = overview.iter().find(|d| d.deck.name() == "Travel").unwrap();
    assert_eq!(travel.language.iso_639_3(), "fra");
    assert_eq!(travel.variant.bcp47_code(), "fr-FR");
    let types: Vec<_> = travel.types.iter().map(|t| (t.flashcard_type.slug(), t.counts)).collect();
    assert_eq!(
        types,
        vec![("vocabulary", counts_of(2, 0, 0)), ("phonetic", counts_of(1, 0, 0))],
        "orthographic has no cards, so it is not a column"
    );

    let empty = overview.iter().find(|d| d.deck.name() == "Empty").unwrap();
    assert!(empty.types.is_empty());
}

#[test]
fn the_counts_follow_the_schedule() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 3);
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(3, 0, 0));

    // Easy: due in 1.3 days. Again: due in ten minutes.
    let first = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&first.uuid, 4, Some(900)).unwrap();
    let second = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&second.uuid, 1, None).unwrap();
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(1, 0, 0), "answered cards are not due yet");

    f.clock.set("2026-10-12 08:10:00");
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(1, 1, 0), "the Again card is learning");
    f.clock.set("2026-10-13 15:12:00");
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(1, 1, 1), "1.3 days later the Easy card is due");
}

#[test]
fn a_session_goes_learning_then_review_then_new() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 3);

    let first = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&first.uuid, 4, None).unwrap(); // Easy: becomes a review card
    let second = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&second.uuid, 1, None).unwrap(); // becomes a learning card
    f.clock.set("2026-10-14 08:00:00");

    let a = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!((a.uuid.as_str(), a.state), (second.uuid.as_str(), CardState::Learning));
    f.review.answer(&a.uuid, 4, None).unwrap();

    let b = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!((b.uuid.as_str(), b.state), (first.uuid.as_str(), CardState::Review));
    f.review.answer(&b.uuid, 3, None).unwrap();

    let c = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!(c.state, CardState::New);
    assert_ne!(c.uuid, first.uuid);
    assert_ne!(c.uuid, second.uuid);
    f.review.answer(&c.uuid, 2, None).unwrap(); // Hard on a new card: it needs a second review

    let again = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!((again.uuid.as_str(), again.state), (c.uuid.as_str(), CardState::Learning), "it comes back in the session");
    f.review.answer(&again.uuid, 3, None).unwrap();

    let done = f.review.next_card("Deck", "vocabulary").unwrap();
    assert!(done.card.is_none());
    assert!(done.counts.is_empty(), "the type is completed");
}

#[test]
fn a_card_answered_again_comes_back_in_the_same_session_until_it_is_recalled() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 1);

    let card = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!(card.state, CardState::New);
    f.review.answer(&card.uuid, 1, None).unwrap();

    // It is only due in ten minutes, but nothing else is waiting, so it is brought forward.
    let back = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!((back.uuid.as_str(), back.state), (card.uuid.as_str(), CardState::Learning));
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(0, 0, 0), "the overview counts only what is due");
    assert_eq!(f.review.next_card("Deck", "vocabulary").unwrap().counts, counts_of(0, 1, 0));
    f.review.answer(&back.uuid, 1, None).unwrap();
    assert_eq!(next(&f, "Deck", "vocabulary").unwrap().uuid, card.uuid, "again and again, until recalled");
}

#[test]
fn a_card_has_its_fields_by_side_with_rich_content_and_nothing_empty() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 1);

    let card = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!(card.flashcard_type.slug(), "vocabulary");
    let fields: Vec<_> = card.fields.iter().map(|f| (f.key.as_str(), f.side, f.is_rich)).collect();
    assert_eq!(
        fields,
        vec![
            ("phrase", Side::Front, true),
            ("word_or_expression", Side::Back, false),
            ("meaning", Side::Back, false),
        ]
    );
    assert_eq!(card.fields[0].content, "<b>phrase 0</b>");
    assert_eq!(card.fields[0].label, "Phrase");
}

#[test]
fn media_comes_with_the_card_even_when_it_is_the_only_content_of_a_field() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let draft = f.editor.add_draft("Deck", "vocabulary").unwrap().remove(0);
    f.editor
        .update_draft(draft.uuid(), values(&[("word_or_expression", "cat"), ("meaning", "an animal")]))
        .unwrap();
    let dir = f.path.with_extension("src");
    std::fs::create_dir_all(&dir).unwrap();
    let png = dir.join("cat.png");
    std::fs::write(&png, [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0]).unwrap();
    f.editor.attach_media(draft.uuid(), "phrase", &png).unwrap();
    assert_eq!(f.editor.publish_drafts("Deck").unwrap().published, 1);

    let card = next(&f, "Deck", "vocabulary").unwrap();
    let phrase = card.fields.iter().find(|field| field.key == "phrase").unwrap();
    assert_eq!(phrase.content, "");
    assert_eq!(phrase.media.len(), 1);
    assert_eq!(phrase.media[0].mime_type(), "image/png");
    let path = f.review.media_path(phrase.media[0].file_name()).unwrap();
    assert!(path.exists(), "the stored file can be previewed");
}

#[test]
fn unknown_decks_types_cards_and_bad_answers_are_reported() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 1);
    let card = next(&f, "Deck", "vocabulary").unwrap();

    assert!(matches!(f.review.next_card("Nope", "vocabulary"), Err(OrchestratorError::NotFound(_))));
    assert!(matches!(f.review.next_card("Deck", "nope"), Err(OrchestratorError::NotFound(_))));
    assert!(matches!(f.review.answer("nope", 3, None), Err(OrchestratorError::NotFound(_))));
    for rating in [0, 5, -1] {
        assert!(matches!(f.review.answer(&card.uuid, rating, None), Err(OrchestratorError::Invalid(_))));
    }
    assert!(matches!(f.review.answer(&card.uuid, 3, Some(-1)), Err(OrchestratorError::Invalid(_))));
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(1, 0, 0), "rejected answers record nothing");
}

#[test]
fn each_type_of_a_deck_is_reviewed_on_its_own() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 1);
    add_cards(&f, "Deck", "phonetic", 2);

    let vocabulary = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&vocabulary.uuid, 3, None).unwrap();
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(0, 0, 0));
    assert_eq!(counts(&f, "Deck", "phonetic"), counts_of(2, 0, 0), "the other type is untouched");
    assert_eq!(next(&f, "Deck", "phonetic").unwrap().flashcard_type.slug(), "phonetic");
}

#[test]
fn each_card_says_how_long_every_answer_would_schedule_it() {
    let f = fixture();
    f.overview.create("Deck".into(), "fra", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 1);

    let new = f.review.next_card("Deck", "vocabulary").unwrap();
    assert_eq!(new.language_tag, "fr-FR");
    let day = 86_400;
    let intervals = new.intervals.unwrap();
    assert_eq!(
        (intervals.again, intervals.hard, intervals.good, intervals.easy),
        (600, 600, 600, day * 13 / 10),
        "a new card needs a second review unless Easy: ten minutes, ten minutes, ten minutes, 1.3 days"
    );

    let uuid = new.card.unwrap().uuid;
    f.review.answer(&uuid, 3, None).unwrap();
    f.clock.set("2026-10-12 08:10:00");
    let second = f.review.next_card("Deck", "vocabulary").unwrap().intervals.unwrap();
    assert_eq!(second.good, day, "the second review graduates the card to a day");

    f.review.answer(&uuid, 3, None).unwrap();
    f.clock.set("2026-10-13 08:10:00");
    let third = f.review.next_card("Deck", "vocabulary").unwrap().intervals.unwrap();
    assert_eq!(third.good, day * 6, "the next good answer waits six days");
}

#[test]
fn a_new_card_answered_good_comes_back_for_a_second_review_in_the_same_wave() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 1);

    let card = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&card.uuid, 3, None).unwrap();

    // Good is not enough for a new card: it comes back at once instead of ending the session.
    let back = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!((back.uuid.as_str(), back.state), (card.uuid.as_str(), CardState::Learning));
    f.review.answer(&back.uuid, 3, None).unwrap();
    assert!(next(&f, "Deck", "vocabulary").is_none(), "graduated: due the next day");
    f.clock.set("2026-10-12 23:59:00");
    assert!(next(&f, "Deck", "vocabulary").is_none());
    f.clock.set("2026-10-13 08:10:00");
    assert_eq!(next(&f, "Deck", "vocabulary").unwrap().state, CardState::Review);
}

#[test]
fn when_nothing_is_due_the_session_says_when_the_next_card_comes() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 2);

    let first = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&first.uuid, 4, None).unwrap(); // Easy: due in 1.3 days
    let second = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&second.uuid, 4, None).unwrap();

    let waiting = f.review.next_card("Deck", "vocabulary").unwrap();
    assert!(waiting.card.is_none() && waiting.intervals.is_none());
    let soonest = 86_400 * 13 / 10;
    assert_eq!(waiting.next_due_in_seconds, Some(soonest), "the soonest one counts");
    assert_eq!(waiting.language_tag, "en-US");

    f.clock.set("2026-10-12 08:05:00");
    assert_eq!(f.review.next_card("Deck", "vocabulary").unwrap().next_due_in_seconds, Some(soonest - 300));
}

/// A deck whose vocabulary type has these daily limits.
fn deck_with(f: &Fixture, new_per_day: i64, max_reviews: i64) {
    let settings = DeckSettings::new(new_per_day, max_reviews, true).unwrap();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    f.overview.update("Deck", "Deck".into(), None, &[("vocabulary".to_owned(), settings)]).unwrap();
}

#[test]
fn the_daily_limit_of_new_cards_holds_the_rest_until_tomorrow() {
    let f = fixture();
    deck_with(&f, 2, 200);
    add_cards(&f, "Deck", "vocabulary", 4);
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(2, 0, 0), "only two of the four are offered");

    for _ in 0..2 {
        let card = next(&f, "Deck", "vocabulary").unwrap();
        assert_eq!(card.state, CardState::New);
        f.review.answer(&card.uuid, 4, None).unwrap(); // Easy: no learning left over
    }
    let done = f.review.next_card("Deck", "vocabulary").unwrap();
    assert!(done.card.is_none());
    assert!(done.daily_limit_reached, "cards are waiting, held back by the limit");
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(0, 0, 0));

    f.clock.set("2026-10-13 00:00:00");
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(2, 0, 0), "a new UTC day lets in two more");
    assert!(!f.review.next_card("Deck", "vocabulary").unwrap().daily_limit_reached);
}

#[test]
fn a_new_card_in_its_first_wave_is_not_cut_by_the_new_card_limit() {
    let f = fixture();
    deck_with(&f, 1, 200);
    add_cards(&f, "Deck", "vocabulary", 2);

    let card = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&card.uuid, 3, None).unwrap(); // the one new card of the day, needs a second pass
    let back = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!((back.uuid.as_str(), back.state), (card.uuid.as_str(), CardState::Learning));
    f.review.answer(&back.uuid, 3, None).unwrap();
    let done = f.review.next_card("Deck", "vocabulary").unwrap();
    assert!(done.card.is_none() && done.daily_limit_reached, "the second new card waits for tomorrow");
}

#[test]
fn the_daily_limit_of_reviews_counts_cards_that_were_recalled_before() {
    let f = fixture();
    deck_with(&f, 20, 1);
    add_cards(&f, "Deck", "vocabulary", 3);
    for _ in 0..3 {
        let card = next(&f, "Deck", "vocabulary").unwrap();
        f.review.answer(&card.uuid, 4, None).unwrap(); // Easy: due again in 1.3 days
    }
    f.clock.set("2026-10-14 08:00:00");
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(0, 0, 1), "three are due, one review is allowed");

    let card = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!(card.state, CardState::Review);
    f.review.answer(&card.uuid, 3, None).unwrap();
    let done = f.review.next_card("Deck", "vocabulary").unwrap();
    assert!(done.card.is_none() && done.daily_limit_reached);

    f.clock.set("2026-10-15 08:00:00");
    assert_eq!(counts(&f, "Deck", "vocabulary").review, 1, "tomorrow one more is allowed");
}

#[test]
fn a_limit_of_zero_offers_nothing_and_the_deck_setting_reaches_the_session() {
    let f = fixture();
    let settings = DeckSettings::new(0, 200, false).unwrap();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    f.overview.update("Deck", "Deck".into(), None, &[("vocabulary".to_owned(), settings)]).unwrap();
    add_cards(&f, "Deck", "vocabulary", 2);
    let waiting = f.review.next_card("Deck", "vocabulary").unwrap();
    assert!(waiting.card.is_none() && waiting.daily_limit_reached);
    assert!(!waiting.autoplay_audio, "the deck turned autoplay off");
}

#[test]
fn the_options_of_a_type_do_not_reach_the_other_types_of_the_deck() {
    let f = fixture();
    deck_with(&f, 1, 200); // vocabulary: one new card a day
    add_cards(&f, "Deck", "vocabulary", 3);
    add_cards(&f, "Deck", "phonetic", 3);
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(1, 0, 0));
    assert_eq!(counts(&f, "Deck", "phonetic"), counts_of(3, 0, 0), "phonetic keeps the default limit of 20");
}

#[test]
fn undoing_an_answer_returns_the_same_card_as_it_was() {
    let f = fixture();
    deck_with(&f, 20, 200);
    add_cards(&f, "Deck", "vocabulary", 2);

    let card = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!(card.state, CardState::New);
    f.review.answer(&card.uuid, 4, None).unwrap(); // Easy: leaves the queue for 1.3 days
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(1, 0, 0));

    let undone = f.review.undo_answer("Deck", "vocabulary", &card.uuid).unwrap();
    let back = undone.card.unwrap();
    assert_eq!((back.uuid.as_str(), back.state), (card.uuid.as_str(), CardState::New), "never reviewed again");
    assert_eq!(undone.counts, counts_of(2, 0, 0), "counts as if it had not been answered");
    assert!(undone.intervals.is_some());
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(2, 0, 0));

    // And it can be answered differently now.
    f.review.answer(&back.uuid, 1, None).unwrap();
    let other = next(&f, "Deck", "vocabulary").unwrap();
    assert_ne!(other.uuid, back.uuid, "the other new card goes first; the missed one waits its ten minutes");
    assert_eq!(other.state, CardState::New);
}

#[test]
fn undo_goes_one_answer_back_each_time_and_restores_the_schedule() {
    let f = fixture();
    deck_with(&f, 20, 200);
    add_cards(&f, "Deck", "vocabulary", 1);
    let card = next(&f, "Deck", "vocabulary").unwrap();

    f.review.answer(&card.uuid, 3, None).unwrap(); // first pass: learning, due in 10 minutes
    f.review.answer(&card.uuid, 3, None).unwrap(); // second pass: graduates, due in a day
    assert!(next(&f, "Deck", "vocabulary").is_none());

    let one = f.review.undo_answer("Deck", "vocabulary", &card.uuid).unwrap();
    assert_eq!(one.card.as_ref().unwrap().state, CardState::Learning, "back to the first pass");
    let day = 86_400;
    assert_eq!(one.intervals.unwrap().good, day, "answering Good again graduates it again");

    let two = f.review.undo_answer("Deck", "vocabulary", &card.uuid).unwrap();
    assert_eq!(two.card.unwrap().state, CardState::New);

    assert!(matches!(
        f.review.undo_answer("Deck", "vocabulary", &card.uuid),
        Err(OrchestratorError::Invalid(_))
    ), "nothing left to undo");
}

#[test]
fn undoing_gives_the_daily_limit_back() {
    let f = fixture();
    deck_with(&f, 1, 200);
    add_cards(&f, "Deck", "vocabulary", 2);
    let card = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&card.uuid, 4, None).unwrap();
    assert!(f.review.next_card("Deck", "vocabulary").unwrap().daily_limit_reached);

    f.review.undo_answer("Deck", "vocabulary", &card.uuid).unwrap();
    let again = f.review.next_card("Deck", "vocabulary").unwrap();
    assert!(!again.daily_limit_reached && again.card.is_some());
}
