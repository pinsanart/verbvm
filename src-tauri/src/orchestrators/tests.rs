use std::collections::BTreeMap;
use std::sync::Arc;

use super::fake_pronunciation::FakePronunciation;
use super::fake_speech::{FakeSpeech, FAKE_MP3};
use super::*;
use crate::dependencies::db::connection::SqliteHandle;
use crate::dependencies::media_store::LocalMediaStore;
use crate::dependencies::sanitizer::AmmoniaSanitizer;
use crate::modules::card_types::{CardTypeApi, CardTypeService, FieldKind, Side};
use crate::modules::decks::{DeckApi, DeckService, DeckSettings};
use crate::modules::flashcards::{Direction, FlashcardApi, FlashcardService};
use crate::modules::languages::{LanguageApi, LanguageService};

struct Fixture {
    languages: Arc<dyn LanguageApi>,
    card_types: Arc<dyn CardTypeApi>,
    flashcards: Arc<dyn FlashcardApi>,
    overview: Arc<DeckOverviewOrchestrator>,
    editor: Arc<FlashcardEditorOrchestrator>,
    speech: Arc<FakeSpeech>,
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
    fixture_with_speech(false)
}

/// `fail` makes the fake speech service report that it is offline.
fn fixture_with_speech(fail: bool) -> Fixture {
    fixture_with(fail, false)
}

/// `dictionary_missing` makes the pronunciation dictionary report that it is not installed.
fn fixture_with(fail: bool, dictionary_missing: bool) -> Fixture {
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
    let speech = Arc::new(FakeSpeech { fail, ..Default::default() });
    let overview = Arc::new(DeckOverviewOrchestrator::new(
        decks.clone(),
        languages.clone(),
        card_types.clone(),
        flashcards.clone(),
    ));
    let editor = Arc::new(FlashcardEditorOrchestrator::new(
        decks,
        languages.clone(),
        card_types.clone(),
        flashcards.clone(),
        Arc::new(AmmoniaSanitizer::new()),
        Arc::new(LocalMediaStore::new(path.with_extension("media"))),
        speech.clone(),
        Arc::new(FakePronunciation { missing: dictionary_missing }),
    ));
    Fixture {
        overview,
        editor,
        languages,
        card_types,
        flashcards,
        speech,
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
fn a_deck_keeps_the_study_options_of_each_type_and_they_can_be_updated() {
    let f = fixture();
    let created = f.overview.create("Travel".into(), "eng", None, None).unwrap();
    assert_eq!(created.settings.len(), created.counts.len(), "every type has options, the defaults at first");
    assert!(created.settings.iter().all(|(_, s)| *s == DeckSettings::default()));
    assert_eq!(
        (DeckSettings::default().new_cards_per_day(), DeckSettings::default().max_reviews_per_day()),
        (20, 200)
    );
    f.overview.create("Other".into(), "eng", None, None).unwrap();

    let custom = DeckSettings::new(5, 50, false).unwrap();
    let updated = f.overview.update("Travel", "Travel".into(), None, &[("phonetic".to_owned(), custom)]).unwrap();
    let of = |overview: &DeckOverview, slug: &str| {
        overview.settings.iter().find(|(t, _)| t.slug() == slug).map(|(_, s)| *s).unwrap()
    };
    assert_eq!(of(&updated, "phonetic"), custom);
    assert_eq!(of(&updated, "vocabulary"), DeckSettings::default(), "the other types keep theirs");

    let listed = f.overview.list().unwrap();
    let travel = listed.iter().find(|d| d.deck.name() == "Travel").unwrap();
    let other = listed.iter().find(|d| d.deck.name() == "Other").unwrap();
    assert_eq!(of(travel, "phonetic"), custom, "stored, not just returned");
    assert_eq!(of(other, "phonetic"), DeckSettings::default(), "another deck is not affected");

    let unknown = f.overview.update("Travel", "Travel".into(), None, &[("nope".to_owned(), custom)]);
    assert!(matches!(unknown, Err(OrchestratorError::NotFound(_))));
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

    let updated = f.overview.update("Trips", " Travel ".into(), None, &[]).unwrap();
    assert_eq!(updated.deck.name(), "Travel");
    assert_eq!(updated.deck.description(), None, "the description is replaced");
    assert_eq!(updated.variant.bcp47_code(), "fr-FR");
    assert_eq!(f.editor.drafts("Travel").unwrap().len(), 1, "the deck keeps what it had");
    assert!(matches!(f.overview.get("Trips"), Err(OrchestratorError::NotFound(_))));

    assert!(matches!(f.overview.update("Travel", "Other".into(), None, &[]), Err(OrchestratorError::Conflict(_))));
    assert!(matches!(f.overview.update("Travel", "  ".into(), None, &[]), Err(OrchestratorError::Invalid(_))));
    assert!(matches!(f.overview.update("Nope", "X".into(), None, &[]), Err(OrchestratorError::NotFound(_))));
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
        .add_field_definition("vocabulary", french.id(), Side::Back, "gender".into(), "Gender".into(), 2, false, FieldKind::Plain, None, false)
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

    assert!(matches!(f.editor.attach_media(draft.uuid(), "word_or_expression", &png), Err(OrchestratorError::Invalid(_))));
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

// ---- saved flashcards ----

fn publish_one(f: &Fixture, deck: &str, pairs: &[(&str, &str)]) -> String {
    let draft = f.editor.add_draft(deck, "vocabulary").unwrap().pop().unwrap();
    f.editor.update_draft(draft.uuid(), values(pairs)).unwrap();
    assert_eq!(f.editor.publish_drafts(deck).unwrap().published, 1);
    f.editor.saved_cards(deck).unwrap()[0].uuid.clone()
}

#[test]
fn saved_cards_are_listed_by_field_key_and_can_be_edited() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let uuid = publish_one(&f, "Deck", &[("phrase", "It's raining"), ("word_or_expression", "rain"), ("meaning", "water")]);

    let saved = f.editor.saved_cards("Deck").unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].type_slug, "vocabulary");
    assert_eq!(saved[0].values["phrase"], "It's raining");

    let updated = f
        .editor
        .update_saved(&uuid, values(&[
            ("phrase", "It's <b>pouring</b><script>x</script>"),
            ("word_or_expression", "rain"),
            ("meaning", "a lot of water"),
        ]))
        .unwrap();
    assert_eq!(updated.uuid, uuid);
    assert_eq!(updated.values["phrase"], "It's <b>pouring</b>", "rich text is sanitized");
    assert_eq!(updated.values["meaning"], "a lot of water");
    assert!(updated.updated_at.is_some());
}

