use std::collections::BTreeMap;
use std::sync::Arc;

use super::*;
use crate::dependencies::db::connection::SqliteHandle;
use crate::dependencies::sanitizer::AmmoniaSanitizer;
use crate::modules::card_types::{CardTypeApi, CardTypeService, Side};
use crate::modules::decks::{DeckApi, DeckService};
use crate::modules::flashcards::{Direction, FlashcardApi, FlashcardService};
use crate::modules::languages::{LanguageApi, LanguageService};

struct Fixture {
    languages: Arc<dyn LanguageApi>,
    card_types: Arc<dyn CardTypeApi>,
    flashcards: Arc<dyn FlashcardApi>,
    overview: DeckOverviewOrchestrator,
    editor: FlashcardEditorOrchestrator,
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
        "verbvm-orchestrators-{}-{:?}.db",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_file(&path);
    let db = Arc::new(SqliteHandle::new(&path).unwrap());
    let decks: Arc<dyn DeckApi> = Arc::new(DeckService::sqlite(db.clone()));
    let languages: Arc<dyn LanguageApi> = Arc::new(LanguageService::sqlite(db.clone()));
    let card_types: Arc<dyn CardTypeApi> = Arc::new(CardTypeService::sqlite(db.clone()));
    let flashcards: Arc<dyn FlashcardApi> = Arc::new(FlashcardService::sqlite(db));
    Fixture {
        overview: DeckOverviewOrchestrator::new(
            decks.clone(),
            languages.clone(),
            card_types.clone(),
            flashcards.clone(),
        ),
        editor: FlashcardEditorOrchestrator::new(
            decks,
            languages.clone(),
            card_types.clone(),
            flashcards.clone(),
            Arc::new(AmmoniaSanitizer::new()),
        ),
        languages,
        card_types,
        flashcards,
        path,
    }
}

fn values(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn keys(form: &EditorForm) -> Vec<&str> {
    form.fields.iter().map(|f| f.key()).collect()
}

// ---- deck overview ----

#[test]
fn creating_a_deck_uses_the_language_default_variant() {
    let f = fixture();
    let created = f.overview.create("Travel".into(), "por", None, Some("Trip words".into())).unwrap();
    assert_eq!(created.variant.bcp47_code(), "pt-BR");
    assert_eq!(created.language.iso_639_3(), "por");
    assert_eq!(created.deck.description(), Some("Trip words"));
    assert_eq!(created.total_cards(), 0);
    assert_eq!(created.counts.len(), 3, "every type is listed, even with no cards");
}

#[test]
fn creating_a_deck_with_an_accent_checks_it_belongs_to_the_language() {
    let f = fixture();
    let deck = f.overview.create("Lisbon".into(), "por", Some("pt-PT"), None).unwrap();
    assert_eq!(deck.variant.bcp47_code(), "pt-PT");
    assert!(matches!(
        f.overview.create("Wrong".into(), "por", Some("en-US"), None),
        Err(OrchestratorError::Invalid(_))
    ));
    assert!(matches!(
        f.overview.create("Unknown".into(), "xxx", None, None),
        Err(OrchestratorError::NotFound(_))
    ));
}

#[test]
fn duplicate_deck_names_conflict() {
    let f = fixture();
    f.overview.create("Travel".into(), "eng", None, None).unwrap();
    assert!(matches!(
        f.overview.create("Travel".into(), "spa", None, None),
        Err(OrchestratorError::Conflict(_))
    ));
}

#[test]
fn the_list_has_counts_by_type_and_delete_removes_the_deck() {
    let f = fixture();
    f.overview.create("B deck".into(), "eng", None, None).unwrap();
    f.overview.create("A deck".into(), "jpn", None, None).unwrap();
    f.editor.add_draft("A deck", "vocabulary").unwrap();
    let draft = f.editor.drafts("A deck", "vocabulary").unwrap().remove(0);
    f.editor
        .update_draft(draft.uuid(), values(&[("phrase", "hi"), ("word_or_expression", "hi"), ("meaning", "greeting")]))
        .unwrap();
    f.editor.publish_drafts("A deck", "vocabulary").unwrap();

    let list = f.overview.list().unwrap();
    let names: Vec<_> = list.iter().map(|o| o.deck.name().to_owned()).collect();
    assert_eq!(names, vec!["A deck", "B deck"]);
    assert_eq!(list[0].total_cards(), 1);
    assert_eq!(list[0].counts[0].0.slug(), "vocabulary");
    assert_eq!(list[0].counts[0].1, 1);
    assert_eq!(list[1].total_cards(), 0);

    f.overview.delete("A deck").unwrap();
    assert!(matches!(f.overview.get("A deck"), Err(OrchestratorError::NotFound(_))));
}

// ---- flashcard editor ----

#[test]
fn the_form_follows_the_deck_language() {
    let f = fixture();
    f.overview.create("English".into(), "eng", None, None).unwrap();
    f.overview.create("Japanese".into(), "jpn", None, None).unwrap();
    let japanese = f.languages.get_language_by_iso("jpn").unwrap();
    f.card_types
        .add_field_definition("vocabulary", japanese.id(), Side::Back, "romaji".into(), "Romaji".into(), 2, false, false, false)
        .unwrap();

    let english = f.editor.form("English", "vocabulary").unwrap();
    assert_eq!(keys(&english), vec!["phrase", "word_or_expression", "meaning"]);
    assert_eq!(english.language.iso_639_3(), "eng");
    assert_eq!(english.variant.bcp47_code(), "en-US");

    let form = f.editor.form("Japanese", "vocabulary").unwrap();
    assert_eq!(keys(&form), vec!["phrase", "word_or_expression", "meaning", "romaji"]);
    assert_eq!(form.flashcard_type.slug(), "vocabulary");
}

#[test]
fn unknown_decks_and_types_are_not_found() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    assert!(matches!(f.editor.form("Nope", "vocabulary"), Err(OrchestratorError::NotFound(_))));
    assert!(matches!(f.editor.form("Deck", "nope"), Err(OrchestratorError::NotFound(_))));
}

