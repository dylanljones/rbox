// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Transaction lifecycle, binary row edits, relocation, and candidate-page allocation.
//!
//! Operations stage complete page images atomically; commit publishes them in memory.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::sync::Arc;

use binrw::{BinRead, BinWrite};

use super::connection::DeviceSqlConnection;
use super::core::{PageChanges, RawRows, ReadStats, RowLocation};
use super::creation::DATA_PAGE_FLAGS;
use super::encoding as row_encoding;
use crate::binary::chain::read_chain_page;
use crate::binary::{
    read_index_entries, read_row, read_row_directory, read_table_chains, CommonPageHeader,
    DataPageHeader, DatabaseKind, FileHeader, PackedRowCounts, PageHeader, PageHeaderKind,
    PageIndex, Row, RowGroup, RowRecord, TableChain, TableKind, PAGE_HEADER_SIZE,
};
use crate::{Error, Result};

/// Exclusive transaction over a connection's committed image.
///
/// Opening performs no page I/O; clean pages are shared with the connection.
/// Staged pages are owned here, and are discarded by rollback or drop. Commit
/// publishes changes in memory; persist with `save` (file-backed connections)
/// or `save_as` (a separate new file).
/// Binary mutations use existing space or allocate validated candidate pages.
/// They do not perform ID assignment, uniqueness checks, or relationship edits.
#[must_use = "a transaction is rolled back unless committed"]
pub struct Transaction<'a, R> {
    pub(super) connection: &'a mut DeviceSqlConnection<R>,
    pub(super) header: FileHeader,
    pub(super) changes: PageChanges,
    pub(super) page_count: u64,
    owners: HashMap<PageIndex, usize>,
    indexes: HashMap<u32, HashMap<u32, RowLocation>>,
    pub(super) identity: Arc<()>,
    pub(super) revisions: HashMap<(PageIndex, u16), u64>,
    pub(super) touched: BTreeMap<PageIndex, BTreeSet<u16>>,
    pub(super) chains: Option<Vec<TableChain>>,
}

impl<'a, R: Read + Seek> Transaction<'a, R> {
    pub(super) fn new(connection: &'a mut DeviceSqlConnection<R>) -> Self {
        Self {
            header: connection.header.clone(),
            page_count: connection.page_count,
            connection,
            changes: PageChanges::new(),
            owners: HashMap::new(),
            indexes: HashMap::new(),
            identity: Arc::new(()),
            revisions: HashMap::new(),
            touched: BTreeMap::new(),
            chains: None,
        }
    }
}

