//! Text to speech. The rest of the app depends on [`SpeechSynthesizer`] only, so a provider can
//! be swapped or added without touching the callers.

pub mod error;
pub mod google_translate;

pub use error::SpeechError;
pub use google_translate::GoogleTranslateSynthesizer;

use async_trait::async_trait;

/// Audio produced by a provider. `bytes` is a complete audio file; the media store detects its
/// type from the content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesizedAudio {
    pub bytes: Vec<u8>,
}

/// A text to speech provider.
#[async_trait]
pub trait SpeechSynthesizer: Send + Sync {
    /// Speaks `text` in the language given as a BCP 47 code, such as `pt-BR`. Plain text only.
    async fn synthesize(&self, text: &str, bcp47: &str) -> Result<SynthesizedAudio, SpeechError>;
}
