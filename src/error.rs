use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Nicodo(nicodo::Error),
    #[error("failed to write config file")]
    Config(std::io::Error),
    #[error("user_session must be specified")]
    UserSessionMustBeSpecified,
    #[error("start, end, or interval is missing")]
    Period,
    #[error("failed to write comment file")]
    Write(std::io::Error),
    #[error("error: {0}")]
    Error(Box<dyn std::error::Error + Send>),
}

impl From<nicodo::Error> for Error {
    fn from(err: nicodo::Error) -> Self {
        Self::Nicodo(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
