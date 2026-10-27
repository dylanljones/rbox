// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared page storage, raw-row traversal, resource types, and file publication helpers.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fs::{File, Metadata};
use std::io::{Read, Seek, SeekFrom, Write};
use std::iter::FusedIterator;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use std::{fs, io};
use tempfile::{NamedTempFile, TempPath};

use crate::binary::{
    read_chain_page, read_row, read_row_directory, validate_reference, DatabaseKind, FileHeader,
    PageIndex, RowDirectory, RowRecord, TableKind, TablePointer,
};
use crate::{Error, Result};

pub(super) type PageChanges = BTreeMap<u64, Arc<[u8]>>;

/// Limits on retained page buffers. These are not a total process-memory limit.
/// ID indexes, metadata and temporary mutation plans require additional memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimits {
    /// Maximum number of clean pages cached; zero disables retention.
    pub cached_pages: usize,
    /// Maximum unique modified pages across committed and transaction overlays.
    /// `None` is unlimited. Save back to the source to release committed buffers.
    /// Transient old/new versions during atomic planning are not counted twice.
    pub dirty_pages: Option<usize>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cached_pages: 16,
            dirty_pages: None,
        }
    }
}

/// Retained page buffers and lookup entries, excluding temporary mutation plans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceUsage {
    /// Page size in bytes.
    pub page_size: u32,
    /// Retained clean pages.
    pub cached_pages: usize,
    /// Committed modified pages that still need persistence.
    pub dirty_pages: usize,
    /// Logical ID-to-location entries retained by the connection.
    pub indexed_rows: usize,
}

/// Physical page reads performed after opening the database.
/// Header-page reads during opening are excluded; decoding is not counted.
/// Streaming writes do not count as cache reads.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReadStats {
    /// Complete pages fetched from the underlying source after cache misses.
    pub page_reads: u64,
}

pub(crate) struct PageCache<R> {
    source: R,
    file_size: u64,
    page_size: u32,
    position: u64,
    pages: VecDeque<(u64, Vec<u8>)>,
    stats: ReadStats,
    capacity: usize,
}

impl<R> PageCache<R> {
    pub(crate) fn new(source: R, file_size: u64, page_size: u32) -> Self {
        Self {
            source,
            file_size,
            page_size,
            position: 0,
            pages: VecDeque::new(),
            stats: ReadStats::default(),
            capacity: 16,
        }
    }

    pub(crate) fn stats(&self) -> ReadStats {
        self.stats
    }
}

impl<R: Read + Seek> Read for PageCache<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() || self.position >= self.file_size {
            return Ok(0);
        }
        let page_size = u64::from(self.page_size);
        let index = self.position / page_size;
        let entry = if let Some(position) = self.pages.iter().position(|(page, _)| *page == index) {
            self.pages
                .remove(position)
                .ok_or_else(|| io::Error::other("missing cached page"))?
        } else {
            let mut bytes = vec![0; self.page_size as usize];
            self.source.seek(SeekFrom::Start(index * page_size))?;
            self.source.read_exact(&mut bytes)?;
            self.stats.page_reads += 1;
            (index, bytes)
        };
        let offset = (self.position % page_size) as usize;
        let count = output.len().min(entry.1.len() - offset);

        output[..count].copy_from_slice(&entry.1[offset..offset + count]);
        self.position += count as u64;
        self.pages.push_front(entry);
        self.pages.truncate(self.capacity);

        Ok(count)
    }
}

impl<R> Seek for PageCache<R> {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let position = match from {
            SeekFrom::Start(position) => Some(position),
            SeekFrom::Current(delta) => self.position.checked_add_signed(delta),
            SeekFrom::End(delta) => self.file_size.checked_add_signed(delta),
        }
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek outside address space"))?;
        self.position = position;
        Ok(position)
    }
}

/// Transaction pages are owned by the transaction, never installed in the
/// connection until commit. Even forgetting the transaction cannot leak them.
pub(super) struct PageView<'a, R> {
    store: &'a mut PageStore<R>,
    changes: Option<&'a PageChanges>,
    file_size: u64,
}

