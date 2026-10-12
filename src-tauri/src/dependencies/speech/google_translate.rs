//! [`SpeechSynthesizer`] over the Google Translate text to speech endpoint.
//!
//! The endpoint is unofficial: it is not documented, has no key and may rate limit or change
//! without notice. It accepts about 200 characters per request, so longer text is refused rather
//! than cut.

use async_trait::async_trait;

use super::{SpeechError, SpeechSynthesizer, SynthesizedAudio};
use crate::dependencies::http::http::{Auth, HttpClient};

const ENDPOINT: &str = "https://translate.google.com/translate_tts";
const MAX_CHARS: usize = 200;
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0";

pub struct GoogleTranslateSynthesizer {
    http: HttpClient,
}

impl GoogleTranslateSynthesizer {
    pub fn new() -> Self {
        Self { http: HttpClient::with_user_agent(USER_AGENT) }
    }
}

impl Default for GoogleTranslateSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

/// The `tl` value for a BCP 47 code. Google takes a few regional codes as they are and the bare
/// language for the rest.
fn language_param(bcp47: &str) -> &str {
    match bcp47 {
        "pt-BR" | "pt-PT" | "zh-CN" | "zh-TW" => bcp47,
        _ => bcp47.split('-').next().unwrap_or(bcp47),
    }
}

#[async_trait]
impl SpeechSynthesizer for GoogleTranslateSynthesizer {
    async fn synthesize(&self, text: &str, bcp47: &str) -> Result<SynthesizedAudio, SpeechError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(SpeechError::EmptyText);
        }
        if text.chars().count() > MAX_CHARS {
            return Err(SpeechError::TextTooLong { max: MAX_CHARS });
        }
        let query = [
            ("ie", "UTF-8"),
            ("client", "tw-ob"),
            ("tl", language_param(bcp47)),
            ("q", text),
        ];
        let bytes = self
            .http
            .get_bytes(ENDPOINT, &query, Auth::None)
            .await
            .map_err(|e| SpeechError::Unavailable(e.to_string()))?;
        Ok(SynthesizedAudio { bytes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regional_codes_are_trimmed_to_the_language_unless_google_distinguishes_them() {
        assert_eq!(language_param("en-US"), "en");
        assert_eq!(language_param("fr-FR"), "fr");
        assert_eq!(language_param("pt-BR"), "pt-BR");
        assert_eq!(language_param("ja"), "ja");
    }

    #[tokio::test]
    async fn refuses_empty_and_long_text_without_a_request() {
        let tts = GoogleTranslateSynthesizer::new();
        assert!(matches!(tts.synthesize("  ", "en-US").await, Err(SpeechError::EmptyText)));
        let long = "a".repeat(MAX_CHARS + 1);
        assert!(matches!(tts.synthesize(&long, "en-US").await, Err(SpeechError::TextTooLong { .. })));
    }
}