#[test]
fn editing_a_saved_card_keeps_the_rules_of_its_type() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let uuid = publish_one(&f, "Deck", &[("phrase", "hi"), ("word_or_expression", "w"), ("meaning", "m")]);

    assert!(matches!(
        f.editor.update_saved(&uuid, values(&[("word_or_expression", "w"), ("meaning", "only the back")])),
        Err(OrchestratorError::Invalid(_))
    ), "the phrase is required");
    assert!(matches!(f.editor.update_saved(&uuid, values(&[("nope", "x"), ("phrase", "p"), ("word_or_expression", "w"), ("meaning", "m")])), Err(OrchestratorError::Invalid(_))));
    assert!(matches!(f.editor.update_saved("nope", values(&[])), Err(OrchestratorError::NotFound(_))));
    assert_eq!(f.editor.saved_cards("Deck").unwrap()[0].values["phrase"], "hi", "nothing changed");
}

#[test]
fn media_of_a_saved_card_can_be_added_removed_and_a_card_deleted() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let uuid = publish_one(&f, "Deck", &[("phrase", "cat"), ("word_or_expression", "cat"), ("meaning", "an animal")]);

    let with = f.editor.attach_saved_media(&uuid, "phrase", &source_file(&f, "a.png", PNG)).unwrap();
    assert_eq!(with.media["phrase"].len(), 1);
    assert!(matches!(
        f.editor.attach_saved_media(&uuid, "word_or_expression", &source_file(&f, "b.png", PNG)),
        Err(OrchestratorError::Invalid(_))
    ));

    let sha = with.media["phrase"][0].sha256().to_owned();
    let without = f.editor.remove_saved_media(&uuid, "phrase", &sha).unwrap();
    assert!(without.media.get("phrase").map_or(true, Vec::is_empty));
    assert_eq!(without.values["phrase"], "cat", "the text stays");

    f.editor.delete_saved(&uuid).unwrap();
    assert!(f.editor.saved_cards("Deck").unwrap().is_empty());
    assert!(matches!(f.editor.delete_saved(&uuid), Err(OrchestratorError::NotFound(_))));
}

