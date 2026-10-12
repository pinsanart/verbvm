use std::collections::BTreeMap;
use std::sync::Arc;

use super::*;
use crate::dependencies::db::connection::SqliteHandle;
use crate::dependencies::media_store::LocalMediaStore;
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
        let _ = std::fs::remove_dir_all(self.path.with_extension("media"));
        let _ = std::fs::remove_dir_all(self.path.with_extension("src"));
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
            Arc::new(LocalMediaStore::new(path.with_extension("media"))),
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

fn keys<'a>(form: &'a EditorForm, type_slug: &str) -> Vec<&'a str> {
    let type_form = form.types.iter().find(|t| t.flashcard_type.slug() == type_slug).unwrap();
    type_form.fields.iter().map(|f| f.key()).collect()
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
    let deck = f.overview.create("London".into(), "eng", Some("en-GB"), None).unwrap();
    assert_eq!(deck.variant.bcp47_code(), "en-GB");
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
        f.overview.create("Travel".into(), "fra", None, None),
        Err(OrchestratorError::Conflict(_))
    ));
}

#[test]
fn updating_a_deck_renames_it_and_keeps_its_language_and_cards() {
    let f = fixture();
    f.overview.create("Trips".into(), "fra", None, Some("Old".into())).unwrap();
    f.overview.create("Other".into(), "eng", None, None).unwrap();
    f.editor.add_draft("Trips", "vocabulary").unwrap();

    let updated = f.overview.update("Trips", " Travel ".into(), None).unwrap();
    assert_eq!(updated.deck.name(), "Travel");
    assert_eq!(updated.deck.description(), None, "the description is replaced");
    assert_eq!(updated.variant.bcp47_code(), "fr-FR");
    assert_eq!(f.editor.drafts("Travel").unwrap().len(), 1, "the deck keeps what it had");
    assert!(matches!(f.overview.get("Trips"), Err(OrchestratorError::NotFound(_))));

    assert!(matches!(f.overview.update("Travel", "Other".into(), None), Err(OrchestratorError::Conflict(_))));
    assert!(matches!(f.overview.update("Travel", "  ".into(), None), Err(OrchestratorError::Invalid(_))));
    assert!(matches!(f.overview.update("Nope", "X".into(), None), Err(OrchestratorError::NotFound(_))));
}

