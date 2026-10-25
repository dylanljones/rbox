// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! DeviceSQL connection lifecycle, reads, creation, resource budgets, and persistence.
//!
//! File operations use binary representations independently of application models.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;

use binrw::{BinRead, BinWrite};
use tempfile::NamedTempFile;

use super::core::{
    replace_file, PageChanges, PageStore, RawRows, ReadStats, ResourceLimits, ResourceUsage,
    RowLocation, SaveStats, SourceFile,
};
use super::creation::{empty_index, CreateOptions, INDEX_CAPACITY};
use super::encoding as row_encoding;
use super::transaction::Transaction;
use crate::binary::chain::read_chain_page;
use crate::binary::{
    read_row, read_row_directory, DatabaseKind, FileHeader, FileHeaderPrefix, IndexEntry,
    PageHeader, PageIndex, RowRecord, TableKind, TablePointer, PAGE_SIZE,
};
use crate::{Error, Result};

/// A DeviceSQL source with lazy reads, a configurable clean cache, and page overlays.
///
/// Opening reads only page zero and checks file length and table-code uniqueness.
/// Chains and row directories are validated as traversed, not at open time.
/// Iteration uses linked-page order, then live slot order, not ID or playlist order.
/// The source must not be modified externally while this handler is open.
/// The clean cache defaults to 16 pages; [`ResourceLimits`] can cap retained
/// clean and dirty buffers. File-backed [`Self::save`] releases dirty buffers
/// after replacing the source; [`Self::save_as`] retains the original source.
/// Saving preserves complete physical pages, including unlinked and opaque data.
/// Transactions support binary row mutations and conservative page allocation.
/// Unsupported allocation profiles return errors; physical-device qualification
/// remains separate from structural saved-file validation.
///
/// ```no_run
/// use rbox_pdb::binary::{DatabaseKind};
/// use rbox_pdb::devicesql::DeviceSqlConnection;
/// # fn main() -> rbox_pdb::Result<()> {
/// let mut db = DeviceSqlConnection::open("export.pdb", DatabaseKind::Standard)?;
/// for table in db.header().tables.clone() {
///     let kind = db.kind().table_kind(table.table_type);
///     for row in db.iter_raw(kind)? {
///         println!("{:?}", row?.page());
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct DeviceSqlConnection<R = File> {
    pub(super) input: PageStore<R>,
    pub(super) header: FileHeader,
    pub(super) kind: DatabaseKind,
    pub(super) page_count: u64,
    pub(super) owners: HashMap<PageIndex, usize>,
    pub(super) indexes: HashMap<u32, HashMap<u32, RowLocation>>,
    pub(super) source_file: Option<SourceFile>,
}

