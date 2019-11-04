#[derive(Debug, failure::Fail)]
pub enum Error {
  #[fail(display = "{}", 0)]
  Nicodo(nicodo::Error),
  #[fail(display = "failed to write config file")]
  Config(std::io::Error),
  #[fail(display = "user_session must be specified")]
  UserSessionMustBeSpecified,
  #[fail(display = "failed to write comment file")]
  Write(std::io::Error),
  #[fail(display = "invalid date")]
  Date,
  #[fail(display = "invalid duration")]
  Duration,
  #[fail(display = "invalid period")]
  Period,
}

impl From<nicodo::Error> for Error {
  fn from(err: nicodo::Error) -> Self {
    Self::Nicodo(err)
  }
}

pub type Result<T> = std::result::Result<T, Error>;
