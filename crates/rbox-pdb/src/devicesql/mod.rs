// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: MIT

//! DeviceSQL storage engine using binary representations.
//!
//! [`connection`] owns file access, creation, caches, and persistence. [`transaction`]
//! owns staged row mutations, allocation, commit, and rollback. [`core`] and
//! [`encoding`] provide shared storage primitives and placement-aware row codecs.

pub mod connection;
pub mod core;
mod creation;
pub mod encoding;
pub mod transaction;

pub use connection::{DeviceSqlConnection, PageSaveStats, RebuildStats};
pub use core::{RawRows, ReadStats, ResourceLimits, ResourceUsage, SaveStats};
pub use creation::{CreateOptions, InitialTable};
pub use transaction::{RowHandle, Transaction};
