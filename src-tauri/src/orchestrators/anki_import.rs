//! Brings the notes of an Anki collection into verbvm as new decks. Only the content comes: the
//! reviews, due dates and tags stay in Anki.
//!
//! The collection is the owner's own: decks `<root>::<Language>::Pronúncia` and `…::Vocabulário`
//! of Basic notes. Each language becomes one deck holding both kinds. A vocabulary note and a
//! pronunciation note with the same sentence become a stack of two cards (see [`convert`] for how
//! the fields map). Everything goes through the editor, so the cards obey the same rules,
//! sanitizing and media limits as the ones written by hand.

mod convert;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::Arc;

use super::error::{OrchestratorError, OrchestratorResult};
use super::{DeckOverviewOrchestrator, FlashcardEditorOrchestrator};
use crate::dependencies::anki_collection::{AnkiCollection, AnkiError, AnkiNote};

/// The languages of the collection: the name Anki uses for the deck, the ISO 639-3 code of the
/// language and the accent the cards are written in. The deck keeps the name it has in Anki.
const LANGUAGES: [(&str, &str, &str); 5] = [
    ("Inglês", "eng", "en-US"),
    ("Espanhol", "spa", "es-ES"),
    ("Francês", "fra", "fr-FR"),
    ("Alemão", "deu", "de-DE"),
    ("Italiano", "ita", "it-IT"),
];

const PRONUNCIATION_DECK: &str = "Pronúncia";
const VOCABULARY_DECK: &str = "Vocabulário";

/// What was brought in for one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckImport {
    pub name: String,
    /// Stacks of two cards (vocabulary and phonetic).
    pub stacks: usize,
    /// Vocabulary cards with no pronunciation note to pair with.
    pub vocabulary: usize,
    /// Phonetic cards with no vocabulary note to pair with.
    pub phonetic: usize,
    /// Images attached to meanings.
    pub images: usize,
    /// Why nothing was created for this language, if so.
    pub skipped: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnkiImportReport {
    pub decks: Vec<DeckImport>,
    pub warnings: Vec<String>,
    /// Anki had the collection open, so changes it had not yet written to the file may be missing.
    pub partial: bool,
}

/// Which notes go together as stacks and which stay alone, by the position of each note.
#[derive(Debug, Default, PartialEq, Eq)]
struct Pairs {
    /// (vocabulary, phonetic)
    stacks: Vec<(usize, usize)>,
    loose_vocabulary: Vec<usize>,
    loose_phonetic: Vec<usize>,
}

/// Pairs the notes with the same sentence. A sentence that appears several times is paired one to
/// one in order, and the notes left over stay alone.
fn pair(vocabulary_keys: &[String], phonetic_keys: &[String]) -> Pairs {
    let mut waiting: HashMap<&str, VecDeque<usize>> = HashMap::new();
    for (index, key) in phonetic_keys.iter().enumerate() {
        waiting.entry(key).or_default().push_back(index);
    }
    let mut pairs = Pairs::default();
    let mut paired = HashSet::new();
    for (index, key) in vocabulary_keys.iter().enumerate() {
        match waiting.get_mut(key.as_str()).and_then(VecDeque::pop_front) {
            Some(phonetic) => {
                paired.insert(phonetic);
                pairs.stacks.push((index, phonetic));
            }
            None => pairs.loose_vocabulary.push(index),
        }
    }
    pairs.loose_phonetic = (0..phonetic_keys.len()).filter(|i| !paired.contains(i)).collect();
    pairs
}

/// The notes of one language, front and back of each.
#[derive(Default)]
struct LanguageNotes {
    vocabulary: Vec<(String, String)>,
    phonetic: Vec<(String, String)>,
}

impl From<AnkiError> for OrchestratorError {
    fn from(error: AnkiError) -> Self {
        match error {
            AnkiError::NotACollection => OrchestratorError::Invalid(error.to_string()),
            AnkiError::Db(_) => OrchestratorError::Internal(error.to_string()),
        }
    }
}

pub struct AnkiImportOrchestrator {
    overview: Arc<DeckOverviewOrchestrator>,
    editor: Arc<FlashcardEditorOrchestrator>,
}