#[test]
fn the_list_has_counts_by_type_and_delete_removes_the_deck() {
    let f = fixture();
    f.overview.create("B deck".into(), "eng", None, None).unwrap();
    f.overview.create("A deck".into(), "fra", None, None).unwrap();
    f.editor.add_draft("A deck", "vocabulary").unwrap();
    let draft = f.editor.drafts("A deck").unwrap().remove(0);
    f.editor
        .update_draft(draft.uuid(), values(&[("phrase", "hi"), ("word_or_expression", "hi"), ("meaning", "greeting")]))
        .unwrap();
    f.editor.publish_drafts("A deck").unwrap();

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
fn the_form_has_every_type_and_follows_the_deck_language() {
    let f = fixture();
    f.overview.create("English".into(), "eng", None, None).unwrap();
    f.overview.create("French".into(), "fra", None, None).unwrap();
    let french = f.languages.get_language_by_iso("fra").unwrap();
    f.card_types
        .add_field_definition("vocabulary", french.id(), Side::Back, "gender".into(), "Gender".into(), 2, false, false, false)
        .unwrap();

    let english = f.editor.form("English").unwrap();
    let slugs: Vec<_> = english.types.iter().map(|t| t.flashcard_type.slug()).collect();
    assert_eq!(slugs, vec!["vocabulary", "phonetic", "orthographic"]);
    assert_eq!(keys(&english, "vocabulary"), vec!["phrase", "word_or_expression", "meaning"]);
    assert_eq!(keys(&english, "phonetic"), vec!["phrase", "ipa_phonetics"]);
    assert_eq!(english.language.iso_639_3(), "eng");
    assert_eq!(english.variant.bcp47_code(), "en-US");

    let form = f.editor.form("French").unwrap();
    assert_eq!(keys(&form, "vocabulary"), vec!["phrase", "word_or_expression", "meaning", "gender"]);
    assert_eq!(keys(&form, "phonetic"), vec!["phrase", "ipa_phonetics"], "other types are not affected");
}

#[test]
fn unknown_decks_and_types_are_not_found() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    assert!(matches!(f.editor.form("Nope"), Err(OrchestratorError::NotFound(_))));
    assert!(matches!(f.editor.drafts("Nope"), Err(OrchestratorError::NotFound(_))));
    assert!(matches!(f.editor.add_draft("Nope", "vocabulary"), Err(OrchestratorError::NotFound(_))));
    assert!(matches!(f.editor.add_draft("Deck", "nope"), Err(OrchestratorError::NotFound(_))));
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
        f.editor.update_draft(draft.uuid(), values(&[("gender", "x")])),
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

    let outcome = f.editor.publish_drafts("Deck").unwrap();
    assert_eq!(outcome.published, 0);
    let missing: Vec<_> = outcome.issues.iter().map(|i| i.field_key.clone().unwrap()).collect();
    assert_eq!(missing, vec!["phrase", "word_or_expression", "meaning"]);
    assert!(outcome.issues.iter().all(|i| i.draft_uuid == list[1].uuid()));
    assert_eq!(f.editor.drafts("Deck").unwrap().len(), 2, "drafts are kept");
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

    let outcome = f.editor.publish_drafts("Deck").unwrap();
    assert_eq!((outcome.published, outcome.issues.len()), (2, 0));
    assert!(f.editor.drafts("Deck").unwrap().is_empty());

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
    let outcome = f.editor.publish_drafts("Deck").unwrap();
    assert_eq!((outcome.published, outcome.issues.len()), (0, 0));
}

// ---- media ----

const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];

fn source_file(f: &Fixture, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let dir = f.path.with_extension("src");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn attaching_media_stores_the_file_and_adds_it_to_the_draft() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let draft = f.editor.add_draft("Deck", "vocabulary").unwrap().remove(0);

    let updated = f.editor.attach_media(draft.uuid(), "phrase", &source_file(&f, "a.png", PNG)).unwrap();
    let attached = &updated.media()["phrase"];
    assert_eq!(attached.len(), 1);
    assert_eq!(attached[0].mime_type(), "image/png");
    let stored = f.editor.media_path(attached[0].file_name()).unwrap();
    assert_eq!(std::fs::read(stored).unwrap(), PNG);
    assert!(f.editor.media_path("../escape.png").is_none());

    let again = f.editor.attach_media(draft.uuid(), "phrase", &source_file(&f, "b.png", PNG)).unwrap();
    assert_eq!(again.media()["phrase"].len(), 1, "the same content counts once");

    let removed = f.editor.remove_media(draft.uuid(), "phrase", attached[0].sha256()).unwrap();
    assert!(removed.media().get("phrase").map_or(true, Vec::is_empty));
}

#[test]
fn media_is_only_accepted_for_fields_that_allow_it_and_for_real_images_and_audio() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let draft = f.editor.add_draft("Deck", "vocabulary").unwrap().remove(0);
    let png = source_file(&f, "a.png", PNG);

    assert!(matches!(f.editor.attach_media(draft.uuid(), "meaning", &png), Err(OrchestratorError::Invalid(_))));
    assert!(matches!(f.editor.attach_media(draft.uuid(), "nope", &png), Err(OrchestratorError::Invalid(_))));
    let text = source_file(&f, "fake.png", b"not an image");
    assert!(matches!(f.editor.attach_media(draft.uuid(), "phrase", &text), Err(OrchestratorError::Invalid(_))));
    assert!(matches!(
        f.editor.attach_media("nope", "phrase", &png),
        Err(OrchestratorError::NotFound(_))
    ));
}

