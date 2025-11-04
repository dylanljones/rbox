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

    #[error("{0}")]
    Error(String),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error(transparent)]
    BinReadError(#[from] binrw::error::Error),

    #[error("XML Error: {0}")]
    XmlError(String),

    #[error("Failed to parse options.json: {0}")]
    OptionError(String),

    #[error("Setting Error: {0}")]
    SettingError(String),

    #[error("Invalid {0} value: {1}")]
    SettingInvalidValue(String, String),

    #[error("Not a {0} file")]
    SettingInvalidFile(String),

    #[error("Anlz Error: {0}")]
    AnlzError(String),

    #[error("Diesel error: {0}")]
    Diesel(#[from] diesel::result::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Connection error: {0}")]
    Connection(#[from] diesel::ConnectionError),

    #[error("Rekordbox is running. Please close Rekordbox and try again.")]
    RekordboxRunning,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Error(s)
    }
}

impl From<quick_xml::de::DeError> for Error {
    fn from(error: quick_xml::de::DeError) -> Self {
        Error::XmlError(error.to_string())
    }
}

impl From<quick_xml::se::SeError> for Error {
    fn from(error: quick_xml::se::SeError) -> Self {
        Error::XmlError(error.to_string())
    }
}

#[cfg(feature = "napi")]
impl From<Error> for NapiError {
    fn from(error: Error) -> Self {
        Self::from_reason(error.to_string())
    }
}
