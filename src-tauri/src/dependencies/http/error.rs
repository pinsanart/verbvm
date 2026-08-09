use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("erro de rede: {0}")]
    Request(#[from] reqwest::Error),

    #[error("http {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
}