fn orthographic_draft(f: &Fixture, phrase: &str) -> String {
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let draft = f.editor.add_draft("Deck", "orthographic").unwrap().remove(0);
    let values = BTreeMap::from([("phrase".to_string(), phrase.to_string())]);
    f.editor.update_draft(draft.uuid(), values).unwrap();
    draft.uuid().to_owned()
}

#[test]
fn the_orthographic_type_has_an_audio_field_spoken_from_the_phrase() {
    let f = fixture();
    let fields = f.card_types.fields_for("orthographic", None).unwrap();
    let summary: Vec<_> = fields.iter().map(|d| (d.side().as_str(), d.key(), d.kind(), d.source_key())).collect();
    assert_eq!(
        summary,
        vec![("FRONT", "audio", FieldKind::Audio, Some("phrase")), ("FRONT", "phrase", FieldKind::Plain, None)]
    );
}

#[tokio::test]
async fn generating_audio_speaks_the_phrase_in_the_decks_language_and_attaches_it() {
    let f = fixture();
    let uuid = orthographic_draft(&f, "  hello there ");

    let draft = f.editor.generate_media(&uuid, "audio").await.unwrap();
    let attached = &draft.media()["audio"];
    assert_eq!(attached.len(), 1);
    assert_eq!(attached[0].mime_type(), "audio/mpeg");
    let stored = f.editor.media_path(attached[0].file_name()).unwrap();
    assert!(std::fs::read(stored).unwrap().starts_with(FAKE_MP3));
    assert_eq!(*f.speech.requests.lock().unwrap(), vec![("hello there".to_string(), "en-US".to_string())]);

    let again = f.editor.generate_media(&uuid, "audio").await.unwrap();
    assert_eq!(again.media()["audio"].len(), 1, "the same audio counts once");
}

#[tokio::test]
async fn generating_audio_needs_text_and_an_audio_field() {
    let f = fixture();
    let uuid = orthographic_draft(&f, "   ");
    assert!(matches!(f.editor.generate_media(&uuid, "audio").await, Err(OrchestratorError::Invalid(_))));
    assert!(matches!(f.editor.generate_media(&uuid, "phrase").await, Err(OrchestratorError::Invalid(_))));
    assert!(matches!(f.editor.generate_media(&uuid, "nope").await, Err(OrchestratorError::Invalid(_))));
    assert!(f.speech.requests.lock().unwrap().is_empty(), "no request when the input is refused");
}

#[tokio::test]
async fn an_unreachable_speech_service_is_reported_and_attaches_nothing() {
    let f = fixture_with_speech(true);
    let uuid = orthographic_draft(&f, "hello");
    assert!(matches!(f.editor.generate_media(&uuid, "audio").await, Err(OrchestratorError::Unavailable(_))));
    let drafts = f.editor.drafts("Deck").unwrap();
    assert!(drafts[0].media().get("audio").map_or(true, Vec::is_empty));
}

#[tokio::test]
async fn generating_audio_for_a_saved_card_uses_its_saved_phrase() {
    let f = fixture();
    let uuid = orthographic_draft(&f, "good morning");
    f.editor.generate_media(&uuid, "audio").await.unwrap();
    assert!(f.editor.publish_drafts("Deck").unwrap().issues.is_empty());

    let card = f.editor.saved_cards("Deck").unwrap().remove(0);
    let values = BTreeMap::from([("phrase".to_string(), "good evening".to_string())]);
    f.editor.update_saved(&card.uuid, values).unwrap();
    let updated = f.editor.generate_saved_media(&card.uuid, "audio").await.unwrap();
    assert_eq!(updated.media["audio"].len(), 1);
    let requests = f.speech.requests.lock().unwrap();
    assert_eq!(requests.last().unwrap().0, "good evening");
}