impl AnkiImportOrchestrator {
    pub fn new(overview: Arc<DeckOverviewOrchestrator>, editor: Arc<FlashcardEditorOrchestrator>) -> Self {
        Self { overview, editor }
    }

    /// Imports the collection in the file. A language whose deck already exists is skipped (so
    /// running it again creates nothing), and a language that fails is removed again, without
    /// stopping the others.
    pub fn import(&self, path: &Path) -> OrchestratorResult<AnkiImportReport> {
        let collection = AnkiCollection::open(path)?;
        let notes = collection.notes()?;
        let mut report = AnkiImportReport { decks: Vec::new(), warnings: Vec::new(), partial: collection.is_partial() };

        let (by_language, ignored) = Self::group(&notes);
        if !ignored.is_empty() {
            report.warnings.push(format!("Notes in these decks were not imported: {}", ignored.join(", ")));
        }

        let existing: HashSet<String> = self.overview.list()?.iter().map(|d| d.deck.name().to_owned()).collect();
        for (anki_name, iso, bcp47) in LANGUAGES {
            let Some(language_notes) = by_language.get(anki_name) else { continue };
            let mut deck = DeckImport { name: anki_name.to_owned(), stacks: 0, vocabulary: 0, phonetic: 0, images: 0, skipped: None };
            if existing.contains(anki_name) {
                deck.skipped = Some("a deck with this name already exists".to_owned());
                report.decks.push(deck);
                continue;
            }
            if let Err(error) = self.import_language(&collection, anki_name, iso, bcp47, language_notes, &mut deck, &mut report.warnings) {
                // Nothing of a language that failed stays, and its drafts go with the deck.
                let _ = self.overview.delete(anki_name);
                deck = DeckImport { stacks: 0, vocabulary: 0, phonetic: 0, images: 0, skipped: Some(error.to_string()), ..deck };
            }
            report.decks.push(deck);
        }
        Ok(report)
    }

    /// The notes by language and kind, and the names of the decks that were not recognised.
    fn group(notes: &[AnkiNote]) -> (HashMap<&'static str, LanguageNotes>, Vec<String>) {
        let mut by_language: HashMap<&'static str, LanguageNotes> = HashMap::new();
        let mut ignored: Vec<String> = Vec::new();
        for note in notes {
            let [.., language, kind] = note.deck_path.as_slice() else {
                Self::remember(&mut ignored, note.deck_path.join("::"));
                continue;
            };
            let known = LANGUAGES.iter().find(|(name, _, _)| name == language);
            let (Some((name, _, _)), [front, back, ..]) = (known, note.fields.as_slice()) else {
                Self::remember(&mut ignored, note.deck_path.join("::"));
                continue;
            };
            let entry = by_language.entry(name).or_default();
            match kind.as_str() {
                PRONUNCIATION_DECK => entry.phonetic.push((front.clone(), back.clone())),
                VOCABULARY_DECK => entry.vocabulary.push((front.clone(), back.clone())),
                _ => Self::remember(&mut ignored, note.deck_path.join("::")),
            }
        }
        (by_language, ignored)
    }

