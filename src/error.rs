use derive_more::From;
use thiserror::Error;

#[derive(Debug, Error, From)]
pub enum Error {
    #[error("{0}")]
    Nicodo(nicodo::Error),
    #[error("{0}")]
    Config(confy::ConfyError),
    #[error("start, end, or interval is missing")]
    Period,
    #[error("{0}")]
    IO(std::io::Error),
    #[error("{0}")]
    Error(Box<dyn std::error::Error + Send>),
}

pub type Result<T> = std::result::Result<T, Error>;