// ---- pronunciation and stacks ----

fn phrase(text: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("phrase".to_string(), text.to_string())])
}

#[test]
fn the_phonetic_type_has_an_ipa_field_made_from_the_phrase() {
    let f = fixture();
    let fields = f.card_types.fields_for("phonetic", None).unwrap();
    let summary: Vec<_> = fields.iter().map(|d| (d.side().as_str(), d.key(), d.kind(), d.source_key())).collect();
    assert_eq!(
        summary,
        vec![("FRONT", "phrase", FieldKind::Plain, None), ("BACK", "ipa_phonetics", FieldKind::Ipa, Some("phrase"))]
    );
    assert!(fields[1].is_rich(), "IPA is HTML, so it is sanitized and rendered like rich text");
}

#[test]
fn generating_the_pronunciation_writes_one_line_per_word_and_names_the_missing_ones() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let draft = f.editor.add_draft("Deck", "phonetic").unwrap().remove(0);
    f.editor.update_draft(draft.uuid(), phrase("Hello, everyone! Zzz")).unwrap();

    let generated = f.editor.generate_pronunciation(draft.uuid(), "ipa_phonetics").unwrap();
    assert_eq!(
        generated.value.values()["ipa_phonetics"],
        "<b>Hello</b>: /həˈɫoʊ/<br><b>everyone</b>: /ˈɛvɹiˌwən/"
    );
    assert_eq!(generated.value.values()["phrase"], "Hello, everyone! Zzz", "the phrase is untouched");
    assert_eq!(generated.warnings, vec!["No pronunciation found for: Zzz"]);

    // Generating again replaces what the field had.
    f.editor.update_draft(draft.uuid(), BTreeMap::from([
        ("phrase".to_string(), "world".to_string()),
        ("ipa_phonetics".to_string(), "old".to_string()),
    ])).unwrap();
    let again = f.editor.generate_pronunciation(draft.uuid(), "ipa_phonetics").unwrap();
    assert_eq!(again.value.values()["ipa_phonetics"], "<b>world</b>: /wɝɫd/");
}

#[test]
fn generating_the_pronunciation_needs_text_a_dictionary_and_an_ipa_field() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let draft = f.editor.add_draft("Deck", "phonetic").unwrap().remove(0);
    assert!(matches!(f.editor.generate_pronunciation(draft.uuid(), "ipa_phonetics"), Err(OrchestratorError::Invalid(_))), "no phrase yet");
    f.editor.update_draft(draft.uuid(), phrase("zzz")).unwrap();
    assert!(matches!(f.editor.generate_pronunciation(draft.uuid(), "ipa_phonetics"), Err(OrchestratorError::Invalid(_))), "no word found");
    assert!(matches!(f.editor.generate_pronunciation(draft.uuid(), "phrase"), Err(OrchestratorError::Invalid(_))), "not an IPA field");

    let none = fixture_with(false, true);
    none.overview.create("Deck".into(), "eng", None, None).unwrap();
    let draft = none.editor.add_draft("Deck", "phonetic").unwrap().remove(0);
    none.editor.update_draft(draft.uuid(), phrase("hello")).unwrap();
    assert!(matches!(none.editor.generate_pronunciation(draft.uuid(), "ipa_phonetics"), Err(OrchestratorError::Unavailable(_))));
}

#[test]
fn a_language_the_dictionary_does_not_have_is_reported_as_unavailable() {
    let f = fixture();
    f.overview.create("Deck".into(), "fra", None, None).unwrap();
    let draft = f.editor.add_draft("Deck", "phonetic").unwrap().remove(0);
    f.editor.update_draft(draft.uuid(), phrase("bonjour")).unwrap();
    assert!(matches!(f.editor.generate_pronunciation(draft.uuid(), "ipa_phonetics"), Err(OrchestratorError::Unavailable(_))));
}

