// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Errors from Rekordbox configuration and path discovery.
//!
//! Discovery distinguishes missing environment variables, unusable paths,
//! invalid configuration, and failures reported by the operating system or
//! parsers. [`Error::Io`], [`Error::Json`], and [`Error::Xml`] retain their
//! underlying errors rather than flattening them into messages.
//!
//! A missing selected database is not always an error:
//! [`crate::locate::locate_library`] returns its location with `available == false`.
//! [`crate::locate::master_db_path`] instead returns [`Error::FileNotFound`].
//! Missing configuration files permit fallback during discovery, whereas calling
//! [`crate::RekordboxOptions::load`] directly returns the file-opening error.

/// Result type shared by configuration and discovery operations.
pub type Result<T> = std::result::Result<T, Error>;

/// A configuration or discovery failure.
///
/// This enum is non-exhaustive; callers should retain a fallback match arm.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A required platform environment variable is missing or empty.
    #[error("environment variable {0} is missing or empty")]
    EnvironmentNotFound(&'static str),

    /// A required path is absent or has the wrong kind (file versus directory).
    #[error("path not found: {0}")]
    FileNotFound(std::path::PathBuf),

    /// Parsed configuration is missing a required value or violates lookup rules.
    #[error("invalid Rekordbox options: {0}")]
    OptionError(String),

    /// A filesystem operation failed, including permission and file-opening errors.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Agent options are invalid JSON or do not have the expected JSON structure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Application preferences could not be deserialized from XML.
    #[error(transparent)]
    Xml(#[from] quick_xml::DeError),

    /// Automatic program lookup is unavailable on the current platform.
    /// Explicit roots can still be inspected with [`crate::locate::find_programs_in`].
    #[error("unsupported platform for Rekordbox program discovery")]
    UnsupportedPlatform,
}