impl DeviceSqlConnection<File> {
    /// Open a file without modifying it. The namespace must be supplied explicitly.
    pub fn open(path: impl AsRef<Path>, kind: DatabaseKind) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        let file = File::open(&path)?;
        let source = SourceFile::new(path, &file)?;
        let mut connection = Self::from_reader(file, kind)?;
        source.check()?;
        connection.source_file = Some(source);
        Ok(connection)
    }

    /// Atomically replace the opened path with the committed image, then rebase.
    ///
    /// Writes a temporary sibling, retains permissions, flushes/syncs its contents,
    /// and replaces the destination. Failure before replacement retains the source
    /// and committed overlay. Successful replacement adopts the already-open new
    /// handle (no fallible reopen), releases dirty pages, and clears read indexes.
    /// A no-op save performs no file writes. This is a full-file replacement, not
    /// selective page writes or a transaction across the two database files.
    ///
    /// Length and modification/creation timestamps are checked before and after
    /// staging. The driver does not lock files: applications must exclude other writers and
    /// path/hard-link aliases. Edits preserving metadata
    /// and changes between the last check and replacement may evade detection.
    /// Directory-entry durability across power loss depends on the filesystem.
    /// Only connections opened with `open` have a source path; use `save_as` for
    /// in-memory databases, then open the result to edit it.
    pub fn save(&mut self) -> Result<SaveStats> {
        let source = self.source_file.as_ref().ok_or(Error::NoSourcePath)?;
        source.check()?;
        if self.modified_page_count() == 0 {
            return Ok(SaveStats::default());
        }
        let (file, replacement) = replace_file(source, |file| {
            self.input.write_to(file)?;
            Ok(())
        })?;
        let bytes_written = replacement.len();
        self.input.rebase(file, bytes_written);
        self.source_file = Some(replacement);
        self.owners.clear();
        self.indexes.clear();
        Ok(SaveStats {
            pages_written: self.page_count,
            bytes_written,
        })
    }

    /// Write committed changed/new pages directly to the opened file and sync it.
    ///
    /// This is a non-atomic save. The application owns backups and recovery: an
    /// error or interruption may leave a partially updated database. After an
    /// attempted write fails, discard this connection, restore or repair the file
    /// as appropriate, and reopen it. No automatic rollback or recovery occurs.
    /// Prefer [`Self::save`] for full-file replacement through a temporary file.
    ///
    /// Writes data/index pages before page zero; that order does not make the save
    /// atomic. Unchanged pages are neither read nor written. No-op saves write
    /// nothing. Success adopts the updated file and releases dirty/read/index
    /// buffers. Failure retains the overlay but invalidates the source once
    /// writing has been attempted, preventing further use of potentially stale data.
    /// The failing write returns [`Error::PageSave`]; subsequent data operations
    /// return [`Error::ConnectionInvalidated`]. Detected external changes use
    /// [`Error::SourceChanged`] instead.
    ///
    /// The driver does not lock files. Applications must exclude other readers
    /// and writers, including Rekordbox and path/hard-link aliases, during the
    /// save. This is not a transaction across database files.
    pub fn save_pages(&mut self) -> Result<PageSaveStats> {
        self.save_pages_with(&mut |_| Ok(()))
    }

    fn save_pages_with(
        &mut self,
        step: &mut impl FnMut(Step) -> io::Result<()>,
    ) -> Result<PageSaveStats> {
        let source = self.source_file.as_ref().ok_or(Error::NoSourcePath)?;
        source.check()?;
        if self.modified_page_count() == 0 {
            return Ok(PageSaveStats::default());
        }
        let path = source.path.clone();
        let mut target = OpenOptions::new().read(true).write(true).open(&path)?;
        source.check()?;
        let size = u64::from(self.header.prefix.page_size);
        let length = self.page_count * size;
        let pages_written = self.modified_page_count() as u64;
        let resize = target.metadata()?.len() != length;
        // Only successful completion can make this source usable again. Metadata
        // alone cannot reliably detect a failed write or sync on every filesystem.
        self.source_file
            .as_mut()
            .ok_or(Error::NoSourcePath)?
            .invalidate();
        let result = (|| -> Result<SourceFile> {
            if resize {
                target.set_len(length)?;
            }
            step(Step::Extended)?;
            let changes = self.input.changes();
            for (&page, bytes) in changes.range(1..).chain(changes.range(..1)) {
                target.seek(SeekFrom::Start(page * size))?;
                target.write_all(bytes)?;
                step(Step::Page(page))?;
            }
            target.sync_all()?;
            step(Step::DatabaseSynced)?;
            SourceFile::new(path.clone(), &target)
        })();
        let source = result.map_err(|error| match error {
            Error::Io(source) => Error::PageSave { path, source },
            error => error,
        })?;
        self.input.rebase(target, length);
        self.source_file = Some(source);
        self.owners.clear();
        self.indexes.clear();
        Ok(PageSaveStats {
            pages_written,
            bytes_written: pages_written * size,
        })
    }

    /// Explicitly discard obsolete physical data and replace the opened file.
    ///
    /// Has the same strict profile and data-discarding contract as [`Self::rebuild`].
    /// Rebuilds fully before replacing anything; failure retains the source and
    /// pending edits. Successful replacement adopts the new header/file and releases
    /// dirty buffers. This writes the whole compacted file, never a partial-page
    /// shrink. File size can stay equal or increase for already densely packed
    /// inputs because the chosen creation profile reserves candidate pages.
    pub fn compact(&mut self) -> Result<RebuildStats> {
        if self.source_file.is_none() {
            return Err(Error::NoSourcePath);
        }
        let (mut rebuilt, stats) = self.rebuild()?;
        let source = self.source_file.as_ref().ok_or(Error::NoSourcePath)?;
        let (file, source) = replace_file(source, |file| {
            rebuilt.write_to(file)?;
            Ok(())
        })?;
        let length = rebuilt.page_count * u64::from(rebuilt.header.prefix.page_size);
        self.input.rebase(file, length);
        self.header = rebuilt.header;
        self.page_count = rebuilt.page_count;
        self.source_file = Some(source);
        self.owners.clear();
        self.indexes.clear();
        Ok(stats)
    }
}