#[test]
fn a_stack_is_three_drafts_that_share_a_stack_and_can_be_edited_alone() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    f.editor.add_draft("Deck", "vocabulary").unwrap();
    let drafts = f.editor.add_stack("Deck").unwrap();
    assert_eq!(drafts.len(), 4);
    let stack: Vec<_> = drafts.iter().filter(|d| d.stack_uuid().is_some()).collect();
    let slugs = f.editor.type_slugs().unwrap();
    assert_eq!(
        stack.iter().map(|d| slugs[&d.flashcard_type_id()].as_str()).collect::<Vec<_>>(),
        vec!["vocabulary", "phonetic", "orthographic"]
    );
    let uuid = stack[0].stack_uuid().unwrap();
    assert!(stack.iter().all(|d| d.stack_uuid() == Some(uuid)));
    assert!(drafts[0].stack_uuid().is_none(), "the regular draft is not part of it");

    // Each card is edited with the ordinary operations, so it can be changed on its own.
    f.editor.update_draft(stack[1].uuid(), phrase("only the phonetic card")).unwrap();
    let drafts = f.editor.drafts("Deck").unwrap();
    assert_eq!(drafts[2].values()["phrase"], "only the phonetic card");
    assert!(drafts[1].values().get("phrase").is_none());

    // One card can leave, or be deleted, without touching the others.
    f.editor.delete_draft(stack[2].uuid()).unwrap();
    assert_eq!(f.editor.drafts("Deck").unwrap().len(), 3);
    let rest = f.editor.unstack(stack[1].uuid()).unwrap();
    assert!(rest.iter().all(|d| d.stack_uuid().is_none()), "one card alone is not a stack");
}

#[test]
fn a_card_can_be_added_back_to_a_stack_with_the_phrase_copied() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let stack = f.editor.add_stack("Deck").unwrap();
    let uuid = stack[0].stack_uuid().unwrap().to_owned();
    f.editor.update_draft(stack[0].uuid(), phrase("<b>Hello</b> there")).unwrap();
    f.editor.delete_draft(stack[2].uuid()).unwrap();

    assert!(matches!(f.editor.add_stack_member(&uuid, "vocabulary"), Err(OrchestratorError::Invalid(_))), "it is already in");
    let drafts = f.editor.add_stack_member(&uuid, "orthographic").unwrap();
    assert_eq!(drafts.len(), 3);
    let added = drafts.iter().find(|d| d.values().contains_key("phrase") && d.uuid() != stack[0].uuid()).unwrap();
    assert_eq!(added.values()["phrase"], "Hello there", "copied as plain text");
    assert!(matches!(f.editor.add_stack_member("nope", "phonetic"), Err(OrchestratorError::NotFound(_))));
}

#[tokio::test]
async fn generating_a_stack_makes_the_audio_and_the_pronunciation() {
    let f = fixture();
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let stack = f.editor.add_stack("Deck").unwrap();
    for draft in &stack {
        f.editor.update_draft(draft.uuid(), phrase("Hello, everyone!")).unwrap();
    }
    let uuid = stack[0].stack_uuid().unwrap();

    let generated = f.editor.generate_stack(uuid).await.unwrap();
    assert!(generated.warnings.is_empty(), "{:?}", generated.warnings);
    let drafts = generated.value;
    assert_eq!(drafts[1].values()["ipa_phonetics"], "<b>Hello</b>: /həˈɫoʊ/<br><b>everyone</b>: /ˈɛvɹiˌwən/");
    assert_eq!(drafts[2].media()["audio"].len(), 1);
    assert_eq!(f.speech.requests.lock().unwrap().len(), 1, "one request for the one audio field");
}

