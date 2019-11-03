#[derive(Debug, failure::Fail)]
pub enum Error {
  #[fail(display = "request: {}", 0)]
  Request(reqwest::Error),
  #[fail(display = "invalid sign in page")]
  InvalidSignInPage,
  #[fail(display = "invalid watch page")]
  InvalidWatchPage,
  #[fail(display = "invalid info: {}", 0)]
  InvalidInfo(serde_json::Error),
  #[fail(display = "invalid key")]
  InvalidKey,
  #[fail(display = "not authorized")]
  NotAuthorized,
  #[fail(display = "serialization error")]
  Serialization,
}

impl From<reqwest::Error> for Error {
  fn from(err: reqwest::Error) -> Self {
    Self::Request(err)
  }
}

pub type Result<T> = std::result::Result<T, Error>;
