use std::collections::BTreeMap;
use std::sync::Arc;

use super::*;
use crate::dependencies::clock::{Clock, FixedClock};
use crate::dependencies::db::connection::SqliteHandle;
use crate::dependencies::media_store::LocalMediaStore;
use crate::dependencies::sanitizer::AmmoniaSanitizer;
use crate::modules::card_types::{CardTypeApi, CardTypeService, Side};
use crate::modules::decks::{DeckApi, DeckService};
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

    // Good: due in a day. Again: due in ten minutes.
    let first = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&first.uuid, 3, Some(900)).unwrap();
    let second = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&second.uuid, 1, None).unwrap();
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(1, 0, 0), "answered cards are not due yet");

    f.clock.set("2026-10-12 08:10:00");
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(1, 1, 0), "the Again card is learning");
    f.clock.set("2026-10-13 08:00:00");
    assert_eq!(counts(&f, "Deck", "vocabulary"), counts_of(1, 1, 1), "a day later the Good card is due");
}

#[test]
fn a_session_goes_learning_then_review_then_new() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 3);

    let first = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&first.uuid, 3, None).unwrap(); // becomes a review card
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
    f.review.answer(&c.uuid, 2, None).unwrap();

    let done = f.review.next_card("Deck", "vocabulary").unwrap();
    assert!(done.card.is_none());
    assert!(done.counts.is_empty(), "the type is completed");
}

#[test]
fn a_card_answered_again_comes_back_after_ten_minutes() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 1);

    let card = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!(card.state, CardState::New);
    f.review.answer(&card.uuid, 1, None).unwrap();
    assert!(next(&f, "Deck", "vocabulary").is_none());

    f.clock.set("2026-10-12 08:09:59");
    assert!(next(&f, "Deck", "vocabulary").is_none());
    f.clock.set("2026-10-12 08:10:00");
    let back = next(&f, "Deck", "vocabulary").unwrap();
    assert_eq!((back.uuid.as_str(), back.state), (card.uuid.as_str(), CardState::Learning));
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
        (600, day, day, day * 13 / 10),
        "a new card: ten minutes, a day, a day, 1.3 days"
    );

    f.review.answer(&new.card.unwrap().uuid, 3, None).unwrap();
    f.clock.set("2026-10-13 08:00:00");
    let again = f.review.next_card("Deck", "vocabulary").unwrap().intervals.unwrap();
    assert_eq!(again.good, day * 6, "the second good answer waits six days");
}

#[test]
fn when_nothing_is_due_the_session_says_when_the_next_card_comes() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    add_cards(&f, "Deck", "vocabulary", 2);

    let first = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&first.uuid, 3, None).unwrap(); // due in a day
    let second = next(&f, "Deck", "vocabulary").unwrap();
    f.review.answer(&second.uuid, 1, None).unwrap(); // due in ten minutes

    let waiting = f.review.next_card("Deck", "vocabulary").unwrap();
    assert!(waiting.card.is_none() && waiting.intervals.is_none());
    assert_eq!(waiting.next_due_in_seconds, Some(600), "the soonest one counts");
    assert_eq!(waiting.language_tag, "en-US");

    f.clock.set("2026-10-12 08:05:00");
    assert_eq!(f.review.next_card("Deck", "vocabulary").unwrap().next_due_in_seconds, Some(300));
}