#[tokio::test]
async fn one_side_of_a_stack_failing_does_not_lose_the_other() {
    let offline = fixture_with_speech(true);
    offline.overview.create("Deck".into(), "eng", None, None).unwrap();
    let stack = offline.editor.add_stack("Deck").unwrap();
    for draft in &stack {
        offline.editor.update_draft(draft.uuid(), phrase("hello")).unwrap();
    }
    let generated = offline.editor.generate_stack(stack[0].stack_uuid().unwrap()).await.unwrap();
    assert_eq!(generated.value[1].values()["ipa_phonetics"], "<b>hello</b>: /həˈɫoʊ/", "the pronunciation was applied");
    assert_eq!(generated.warnings.len(), 1);
    assert!(generated.warnings[0].starts_with("Orthographic, Audio"), "{}", generated.warnings[0]);

    let no_dictionary = fixture_with(false, true);
    no_dictionary.overview.create("Deck".into(), "eng", None, None).unwrap();
    let stack = no_dictionary.editor.add_stack("Deck").unwrap();
    for draft in &stack {
        no_dictionary.editor.update_draft(draft.uuid(), phrase("hello")).unwrap();
    }
    let generated = no_dictionary.editor.generate_stack(stack[0].stack_uuid().unwrap()).await.unwrap();
    assert_eq!(generated.value[2].media()["audio"].len(), 1, "the audio was applied");
    assert!(generated.warnings[0].starts_with("Phonetic, IPA phonetics"), "{}", generated.warnings[0]);
}

#[tokio::test]
async fn generating_audio_again_for_a_new_phrase_replaces_the_old_audio() {
    let f = fixture();
    let uuid = orthographic_draft(&f, "first phrase");
    f.editor.generate_media(&uuid, "audio").await.unwrap();
    // The fake voice makes a different file for a different phrase.
    f.editor.update_draft(&uuid, phrase("second phrase")).unwrap();
    let draft = f.editor.generate_media(&uuid, "audio").await.unwrap();
    assert_eq!(draft.media()["audio"].len(), 1, "only the audio of the current phrase is kept");
}

fn publish_a_stack(f: &Fixture) -> Vec<SavedCard> {
    f.overview.create("Deck".into(), "eng", None, None).unwrap();
    let stack = f.editor.add_stack("Deck").unwrap();
    let values = [
        BTreeMap::from([
            ("phrase".to_string(), "Hello".to_string()),
            ("word_or_expression".to_string(), "hello".to_string()),
            ("meaning".to_string(), "a greeting".to_string()),
        ]),
        BTreeMap::from([
            ("phrase".to_string(), "Hello".to_string()),
            ("ipa_phonetics".to_string(), "<b>hello</b>: /həˈɫoʊ/".to_string()),
        ]),
        BTreeMap::from([("phrase".to_string(), "Hello".to_string())]),
    ];
    for (draft, values) in stack.iter().zip(values) {
        f.editor.update_draft(draft.uuid(), values).unwrap();
    }
    let source = f.path.with_extension("src");
    std::fs::create_dir_all(&source).unwrap();
    let mp3 = source.join("hello.mp3");
    std::fs::write(&mp3, FAKE_MP3).unwrap();
    f.editor.attach_media(stack[2].uuid(), "audio", &mp3).unwrap();
    let outcome = f.editor.publish_drafts("Deck").unwrap();
    assert!(outcome.issues.is_empty(), "{:?}", outcome.issues);
    f.editor.saved_cards("Deck").unwrap()
}

#[test]
fn publishing_a_stack_saves_three_cards_linked_by_the_stack() {
    let f = fixture();
    let cards = publish_a_stack(&f);
    assert_eq!(cards.len(), 3);
    let stack = cards[0].stack_uuid.clone().expect("linked");
    assert!(cards.iter().all(|c| c.stack_uuid.as_deref() == Some(stack.as_str())));
    let mut types: Vec<_> = cards.iter().map(|c| c.type_slug.as_str()).collect();
    types.sort_unstable();
    assert_eq!(types, vec!["orthographic", "phonetic", "vocabulary"]);
}