impl<R: Read + Seek> Transaction<'_, R> {
    /// Explicit standard/extended namespace of the borrowed connection.
    pub fn kind(&self) -> DatabaseKind {
        self.connection.kind
    }

    /// Header in this transaction's view.
    pub fn header(&self) -> &FileHeader {
        &self.header
    }

    /// Shared clean-page cache statistics; these are not rolled back.
    pub fn read_stats(&self) -> ReadStats {
        self.connection.read_stats()
    }

    /// Number of complete pages staged by this transaction.
    pub fn modified_page_count(&self) -> usize {
        self.changes.len()
    }

    /// Number of complete pages visible here, including staged file growth.
    pub fn page_count(&self) -> u64 {
        self.page_count
    }

    /// Read live binary row windows through staged, committed, then source pages.
    pub fn iter_raw(&mut self, table: TableKind) -> Result<RawRows<'_, R>> {
        self.connection.check_source()?;
        RawRows::new(
            self.connection
                .input
                .view_at(Some(&self.changes), self.page_count),
            &self.header,
            self.connection.kind,
            self.page_count,
            &mut self.owners,
            table,
        )
    }
    #[allow(dead_code, reason = "Retained for the deferred model layer")]
    pub(crate) fn check_source(&self) -> Result<()> {
        self.connection.check_source()
    }
    #[allow(dead_code, reason = "Retained for the deferred model layer")]
    pub(crate) fn check_kind(&self, table: TableKind) -> Result<()> {
        self.connection.check_kind(table)
    }

    // Only the model layer supplies scalar ID readers. Indexes belong to this
    // overlay, never the committed connection, and publish only after a full scan.
    #[allow(dead_code, reason = "Retained for the deferred model layer")]
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
    #[allow(dead_code, reason = "Retained for the deferred model layer")]
    pub(crate) fn read_indexed_row(
        &mut self,
        table: TableKind,
        location: RowLocation,
    ) -> Result<RowRecord> {
        self.check_source()?;
        let owner = self
            .owners
            .get(&location.page)
            .copied()
            .ok_or(Error::InvalidRowDirectory {
                page: location.page,
                reason: "indexed page has no table owner",
            })?;
        let size = self.header.prefix.page_size;
        let kind = self.kind();
        let mut input = self
            .connection
            .input
            .view_at(Some(&self.changes), self.page_count);
        let page = read_chain_page(&mut input, size, location.page, owner, table.code())?;
        let directory = read_row_directory(&mut input, size, &page)?;
        read_row(&mut input, kind, &directory, location.slot)?.ok_or(Error::InvalidRowDirectory {
            page: location.page,
            reason: "indexed row is no longer live",
        })
    }

    // Only the binary mutation planner may call this. It must provide complete,
    // consistent page images including mutation metadata.
    // No arbitrary page replacement API is exposed publicly.
    pub(super) fn stage_pages(&mut self, changes: PageChanges) -> Result<()> {
        self.stage_growth(changes, self.page_count)
    }

    pub(super) fn stage_growth(&mut self, changes: PageChanges, page_count: u64) -> Result<()> {
        if page_count < self.page_count {
            return Err(Error::InvalidPageWrite {
                page: page_count,
                reason: "transactional shrinking is unsupported",
            });
        }
        self.connection
            .input
            .validate_changes(&changes, &self.changes, page_count)?;
        let header = if let Some(bytes) = changes.get(&0) {
            let header = FileHeader::read_le(&mut Cursor::new(bytes.as_ref()))?;
            if header.prefix.page_size != self.header.prefix.page_size
                || header.tables.len() != self.header.tables.len()
                || header
                    .tables
                    .iter()
                    .zip(&self.header.tables)
                    .any(|(a, b)| a.table_type != b.table_type || a.first_page != b.first_page)
            {
                return Err(Error::InvalidPageWrite {
                    page: 0,
                    reason: "page size, table namespace and first pages must remain unchanged",
                });
            }
            Some(header)
        } else {
            None
        };
        let structural = header
            .as_ref()
            .is_some_and(|h| h.tables != self.header.tables)
            || page_count != self.page_count;
        if structural {
            let mut proposed = self.changes.clone();
            proposed.extend(changes.clone());
            read_table_chains(
                &mut self.connection.input.view_at(Some(&proposed), page_count),
                header.as_ref().unwrap_or(&self.header),
            )?;
        }
        // All fallible preparation is above this point.
        self.changes.extend(changes);
        self.page_count = page_count;
        if structural {
            self.chains = None;
        }
        if let Some(header) = header {
            self.header = header;
        }
        self.owners.clear();
        self.indexes.clear();
        Ok(())
    }

    // Prepare multi-page operations in a private fork. Page images are Arc-backed:
    // only map entries/metadata are cloned, never the file or unchanged payloads.
    // Nothing is published until every fallible suboperation has succeeded.
    pub(super) fn atomic<T>(
        &mut self,
        operation: impl FnOnce(&mut Transaction<'_, R>) -> Result<T>,
    ) -> Result<T> {
        let mut plan = Transaction {
            connection: &mut *self.connection,
            header: self.header.clone(),
            changes: self.changes.clone(),
            page_count: self.page_count,
            owners: HashMap::new(),
            indexes: HashMap::new(),
            identity: self.identity.clone(),
            revisions: self.revisions.clone(),
            touched: self.touched.clone(),
            chains: self.chains.clone(),
        };
        let result = operation(&mut plan)?;
        self.header = plan.header;
        self.changes = plan.changes;
        self.page_count = plan.page_count;
        self.revisions = plan.revisions;
        self.touched = plan.touched;
        self.chains = plan.chains;
        self.owners.clear();
        self.indexes.clear();
        Ok(result)
    }

    /// Publish staged pages and header together in memory. No disk writes occur.
    /// Derived lookup/ownership caches are invalidated when pages changed.
    /// An empty commit preserves the connection's caches and original bytes.
    pub fn commit(self) -> Result<()> {
        self.connection.check_source()?;
        if !self.changes.is_empty() {
            self.connection.input.install(self.changes);
            self.connection.input.grow(self.page_count);
            self.connection.page_count = self.page_count;
            self.connection.header = self.header;
            self.connection.indexes.clear();
            self.connection.owners.clear();
        }
        Ok(())
    }

    /// Discard this transaction's staged pages. Dropping it has the same effect.
    /// Previously committed changes remain intact.
    pub fn rollback(self) {}

    // Validate all linked page ownership once before exposing mutable targets.
    // This is deliberately stronger than lazy, selected-table read validation.
    pub(super) fn ensure_chains(&mut self, table: TableKind) -> Result<()> {
        self.connection.check_source()?;
        self.connection.check_kind(table)?;
        if self.chains.is_none() {
            let chains = read_table_chains(
                &mut self
                    .connection
                    .input
                    .view_at(Some(&self.changes), self.page_count),
                &self.header,
            )?;
            self.chains = Some(chains);
        }
        Ok(())
    }

    fn belongs(&mut self, table: TableKind, page: PageIndex) -> Result<bool> {
        self.ensure_chains(table)?;
        Ok(self.chains.as_ref().into_iter().flatten().any(|chain| {
            chain.table.table_type == table.code()
                && chain
                    .pages
                    .iter()
                    .any(|p| p.common.page_index == page && !p.kind.is_index())
        }))
    }

    pub(super) fn page_bytes(&mut self, page: PageIndex) -> Result<Vec<u8>> {
        let mut bytes = vec![0; self.header.prefix.page_size as usize];
        let mut input = self
            .connection
            .input
            .view_at(Some(&self.changes), self.page_count);
        input.seek(SeekFrom::Start(
            u64::from(page.0) * u64::from(self.header.prefix.page_size),
        ))?;
        input.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    fn current_row(&mut self, table: TableKind, page: PageIndex, slot: u16) -> Result<RowRecord> {
        let missing = || Error::RowNotFound { table, page, slot };
        if !self.belongs(table, page)? {
            return Err(missing());
        }
        let bytes = self.page_bytes(page)?;
        let header = PageHeader::read_le(&mut Cursor::new(&bytes))?;
        let kind = self.kind();
        let mut input = self
            .connection
            .input
            .view_at(Some(&self.changes), self.page_count);
        let directory = read_row_directory(&mut input, self.header.prefix.page_size, &header)?;
        read_row(&mut input, kind, &directory, slot)?.ok_or_else(missing)
    }

    /// Resolve a physical location into a checked transaction-scoped row handle.
    /// First use validates all table chains; only the target row payload is read.
    pub fn locate(&mut self, table: TableKind, page: PageIndex, slot: u16) -> Result<RowHandle> {
        self.current_row(table, page, slot)?;
        Ok(self.handle(table, page, slot))
    }

    fn handle(&self, table: TableKind, page: PageIndex, slot: u16) -> RowHandle {
        RowHandle {
            identity: self.identity.clone(),
            revision: *self.revisions.get(&(page, slot)).unwrap_or(&0),
            table,
            page,
            slot,
        }
    }

    fn check_handle(&self, handle: &RowHandle) -> Result<()> {
        if !Arc::ptr_eq(&self.identity, &handle.identity)
            || handle.revision
                != *self
                    .revisions
                    .get(&(handle.page, handle.slot))
                    .unwrap_or(&0)
        {
            return Err(Error::InvalidRowHandle);
        }
        Ok(())
    }

    /// Read the current binary row window behind a valid handle.
    pub fn read(&mut self, handle: &RowHandle) -> Result<RowRecord> {
        self.check_handle(handle)?;
        self.current_row(handle.table, handle.page, handle.slot)
    }

    /// Update binary fields, returning a refreshed (possibly relocated) handle.
    /// Fixed-prefix edits, fitting inline edits, and offset-string edits fitting
    /// their original encoded frames preserve remaining row bytes and offsets.
    /// This includes readable far tags and nonempty reserved property strings.
    /// A shortened frame's known retired suffix is cleared; other gaps/tails stay.
    /// If individual frames do not fit, strings may be repacked inside the old
    /// known content extent (never window-tail slack). This can widen My Tag
    /// offsets to the readable far subtype without allocating more space.
    /// Growing inline rows and supported string-layout changes append a replacement
    /// and tombstone the old slot atomically. Offset arrays and slot fields are
    /// recalculated on relocation; raw frames and modeled unknown fields survive.
    /// Unsupported layouts or nonzero unmodeled gaps return an error.
    /// No-op updates preserve handles and metadata. ID uniqueness is not enforced.
    ///
    /// ```no_run
    /// use rbox_pdb::binary::{DatabaseKind, PageType, TableKind};
    /// use rbox_pdb::devicesql::DeviceSqlConnection;
    /// use rbox_pdb::binary::{Row, StandardRow};
    /// # fn main() -> rbox_pdb::Result<()> {
    /// let mut connection = DeviceSqlConnection::open("export.pdb", DatabaseKind::Standard)?;
    /// let mut tx = connection.transaction();
    /// let table = TableKind::Standard(PageType::Tracks);
    /// let first = tx.iter_raw(table)?.next().transpose()?;
    /// if let Some(record) = first {
    ///     let handle = tx.locate(table, record.page(), record.slot().slot)?;
    ///     let mut binary = tx.read(&handle)?.decode()?;
    ///     if let Row::Standard(StandardRow::Track(track)) = &mut binary {
    ///         track.header.rating = 4;
    ///         let refreshed = tx.update(&handle, &binary)?;
    ///         let updated = tx.read(&refreshed)?;
    ///     }
    /// }
    /// tx.commit()?;
    /// connection.save_as("edited-export.pdb")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update(&mut self, handle: &RowHandle, row: &Row) -> Result<RowHandle> {
        let original = self.read(handle)?;
        let decoded = original.decode()?;
        if decoded == *row {
            return Ok(handle.clone());
        }
        let encoding = row_encoding::update(handle.table, &decoded, row, original.as_bytes())
            .or_else(|error| match error {
                Error::UnsupportedMutation { .. } => {
                    row_encoding::repack_existing(handle.table, &decoded, row, original.as_bytes())
                }
                error => Err(error),
            });
        let (old, new) = match encoding {
            Ok(pair) => pair,
            Err(Error::UnsupportedMutation { .. }) => {
                row_encoding::layout(handle.table, row)?;
                row_encoding::check_relocation(handle.table, &decoded, original.as_bytes())?;
                return self.atomic(|plan| {
                    let replacement = plan.insert(handle.table, row)?;
                    plan.delete(handle)?;
                    Ok(replacement)
                });
            }
            Err(error) => return Err(error),
        };
        if !original.as_bytes().starts_with(&old) {
            return Err(row_encoding::unsupported(
                handle.table,
                "original field encoding is not byte-preserving",
            ));
        }
        if new.len() > old.len() {
            return self.atomic(|plan| {
                let replacement = plan.insert(handle.table, row)?;
                plan.delete(handle)?;
                Ok(replacement)
            });
        }
        if old == new {
            return Ok(handle.clone());
        }
        let next_revision = handle
            .revision
            .checked_add(1)
            .ok_or(Error::MutationOverflow)?;
        let mut bytes = self.page_bytes(handle.page)?;
        let start = PAGE_HEADER_SIZE + usize::from(original.slot().heap_offset);
        bytes[start..start + new.len()].copy_from_slice(&new);
        self.finish_page(handle.page, handle.slot, bytes, false)?;
        self.revisions
            .insert((handle.page, handle.slot), next_revision);
        Ok(self.handle(handle.table, handle.page, handle.slot))
    }

    /// Tombstone a live row without reclaiming its payload or directory slot.
    /// Also works for opaque row payloads. Does not cascade to other tables.
    pub fn delete(&mut self, handle: &RowHandle) -> Result<()> {
        self.read(handle)?;
        let next_revision = handle
            .revision
            .checked_add(1)
            .ok_or(Error::MutationOverflow)?;
        let mut bytes = self.page_bytes(handle.page)?;
        let mut header = PageHeader::read_le(&mut Cursor::new(&bytes))?;
        let mut groups = self.groups(&header)?;
        groups[usize::from(handle.slot / RowGroup::MAX_SLOTS)].presence_flags &=
            !(1u16 << (handle.slot % RowGroup::MAX_SLOTS));
        header.common.counts = PackedRowCounts::new(
            header.common.counts.allocated(),
            header.common.counts.live() - 1,
        )?;
        header.common.flags = header.common.flags.with_transaction_flag(true);
        header.write_le(&mut Cursor::new(&mut bytes[..]))?;
        RowGroup::write_footer(&mut bytes, &groups)?;
        self.finish_page(handle.page, handle.slot, bytes, false)?;
        self.revisions
            .insert((handle.page, handle.slot), next_revision);
        Ok(())
    }

    /// Append a binary row to a linked data page, allocating a candidate if needed.
    /// Known inline layouts, tracks, artists/albums, near My Tags, export properties,
    /// and the empty-string extended property profile are supported. Offsets and
    /// page-local slot fields are recalculated; IDs/unknown defaults are not generated.
    /// Unsupported profiles return `UnsupportedMutation`; rows too large for a page
    /// return `NoRowSpace`. Deleted slots are never reused.
    pub fn insert(&mut self, table: TableKind, row: &Row) -> Result<RowHandle> {
        if let Some(handle) = self.try_insert(table, row)? {
            return Ok(handle);
        }
        self.atomic(|plan| {
            plan.allocate_data_page(table)?;
            plan.try_insert(table, row)?.ok_or(Error::NoRowSpace {
                table,
                bytes: row_encoding::layout(table, row)?.allocation,
            })
        })
    }

    fn try_insert(&mut self, table: TableKind, row: &Row) -> Result<Option<RowHandle>> {
        self.connection.check_kind(table)?;
        let layout = row_encoding::layout(table, row)?;
        let allocation = layout.allocation;
        if PAGE_HEADER_SIZE + allocation + RowGroup::footer_size(1)
            > self.header.prefix.page_size as usize
        {
            return Err(Error::NoRowSpace {
                table,
                bytes: allocation,
            });
        }
        self.ensure_chains(table)?;
        let pages: Vec<_> = self
            .chains
            .as_ref()
            .into_iter()
            .flatten()
            .filter(|c| c.table.table_type == table.code())
            .flat_map(|c| c.pages.iter())
            .filter(|p| !p.kind.is_index())
            .map(|p| p.common.page_index)
            .collect();
        for page in pages.into_iter().rev() {
            let mut bytes = self.page_bytes(page)?;
            let mut header = PageHeader::read_le(&mut Cursor::new(&bytes))?;
            let n = header.common.counts.allocated();
            let live = header.common.counts.live();
            if n == PackedRowCounts::MAX_ALLOCATED
                || live == PackedRowCounts::MAX_LIVE
                || !layout.fits_slot(n)
            {
                continue;
            }
            let used = usize::from(header.common.used_size);
            let footer = RowGroup::footer_size(n + 1);
            if used % 4 != 0
                || PAGE_HEADER_SIZE + used + allocation + footer > bytes.len()
                || used + allocation > u16::MAX as usize
            {
                continue;
            }
            let mut groups = self.groups(&header)?;
            if n % RowGroup::MAX_SLOTS == 0 {
                groups.push(RowGroup {
                    offsets: Vec::new(),
                    presence_flags: 0,
                    transaction_flags: 0,
                });
            }
            let group = &mut groups[usize::from(n / RowGroup::MAX_SLOTS)];
            group.offsets.insert(0, used as u16);
            group.presence_flags |= 1 << (n % RowGroup::MAX_SLOTS);
            let start = PAGE_HEADER_SIZE + used;
            let encoded = layout.at_slot(n)?;
            bytes[start..start + encoded.len()].copy_from_slice(&encoded);
            bytes[start + encoded.len()..start + allocation].fill(0);
            header.common.counts = PackedRowCounts::new(n + 1, live + 1)?;
            header.common.used_size = (used + allocation) as u16;
            header.common.free_size =
                (bytes.len() - PAGE_HEADER_SIZE - used - allocation - footer) as u16;
            header.write_le(&mut Cursor::new(&mut bytes[..]))?;
            RowGroup::write_footer(&mut bytes, &groups)?;
            self.finish_page(page, n, bytes, true)?;
            return Ok(Some(self.handle(table, page, n)));
        }
        Ok(None)
    }

    fn groups(&mut self, header: &PageHeader) -> Result<Vec<RowGroup>> {
        let mut input = self
            .connection
            .input
            .view_at(Some(&self.changes), self.page_count);
        Ok(
            read_row_directory(&mut input, self.header.prefix.page_size, header)?
                .groups()
                .to_vec(),
        )
    }

    // Prepare all bytes and metadata locally before publishing a single operation.
    fn finish_page(
        &mut self,
        page: PageIndex,
        slot: u16,
        mut bytes: Vec<u8>,
        inserted: bool,
    ) -> Result<()> {
        let mut header = PageHeader::read_le(&mut Cursor::new(&bytes))?;
        let mut file = self.header.clone();
        let first_touch = !self.touched.contains_key(&page);
        if first_touch {
            header.common.sequence = file.prefix.sequence;
            file.prefix.sequence = file
                .prefix
                .sequence
                .checked_add(1)
                .ok_or(Error::MutationOverflow)?;
        }
        let mut touched = self.touched.get(&page).cloned().unwrap_or_default();
        touched.insert(slot);
        let PageHeaderKind::Data(ref mut data) = header.kind else {
            return Err(Error::InvalidPageWrite {
                page: u64::from(page.0),
                reason: "mutations require a data page",
            });
        };
        data.transaction_row_count =
            u16::try_from(touched.len()).map_err(|_| Error::MutationOverflow)?;
        data.transaction_row_index = touched.first().copied().ok_or(Error::MutationOverflow)?;
        // Read the modified footer from a bounded local page. Avoid allocating
        // an entire file merely to give the absolute-offset reader a page.
        let n = header.common.counts.allocated();
        let mut groups = RowGroup::read_footer(&bytes, n)?;
        for (index, group) in groups.iter_mut().enumerate() {
            let allocated_mask = if group.offsets.len() == usize::from(RowGroup::MAX_SLOTS) {
                u16::MAX
            } else {
                (1u16 << group.offsets.len()) - 1
            };
            if first_touch {
                group.transaction_flags &= !allocated_mask;
            }
            if inserted && index == usize::from(slot / RowGroup::MAX_SLOTS) {
                group.transaction_flags &= !(1 << (slot % RowGroup::MAX_SLOTS));
            }
            for &touched_slot in touched.range(
                (index as u16 * RowGroup::MAX_SLOTS)
                    ..(index as u16 * RowGroup::MAX_SLOTS + RowGroup::MAX_SLOTS),
            ) {
                group.transaction_flags |= 1 << (touched_slot % RowGroup::MAX_SLOTS);
            }
        }
        header.write_le(&mut Cursor::new(&mut bytes[..]))?;
        RowGroup::write_footer(&mut bytes, &groups)?;
        let mut page_zero = Cursor::new(Vec::new());
        file.write_le(&mut page_zero)?;
        self.stage_pages(PageChanges::from([
            (0, Arc::from(page_zero.into_inner())),
            (u64::from(page.0), Arc::from(bytes)),
        ]))?;
        self.touched.insert(page, touched);
        Ok(())
    }

    // Called only inside an atomic mutation plan, before appending its first row.
    pub(super) fn allocate_data_page(&mut self, table: TableKind) -> Result<()> {
        self.ensure_chains(table)?;
        let unsupported = || {
            row_encoding::unsupported(
                table,
                "unsupported or occupied allocation candidate/index profile",
            )
        };
        let chain = self
            .chains
            .as_ref()
            .and_then(|chains| chains.iter().find(|c| c.table.table_type == table.code()))
            .cloned()
            .ok_or_else(unsupported)?;
        let first = chain.pages.first().ok_or_else(unsupported)?;
        let last = chain.pages.last().ok_or_else(unsupported)?;
        let PageHeaderKind::Index(index) = first.kind else {
            return Err(unsupported());
        };
        if chain.pages.iter().skip(1).any(|p| p.kind.is_index())
            || index.page_index != first.common.page_index
            || (chain.pages.len() > 1 && index.data_page != chain.pages[1].common.page_index)
            || (chain.pages.len() == 1
                && index.data_page != PageIndex::NONE
                && index.data_page != chain.table.empty_candidate)
        {
            return Err(unsupported());
        }
        // Do not invent initialization values for an unrecognized page profile.
        for page in &chain.pages {
            if page.common.signature != 0 || page.common.unknown_14 != 0 {
                return Err(unsupported());
            }
            if let PageHeaderKind::Data(data) = page.kind {
                if data.unknown_24 != 0
                    || data.unknown_26 != 0
                    || page.common.flags.with_transaction_flag(false) != DATA_PAGE_FLAGS
                {
                    return Err(unsupported());
                }
            }
        }
        let candidate = chain.table.empty_candidate;
        let next = self.header.prefix.next_unused_page;
        if candidate.0 == 0
            || candidate.0 >= PageIndex::NONE.0
            || next.0 == 0
            || next.0 >= PageIndex::NONE.0 - 1
            || next.0 <= candidate.0
            || last.common.next_page != candidate
            || u64::from(next.0) < self.page_count
            || u64::from(candidate.0) > self.page_count + self.header.tables.len() as u64
            || u64::from(next.0) > self.page_count + self.header.tables.len() as u64 + 1
        {
            return Err(unsupported());
        }
        for other in self.chains.as_ref().into_iter().flatten() {
            if other
                .pages
                .iter()
                .any(|p| p.common.page_index == candidate || p.common.page_index == next)
                || (other.table.table_type != table.code()
                    && other.table.empty_candidate == candidate)
                || other.table.empty_candidate == next
                || (other.table.table_type != table.code()
                    && other.pages.last().is_some_and(|p| p.common.next_page == candidate || p.common.next_page == next))
                || (other.table.table_type != table.code()
                    && other.pages.iter().any(|p| matches!(p.kind, PageHeaderKind::Index(i) if i.data_page == candidate || i.data_page == next)))
            {
                return Err(unsupported());
            }
            // A zero-filled candidate may still be referenced by an index array.
            // Flags do not establish whether a reference is safe to reclaim, so
            // conservatively reserve every declared nonempty reference. Reading
            // through the bounded codec also rejects malformed entry counts.
            for page in other.pages.iter().filter(|p| p.kind.is_index()) {
                let entries = read_index_entries(
                    &mut self
                        .connection
                        .input
                        .view_at(Some(&self.changes), self.page_count),
                    self.header.prefix.page_size,
                    page,
                )?;
                if entries.iter().any(|entry| {
                    !entry.is_empty()
                        && (entry.page_index() == candidate || entry.page_index() == next)
                }) {
                    return Err(unsupported());
                }
            }
        }
        if u64::from(candidate.0) < self.page_count
            && self.page_bytes(candidate)?.iter().any(|&b| b != 0)
        {
            return Err(unsupported());
        }

        let size = self.header.prefix.page_size as usize;
        let mut file = self.header.clone();
        let mut changes = PageChanges::new();
        // Link-only edits also take a sequence, without claiming a row was touched.
        let mut previous = self.page_bytes(last.common.page_index)?;
        let mut previous_header = PageHeader::read_le(&mut Cursor::new(&previous))?;
        previous_header.common.next_page = candidate;
        if !self.touched.contains_key(&last.common.page_index) {
            previous_header.common.sequence = file.prefix.sequence;
            file.prefix.sequence = file
                .prefix
                .sequence
                .checked_add(1)
                .ok_or(Error::MutationOverflow)?;
        }
        if let PageHeaderKind::Index(ref mut index) = previous_header.kind {
            index.data_page = candidate;
        }
        previous_header.write_le(&mut Cursor::new(&mut previous[..]))?;
        changes.insert(u64::from(last.common.page_index.0), Arc::from(previous));
        let page = PageHeader {
            common: CommonPageHeader {
                signature: 0,
                page_index: candidate,
                table_type: table.code(),
                next_page: next,
                sequence: 0,
                unknown_14: 0,
                counts: PackedRowCounts::new(0, 0)?,
                flags: DATA_PAGE_FLAGS,
                free_size: u16::try_from(size - PAGE_HEADER_SIZE)
                    .map_err(|_| Error::MutationOverflow)?,
                used_size: 0,
            },
            kind: PageHeaderKind::Data(DataPageHeader {
                transaction_row_count: DataPageHeader::NO_TRANSACTION,
                transaction_row_index: DataPageHeader::NO_TRANSACTION,
                unknown_24: 0,
                unknown_26: 0,
            }),
        };
        let count = self.page_count.max(u64::from(candidate.0) + 1);
        for p in self.page_count..count {
            changes.insert(p, Arc::from(vec![0; size]));
        }
        let mut bytes = vec![0; size];
        page.write_le(&mut Cursor::new(&mut bytes[..]))?;
        changes.insert(u64::from(candidate.0), Arc::from(bytes));
        let pointer = file
            .tables
            .iter_mut()
            .find(|p| p.table_type == table.code())
            .ok_or_else(unsupported)?;
        pointer.last_page = candidate;
        pointer.empty_candidate = next;
        file.prefix.next_unused_page = PageIndex(next.0 + 1);
        let mut zero = Cursor::new(Vec::new());
        file.write_le(&mut zero)?;
        changes.insert(0, Arc::from(zero.into_inner()));
        self.stage_growth(changes, count)?;
        // Distinguish sequence assignment from row-mask clearing on a later edit.
        // A later row edit may take another sequence; both remain below file.sequence.
        Ok(())
    }
}

