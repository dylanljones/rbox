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
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Failed to parse options.json: {0}")]
    OptionError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "napi")]
impl From<Error> for NapiError {
    fn from(error: Error) -> Self {
        Self::from_reason(error.to_string())
    }
}