impl<R: Read + Seek> Read for PageView<'_, R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let cache = &mut self.store.cache;
        if output.is_empty() || cache.position >= self.file_size {
            return Ok(0);
        }
        let page = cache.position / u64::from(cache.page_size);
        let bytes = self
            .changes
            .and_then(|pages| pages.get(&page))
            .or_else(|| self.store.committed.get(&page));
        if let Some(bytes) = bytes {
            let offset = (cache.position % u64::from(cache.page_size)) as usize;
            let count = output.len().min(bytes.len() - offset);
            output[..count].copy_from_slice(&bytes[offset..offset + count]);
            cache.position += count as u64;
            Ok(count)
        } else {
            if cache.position >= cache.file_size {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "missing new page image",
                ));
            }
            cache.read(output)
        }
    }
}

impl<R> Seek for PageView<'_, R> {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let from = match from {
            SeekFrom::End(delta) => {
                SeekFrom::Start(self.file_size.checked_add_signed(delta).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "seek outside address space")
                })?)
            }
            other => other,
        };
        self.store.cache.seek(from)
    }
}

/// Clean pages are evictable; committed replacements remain authoritative.
pub(super) struct PageStore<R> {
    cache: PageCache<R>,
    committed: PageChanges,
    page_count: u64,
    limits: ResourceLimits,
}

impl<R> PageStore<R> {
    pub(super) fn new(source: R, file_size: u64, page_size: u32) -> Self {
        Self {
            cache: PageCache::new(source, file_size, page_size),
            committed: BTreeMap::new(),
            page_count: file_size / u64::from(page_size),
            limits: ResourceLimits::default(),
        }
    }

    pub(super) fn stats(&self) -> ReadStats {
        self.cache.stats()
    }

    pub(super) fn modified_page_count(&self) -> usize {
        self.committed.len()
    }

    pub(super) fn changes(&self) -> &PageChanges {
        &self.committed
    }

    pub(super) fn limits(&self) -> ResourceLimits {
        self.limits
    }

    pub(super) fn set_limits(&mut self, limits: ResourceLimits) -> Result<()> {
        if let Some(limit) = limits.dirty_pages {
            if self.committed.len() > limit {
                return Err(Error::DirtyPageLimit {
                    limit,
                    required: self.committed.len(),
                });
            }
        }
        self.limits = limits;
        self.cache.capacity = limits.cached_pages;
        self.cache.pages.truncate(limits.cached_pages);
        Ok(())
    }

    pub(super) fn cached_pages(&self) -> usize {
        self.cache.pages.len()
    }

    pub(super) fn clear_cache(&mut self) {
        self.cache.pages.clear();
    }

    pub(super) fn rebase(&mut self, source: R, file_size: u64) {
        let stats = self.cache.stats;
        self.cache = PageCache::new(source, file_size, self.cache.page_size);
        self.cache.capacity = self.limits.cached_pages;
        self.cache.stats = stats;
        self.committed.clear();
        self.page_count = file_size / u64::from(self.cache.page_size);
    }

    pub(super) fn view<'a>(&'a mut self, changes: Option<&'a PageChanges>) -> PageView<'a, R> {
        self.view_at(changes, self.page_count)
    }

    pub(super) fn view_at<'a>(
        &'a mut self,
        changes: Option<&'a PageChanges>,
        page_count: u64,
    ) -> PageView<'a, R> {
        let file_size = page_count * u64::from(self.cache.page_size);
        PageView {
            store: self,
            changes,
            file_size,
        }
    }

    // Validate the binary mutation layer's complete operation
    // before extending an overlay, so a bad later page cannot stage a prefix.
    pub(super) fn validate_changes(
        &self,
        changes: &PageChanges,
        staged: &PageChanges,
        page_count: u64,
    ) -> Result<()> {
        if let Some(limit) = self.limits.dirty_pages {
            let required = self.committed.len()
                + staged
                    .keys()
                    .filter(|p| !self.committed.contains_key(p))
                    .count()
                + changes
                    .keys()
                    .filter(|p| !self.committed.contains_key(p) && !staged.contains_key(p))
                    .count();
            if required > limit {
                return Err(Error::DirtyPageLimit { limit, required });
            }
        }
        if page_count < self.page_count
            || page_count
                .checked_mul(u64::from(self.cache.page_size))
                .is_none()
        {
            return Err(Error::InvalidPageWrite {
                page: page_count,
                reason: "invalid logical file length",
            });
        }
        for (&page, bytes) in changes {
            if page >= page_count {
                return Err(Error::InvalidPageWrite {
                    page,
                    reason: "page is outside the logical file",
                });
            }
            if bytes.len() != self.cache.page_size as usize {
                return Err(Error::InvalidPageWrite {
                    page,
                    reason: "replacement is not one complete page",
                });
            }
        }
        for page in self.page_count..page_count {
            if !changes.contains_key(&page) && !staged.contains_key(&page) {
                return Err(Error::InvalidPageWrite {
                    page,
                    reason: "new page image is missing",
                });
            }
        }
        Ok(())
    }

    pub(super) fn install(&mut self, changes: PageChanges) {
        self.committed.extend(changes);
    }

    pub(super) fn grow(&mut self, page_count: u64) {
        self.page_count = page_count;
    }
}