impl<R: Read + Seek> DeviceSqlConnection<R> {
    /// Own a seekable source, reading its header from absolute offset zero.
    pub fn from_reader(mut input: R, kind: DatabaseKind) -> Result<Self> {
        input.seek(SeekFrom::Start(0))?;
        let header = FileHeader::read_le(&mut input)?;
        let file_size = input.seek(SeekFrom::End(0))?;
        let page_size = header.prefix.page_size;
        if file_size < u64::from(page_size) || file_size % u64::from(page_size) != 0 {
            return Err(Error::InvalidFileLayout {
                file_size,
                page_size,
            });
        }
        let mut tables = HashSet::new();
        for table in &header.tables {
            if !tables.insert(table.table_type) {
                return Err(Error::DuplicateTable(table.table_type));
            }
        }
        Ok(Self {
            input: PageStore::new(input, file_size, page_size),
            header,
            kind,
            page_count: file_size / u64::from(page_size),
            owners: HashMap::new(),
            indexes: HashMap::new(),
            source_file: None,
        })
    }

    /// The explicitly selected table namespace.
    pub fn kind(&self) -> DatabaseKind {
        self.kind
    }

    /// Current committed header, including table descriptors and unknown bytes.
    pub fn header(&self) -> &FileHeader {
        &self.header
    }

    /// Complete page fetches after opening; useful for observing lazy access.
    pub fn read_stats(&self) -> ReadStats {
        self.input.stats()
    }

    pub(crate) fn check_kind(&self, table: TableKind) -> Result<()> {
        if table.database_kind() != self.kind {
            return Err(Error::WrongDatabaseKind {
                database: self.kind,
                table,
            });
        }
        Ok(())
    }

