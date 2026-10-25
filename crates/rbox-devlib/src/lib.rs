// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Read, create, and edit Rekordbox DeviceSQL exports (`export.pdb` and `exportExt.pdb`).
//!
//! [`binary`] provides on-disk structures and lossless codecs. [`devicesql`] provides
//! lazy reads, transactional binary edits, page allocation, and persistence through
//! [`DeviceSqlConnection`]. Connection and transaction types are also re-exported
//! at the crate root.
//! [`models`] adds detached track and catalog models with explicit reads and insertions.
//! [`DeviceLibrary`] opens, creates, and saves the standard and extended files together.
//!
//! This crate is experimental. Applications are responsible for backups and recovery;
//! [`DeviceSqlConnection::save_pages`] writes changed pages non-atomically.

pub mod binary;
pub mod database;
pub mod devicesql;
pub mod error;
pub mod models;

pub use binary::DatabaseKind;
pub use database::{DeviceLibrary, LibraryCreateOptions, LibrarySaveStats};
pub use devicesql::{CreateOptions, DeviceSqlConnection, Transaction};
pub use error::{Error, Result};