#[test]
fn the_cards_of_a_saved_stack_are_edited_together_all_or_nothing() {
    let f = fixture();
    let cards = publish_a_stack(&f);
    let by_type = |slug: &str| cards.iter().find(|c| c.type_slug == slug).unwrap().clone();
    let (vocabulary, phonetic, orthographic) = (by_type("vocabulary"), by_type("phonetic"), by_type("orthographic"));

    let updated = f.editor.update_saved_many(vec![
        (vocabulary.uuid.clone(), BTreeMap::from([
            ("phrase".to_string(), "Goodbye".to_string()),
            ("word_or_expression".to_string(), "bye".to_string()),
            ("meaning".to_string(), "farewell".to_string()),
        ])),
        (phonetic.uuid.clone(), BTreeMap::from([
            ("phrase".to_string(), "Goodbye".to_string()),
            ("ipa_phonetics".to_string(), "<b>bye</b>".to_string()),
        ])),
        (orthographic.uuid.clone(), phrase("Goodbye")),
    ]).unwrap();
    assert!(updated.iter().all(|c| c.values["phrase"] == "Goodbye"));

    // One card that does not pass leaves all of them as they were.
    let refused = f.editor.update_saved_many(vec![
        (vocabulary.uuid.clone(), BTreeMap::from([
            ("phrase".to_string(), "Changed".to_string()),
            ("word_or_expression".to_string(), "x".to_string()),
            ("meaning".to_string(), "y".to_string()),
        ])),
        (phonetic.uuid.clone(), phrase("Changed")), // the IPA field is required
    ]);
    assert!(matches!(refused, Err(OrchestratorError::Invalid(_))));
    let again = f.editor.saved_cards("Deck").unwrap();
    assert!(again.iter().all(|c| c.values["phrase"] == "Goodbye"), "nothing was stored");
}

#[test]
fn a_saved_card_can_leave_its_stack_and_a_stack_can_be_deleted() {
    let f = fixture();
    let cards = publish_a_stack(&f);
    let stack = cards[0].stack_uuid.clone().unwrap();

    let free = f.editor.unstack_saved(&cards[0].uuid).unwrap();
    assert!(free.stack_uuid.is_none());
    let rest = f.editor.saved_cards("Deck").unwrap();
    assert_eq!(rest.iter().filter(|c| c.stack_uuid.is_some()).count(), 2, "two are still a stack");

    f.editor.delete_saved_stack(&stack).unwrap();
    let left = f.editor.saved_cards("Deck").unwrap();
    assert_eq!(left.len(), 1, "only the card that left is kept");
    assert_eq!(left[0].uuid, cards[0].uuid);
}

#[test]
fn deleting_one_card_of_a_saved_stack_leaves_the_others_stacked() {
    let f = fixture();
    let cards = publish_a_stack(&f);
    f.editor.delete_saved(&cards[2].uuid).unwrap();
    let rest = f.editor.saved_cards("Deck").unwrap();
    assert_eq!(rest.len(), 2);
    assert!(rest.iter().all(|c| c.stack_uuid.is_some()));
    f.editor.delete_saved(&cards[1].uuid).unwrap();
    assert!(f.editor.saved_cards("Deck").unwrap()[0].stack_uuid.is_none(), "one card alone is not a stack");
}

#[tokio::test]
async fn generating_a_saved_stack_uses_the_saved_text() {
    let f = fixture();
    let cards = publish_a_stack(&f);
    let stack = cards[0].stack_uuid.clone().unwrap();
    let phonetic = cards.iter().find(|c| c.type_slug == "phonetic").unwrap();
    f.editor.update_saved_many(vec![(phonetic.uuid.clone(), BTreeMap::from([
        ("phrase".to_string(), "world".to_string()),
        ("ipa_phonetics".to_string(), "old".to_string()),
    ]))]).unwrap();

    let generated = f.editor.generate_saved_stack(&stack).await.unwrap();
    assert_eq!(generated.value.len(), 3);
    let phonetic = generated.value.iter().find(|c| c.type_slug == "phonetic").unwrap();
    assert_eq!(phonetic.values["ipa_phonetics"], "<b>world</b>: /wɝɫd/");
    let orthographic = generated.value.iter().find(|c| c.type_slug == "orthographic").unwrap();
    assert_eq!(orthographic.media["audio"].len(), 1);
    assert!(generated.warnings.is_empty(), "{:?}", generated.warnings);
}
