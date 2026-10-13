//! Pronunciation (IPA) of words. The rest of the app depends on [`PronunciationSource`] only, so
//! the dictionary behind it can be swapped without touching the callers.

pub mod error;
pub mod ipa_dictionary;

pub use error::PronunciationError;
pub use ipa_dictionary::{DictionaryLanguage, ImportReport, IpaDictionary};

/// Where pronunciations come from.
pub trait PronunciationSource: Send + Sync {
    /// The pronunciation of a word, with its slashes (`/həˈləʊ/`), in the language given as a
    /// BCP 47 code such as `en-GB`. `None` when the dictionary has no such word. Fails when the
    /// source is not installed or has nothing for the language.
    fn lookup(&self, language: &str, word: &str) -> Result<Option<String>, PronunciationError>;
}