#[test]
fn a_required_field_with_only_media_can_be_published() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let draft = f.editor.add_draft("Deck", "vocabulary").unwrap().remove(0);
    f.editor.update_draft(draft.uuid(), values(&[("word_or_expression", "cat"), ("meaning", "an animal")])).unwrap();

    let blocked = f.editor.publish_drafts("Deck").unwrap();
    assert_eq!(blocked.issues.len(), 1, "the phrase has neither text nor media");

    f.editor.attach_media(draft.uuid(), "phrase", &source_file(&f, "cat.png", PNG)).unwrap();
    let outcome = f.editor.publish_drafts("Deck").unwrap();
    assert_eq!((outcome.published, outcome.issues.len()), (1, 0));

    let deck_id = f.overview.get("Deck").unwrap().deck.id().unwrap();
    let cards = f.flashcards.list_flashcards(deck_id).unwrap();
    assert_eq!(cards[0].media().len(), 1);
    assert_eq!(cards[0].fields().len(), 3);
}

// ---- one list for every type ----

#[test]
fn a_deck_mixes_types_in_one_list_and_each_draft_uses_its_own_fields() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    f.editor.add_draft("Deck", "vocabulary").unwrap();
    let list = f.editor.add_draft("Deck", "phonetic").unwrap();
    let kinds = |l: &[crate::modules::flashcards::FlashcardDraft]| -> Vec<i64> {
        l.iter().map(|d| d.flashcard_type_id()).collect()
    };
    assert_eq!(list.len(), 2);
    assert_ne!(kinds(&list)[0], kinds(&list)[1]);

    let (vocabulary, phonetic) = (list[0].uuid().to_owned(), list[1].uuid().to_owned());
    // ipa_phonetics belongs to phonetic only, meaning to vocabulary only.
    assert!(f.editor.update_draft(&phonetic, values(&[("ipa_phonetics", "/hi/")])).is_ok());
    assert!(matches!(
        f.editor.update_draft(&phonetic, values(&[("meaning", "x")])),
        Err(OrchestratorError::Invalid(_))
    ));
    assert!(f.editor.update_draft(&vocabulary, values(&[("meaning", "x")])).is_ok());
    assert!(matches!(
        f.editor.update_draft(&vocabulary, values(&[("ipa_phonetics", "x")])),
        Err(OrchestratorError::Invalid(_))
    ));

    let moved = f.editor.move_draft(&phonetic, Direction::Up).unwrap();
    assert_eq!(moved[0].uuid(), phonetic, "moving works across types");
}

#[test]
fn publishing_a_mixed_list_creates_cards_of_each_type_or_nothing() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    f.editor.add_draft("Deck", "vocabulary").unwrap();
    let list = f.editor.add_draft("Deck", "phonetic").unwrap();
    f.editor
        .update_draft(list[0].uuid(), values(&[("phrase", "Hi"), ("word_or_expression", "hi"), ("meaning", "greeting")]))
        .unwrap();

    // The phonetic draft still lacks its required fields, so the whole publish is blocked.
    let blocked = f.editor.publish_drafts("Deck").unwrap();
    assert_eq!(blocked.published, 0);
    let keys: Vec<_> = blocked.issues.iter().map(|i| i.field_key.clone().unwrap()).collect();
    assert_eq!(keys, vec!["phrase", "ipa_phonetics"]);
    assert!(blocked.issues.iter().all(|i| i.draft_uuid == list[1].uuid()));
    assert_eq!(f.overview.get("Deck").unwrap().total_cards(), 0);

    f.editor.update_draft(list[1].uuid(), values(&[("phrase", "Hi"), ("ipa_phonetics", "/haɪ/")])).unwrap();
    let outcome = f.editor.publish_drafts("Deck").unwrap();
    assert_eq!((outcome.published, outcome.issues.len()), (2, 0));

    let deck = f.overview.get("Deck").unwrap();
    let counts: Vec<_> = deck.counts.iter().map(|(t, n)| (t.slug().to_owned(), *n)).collect();
    assert_eq!(
        counts,
        vec![("vocabulary".to_string(), 1), ("phonetic".to_string(), 1), ("orthographic".to_string(), 0)]
    );
    assert!(f.editor.drafts("Deck").unwrap().is_empty());
}
