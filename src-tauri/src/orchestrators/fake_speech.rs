//! A [`SpeechSynthesizer`] for tests: no network, records what it was asked to speak.

use std::sync::Mutex;

use async_trait::async_trait;

use crate::dependencies::speech::{SpeechError, SpeechSynthesizer, SynthesizedAudio};

/// A minimal MP3 header the media store accepts. Generated audio starts with it.
pub const FAKE_MP3: &[u8] = b"ID3\x03\x00\x00\x00\x00\x00\x00audio-bytes";

#[derive(Default)]
pub struct FakeSpeech {
    pub fail: bool,
    /// The `(text, language tag)` of each request.
    pub requests: Mutex<Vec<(String, String)>>,
}

#[async_trait]
impl SpeechSynthesizer for FakeSpeech {
    async fn synthesize(&self, text: &str, bcp47: &str) -> Result<SynthesizedAudio, SpeechError> {
        self.requests.lock().unwrap().push((text.to_owned(), bcp47.to_owned()));
        if self.fail {
            return Err(SpeechError::Unavailable("offline".into()));
        }
        // The text is appended, so different phrases give different files, as a real voice would.
        let mut bytes = FAKE_MP3.to_vec();
        bytes.extend_from_slice(text.as_bytes());
        Ok(SynthesizedAudio { bytes })
    }
}
