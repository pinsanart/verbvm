//! Turns a phrase into the text of an IPA field: one line per word, `<b>word</b>: /ipa/`.
//!
//! Pure functions over a [`PronunciationSource`], so the rules can be tested without a dictionary.

use crate::dependencies::pronunciation::{PronunciationError, PronunciationSource};

/// The IPA text of a phrase and the words that had no pronunciation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transcription {
    /// HTML with `<b>` and `<br>`, empty when no word was found.
    pub html: String,
    /// The words with no pronunciation, as written, without repeats.
    pub missing: Vec<String>,
}

/// The words of a phrase, in order. A word is letters and digits, and it may have `'`, `’` or `-`
/// inside it (`don't`, `l'homme`, `well-known`). `’` is normalized to `'`.
pub fn words(phrase: &str) -> Vec<String> {
    let chars: Vec<char> = phrase.chars().collect();
    let is_connector = |c: char| matches!(c, '\'' | '’' | '-');
    let mut words = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_alphanumeric() {
            i += 1;
            continue;
        }
        let mut word = String::new();
        while i < chars.len() {
            let c = chars[i];
            let inside = is_connector(c) && chars.get(i + 1).is_some_and(|next| next.is_alphanumeric());
            if c.is_alphanumeric() || inside {
                word.push(if c == '’' { '\'' } else { c });
                i += 1;
            } else {
                break;
            }
        }
        words.push(word);
    }
    words
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// The pronunciation of one word, or of its parts when the word itself is not in the dictionary:
/// split after an apostrophe (`l'homme` is `l'` and `homme`) or at a hyphen.
fn pronounce(
    word: &str,
    language: &str,
    source: &dyn PronunciationSource,
) -> Result<Option<String>, PronunciationError> {
    if let Some(ipa) = source.lookup(language, word)? {
        return Ok(Some(ipa));
    }
    let parts: Vec<&str> = if word.contains('\'') {
        word.split_inclusive('\'').collect()
    } else if word.contains('-') {
        word.split('-').collect()
    } else {
        return Ok(None);
    };
    let mut joined = String::new();
    for part in parts {
        match source.lookup(language, part)? {
            Some(ipa) => joined.push_str(ipa.trim_matches('/')),
            None => return Ok(None),
        }
    }
    Ok(Some(format!("/{joined}/")))
}

/// Looks up every word of `phrase` for `language` (a BCP 47 code). Words repeated in the phrase
/// are listed once. Fails when the source itself cannot answer (not installed, no data for the
/// language).
pub fn transcribe(
    phrase: &str,
    language: &str,
    source: &dyn PronunciationSource,
) -> Result<Transcription, PronunciationError> {
    let mut seen = std::collections::HashSet::new();
    let mut lines = Vec::new();
    let mut missing = Vec::new();
    for word in words(phrase) {
        if !seen.insert(word.to_lowercase()) {
            continue;
        }
        match pronounce(&word, language, source)? {
            Some(ipa) => lines.push(format!("<b>{}</b>: {}", escape(&word), escape(&ipa))),
            None => missing.push(word),
        }
    }
    Ok(Transcription { html: lines.join("<br>"), missing })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Fake {
        entries: HashMap<(&'static str, &'static str), &'static str>,
    }

    impl Fake {
        fn new(entries: &[(&'static str, &'static str, &'static str)]) -> Self {
            Self { entries: entries.iter().map(|(l, w, i)| ((*l, *w), *i)).collect() }
        }
    }

    impl PronunciationSource for Fake {
        fn lookup(&self, language: &str, word: &str) -> Result<Option<String>, PronunciationError> {
            if language == "xx-XX" {
                return Err(PronunciationError::Unsupported(language.to_owned()));
            }
            let find = |w: &str| self.entries.iter().find(|((l, k), _)| *l == language && *k == w).map(|(_, i)| i.to_string());
            Ok(find(word).or_else(|| find(&word.to_lowercase())))
        }
    }

    #[test]
    fn words_skip_punctuation_and_keep_inner_connectors() {
        assert_eq!(words("Hello, everyone!"), vec!["Hello", "everyone"]);
        assert_eq!(words("don't stop"), vec!["don't", "stop"]);
        assert_eq!(words("l’homme"), vec!["l'homme"], "the typographic apostrophe is normalized");
        assert_eq!(words("well-known -- 'quoted' word"), vec!["well-known", "quoted", "word"]);
        assert_eq!(words("¿Qué tal? ¡Bien!"), vec!["Qué", "tal", "Bien"]);
        assert_eq!(words("Straße 42"), vec!["Straße", "42"]);
        assert!(words("... !?").is_empty());
    }

    #[test]
    fn one_line_per_word_with_the_word_in_bold() {
        let source = Fake::new(&[("en-GB", "hello", "/həˈləʊ/"), ("en-GB", "everyone", "/ˈevriwʌn/")]);
        let result = transcribe("Hello, everyone!", "en-GB", &source).unwrap();
        assert_eq!(result.html, "<b>Hello</b>: /həˈləʊ/<br><b>everyone</b>: /ˈevriwʌn/");
        assert!(result.missing.is_empty());
    }

    #[test]
    fn repeated_words_are_listed_once_and_unknown_ones_are_reported() {
        let source = Fake::new(&[("en-US", "go", "/ɡoʊ/")]);
        let result = transcribe("Go go GO zzz zzz", "en-US", &source).unwrap();
        assert_eq!(result.html, "<b>Go</b>: /ɡoʊ/");
        assert_eq!(result.missing, vec!["zzz"]);
        assert_eq!(transcribe("zzz", "en-US", &source).unwrap().html, "", "nothing found is an empty text");
    }

    #[test]
    fn an_elision_or_a_hyphenated_word_is_pronounced_from_its_parts() {
        let source = Fake::new(&[
            ("fr-FR", "l'", "/l/"),
            ("fr-FR", "homme", "/ɔm/"),
            ("en-US", "well", "/wɛl/"),
            ("en-US", "known", "/noʊn/"),
        ]);
        let french = transcribe("l'homme", "fr-FR", &source).unwrap();
        assert_eq!(french.html, "<b>l'homme</b>: /lɔm/");
        let english = transcribe("well-known", "en-US", &source).unwrap();
        assert_eq!(english.html, "<b>well-known</b>: /wɛlnoʊn/");
        assert_eq!(transcribe("l'inconnu", "fr-FR", &source).unwrap().missing, vec!["l'inconnu"], "every part is needed");
    }

    #[test]
    fn markup_in_a_word_is_escaped_and_source_errors_pass_through() {
        let source = Fake::new(&[("en-US", "a", "/ə/")]);
        assert_eq!(transcribe("a<b>", "en-US", &source).unwrap().html, "<b>a</b>: /ə/", "angle brackets are not word characters");
        assert!(matches!(transcribe("a", "xx-XX", &source), Err(PronunciationError::Unsupported(_))));
        assert_eq!(escape("a&b"), "a&amp;b");
    }
}
