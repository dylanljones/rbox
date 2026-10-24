// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Read, create, and edit Rekordbox DeviceSQL exports (`export.pdb` and `exportExt.pdb`).
//!
//! [`binary`] provides on-disk structures and lossless codecs. [`devicesql`] provides
//! lazy reads, transactional binary edits, page allocation, and persistence through
//! [`DeviceSqlConnection`]. Connection and transaction types are also re-exported
//! at the crate root.
//!
//! This crate is experimental. Applications are responsible for backups and recovery;
//! [`DeviceSqlConnection::save_pages`] writes changed pages non-atomically.

pub mod binary;
pub mod devicesql;
pub mod error;

pub use devicesql::{
    CreateOptions, DeviceSqlConnection, InitialTable, PageSaveStats, RawRows, ReadStats,
    RebuildStats, ResourceLimits, ResourceUsage, RowHandle, SaveStats, Transaction,
};
pub use error::{Error, Result};