    /// Iterate original live-row windows, including opaque layouts.
    /// A missing table is empty; a wrong namespace is an error. Construction
    /// performs no page reads. An I/O or layout error ends the iterator.
    pub fn iter_raw(&mut self, table: TableKind) -> Result<RawRows<'_, R>> {
        self.check_source()?;
        RawRows::new(
            self.input.view(None),
            &self.header,
            self.kind,
            self.page_count,
            &mut self.owners,
            table,
        )
    }

    // Only crate-owned model implementations supply ID schemas: one per table.
    // Build the index completely before publishing it; failed scans are retryable.
    pub(crate) fn index_rows(
        &mut self,
        table: TableKind,
        read_id: fn(&RowRecord) -> Result<u32>,
    ) -> Result<&HashMap<u32, RowLocation>> {
        self.check_source()?;
        self.check_kind(table)?;
        let code = table.code();
        if !self.indexes.contains_key(&code) {
            let mut index = HashMap::new();
            for record in self.iter_raw(table)? {
                let record = record?;
                let id = read_id(&record)?;
                let location = RowLocation {
                    page: record.page(),
                    slot: record.slot().slot,
                };
                if index.insert(id, location).is_some() {
                    return Err(Error::DuplicateId { table, id });
                }
            }
            self.indexes.insert(code, index);
        }
        Ok(self.indexes.entry(code).or_default())
    }

    pub(crate) fn read_indexed_row(
        &mut self,
        table: TableKind,
        location: RowLocation,
    ) -> Result<RowRecord> {
        self.check_source()?;
        let table_index =
            self.owners
                .get(&location.page)
                .copied()
                .ok_or(Error::InvalidRowDirectory {
                    page: location.page,
                    reason: "indexed page has no table owner",
                })?;
        let page_size = self.header.prefix.page_size;
        let page = read_chain_page(
            &mut self.input,
            page_size,
            location.page,
            table_index,
            table.code(),
        )?;
        let directory = read_row_directory(&mut self.input, page_size, &page)?;
        let record = read_row(&mut self.input, self.kind, &directory, location.slot)?.ok_or(
            Error::InvalidRowDirectory {
                page: location.page,
                reason: "indexed row is no longer live",
            },
        )?;
        Ok(record)
    }

    /// Begin an exclusive in-memory transaction without loading any data pages.
    /// Nested transactions and durable database transactions are not supported.
    ///
    /// ```no_run
    /// use rbox_pdb::binary::{DatabaseKind, PageType, TableKind};
    /// use rbox_pdb::devicesql::DeviceSqlConnection;
    /// # fn main() -> rbox_pdb::Result<()> {
    /// let mut connection = DeviceSqlConnection::open("export.pdb", DatabaseKind::Standard)?;
    /// let mut tx = connection.transaction();
    /// for record in tx.iter_raw(TableKind::Standard(PageType::Tracks))? {
    ///     println!("{:?}", record?.decode()?);
    /// }
    /// tx.rollback(); // Dropping tx also rolls back.
    /// # Ok(())
    /// # }
    /// ```
    pub fn transaction(&mut self) -> Transaction<'_, R> {
        Transaction::new(self)
    }

    /// Number of committed replacement pages retained above the original source.
    /// Saving a copy does not clear them, since the source remains unchanged.
    pub fn modified_page_count(&self) -> usize {
        self.input.modified_page_count()
    }

    /// Number of complete pages in the committed logical file, including page zero.
    pub fn page_count(&self) -> u64 {
        self.page_count
    }

    /// Current page-buffer limits.
    pub fn resource_limits(&self) -> ResourceLimits {
        self.input.limits()
    }

    /// Set page-buffer limits and immediately evict excess clean pages.
    /// Refuses a dirty budget smaller than the committed overlay, without changes.
    /// A transaction cannot coexist with this mutable borrow.
    pub fn set_resource_limits(&mut self, limits: ResourceLimits) -> Result<()> {
        self.input.set_limits(limits)
    }

    /// Inspect retained buffers and lookup-index size without performing I/O.
    pub fn resource_usage(&self) -> ResourceUsage {
        ResourceUsage {
            page_size: self.header.prefix.page_size,
            cached_pages: self.input.cached_pages(),
            dirty_pages: self.input.modified_page_count(),
            indexed_rows: self.indexes.values().map(|index| index.len()).sum(),
        }
    }

    /// Release clean page and derived lookup/ownership caches. Dirty pages remain.
    /// Future reads rebuild caches lazily; statistics are retained.
    pub fn clear_read_caches(&mut self) {
        self.input.clear_cache();
        self.indexes.clear();
        self.owners.clear();
    }

    pub(crate) fn check_source(&self) -> Result<()> {
        if let Some(source) = &self.source_file {
            source.check()?;
        }
        Ok(())
    }
    /// Stream the committed file image at the output's current position.
    ///
    /// Includes page zero, unlinked pages, slack, and opaque bytes without row
    /// decoding. Working memory is one page in addition to the existing cache
    /// and committed changes. Does not flush, truncate, or rewind the output.
    /// On error the output may contain a prefix; connection state is unchanged.
    /// The output must not alias the source. Prefer [`Self::save_as`] for files.
    pub fn write_to<W: Write>(&mut self, output: &mut W) -> Result<u64> {
        if let Some(source) = &self.source_file {
            source.check()?;
        }
        let written = self.input.write_to(output)?;
        if let Some(source) = &self.source_file {
            source.check()?;
        }
        Ok(written)
    }

    /// Save the committed image to a new path without overwriting any file.
    ///
    /// Streams to a temporary file beside the destination, flushes/syncs it,
    /// then publishes it with no-clobber semantics. A failed copy leaves no
    /// partial destination. An existing path (including the source or an alias)
    /// is an error. This is not an in-place save or a multi-file transaction.
    ///
    /// The connection continues to use its original source and committed
    /// overlay; saving neither rebases the connection nor clears that overlay.
    /// Source data must remain unchanged while the connection is open. This
    /// does not promise crash durability of the directory entry on all systems.
    ///
    /// ```no_run
    /// use rbox_pdb::binary::{DatabaseKind};
    /// use rbox_pdb::devicesql::DeviceSqlConnection;
    /// # fn main() -> rbox_pdb::Result<()> {
    /// let mut connection = DeviceSqlConnection::open("export.pdb", DatabaseKind::Standard)?;
    /// connection.save_as("export-copy.pdb")?; // Destination must not exist.
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_as(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        match self.save_new_file(path) {
            Err(Error::Io(source)) => Err(Error::Save {
                path: path.to_path_buf(),
                source,
            }),
            result => result,
        }
    }

    fn save_new_file(&mut self, path: &Path) -> Result<()> {
        if let Some(source) = &self.source_file {
            source.check()?;
        }
        match fs::symlink_metadata(path) {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "destination already exists",
                )
                .into())
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temporary = NamedTempFile::new_in(parent)?;
        self.input.write_to(temporary.as_file_mut())?;
        temporary.as_file_mut().flush()?;
        temporary.as_file().sync_all()?;
        if let Some(source) = &self.source_file {
            source.check()?;
        }
        temporary
            .persist_noclobber(path)
            .map_err(|error| error.error)?;
        Ok(())
    }

    /// Repack supported live rows into a detached database with fresh index pages.
    ///
    /// This explicit, non-lossless operation discards tombstones, unlinked pages,
    /// row-window tails, old index arrays and historical page metadata. Live binary
    /// fields/raw string frames and their order survive; offsets and slot headers
    /// are recalculated. Nonzero unmodeled inter-string gaps, opaque live rows,
    /// unsupported string allocations and unfamiliar container profiles are errors.
    /// It never silently omits a live row and never changes the source or writes
    /// any file. The returned connection can be inspected and saved separately.
    ///
    /// Uses the existing 4096-byte, standard-20/extended-9-table creation profile.
    /// Header `unknown_10` is preserved and sequences start at the source's committed
    /// sequence. Other nonzero opaque header state is rejected. Fresh empty index
    /// arrays match the creation profile; this is not general native index repair.
    /// Result buffers obey the source's resource limits; metadata/temporary work
    /// remains outside that budget. Device qualification remains separate.
    pub fn rebuild(&mut self) -> Result<(DeviceSqlConnection<Cursor<Vec<u8>>>, RebuildStats)> {
        self.check_source()?;
        let count = match self.kind {
            DatabaseKind::Standard => 20,
            DatabaseKind::Extended => 9,
        };
        if self.header.prefix.page_size as usize != PAGE_SIZE
            || self.header.prefix.signature != 0
            || self.header.prefix.unknown_18 != 0
            || self.header.trailing.iter().any(|&b| b != 0)
            || self.header.tables.len() != count
            || self
                .header
                .tables
                .iter()
                .enumerate()
                .any(|(i, t)| t.table_type != i as u32)
        {
            return Err(Error::InvalidCreation {
                reason: "rebuild requires the canonical directory and no opaque file-header state",
            });
        }
        let mut empty = DeviceSqlConnection::create(self.kind)?;
        let mut bytes = Vec::new();
        empty.write_to(&mut bytes)?;
        let mut header = empty.header.clone();
        header.prefix.unknown_10 = self.header.prefix.unknown_10;
        header.prefix.sequence = self.header.prefix.sequence.max(2);
        header.write_le(&mut Cursor::new(&mut bytes[..]))?;
        let mut rebuilt = DeviceSqlConnection::from_reader(Cursor::new(bytes), self.kind)?;
        rebuilt.set_resource_limits(self.resource_limits())?;
        let mut tx = rebuilt.transaction();
        let mut live_rows = 0;
        for code in 0..count {
            let table = self.kind.table_kind(code as u32);
            for record in self.iter_raw(table)? {
                let record = record?;
                let row = record.decode()?;
                row_encoding::check_relocation(table, &row, record.as_bytes())?;
                tx.insert(table, &row)?;
                live_rows += 1;
            }
        }
        tx.commit()?;
        if let Some(source) = &self.source_file {
            source.check()?;
        }
        let stats = RebuildStats {
            live_rows,
            original_pages: self.page_count,
            rebuilt_pages: rebuilt.page_count,
        };
        Ok((rebuilt, stats))
    }
}