impl<R: Read + Seek> PageStore<R> {
    /// Copy every physical page, including unlinked/opaque ones. Saving does not
    /// populate/evict the read cache or change its virtual cursor/statistics.
    pub(super) fn write_to<W: Write>(&mut self, output: &mut W) -> io::Result<u64> {
        let size = self.cache.page_size as usize;
        let mut buffer = vec![0; size];
        for page in 0..self.page_count {
            if let Some(bytes) = self.committed.get(&page) {
                output.write_all(bytes)?;
            } else {
                self.cache
                    .source
                    .seek(SeekFrom::Start(page * size as u64))?;
                self.cache.source.read_exact(&mut buffer)?;
                output.write_all(&buffer)?;
            }
        }
        Ok(self.page_count * size as u64)
    }
}

impl<R: Read + Seek> Read for PageStore<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.view(None).read(output)
    }
}

impl<R> Seek for PageStore<R> {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        self.view(None).seek(from)
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code, reason = "Retained for the deferred model layer")]
pub(crate) struct RowLocation {
    pub(crate) page: PageIndex,
    pub(crate) slot: u16,
}

/// Fallible streaming access to original row windows. Fused after EOF or an error.
pub struct RawRows<'a, R> {
    input: PageView<'a, R>,
    kind: DatabaseKind,
    page_size: u32,
    page_count: u64,
    owners: &'a mut HashMap<PageIndex, usize>,
    descriptor: Option<(usize, TablePointer)>,
    next_page: Option<PageIndex>,
    visited: HashSet<PageIndex>,
    directory: Option<RowDirectory>,
    slots: std::vec::IntoIter<u16>,
    done: bool,
}

impl<'a, R: Read + Seek> RawRows<'a, R> {
    pub(super) fn new(
        input: PageView<'a, R>,
        header: &FileHeader,
        kind: DatabaseKind,
        page_count: u64,
        owners: &'a mut HashMap<PageIndex, usize>,
        table: TableKind,
    ) -> Result<Self> {
        if table.database_kind() != kind {
            return Err(Error::WrongDatabaseKind {
                database: kind,
                table,
            });
        }
        let descriptor = header
            .tables
            .iter()
            .copied()
            .enumerate()
            .find(|(_, pointer)| pointer.table_type == table.code());
        if let Some((index, pointer)) = descriptor {
            validate_reference(pointer.first_page, page_count, index)?;
            validate_reference(pointer.last_page, page_count, index)?;
        }
        Ok(Self {
            input,
            kind,
            page_size: header.prefix.page_size,
            page_count,
            owners,
            descriptor,
            next_page: descriptor.map(|(_, table)| table.first_page),
            visited: HashSet::new(),
            directory: None,
            slots: Vec::new().into_iter(),
            done: false,
        })
    }

    fn read_next(&mut self) -> Result<Option<RowRecord>> {
        let Some((table_index, table)) = self.descriptor else {
            return Ok(None);
        };
        loop {
            if let Some(slot) = self.slots.next() {
                let directory = self.directory.as_ref().ok_or(Error::InvalidRowDirectory {
                    page: table.first_page,
                    reason: "missing current row directory",
                })?;
                return read_row(&mut self.input, self.kind, directory, slot);
            }
            let Some(current) = self.next_page else {
                return Ok(None);
            };
            validate_reference(current, self.page_count, table_index)?;
            if !self.visited.insert(current) {
                return Err(Error::PageCycle {
                    table_index,
                    page: current,
                });
            }
            if let Some(&owner) = self.owners.get(&current) {
                if owner != table_index {
                    return Err(Error::SharedPage {
                        page: current,
                        first_table: owner,
                        second_table: table_index,
                    });
                }
            }
            let page_size = self.page_size;
            let page = read_chain_page(
                &mut self.input,
                page_size,
                current,
                table_index,
                table.table_type,
            )?;
            self.owners.insert(current, table_index);
            self.next_page = if current == table.last_page {
                None
            } else {
                Some(page.common.next_page)
            };
            if !page.kind.is_index() {
                let directory = read_row_directory(&mut self.input, page_size, &page)?;
                self.slots = directory
                    .slots()
                    .filter(|slot| slot.live)
                    .map(|slot| slot.slot)
                    .collect::<Vec<_>>()
                    .into_iter();
                self.directory = Some(directory);
            }
        }
    }
}

