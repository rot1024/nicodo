#[derive(Debug, failure::Fail)]
pub enum Error {
  #[fail(display = "{}", 0)]
  Nicodo(nicodo::Error),
  #[fail(display = "failed to write config file")]
  Config(std::io::Error),
  #[fail(display = "session must be specified")]
  SessionMustBeSpecified,
}

impl From<nicodo::Error> for Error {
  fn from(err: nicodo::Error) -> Self {
    Self::Nicodo(err)
  }
}

pub type Result<T> = std::result::Result<T, Error>;