    fn remember(names: &mut Vec<String>, name: String) {
        if !names.contains(&name) {
            names.push(name);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn import_language(
        &self,
        collection: &AnkiCollection,
        name: &str,
        iso: &str,
        bcp47: &str,
        notes: &LanguageNotes,
        deck: &mut DeckImport,
        warnings: &mut Vec<String>,
    ) -> OrchestratorResult<()> {
        self.overview.create(name.to_owned(), iso, Some(bcp47), None)?;

        let vocabulary_keys: Vec<String> = notes.vocabulary.iter().map(|(front, _)| convert::sentence_key(front)).collect();
        let phonetic_keys: Vec<String> = notes.phonetic.iter().map(|(front, _)| convert::sentence_key(front)).collect();
        let pairs = pair(&vocabulary_keys, &phonetic_keys);

        for (v, p) in &pairs.stacks {
            let drafts = self.editor.add_stack_of(name, &["vocabulary", "phonetic"])?;
            self.fill_vocabulary(collection, name, drafts[0].uuid(), &notes.vocabulary[*v], deck, warnings)?;
            self.fill_phonetic(drafts[1].uuid(), &notes.phonetic[*p])?;
            deck.stacks += 1;
        }
        for v in &pairs.loose_vocabulary {
            let draft = self.editor.new_draft(name, "vocabulary")?;
            self.fill_vocabulary(collection, name, draft.uuid(), &notes.vocabulary[*v], deck, warnings)?;
            deck.vocabulary += 1;
        }
        for p in &pairs.loose_phonetic {
            let draft = self.editor.new_draft(name, "phonetic")?;
            self.fill_phonetic(draft.uuid(), &notes.phonetic[*p])?;
            deck.phonetic += 1;
        }

        let outcome = self.editor.publish_drafts(name)?;
        if let Some(first) = outcome.issues.first() {
            return Err(OrchestratorError::Invalid(format!(
                "{} cards did not pass the checks, the first: {}",
                outcome.issues.len(),
                first.message
            )));
        }
        Ok(())
    }

    fn fill_vocabulary(
        &self,
        collection: &AnkiCollection,
        deck_name: &str,
        draft_uuid: &str,
        (front, back): &(String, String),
        deck: &mut DeckImport,
        warnings: &mut Vec<String>,
    ) -> OrchestratorResult<()> {
        let fields = convert::vocabulary(front, back);
        let values = std::collections::BTreeMap::from([
            ("phrase".to_owned(), fields.phrase),
            ("word_or_expression".to_owned(), fields.word_or_expression),
            ("meaning".to_owned(), fields.meaning),
        ]);
        self.editor.update_draft(draft_uuid, values)?;
        if let Some(warning) = fields.warning {
            warnings.push(format!("{deck_name}: {warning}"));
        }
        for image in fields.images {
            let Some(path) = collection.media_path(&image).filter(|p| p.is_file()) else {
                warnings.push(format!("{deck_name}: the image {image} is not in the collection's media folder"));
                continue;
            };
            match self.editor.attach_media(draft_uuid, "meaning", &path) {
                Ok(_) => deck.images += 1,
                Err(error) => warnings.push(format!("{deck_name}: the image {image} was not attached: {error}")),
            }
        }
        Ok(())
    }

    fn fill_phonetic(&self, draft_uuid: &str, (front, back): &(String, String)) -> OrchestratorResult<()> {
        let fields = convert::phonetic(front, back);
        let values = std::collections::BTreeMap::from([
            ("phrase".to_owned(), fields.phrase),
            ("ipa_phonetics".to_owned(), fields.ipa),
        ]);
        self.editor.update_draft(draft_uuid, values)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(texts: &[&str]) -> Vec<String> {
        texts.iter().map(|t| convert::sentence_key(t)).collect()
    }

    #[test]
    fn notes_with_the_same_sentence_are_paired_and_the_rest_stay_alone() {
        let pairs = pair(&keys(&["a", "b", "c"]), &keys(&["c", "a", "z"]));
        assert_eq!(pairs.stacks, vec![(0, 1), (2, 0)]);
        assert_eq!(pairs.loose_vocabulary, vec![1]);
        assert_eq!(pairs.loose_phonetic, vec![2]);
    }

    #[test]
    fn a_repeated_sentence_is_paired_one_to_one_in_order() {
        let pairs = pair(&keys(&["x", "x", "x"]), &keys(&["x", "x"]));
        assert_eq!(pairs.stacks, vec![(0, 0), (1, 1)]);
        assert_eq!(pairs.loose_vocabulary, vec![2], "the third has nothing left to pair with");
        assert!(pairs.loose_phonetic.is_empty());

        let more_phonetic = pair(&keys(&["x"]), &keys(&["x", "x"]));
        assert_eq!(more_phonetic.stacks, vec![(0, 0)]);
        assert_eq!(more_phonetic.loose_phonetic, vec![1]);
    }

    #[test]
    fn bold_and_spacing_do_not_keep_two_sentences_apart() {
        let pairs = pair(&keys(&["Conseguimos, <strong>a lo más</strong>,  medio kilo."]), &keys(&["Conseguimos, a lo más, medio kilo."]));
        assert_eq!(pairs.stacks.len(), 1);
    }
}
