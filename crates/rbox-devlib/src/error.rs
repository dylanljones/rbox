// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Errors shared by DeviceSQL parsing, serialization, and text conversion.

use crate::binary::core::TableKind;
use crate::binary::PageIndex;

/// Result returned by DeviceSQL's fallible public helpers.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned while reading, authoring, or writing DeviceSQL data.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An operation requires a library's absent extended database.
    #[error("this Device Library has no extended database")]
    MissingExtendedDatabase,

    /// Standard and extended paths resolve to the same file name.
    #[error("standard and extended database paths refer to the same file: {0}")]
    AliasedDatabaseFiles(std::path::PathBuf),

    /// A library write failed. Completion refers to the standard-file operation,
    /// including a successful no-op save, not atomic publication of both files.
    /// The failing file can also be partially modified after a direct page write.
    #[error("Device Library {operation} failed for {database:?} (standard completed: {standard_completed}): {source}")]
    LibraryWrite {
        operation: &'static str,
        database: crate::binary::DatabaseKind,
        standard_completed: bool,
        #[source]
        source: Box<Error>,
    },

    /// A value supplied to a model cannot be used for authoring.
    #[error("invalid {field} for {table:?}: {reason}")]
    InvalidModelField {
        table: TableKind,
        field: &'static str,
        reason: &'static str,
    },

    /// The selected row does not have a supported layout for the requested model.
    #[error("unsupported model row {slot} on page {page:?} in {table:?}")]
    UnsupportedModelRow {
        page: PageIndex,
        table: TableKind,
        slot: u16,
    },

    /// A model field could not be converted from its original string frame.
    #[error("cannot decode {field} in row {slot} on page {page:?} in {table:?}: {source}")]
    ModelText {
        page: PageIndex,
        table: TableKind,
        slot: u16,
        field: &'static str,
        #[source]
        source: Box<Error>,
    },

    /// Explicit initialization metadata cannot form a committed new database.
    #[error("invalid DeviceSQL creation options: {reason}")]
    InvalidCreation { reason: &'static str },

    /// A mutation cannot be represented by the currently supported writer.
    #[error("unsupported mutation in {table:?}: {reason}")]
    UnsupportedMutation {
        table: TableKind,
        reason: &'static str,
    },

    /// A physical row is absent or outside the selected table's linked pages.
    #[error("no live row {slot} on page {page:?} in {table:?}")]
    RowNotFound {
        table: TableKind,
        page: PageIndex,
        slot: u16,
    },

    /// A handle belongs to another transaction or its row was changed/deleted.
    #[error("foreign or stale transaction row handle")]
    InvalidRowHandle,

    /// The row allocation and directory cannot fit within one data page.
    #[error("no data page in {table:?} can fit {bytes} allocated bytes")]
    NoRowSpace { table: TableKind, bytes: usize },

    /// Checked sequence/revision arithmetic would overflow.
    #[error("DeviceSQL mutation counter overflow")]
    MutationOverflow,

    /// Source metadata changed externally or the opened path disappeared.
    #[error("DeviceSQL source changed since opening: {0}")]
    SourceChanged(std::path::PathBuf),

    /// A failed direct write invalidated this connection; discard it and reopen
    /// after the application has restored or repaired the file as appropriate.
    #[error("DeviceSQL connection invalidated by a failed direct write: {0}")]
    ConnectionInvalidated(std::path::PathBuf),

    /// Saving back requires a connection opened with a filesystem path.
    #[error("save requires a connection opened by DeviceSqlConnection::open")]
    NoSourcePath,

    /// The unique dirty-page budget would be exceeded; the operation is not staged.
    #[error("dirty page limit {limit} exceeded: {required} pages required")]
    DirtyPageLimit { limit: usize, required: usize },

    /// Publishing a saved copy or full-file replacement failed.
    #[error("cannot save DeviceSQL file to {path}: {source}")]
    Save {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// A direct page write was attempted and failed. The destination may be
    /// partially updated and the connection is now invalidated.
    #[error(
        "DeviceSQL page save failed at {path}; destination may be partially updated: {source}"
    )]
    PageSave {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// A staged physical page has an invalid index or byte length.
    #[error("invalid staged page {page}: {reason}")]
    InvalidPageWrite { page: u64, reason: &'static str },

    /// A typed operation was requested in the wrong database namespace.
    #[error("{table:?} is not a table in a {database:?} database")]
    WrongDatabaseKind {
        database: crate::binary::DatabaseKind,
        table: TableKind,
    },

    /// More than one directory entry declares the same table code.
    #[error("duplicate table directory code {0}")]
    DuplicateTable(u32),

    /// An ID lookup cannot resolve an ambiguous live-row identity.
    #[error("duplicate live ID {id} in {table:?}")]
    DuplicateId { table: TableKind, id: u32 },

    /// A model mutation would violate a unique constraint on these fields.
    #[error("unique constraint failed: {table:?} {fields:?}")]
    UniqueConstraintViolation {
        table: TableKind,
        fields: &'static [&'static str],
    },

    /// A model update could not find its original ID in the current database.
    #[error("no model with ID {id} in {table:?}")]
    ModelNotFound { table: TableKind, id: u32 },

    /// A model without a unique ID no longer matches its original row location.
    #[error("model row {slot} on page {page:?} in {table:?} changed since it was read")]
    StaleModelRow {
        table: TableKind,
        page: PageIndex,
        slot: u16,
    },

    /// An underlying stream or filesystem operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// A binary structure is invalid, truncated, or could not be serialized.
    #[error(transparent)]
    Binary(#[from] binrw::Error),

    /// Unsupported page size, missing header page, or an incomplete physical page.
    #[error("invalid file layout: {file_size} bytes with page size {page_size}")]
    InvalidFileLayout { file_size: u64, page_size: u32 },

    /// A chain endpoint or successor references page zero, a sentinel, or a page beyond EOF.
    #[error("table directory entry {table_index} references invalid page {page:?}")]
    InvalidPageReference { table_index: usize, page: PageIndex },

    /// A page was revisited before reaching the table's declared last page.
    #[error("cycle in table directory entry {table_index} at page {page:?}")]
    PageCycle { table_index: usize, page: PageIndex },

    /// Two table-directory entries claim the same physical page.
    #[error("page {page:?} is shared by table directory entries {first_table} and {second_table}")]
    SharedPage {
        page: PageIndex,
        first_table: usize,
        second_table: usize,
    },

    /// The stored self-index does not match the page's physical location.
    #[error("page {expected:?} in table directory entry {table_index} has self-index {actual:?}")]
    PageIndexMismatch {
        table_index: usize,
        expected: PageIndex,
        actual: PageIndex,
    },

    /// A linked page's raw table code disagrees with its directory entry.
    #[error("page {page:?} in table directory entry {table_index} has table code {actual}, expected {expected}")]
    PageTableMismatch {
        table_index: usize,
        page: PageIndex,
        expected: u32,
        actual: u32,
    },

    /// A linked page header could not be decoded.
    #[error("cannot read page {page:?} in table directory entry {table_index}: {source}")]
    PageRead {
        table_index: usize,
        page: PageIndex,
        #[source]
        source: binrw::Error,
    },

    /// Counts cannot fit the page header's 13-bit allocated / 11-bit live fields.
    #[error("page row counts exceed their bit widths: {allocated} allocated, {live} live")]
    InvalidRowCounts { allocated: u16, live: u16 },

    /// A data-page directory violates a structural bound or count invariant.
    #[error("invalid row directory on page {page:?}: {reason}")]
    InvalidRowDirectory {
        page: PageIndex,
        reason: &'static str,
    },

    /// Reading a data-page directory failed at the underlying stream.
    #[error("cannot read row directory on page {page:?}: {source}")]
    RowDirectoryRead {
        page: PageIndex,
        #[source]
        source: std::io::Error,
    },

    /// An index-entry array or containing page violates a structural bound.
    #[error("invalid index entries on page {page:?}: {reason}")]
    InvalidIndexEntries {
        page: PageIndex,
        reason: &'static str,
    },

    /// Reading the declared index-entry array failed.
    #[error("cannot read index entries on page {page:?}: {source}")]
    IndexEntriesRead {
        page: PageIndex,
        #[source]
        source: binrw::Error,
    },

    /// Reading a selected live-row window failed.
    #[error("cannot read row {slot} on page {page:?} in {table:?}: {source}")]
    RowRead {
        page: PageIndex,
        table: TableKind,
        slot: u16,
        #[source]
        source: std::io::Error,
    },

    /// A supported row layout is malformed or exceeds its bounded window.
    #[error("cannot decode row {slot} on page {page:?} in {table:?}: {source}")]
    RowDecode {
        page: PageIndex,
        table: TableKind,
        slot: u16,
        #[source]
        source: binrw::Error,
    },

    /// An ASCII representation contained non-ASCII bytes or text.
    #[error("non-ASCII data in an ASCII string")]
    NonAscii,

    /// Invalid UTF-16, including an odd number of payload bytes.
    #[error("invalid UTF-16 payload")]
    InvalidUtf16,

    /// An explicit string encoding cannot represent the frame length.
    #[error("string exceeds its encoding's length limit")]
    StringTooLong,

    /// A string supplied for authoring contained an embedded NUL.
    #[error("embedded NUL in authored text")]
    EmbeddedNul,

    /// Invalid ISRC input or an unrecognized ISRC frame.
    #[error("not a recognized ISRC value or frame")]
    InvalidIsrc,

    /// Unknown text marker; the original frame can still be copied.
    #[error("unsupported text marker {0:#x}")]
    UnsupportedStringMarker(u8),
}