/// A live row identity scoped to one transaction. Cloneable, but invalid after
/// that row is updated/deleted or the transaction ends. Obtain another handle
/// through `locate` after an update, or use the handle returned by `update`.
#[derive(Debug, Clone)]
pub struct RowHandle {
    identity: Arc<()>,
    revision: u64,
    table: TableKind,
    page: PageIndex,
    slot: u16,
}

impl RowHandle {
    /// Explicit standard/extended table namespace.
    pub fn table(&self) -> TableKind {
        self.table
    }
    /// Physical page in the current transaction.
    pub fn page(&self) -> PageIndex {
        self.page
    }
    /// Page-local directory slot.
    pub fn slot(&self) -> u16 {
        self.slot
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use binrw::BinWrite;
    use rbox_testing::data::{create_temp_dir, export_dir};

    use super::*;
    use crate::binary::{ExtPageType, MyTagHeader, PageType, RowRecord, TrackHeader};

    fn id(record: &RowRecord) -> Result<u32> {
        let mut input = Cursor::new(record.as_bytes());
        Ok(match record.table().database_kind() {
            DatabaseKind::Standard => TrackHeader::read_le(&mut input)?.id,
            DatabaseKind::Extended => MyTagHeader::read_le(&mut input)?.id,
        })
    }

    // These controlled binary edits prove overlay mechanics only. In
    // particular they do NOT establish the metadata rules for PDB mutations.
    #[test]
    fn binary_overlay_reads_rollback_commit_and_save_work_for_both_kinds() -> Result<()> {
        let temporary = create_temp_dir();
        for (kind, name, table) in [
            (
                DatabaseKind::Standard,
                "export.pdb",
                TableKind::Standard(PageType::Tracks),
            ),
            (
                DatabaseKind::Extended,
                "exportExt.pdb",
                TableKind::Extended(ExtPageType::MyTags),
            ),
        ] {
            let source = temporary.path().join(name);
            fs::copy(export_dir(7).join("rekordbox").join(name), &source)?;
            let original = fs::read(&source)?;
            let mut connection = DeviceSqlConnection::open(&source, kind)?;
            let record = connection.iter_raw(table)?.next().unwrap()?;
            let original_id = id(&record)?;
            let index = connection.index_rows(table, id)?;
            assert!(index.contains_key(&original_id));
            assert!(!index.contains_key(&u32::MAX));
            let size = connection.header.prefix.page_size as usize;
            let page_start = record.page().0 as usize * size;
            let mut page = original[page_start..page_start + size].to_vec();
            let row_start = 40 + usize::from(record.slot().heap_offset);
            let mut input = Cursor::new(record.as_bytes());
            let mut output = Cursor::new(&mut page[row_start..]);
            match kind {
                DatabaseKind::Standard => {
                    let mut header = TrackHeader::read_le(&mut input)?;
                    header.id = u32::MAX;
                    header.write_le(&mut output)?;
                }
                DatabaseKind::Extended => {
                    let mut header = MyTagHeader::read_le(&mut input)?;
                    header.id = u32::MAX;
                    header.write_le(&mut output)?;
                }
            }
            let mut header = connection.header.clone();
            header.prefix.sequence += 1;
            let mut page_zero = Cursor::new(Vec::new());
            header.write_le(&mut page_zero)?;
            let changes = PageChanges::from([
                (0, Arc::from(page_zero.into_inner())),
                (u64::from(record.page().0), Arc::from(page.clone())),
            ]);

            // Drop and explicit rollback both discard pages and header changes.
            for explicit in [false, true] {
                let mut tx = connection.transaction();
                tx.stage_pages(changes.clone())?;
                assert_eq!(tx.modified_page_count(), 2);
                assert_eq!(tx.header(), &header);
                let staged = tx.iter_raw(table)?.next().unwrap()?;
                assert_eq!(id(&staged)?, u32::MAX);
                staged.decode()?;
                if explicit {
                    tx.rollback();
                }
            }
            assert_eq!(connection.iter_raw(table)?.next().unwrap()?, record);
            assert!(connection.index_rows(table, id)?.contains_key(&original_id));
            assert_eq!(connection.modified_page_count(), 0);
            let mut unchanged = Vec::new();
            connection.write_to(&mut unchanged)?;
            assert_eq!(unchanged, original);

            // Forgetting a guard leaks its allocations, not its uncommitted data.
            let mut tx = connection.transaction();
            tx.stage_pages(changes.clone())?;
            std::mem::forget(tx);
            assert_eq!(connection.iter_raw(table)?.next().unwrap()?, record);

            let mut tx = connection.transaction();
            tx.stage_pages(changes)?;
            tx.commit()?;
            assert_eq!(connection.header(), &header);
            assert_eq!(connection.modified_page_count(), 2);
            let rebuilt = connection.index_rows(table, id)?;
            assert!(!rebuilt.contains_key(&original_id));
            assert!(rebuilt.contains_key(&u32::MAX));

            // A later rollback must retain earlier commits, not revert to disk.
            let mut tx = connection.transaction();
            tx.stage_pages(PageChanges::from([(
                u64::from(record.page().0),
                Arc::from(original[page_start..page_start + size].to_vec()),
            )]))?;
            assert_eq!(id(&tx.iter_raw(table)?.next().unwrap()?)?, original_id);
            tx.rollback();
            assert_eq!(id(&connection.iter_raw(table)?.next().unwrap()?)?, u32::MAX);

            let mut expected = original.clone();
            expected[page_start..page_start + size].copy_from_slice(&page);
            header.write_le(&mut Cursor::new(&mut expected[..size]))?;
            let destination = temporary.path().join(format!("staged-{name}"));
            connection.save_as(&destination)?;
            assert_eq!(fs::read(&destination)?, expected);
            assert_eq!(fs::read(&source)?, original);
            let mut reopened = DeviceSqlConnection::open(&destination, kind)?;
            assert_eq!(reopened.header(), &header);
            assert_eq!(id(&reopened.iter_raw(table)?.next().unwrap()?)?, u32::MAX);
            let mut repeat = Vec::new();
            connection.write_to(&mut repeat)?;
            assert_eq!(repeat, expected); // Save did not discard the overlay.
        }
        Ok(())
    }

    #[test]
    fn bad_page_batch_keeps_previous_staged_state_and_header() -> Result<()> {
        let mut bytes = vec![0; 3 * 512];
        bytes[4..8].copy_from_slice(&512u32.to_le_bytes());
        let mut connection =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        let mut tx = connection.transaction();
        tx.stage_pages(PageChanges::from([(1, Arc::from(vec![7; 512]))]))?;
        for bad in [
            PageChanges::from([(1, Arc::from(vec![8; 512])), (2, Arc::from(vec![0; 511]))]),
            PageChanges::from([(1, Arc::from(vec![8; 512])), (3, Arc::from(vec![0; 512]))]),
            PageChanges::from([(0, Arc::from(vec![0; 512])), (1, Arc::from(vec![8; 512]))]),
        ] {
            assert!(tx.stage_pages(bad).is_err());
            assert_eq!(tx.modified_page_count(), 1);
            assert_eq!(tx.changes.get(&1).unwrap().as_ref(), &[7; 512]);
            assert_eq!(tx.header().prefix.page_size, 512);
        }
        tx.commit()?;
        let mut saved = Vec::new();
        connection.write_to(&mut saved)?;
        assert_eq!(&saved[512..1024], &[7; 512]);
        assert_eq!(&saved[1024..], &[0; 512]);
        Ok(())
    }
}