impl DeviceSqlConnection<Cursor<Vec<u8>>> {
    /// Create a database from explicit binary rows and initial header metadata.
    ///
    /// This is an initialization operation, not a replay of historical updates.
    /// The existing transaction writer handles row placement, padding, footers and
    /// page growth; initial page sequences are then assigned from `options` before
    /// publishing the connection. Invalid namespaces, duplicate tables, unsupported
    /// rows, zero data-page sequences and uncommitted page sequences are rejected.
    /// No file is read or written. Both standard and extended namespaces are supported.
    ///
    /// `examples/reproduce_empty.rs` supplies the observed configuration of the
    /// native empty standard fixture and verifies whole-file equality.
    /// [`CreateOptions::standard`] and [`CreateOptions::extended`] supply opt-in
    /// configuration defaults without reading any source database.
    pub fn create_with_options(kind: DatabaseKind, options: &CreateOptions) -> Result<Self> {
        let invalid = |reason| Error::InvalidCreation { reason };
        if options.sequence <= 1 {
            return Err(invalid(
                "file sequence must be greater than index sequence 1",
            ));
        }
        let mut db = Self::create(kind)?;
        let mut seen = HashSet::new();
        for seed in &options.tables {
            db.check_kind(seed.table)?;
            if !db
                .header
                .tables
                .iter()
                .any(|t| t.table_type == seed.table.code())
            {
                return Err(invalid("initial table is outside the creation directory"));
            }
            if !seen.insert(seed.table.code()) {
                return Err(invalid("duplicate initial table"));
            }
            if seed.first_sequence == 0 || seed.first_sequence >= options.sequence {
                return Err(invalid(
                    "initial data-page sequence is outside the committed range",
                ));
            }
        }
        let mut tx = db.transaction();
        for seed in &options.tables {
            for row in &seed.rows {
                tx.insert(seed.table, row)?;
            }
        }
        let mut changes = PageChanges::new();
        for seed in &options.tables {
            tx.ensure_chains(seed.table)?;
            // Collect page numbers before reading through the mutable page cache.
            let pages = tx
                .chains
                .as_ref()
                .into_iter()
                .flatten()
                .filter(|c| c.table.table_type == seed.table.code())
                .flat_map(|c| c.pages.iter().map(|p| p.common.page_index))
                .collect::<Vec<_>>();
            for (index, page) in pages.into_iter().enumerate() {
                let sequence = if index == 0 {
                    1
                } else {
                    seed.first_sequence
                        .checked_add(u32::try_from(index - 1).map_err(|_| Error::MutationOverflow)?)
                        .ok_or(Error::MutationOverflow)?
                };
                if sequence >= options.sequence {
                    return Err(invalid(
                        "initial data-page sequence is outside the committed range",
                    ));
                }
                let mut bytes = tx.page_bytes(page)?;
                let mut header = PageHeader::read_le(&mut Cursor::new(&bytes))?;
                header.common.sequence = sequence;
                header.write_le(&mut Cursor::new(&mut bytes[..]))?;
                changes.insert(u64::from(page.0), Arc::from(bytes));
            }
        }
        let mut header = tx.header.clone();
        header.prefix.unknown_10 = options.unknown_10;
        header.prefix.sequence = options.sequence;
        let mut bytes = Cursor::new(Vec::new());
        header.write_le(&mut bytes)?;
        changes.insert(0, Arc::from(bytes.into_inner()));
        tx.stage_pages(changes)?;
        tx.commit()?;
        Ok(db)
    }

