// Author: Dylan Jones
// Date:   2025-11-01

#[cfg(feature = "napi")]
use napi::Error as NapiError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Enviroment variable {0} is not set")]
    EnvionmentNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to parse options.json: {0}")]
    OptionError(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("XML Error: {0}")]
    XmlError(String),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "napi")]
impl From<Error> for NapiError {
    fn from(error: Error) -> Self {
        Self::from_reason(error.to_string())
    }
}
