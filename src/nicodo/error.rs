use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("request: {0}")]
    Request(reqwest::Error),
    #[error("invalid sign in page")]
    InvalidSignInPage,
    #[error("invalid watch page")]
    InvalidWatchPage,
    #[error("invalid info: {0}")]
    InvalidInfo(serde_json::Error),
    #[error("invalid key")]
    InvalidKey,
    #[error("not authorized")]
    NotAuthorized,
    #[error("serialization error")]
    Serialization,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Request(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