    /// Create an empty standard or extended database in memory.
    ///
    /// Uses the 4096-byte rbxport creation profile: standard tables 0..=19 or
    /// extended tables 0..=8, each with an index page and reserved candidate.
    /// All tables are empty, including configuration and property tables. No IDs,
    /// timestamps, master-library identities, or application defaults are generated.
    /// This creates a binary container, not a complete player-ready USB export.
    ///
    /// Insert binary rows through [`Self::transaction`], commit in memory, then
    /// persist with [`Self::save_as`] or [`Self::write_to`]. No path is created or
    /// opened by this constructor. Saved files remain editable through the same
    /// transaction API. Physical-device compatibility is not yet qualified.
    ///
    /// ```no_run
    /// use rbox_pdb::binary::{DatabaseKind, DeviceSqlString, NamedRow, PageType, Row, StandardRow, StringEncoding, TableKind};
    /// use rbox_pdb::devicesql::DeviceSqlConnection;
    /// # fn main() -> rbox_pdb::Result<()> {
    /// let mut db = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    /// let mut tx = db.transaction();
    /// tx.insert(TableKind::Standard(PageType::Genres),
    ///     &Row::Standard(StandardRow::Genre(NamedRow {
    ///         id: 1,
    ///         name: DeviceSqlString::encode("House", StringEncoding::ShortAscii)?,
    ///     })))?;
    /// tx.commit()?;
    /// db.save_as("new-export.pdb")?; // Refuses an existing destination.
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(kind: DatabaseKind) -> Result<Self> {
        let table_count = match kind {
            DatabaseKind::Standard => 20,
            DatabaseKind::Extended => 9,
        };
        let tables = (0..table_count)
            .map(|table_type| {
                let first_page = PageIndex(2 * table_type + 1);
                TablePointer {
                    table_type,
                    first_page,
                    last_page: first_page,
                    empty_candidate: PageIndex(first_page.0 + 1),
                }
            })
            .collect::<Vec<_>>();
        let header = FileHeader {
            prefix: FileHeaderPrefix {
                signature: 0,
                page_size: PAGE_SIZE as u32,
                table_count,
                next_unused_page: PageIndex(2 * table_count + 1),
                // rbxport writes 1; native files also contain 5. Neither is a
                // reliable version discriminator. Do not alter parser semantics.
                unknown_10: 1,
                // All initial index pages have sequence 1. The committed file
                // must name a later sequence, so recovery does not roll them back.
                sequence: 2,
                unknown_18: 0,
            },
            trailing: vec![
                0;
                PAGE_SIZE
                    - FileHeaderPrefix::BINARY_SIZE as usize
                    - TablePointer::BINARY_SIZE as usize * table_count as usize
            ],
            tables,
        };
        // Page zero, alternating index/candidate pages, ending at the last index.
        // Internal zero candidate pages exist; the final reservation is not stored.
        let mut output = Cursor::new(vec![0; 2 * table_count as usize * PAGE_SIZE]);
        header.write_le(&mut output)?;
        for table in &header.tables {
            output.set_position(
                table
                    .first_page
                    .file_offset(PAGE_SIZE as u32)
                    .ok_or(Error::MutationOverflow)?,
            );
            empty_index(table)?.write_le(&mut output)?;
            for _ in 0..INDEX_CAPACITY {
                IndexEntry::EMPTY.write_le(&mut output)?;
            }
            // The remaining 20 bytes are zero, as in the reference index page.
        }
        Self::from_reader(Cursor::new(output.into_inner()), kind)
    }
}

