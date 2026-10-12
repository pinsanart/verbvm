use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpeechError {
    #[error("there is no text to speak")]
    EmptyText,

    #[error("the text is too long to speak: at most {max} characters")]
    TextTooLong { max: usize },

    #[error("the speech service could not be reached: {0}")]
    Unavailable(String),
}
