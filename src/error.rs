use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("API error (status {status}): {message}")]
    Api {
        status: reqwest::StatusCode,
        message: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Server refresh failed")]
    ServerRefreshFailed,

    #[error("Server ({0}) has no connections")]
    ServerNotFound(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl Error {
    pub(crate) fn is_transport_failure(&self) -> bool {
        match self {
            Error::Network(e) => {
                e.is_connect()
                    || e.is_timeout()
                    || (e.status().is_none()
                        && !e.is_decode()
                        && !e.is_builder()
                        && !e.is_redirect())
            }
            _ => false,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
