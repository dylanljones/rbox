// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Errors and result types for master database operations.
//!
//! Combines shared database and filesystem errors with analysis-file failures,
//! playlist XML serialization errors, invalid operations, and date parsing errors.

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] rbox_core::error::Error),

    #[error(transparent)]
    Anlz(#[from] rbox_anlz::error::AnlzError),

    #[error("Rekordbox is running. Close it before modifying the database.")]
    RekordboxRunning,

    #[error(transparent)]
    Database(#[from] rbox_db::error::Error),

    #[error("File operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse XML: {0}")]
    XmlRead(#[from] quick_xml::DeError),

    #[error("Failed to serialize XML: {0}")]
    XmlWrite(#[from] quick_xml::SeError),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Invalid datetime '{value}': {source}")]
    InvalidDateTime {
        value: String,
        #[source]
        source: chrono::ParseError,
    },

    #[error("Failed to generate a valid ID after {attempts} attempts")]
    IdGeneration { attempts: usize },
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