impl<R: Read + Seek> Iterator for RawRows<'_, R> {
    type Item = Result<RowRecord>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.read_next() {
            Ok(Some(record)) => Some(Ok(record)),
            Ok(None) => {
                self.done = true;
                None
            }
            Err(error) => {
                self.done = true;
                Some(Err(error))
            }
        }
    }
}

impl<R: Read + Seek> FusedIterator for RawRows<'_, R> {}

#[derive(Clone, PartialEq, Eq)]
struct SourceStamp {
    len: u64,
    modified: SystemTime,
    created: Option<SystemTime>,
}

impl SourceStamp {
    fn read(metadata: &Metadata) -> io::Result<Self> {
        Ok(Self {
            len: metadata.len(),
            modified: metadata.modified()?,
            created: metadata.created().ok(),
        })
    }
}

pub(super) struct SourceFile {
    pub(super) path: PathBuf,
    stamp: SourceStamp,
    usable: bool,
}

impl SourceFile {
    pub(super) fn len(&self) -> u64 {
        self.stamp.len
    }

    pub(super) fn new(path: PathBuf, file: &File) -> Result<Self> {
        Ok(Self {
            path,
            stamp: SourceStamp::read(&file.metadata()?)?,
            usable: true,
        })
    }

    pub(super) fn check(&self) -> Result<()> {
        if !self.usable {
            return Err(Error::ConnectionInvalidated(self.path.clone()));
        }
        let metadata = fs::metadata(&self.path).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                Error::SourceChanged(self.path.clone())
            } else {
                Error::Io(error)
            }
        })?;
        if SourceStamp::read(&metadata)? != self.stamp {
            return Err(Error::SourceChanged(self.path.clone()));
        }
        Ok(())
    }

    pub(super) fn invalidate(&mut self) {
        self.usable = false;
    }
}

/// Prepare a replacement completely before publication; adopt its open handle.
pub(super) fn replace_file(
    source: &SourceFile,
    write: impl FnOnce(&mut File) -> Result<()>,
) -> Result<(File, SourceFile)> {
    let path = &source.path;
    source.check()?;
    let parent = path.parent().ok_or(Error::NoSourcePath)?;
    let mut temporary = NamedTempFile::new_in(parent)?;
    write(temporary.as_file_mut())?;
    temporary.as_file_mut().flush()?;
    // std::fs::rename also handles open Windows destinations, unlike the
    // MoveFileExW-only tempfile persist. keep clears the temporary attribute.
    let (file, temporary_path) = temporary.keep().map_err(|e| e.error)?;
    let temporary_path = TempPath::try_from_path(temporary_path)?;
    file.set_permissions(fs::metadata(path)?.permissions())?;
    file.sync_all()?;
    source.check()?;
    fs::rename(&temporary_path, path).map_err(|source| Error::Save {
        path: path.clone(),
        source,
    })?;
    // Windows replacement can preserve the destination's creation timestamp.
    // Capture the published file's metadata, not the temporary file's stamp.
    let replacement = SourceFile::new(path.clone(), &file)?;
    Ok((file, replacement))
}

/// Physical output performed by a successful save back to the source.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SaveStats {
    /// Complete pages written to the replacement file, including unchanged pages.
    pub pages_written: u64,
    /// Total bytes written; zero for a no-op save.
    pub bytes_written: u64,
}

#[cfg(test)]
mod tests {

    use std::io::Cursor;

    use super::*;