#[test]
fn updating_a_draft_sanitizes_rich_fields_and_rejects_unknown_keys() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let draft = f.editor.add_draft("Deck", "vocabulary").unwrap().remove(0);

    let saved = f
        .editor
        .update_draft(
            draft.uuid(),
            values(&[
                ("phrase", "<b>Hi</b><script>alert(1)</script>"),
                ("meaning", "<b>kept as typed</b>"),
            ]),
        )
        .unwrap();
    let phrase = &saved.values()["phrase"];
    assert!(phrase.contains("<b>Hi</b>") && !phrase.contains("script"));
    assert_eq!(saved.values()["meaning"], "<b>kept as typed</b>", "plain fields are text and stay as typed");

    assert!(matches!(
        f.editor.update_draft(draft.uuid(), values(&[("romaji", "x")])),
        Err(OrchestratorError::Invalid(_))
    ));
}

#[test]
fn structural_actions_return_the_updated_list() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    f.editor.add_draft("Deck", "vocabulary").unwrap();
    let list = f.editor.add_draft("Deck", "vocabulary").unwrap();
    let (a, b) = (list[0].uuid().to_owned(), list[1].uuid().to_owned());

    let list = f.editor.move_draft(&b, Direction::Up).unwrap();
    assert_eq!(list.iter().map(|d| d.uuid()).collect::<Vec<_>>(), vec![b.as_str(), a.as_str()]);
    let list = f.editor.duplicate_draft(&a).unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[2].uuid() == a, false, "the copy comes after the original");
    let list = f.editor.delete_draft(&b).unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn publishing_reports_missing_required_fields_and_saves_nothing() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    f.editor.add_draft("Deck", "vocabulary").unwrap();
    let list = f.editor.add_draft("Deck", "vocabulary").unwrap();
    f.editor
        .update_draft(list[0].uuid(), values(&[("phrase", "Hi"), ("word_or_expression", "hi"), ("meaning", "greeting")]))
        .unwrap();
    // The second draft only has a phrase made of an empty tag.
    f.editor.update_draft(list[1].uuid(), values(&[("phrase", "<br>")])).unwrap();

    let outcome = f.editor.publish_drafts("Deck", "vocabulary").unwrap();
    assert_eq!(outcome.published, 0);
    let missing: Vec<_> = outcome.issues.iter().map(|i| i.field_key.clone().unwrap()).collect();
    assert_eq!(missing, vec!["phrase", "word_or_expression", "meaning"]);
    assert!(outcome.issues.iter().all(|i| i.draft_uuid == list[1].uuid()));
    assert_eq!(f.editor.drafts("Deck", "vocabulary").unwrap().len(), 2, "drafts are kept");
    let deck = f.overview.get("Deck").unwrap();
    assert_eq!(deck.total_cards(), 0);
}

#[test]
fn publishing_valid_drafts_creates_flashcards_and_clears_the_list() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    f.editor.add_draft("Deck", "vocabulary").unwrap();
    let list = f.editor.add_draft("Deck", "vocabulary").unwrap();
    for (draft, phrase) in list.iter().zip(["It's <b>raining</b>", "Good morning"]) {
        f.editor
            .update_draft(draft.uuid(), values(&[("phrase", phrase), ("word_or_expression", "w"), ("meaning", "m")]))
            .unwrap();
    }

    let outcome = f.editor.publish_drafts("Deck", "vocabulary").unwrap();
    assert_eq!((outcome.published, outcome.issues.len()), (2, 0));
    assert!(f.editor.drafts("Deck", "vocabulary").unwrap().is_empty());

    let deck = f.overview.get("Deck").unwrap();
    assert_eq!(deck.total_cards(), 2);
    let cards = f.flashcards.list_flashcards(deck.deck.id().unwrap()).unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].fields().len(), 3);
}

#[test]
fn publishing_with_no_drafts_does_nothing() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let outcome = f.editor.publish_drafts("Deck", "vocabulary").unwrap();
    assert_eq!((outcome.published, outcome.issues.len()), (0, 0));
}
