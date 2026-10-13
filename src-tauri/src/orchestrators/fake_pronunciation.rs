//! A [`PronunciationSource`] for tests: a few English words, or nothing installed.

use crate::dependencies::pronunciation::{PronunciationError, PronunciationSource};

pub struct FakePronunciation {
    /// The dictionary is not installed.
    pub missing: bool,
}

impl FakePronunciation {
    pub fn installed() -> Self {
        Self { missing: false }
    }
}

impl PronunciationSource for FakePronunciation {
    fn lookup(&self, language: &str, word: &str) -> Result<Option<String>, PronunciationError> {
        if self.missing {
            return Err(PronunciationError::NotInstalled);
        }
        if language != "en-US" {
            return Err(PronunciationError::Unsupported(language.to_owned()));
        }
        Ok(match word.to_lowercase().as_str() {
            "hello" => Some("/həˈɫoʊ/".to_owned()),
            "everyone" => Some("/ˈɛvɹiˌwən/".to_owned()),
            "world" => Some("/wɝɫd/".to_owned()),
            _ => None,
        })
    }
}