/// Physical database writes performed by a successful selective page save.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PageSaveStats {
    /// Complete changed/new pages written, including page zero.
    pub pages_written: u64,
    /// Bytes written to the database; unchanged pages are never rewritten.
    pub bytes_written: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Extended,
    Page(u64),
    DatabaseSynced,
}

/// Logical rows retained and physical size before/after an explicit rebuild.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RebuildStats {
    /// Number of live rows copied, in original table and live-slot order.
    pub live_rows: u64,
    /// Physical page count of the source, including unlinked pages.
    pub original_pages: u64,
    /// Physical page count of the freshly packed result.
    pub rebuilt_pages: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    use crate::binary::{
        DatabaseKind, ExtPageType, ExtendedRow, PageType, PlaylistEntryRow, Row, StandardRow,
        TableKind, TrackMyTagRow,
    };
    use rbox_testing::data::create_temp_dir;

    #[test]
    fn creation_roundtrips_empty_and_populated_profiles_in_both_namespaces() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let populated = match kind {
                DatabaseKind::Standard => CreateOptions::standard("2026-10-24", "Library 日本語")?,
                DatabaseKind::Extended => CreateOptions::extended(42)?,
            };
            for options in [CreateOptions::default(), populated] {
                let mut engine = DeviceSqlConnection::create_with_options(kind, &options)?;
                let mut bytes = Vec::new();
                engine.write_to(&mut bytes)?;
                let mut reopened =
                    DeviceSqlConnection::from_reader(Cursor::new(bytes.clone()), kind)?;
                assert_eq!(reopened.header(), engine.header());
                assert_eq!(reopened.header().prefix.unknown_10, options.unknown_10);
                assert_eq!(reopened.header().prefix.sequence, options.sequence);
                for pointer in reopened.header().tables.clone() {
                    let table = kind.table_kind(pointer.table_type);
                    let seed = options.tables.iter().find(|seed| seed.table == table);
                    let rows = reopened
                        .iter_raw(table)?
                        .map(|record| record?.decode())
                        .collect::<Result<Vec<_>>>()?;
                    assert_eq!(
                        rows.len(),
                        seed.map_or(0, |seed| seed.rows.len()),
                        "{table:?}"
                    );
                    for row in rows {
                        match row {
                            Row::Standard(StandardRow::ExportProperty(property)) => {
                                assert_eq!(property.date.text()?, "2026-10-24");
                                assert_eq!(
                                    property.name.text()?,
                                    "Library \u{65e5}\u{672c}\u{8a9e}"
                                );
                                assert_eq!(property.track_count, 0);
                            }
                            Row::Extended(ExtendedRow::DatabaseProperty(property)) => {
                                assert_eq!(property.header.my_tag_master_db_id, 42);
                                assert_eq!(property.header.unknown_words, [0; 5]);
                                for string in property.unknown_strings {
                                    assert_eq!(string.as_bytes(), [3]);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                let mut saved = Vec::new();
                reopened.write_to(&mut saved)?;
                assert_eq!(saved, bytes, "{kind:?}");
            }
        }
        Ok(())
    }

    fn table(kind: DatabaseKind) -> TableKind {
        match kind {
            DatabaseKind::Standard => TableKind::Standard(PageType::PlaylistEntries),
            DatabaseKind::Extended => TableKind::Extended(ExtPageType::TrackMyTags),
        }
    }

    fn row(kind: DatabaseKind, id: u32) -> Row {
        match kind {
            DatabaseKind::Standard => Row::Standard(StandardRow::PlaylistEntry(PlaylistEntryRow {
                position: id,
                track_id: id,
                playlist_id: 1,
            })),
            DatabaseKind::Extended => Row::Extended(ExtendedRow::TrackMyTag(TrackMyTagRow {
                unknown_00: 0,
                track_id: id,
                tag_id: 1,
                reserved_string_marker: 3,
                reserved_tail: [0; 3],
            })),
        }
    }

    fn pending(path: &Path, kind: DatabaseKind) -> Result<DeviceSqlConnection> {
        DeviceSqlConnection::create(kind)?.save_as(path)?;
        let mut db = DeviceSqlConnection::open(path, kind)?;
        // A no-op creates no extra files.
        assert_eq!(db.save_pages()?, PageSaveStats::default());
        assert_eq!(fs::read_dir(path.parent().unwrap())?.count(), 1);
        let mut tx = db.transaction();
        for id in 0..700 {
            tx.insert(table(kind), &row(kind, id))?;
        }
        tx.commit()?;
        Ok(db)
    }

    #[test]
    fn saves_only_dirty_pages_header_last_and_releases_buffers() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let temp = create_temp_dir();
            let path = temp.path().join("pages.pdb");
            let mut db = pending(&path, kind)?;
            for pass in 0..3 {
                if pass != 0 {
                    let mut tx = db.transaction();
                    let first = tx.iter_raw(table(kind))?.next().unwrap()?;
                    let handle = tx.locate(table(kind), first.page(), first.slot().slot)?;
                    if pass == 1 {
                        tx.update(&handle, &row(kind, 900))?;
                    } else {
                        tx.delete(&handle)?;
                    }
                    tx.commit()?;
                }
                let before = fs::read(&path)?;
                let dirty = db.input.changes().keys().copied().collect::<Vec<_>>();
                let mut expected = Vec::new();
                db.write_to(&mut expected)?;
                let reads = db.read_stats().page_reads;
                let mut writes = Vec::new();
                let stats = db.save_pages_with(&mut |step| {
                    if let Step::Page(page) = step {
                        writes.push(page);
                    }
                    Ok(())
                })?;
                assert_eq!(writes.last(), Some(&0));
                writes.sort_unstable();
                assert_eq!(writes, dirty);
                assert_eq!(stats.pages_written, dirty.len() as u64);
                assert!(stats.pages_written < db.page_count());
                assert_eq!(stats.bytes_written, stats.pages_written * 4096);
                assert_eq!(db.read_stats().page_reads, reads);
                assert_eq!(fs::read(&path)?, expected);
                for (page, bytes) in before.chunks_exact(4096).enumerate() {
                    if !dirty.contains(&(page as u64)) {
                        assert_eq!(bytes, &expected[page * 4096..(page + 1) * 4096]);
                    }
                }
                assert_eq!(db.resource_usage().dirty_pages, 0);
                assert_eq!(db.resource_usage().cached_pages, 0);
                assert_eq!(db.save_pages()?, PageSaveStats::default());
                // The driver creates no sidecar files.
                assert_eq!(fs::read_dir(temp.path())?.count(), 1);
                let mut reopened = DeviceSqlConnection::open(&path, kind)?;
                assert_eq!(
                    reopened
                        .iter_raw(table(kind))?
                        .collect::<Result<Vec<_>>>()?
                        .len(),
                    if pass == 2 { 699 } else { 700 }
                );
            }
        }
        Ok(())
    }

    #[test]
    fn failed_writes_invalidate_connection_and_application_restores_backup() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            // Inject failure after extension, a data page, the header, and sync.
            for point in 0..4 {
                let temp = create_temp_dir();
                let path = temp.path().join("failed.pdb");
                let mut db = pending(&path, kind)?;
                let backup = fs::read(&path)?;
                let dirty = db.modified_page_count();
                let result = db.save_pages_with(&mut |step| {
                    let stop = match step {
                        Step::Extended => point == 0,
                        Step::Page(page) => (point == 1 && page != 0) || (point == 2 && page == 0),
                        Step::DatabaseSynced => point == 3,
                    };
                    if stop {
                        Err(io::Error::other("injected write failure"))
                    } else {
                        Ok(())
                    }
                });
                assert!(matches!(result, Err(Error::PageSave { .. })));
                assert_ne!(fs::read(&path)?, backup);
                assert_eq!(db.modified_page_count(), dirty);
                assert!(matches!(
                    db.iter_raw(table(kind)),
                    Err(Error::ConnectionInvalidated(_))
                ));
                assert!(matches!(
                    db.save_pages(),
                    Err(Error::ConnectionInvalidated(_))
                ));
                assert!(matches!(db.save(), Err(Error::ConnectionInvalidated(_))));
                assert!(matches!(db.compact(), Err(Error::ConnectionInvalidated(_))));
                let mut tx = db.transaction();
                assert!(matches!(
                    tx.insert(table(kind), &row(kind, 900)),
                    Err(Error::ConnectionInvalidated(_))
                ));
                tx.rollback();
                drop(db);
                // Recovery belongs to the application; the driver creates no backup.
                fs::write(&path, backup)?;
                let mut reopened = DeviceSqlConnection::open(&path, kind)?;
                assert!(reopened.iter_raw(table(kind))?.next().is_none());
                let mut tx = reopened.transaction();
                tx.insert(table(kind), &row(kind, 901))?;
                tx.commit()?;
                reopened.save_pages()?;
                assert_eq!(fs::read_dir(temp.path())?.count(), 1);
            }
        }
        Ok(())
    }
}
