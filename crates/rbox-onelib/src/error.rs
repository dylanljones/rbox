// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Errors and result types for One Library database operations.
//!
//! Wraps shared database and core errors alongside filesystem failures, missing
//! paths, and date parsing errors that retain the invalid input and its cause.

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] rbox_core::error::Error),

    /// A required path is absent or has the wrong kind (file versus directory).
    #[error("path not found: {0}")]
    FileNotFound(std::path::PathBuf),

    #[error(transparent)]
    Database(#[from] rbox_db::error::Error),

    #[error("File operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid datetime '{value}': {source}")]
    InvalidDateTime {
        value: String,
        #[source]
        source: chrono::ParseError,
    },
}

impl From<diesel::result::Error> for Error {
    fn from(error: diesel::result::Error) -> Self {
        Self::Database(error.into())
    }
}

impl From<diesel::r2d2::PoolError> for Error {
    fn from(error: diesel::r2d2::PoolError) -> Self {
        Self::Database(error.into())
    }
}
