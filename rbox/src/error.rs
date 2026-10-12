// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Errors shared by applications using multiple Rbox components.
//!
//! Component methods retain their own error types. Their errors convert into
//! [`Error`] with `?` in functions returning [`Result`], preserving the original
//! error values and messages. Optional variants follow the component features.

/// A result that can contain an error from any enabled Rbox component.
pub type Result<T> = std::result::Result<T, Error>;

/// A failure from a Rbox component.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Configuration, path discovery, or other core utility failure.
    #[error(transparent)]
    Core(#[from] rbox_core::error::Error),

    /// Shared database connection, pool, or query failure.
    #[cfg(any(feature = "master-db", feature = "one-library"))]
    #[error(transparent)]
    Database(#[from] rbox_db::error::Error),

    /// Master database operation failure.
    #[cfg(feature = "master-db")]
    #[error(transparent)]
    MasterDb(#[from] rbox_masterdb::error::Error),

    /// One Library database operation failure.
    #[cfg(feature = "one-library")]
    #[error(transparent)]
    OneLibrary(#[from] rbox_onelib::error::Error),

    /// Analysis file parsing, editing, or writing failure.
    #[cfg(feature = "anlz")]
    #[error(transparent)]
    Anlz(#[from] rbox_anlz::error::AnlzError),

    /// Rekordbox XML operation failure.
    #[cfg(feature = "xml")]
    #[error(transparent)]
    Xml(#[from] rbox_xml::XmlError),

    /// Device settings operation failure.
    #[cfg(feature = "settings")]
    #[error(transparent)]
    Setting(#[from] rbox_setting::SettingError),
}