    #[test]
    fn logical_growth_requires_complete_images_and_changes_end_relative_seeks() -> Result<()> {
        let mut store = PageStore::new(Cursor::new(vec![9; 512]), 512, 512);
        let staged = PageChanges::from([(1, Arc::from(vec![7; 512]))]);
        assert!(store
            .validate_changes(&staged, &PageChanges::new(), 3)
            .is_err());
        store.validate_changes(&staged, &PageChanges::new(), 2)?;
        let mut view = store.view_at(Some(&staged), 2);
        assert_eq!(view.seek(SeekFrom::End(-2))?, 1022);
        let mut bytes = [0; 3];
        assert_eq!(view.read(&mut bytes)?, 2);
        assert_eq!(&bytes[..2], &[7, 7]);
        assert_eq!(view.read(&mut bytes)?, 0);
        drop(view);
        assert_eq!(store.seek(SeekFrom::End(0))?, 512);
        store.install(staged);
        store.grow(2);
        assert_eq!(store.seek(SeekFrom::End(0))?, 1024);
        let mut saved = Vec::new();
        assert_eq!(store.write_to(&mut saved)?, 1024);
        assert_eq!(&saved[512..], &[7; 512]);
        let extra = PageChanges::from([(2, Arc::from(vec![8; 512]))]);
        store.validate_changes(&extra, &PageChanges::new(), 3)?;
        let mut view = store.view_at(Some(&extra), 3);
        view.seek(SeekFrom::Start(1022))?;
        let mut crossing = [0; 4];
        view.read_exact(&mut crossing)?;
        assert_eq!(crossing, [7, 7, 8, 8]);
        Ok(())
    }

    #[test]
    fn overlays_survive_clean_cache_eviction_and_reads_cross_layer_boundaries() -> io::Result<()> {
        let bytes: Vec<_> = (0..20).flat_map(|page| [page; 512]).collect();
        let mut store = PageStore::new(Cursor::new(bytes.clone()), bytes.len() as u64, 512);
        store.install(PageChanges::from([(1, Arc::from(vec![91; 512]))]));
        let staged =
            PageChanges::from([(1, Arc::from(vec![92; 512])), (2, Arc::from(vec![93; 512]))]);
        let mut view = store.view(Some(&staged));
        view.seek(SeekFrom::Start(510))?;
        let mut data = vec![0; 1028];
        view.read_exact(&mut data)?;
        assert_eq!(&data[..2], &[0, 0]);
        assert_eq!(&data[2..514], &[92; 512]);
        assert_eq!(&data[514..1026], &[93; 512]);
        assert_eq!(&data[1026..], &[3, 3]);
        drop(view);
        for page in 3..20 {
            store.seek(SeekFrom::Start(page * 512))?;
            store.read_exact(&mut [0])?;
        }
        store.seek(SeekFrom::Start(512))?;
        let before = store.stats();
        let mut byte = [0];
        store.read_exact(&mut byte)?;
        assert_eq!(byte, [91]);
        assert_eq!(store.stats(), before);
        let mut saved = Vec::new();
        store.write_to(&mut saved)?;
        assert_eq!(&saved[512..1024], &[91; 512]);
        assert_eq!(&saved[1024..], &bytes[1024..]);
        assert_eq!(store.stream_position()?, 513);
        assert_eq!(store.stats(), before);
        Ok(())
    }

    #[test]
    fn absolute_seeks_cross_page_reads_eof_and_eviction() -> io::Result<()> {
        let bytes: Vec<_> = (0..18).flat_map(|page| [page; 512]).collect();
        let mut cache = PageCache::new(Cursor::new(bytes), 18 * 512, 512);
        cache.seek(SeekFrom::Start(510))?;
        let mut boundary = [0; 4];
        cache.read_exact(&mut boundary)?;
        assert_eq!(boundary, [0, 0, 1, 1]);
        assert_eq!(cache.stats().page_reads, 2);
        cache.seek(SeekFrom::Current(-4))?;
        cache.read_exact(&mut boundary)?;
        assert_eq!(cache.stats().page_reads, 2);
        for page in 2..18 {
            cache.seek(SeekFrom::Start(page * 512))?;
            cache.read_exact(&mut boundary)?;
        }
        assert_eq!(cache.pages.len(), 16);
        cache.seek(SeekFrom::Start(0))?;
        cache.read_exact(&mut boundary)?;
        assert_eq!(cache.stats().page_reads, 19);
        assert_eq!(boundary, [0; 4]);
        assert_eq!(cache.seek(SeekFrom::End(0))?, 18 * 512);
        assert_eq!(cache.read(&mut boundary)?, 0);
        assert!(cache.seek(SeekFrom::Current(-100_000)).is_err());
        Ok(())
    }

    #[test]
    fn partial_source_read_is_an_error_and_does_not_cache_a_partial_page() -> io::Result<()> {
        let mut cache = PageCache::new(Cursor::new(vec![0; 511]), 512, 512);
        assert_eq!(
            cache.read(&mut [0]).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        assert_eq!(cache.stats().page_reads, 0);
        assert!(cache.pages.is_empty());
        assert_eq!(cache.stream_position()?, 0);
        Ok(())
    }
}
