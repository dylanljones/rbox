// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Low-level DeviceSQL structures, bounded row decoding, and lossless field codecs.
//!
//! These APIs expose the on-disk representation for both database namespaces.
//! They do not implement database editing or page allocation. Prefer
//! [`crate::devicesql::DeviceSqlConnection`] for lazy reads and transactional binary edits.

pub mod chain;
pub mod core;
pub mod rows;
pub mod string;

pub use chain::{read_chain_page, read_table_chains, validate_reference, TableChain};
pub use core::{
    read_index_entries, read_row_directory, CommonPageHeader, DataPageHeader, DatabaseKind,
    ExtPageType, FileHeader, FileHeaderPrefix, IndexEntry, IndexPageHeader, PackedRowCounts,
    PageFlags, PageHeader, PageHeaderKind, PageIndex, PageType, RowDirectory, RowGroup, RowSlot,
    TableKind, TablePointer, NO_PAGE, PAGE_HEADER_SIZE, PAGE_SIZE,
};

pub use rows::*;
pub use string::{DeviceSqlString, StringEncoding};
