// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Lazy access to ANLZ files, with ordered sections and selective serialization.
//!
//! [`Anlz`] owns the source file and caches decoded sections. The binary models
//! and their eager parsers are defined in [`super::sections`].

use binrw::io::{Read, Seek, Write};
use binrw::{BinRead, BinWrite};
use std::io::{Cursor, SeekFrom};
use std::ops::Range;

use crate::error::{AnlzError, InvalidFileReason, Result};
use crate::sections::*;

/// One section occurrence in the file's ordered sequence.
///
/// Duplicate tags have separate entries. New entries are decoded and marked
/// for serialization; unloaded entries always have a source byte range.
struct SectionEntry {
    /// Complete section range, including its header, in the current source.
    /// Absent for a newly inserted section until it is saved to that source.
    original_range: Option<Range<u64>>,
    /// Indexed tag, used until the section is decoded.
    tag: AnlzTag,
    /// Cached section, including any edits made through a mutable getter.
    decoded: Option<Section>,
    /// Serialize on save instead of copying source bytes. Set before exposing
    /// mutable access, since subsequent changes to the reference cannot be tracked.
    potentially_modified: bool,
}

impl SectionEntry {
    /// Prefer the live tag so low-level edits cannot leave lookups stale.
    fn tag(&self) -> &AnlzTag {
        self.decoded.as_ref().map_or(&self.tag, |s| &s.tag)
    }
}

/// Lazily read and edit a Rekordbox `.DAT`, `.EXT`, or `.2EX` analysis file.
///
/// Opening reads the file header and indexes the section boundaries. Payloads
/// are decoded and cached on first access; cached sections remain in memory
/// until this handler is consumed or dropped.
///
/// # Access and editing
///
/// Shared section getters (`get_*`) borrow cached data without marking it modified. They
/// still require `&mut self` because a first access may read and populate the cache.
/// The `get_mut_*` methods mark a returned section for serialization, even if the
/// caller ultimately makes no changes. Edits remain in memory until
/// [`dump`](Self::dump) or [`dump_copy`](Self::dump_copy) succeeds.
///
/// Optional getters return `Ok(None)` when no matching section is present.
/// Typed getters also return `None` if a low-level edit has made a section's tag
/// inconsistent with its content; saving rejects that inconsistency. I/O and
/// payload parsing failures are returned as errors when the section is accessed.
/// An error in an unread payload therefore need not prevent opening the file.
///
/// # Saving and source lifetime
///
/// Saves write a complete output file in the original section order. Sections
/// accessed mutably are serialized; untouched sections, including unknown tags,
/// are copied from the source byte-for-byte. Duplicate tags remain distinct.
///
/// Keep the backing file unchanged externally while this handler is alive:
/// cached values and indexed byte ranges are tied to that source. Dropping the
/// handler closes its file handle and discards unsaved edits.
///
/// # Examples
///
/// ```no_run
/// use rbox_anlz::Anlz;
///
/// # fn main() -> rbox_anlz::Result<()> {
/// let mut file = Anlz::load("ANLZ0000.DAT")?;
/// if let Some(grid) = file.get_beat_grid()? {
///     println!("{} beats", grid.beats.len());
/// }
/// if let Some(grid) = file.get_mut_beat_grid()? {
///     grid.set_grid_offset(25);
/// }
/// file.dump_copy("ANLZ_EDITED.DAT")?;
/// # Ok(())
/// # }
/// ```
pub struct Anlz {
    /// Canonical source path; distinct from the audio path stored in PPTH.
    path: std::path::PathBuf,
    /// Closed immediately before replacing the source. A failed replacement
    /// leaves this empty until the next operation that needs source bytes.
    source: Option<std::fs::File>,
    /// PMAI header from the most recent load or successful source save.
    header: Header,
    /// Opaque bytes following the common 12-byte PMAI header.
    header_data: Vec<u8>,
    /// Authoritative serialization order, including duplicate and unknown tags.
    sections: Vec<SectionEntry>,
}

impl Anlz {
    /// Open an analysis file and index its sections without decoding their payloads.
    ///
    /// The path is canonicalized and a file handle is retained for later reads.
    /// The file-header extension is loaded; section bodies are skipped using their
    /// declared lengths. Opening does not validate individual payloads.
    ///
    /// # AnlzErrors
    ///
    /// Returns an error if the file cannot be opened or read, the PMAI header is
    /// invalid, its declared total size differs from the physical file size, or a
    /// section has invalid or truncated outer bounds.
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let path = std::fs::canonicalize(path)?;
        let mut source = std::fs::File::open(&path)?;
        let (header, header_data, sections) = Self::index(&mut source)?;
        Ok(Self {
            path,
            source: Some(source),
            header,
            header_data,
            sections,
        })
    }

    /// Scan common headers, checking bounds before seeking over section bodies.
    /// The ordered entries are also the authoritative order used when saving.
    fn index<R: Read + Seek>(source: &mut R) -> Result<(Header, Vec<u8>, Vec<SectionEntry>)> {
        let physical_size = source.seek(SeekFrom::End(0))?;
        source.seek(SeekFrom::Start(0))?;
        let header = Self::read_header(source)?;
        if header.tag != AnlzTag::File
            || header.size < 12
            || header.total_size < header.size
            || u64::from(header.total_size) != physical_size
        {
            return Err(AnlzError::InvalidFile {
                reason: InvalidFileReason::HeaderOrLength,
                offset: 0,
            });
        }
        let mut header_data = vec![0; (header.size - 12) as usize];
        source.read_exact(&mut header_data)?;
        let mut offset = u64::from(header.size);
        let mut sections = Vec::new();
        while offset < physical_size {
            if physical_size - offset < 12 {
                return Err(AnlzError::InvalidFile {
                    reason: InvalidFileReason::TruncatedSectionHeader,
                    offset,
                });
            }
            source.seek(SeekFrom::Start(offset))?;
            let h = Self::read_header(source)?;
            let end =
                offset
                    .checked_add(u64::from(h.total_size))
                    .ok_or(AnlzError::InvalidFile {
                        reason: InvalidFileReason::SectionOffsetOverflow,
                        offset,
                    })?;
            if h.size < 12 || h.total_size < h.size || end > physical_size {
                return Err(AnlzError::InvalidFile {
                    reason: InvalidFileReason::InvalidSectionBounds,
                    offset,
                });
            }
            sections.push(SectionEntry {
                original_range: Some(offset..end),
                tag: h.tag,
                decoded: None,
                potentially_modified: false,
            });
            offset = end;
        }
        Ok((header, header_data, sections))
    }

    fn read_header<R: Read>(source: &mut R) -> Result<Header> {
        // AnlzTag's derived reader backtracks through enum alternatives. Keep
        // those small reads in memory rather than repeating filesystem calls.
        let mut bytes = [0; 12];
        source.read_exact(&mut bytes)?;
        Ok(Header::read_be(&mut Cursor::new(bytes))?)
    }

    /// Reopen the original source after a failed attempt to replace it.
    fn source(&mut self) -> Result<&mut std::fs::File> {
        if self.source.is_none() {
            self.source = Some(std::fs::File::open(&self.path)?);
        }
        self.source
            .as_mut()
            .ok_or(AnlzError::InvalidState("source file was not opened"))
    }

    /// Return the number of top-level sections, excluding the PMAI file header.
    ///
    /// Includes duplicate tags, unknown tags, and newly inserted sections.
    /// This uses the index and does not read or decode payloads.
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// Populate one valid index on success; leave failed decodes uncached.
    fn decode(&mut self, index: usize) -> Result<()> {
        if self.sections[index].decoded.is_none() {
            self.source()?;
        }
        let entry = &mut self.sections[index];
        if entry.decoded.is_none() {
            let range = entry
                .original_range
                .as_ref()
                .ok_or(AnlzError::InvalidState(
                    "an unloaded section has no source range",
                ))?;
            let offset = range.start;
            let tag = entry.tag.clone();
            let result = (|| -> Result<Section> {
                let source = self
                    .source
                    .as_mut()
                    .ok_or(AnlzError::InvalidState("source file was not opened"))?;
                source.seek(SeekFrom::Start(range.start))?;
                let mut bytes = vec![0; (range.end - range.start) as usize];
                source.read_exact(&mut bytes)?;
                // Isolate the parser from adjacent sections, including its seeks
                // and enum backtracking. Drop these raw bytes after decoding.
                let mut reader = Cursor::new(bytes);
                let section = Section::read_be(&mut reader)?;
                if reader.position() != range.end - range.start {
                    return Err(AnlzError::InvalidFile {
                        reason: InvalidFileReason::SectionSizeMismatch,
                        offset,
                    });
                }
                Ok(section)
            })();
            entry.decoded = Some(result.map_err(|source| AnlzError::SectionDecode {
                index,
                tag,
                offset,
                source: Box::new(source),
            })?);
        }
        Ok(())
    }

    /// Borrow the section at a zero-based position in the current file order.
    ///
    /// Decodes and caches that section on first access without marking it modified.
    /// Returns `Ok(None)` for an out-of-range index and an error if decoding fails.
    /// Indices refer to positions, not stable identifiers: inserting a missing
    /// PPTH through [`set_path`](Self::set_path) shifts existing positions by one.
    pub fn get_section(&mut self, index: usize) -> Result<Option<&Section>> {
        if index >= self.sections.len() {
            return Ok(None);
        }
        self.decode(index)?;
        Ok(self.sections[index].decoded.as_ref())
    }

    /// Mutably borrow the section at a zero-based position in the current file order.
    ///
    /// Uses the same loading and bounds behavior as [`get_section`](Self::get_section).
    /// A successful access marks the section for serialization even if no fields
    /// are changed. When editing the whole section, keep its tag and content variant
    /// consistent; [`dump_copy`](Self::dump_copy) checks this before committing output.
    pub fn get_mut_section(&mut self, index: usize) -> Result<Option<&mut Section>> {
        if index >= self.sections.len() {
            return Ok(None);
        }
        self.decode(index)?;
        self.sections[index].potentially_modified = true;
        Ok(self.sections[index].decoded.as_mut())
    }

    /// Mutably borrow the first section with `tag`, in file order.
    ///
    /// Returns `Ok(None)` if absent. Only the selected section is decoded and marked
    /// for serialization. Use [`get_mut_sections_by_tag`](Self::get_mut_sections_by_tag)
    /// to edit every occurrence of a repeated tag. Reading or parsing can fail.
    pub fn get_mut_section_by_tag(&mut self, tag: AnlzTag) -> Result<Option<&mut Section>> {
        match self.sections.iter().position(|s| s.tag() == &tag) {
            Some(i) => self.get_mut_section(i),
            None => Ok(None),
        }
    }

    /// Borrow the first section with `tag`, in file order.
    ///
    /// Returns `Ok(None)` if absent. Only the selected section is decoded; the
    /// access does not mark it modified. Reading or parsing can fail.
    /// Use indexed access to select a later occurrence of a repeated tag.
    pub fn get_section_by_tag(&mut self, tag: AnlzTag) -> Result<Option<&Section>> {
        match self.sections.iter().position(|s| s.tag() == &tag) {
            Some(i) => self.get_section(i),
            None => Ok(None),
        }
    }

    /// Mutably borrow every section with `tag`, preserving file order.
    ///
    /// Returns an empty vector if absent. All matches are decoded before any are
    /// marked for serialization. If a decode fails, the error is returned and
    /// earlier successful decodes remain cached; this call adds no modification
    /// flags. On success, every returned section is marked for serialization.
    pub fn get_mut_sections_by_tag(&mut self, tag: AnlzTag) -> Result<Vec<&mut Section>> {
        let indices: Vec<_> = self
            .sections
            .iter()
            .enumerate()
            .filter_map(|(i, s)| (s.tag() == &tag).then_some(i))
            .collect();
        for &i in &indices {
            self.decode(i)?;
        }
        Ok(self
            .sections
            .iter_mut()
            .enumerate()
            .filter_map(|(i, s)| {
                if indices.binary_search(&i).is_ok() {
                    s.potentially_modified = true;
                    s.decoded.as_mut()
                } else {
                    None
                }
            })
            .collect())
    }

    /// Return whether any section currently has `tag`, without reading payloads.
    ///
    /// Uses cached tags when available, so tag edits through mutable section
    /// access are reflected immediately.
    pub fn contains(&self, tag: AnlzTag) -> bool {
        self.sections.iter().any(|s| s.tag() == &tag)
    }

    /// Return section names in file order, retaining duplicate and unknown tags.
    ///
    /// Names use [`AnlzTag::to_string`] (for example, `"BeatGrid"`, not `"PQTZ"`).
    /// This reads no payloads; the `Result` return type is retained for API compatibility.
    pub fn get_tags(&self) -> Result<Vec<String>> {
        Ok(self.sections.iter().map(|s| s.tag().to_string()).collect())
    }

    /// Consume the handler and materialize all sections as owned [`AnlzData`].
    ///
    /// Includes cached edits, decodes every remaining section, and recalculates the
    /// file-header sizes. No output is written. Returns an error if any remaining
    /// section cannot be read or parsed; the handler is consumed even on failure.
    pub fn into_data(mut self) -> Result<AnlzData> {
        for i in 0..self.sections.len() {
            self.decode(i)?;
        }
        let mut data = AnlzData {
            header: self.header,
            header_data: self.header_data,
            sections: self
                .sections
                .into_iter()
                .map(|s| {
                    s.decoded.ok_or(AnlzError::InvalidState(
                        "a materialized section was not decoded",
                    ))
                })
                .collect::<Result<Vec<_>>>()?,
        };
        data.update_total_size();
        Ok(data)
    }

    /// Write the current contents to `path`, replacing an existing destination.
    ///
    /// Writes a complete file through a temporary sibling, preserving section order.
    /// Sections marked for serialization are encoded from their cached values;
    /// untouched sections are copied directly from the source, even when cached.
    /// Consequently, saving does not validate the payloads of untouched sections.
    /// Lengths are updated from the output, which is flushed and synced before
    /// replacing the destination.
    ///
    /// If `path` resolves to the original source path, behaves like [`dump`](Self::dump):
    /// adopts the new file, refreshes byte ranges, and clears modification flags.
    /// Otherwise the source, cached edits, and modification flags are retained.
    ///
    /// # Errors
    ///
    /// Returns errors from source reads, serialization, destination creation or
    /// replacement, inconsistent tag/content pairs, and output exceeding the
    /// format's 32-bit file length. The destination's parent must already exist.
    /// Failures before replacement leave the destination and cached edits intact
    /// so the caller can correct the problem and retry.
    pub fn dump_copy<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        let destination = if path.exists() {
            std::fs::canonicalize(path)?
        } else {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| std::path::Path::new("."));
            std::fs::canonicalize(parent)?.join(
                path.file_name()
                    .ok_or_else(|| AnlzError::InvalidOutputPath(path.to_path_buf()))?,
            )
        };
        let replace_source = destination == self.path;
        self.source()?;
        let output_parent = destination
            .parent()
            .ok_or_else(|| AnlzError::InvalidOutputPath(destination.clone()))?;
        let mut output = tempfile::NamedTempFile::new_in(output_parent)?;
        self.header.write_be(&mut output)?;
        output.write_all(&self.header_data)?;
        let mut ranges = Vec::with_capacity(self.sections.len());
        for entry in &self.sections {
            let start = output.stream_position()?;
            if entry.potentially_modified {
                let section = entry.decoded.as_ref().ok_or(AnlzError::InvalidState(
                    "a modified section was not decoded",
                ))?;
                if !section.tag_matches_content() {
                    return Err(AnlzError::TagContentMismatch {
                        tag: section.tag.clone(),
                    });
                }
                section.write_be(&mut output)?;
            } else {
                let range = entry
                    .original_range
                    .as_ref()
                    .ok_or(AnlzError::InvalidState(
                        "an unchanged section has no source range",
                    ))?;
                let source = self
                    .source
                    .as_mut()
                    .ok_or(AnlzError::InvalidState("source file was not opened"))?;
                source.seek(SeekFrom::Start(range.start))?;
                let len = range.end - range.start;
                let copied = std::io::copy(&mut source.take(len), &mut output)?;
                if copied != len {
                    return Err(AnlzError::InvalidFile {
                        reason: InvalidFileReason::BackingFileTruncated,
                        offset: range.start + copied,
                    });
                }
            }
            let end = output.stream_position()?;
            if end > u64::from(u32::MAX) {
                return Err(AnlzError::OutputTooLarge { size: end });
            }
            if entry.potentially_modified {
                // Cached binary structs can retain their original size fields.
                // Patch the common section length using the bytes actually written.
                output.seek(SeekFrom::Start(start + 8))?;
                ((end - start) as u32).write_be(&mut output)?;
                output.seek(SeekFrom::Start(end))?;
            }
            ranges.push(start..end);
        }
        let mut header = self.header.clone();
        let output_size = output.stream_position()?;
        header.total_size = u32::try_from(output_size)
            .map_err(|_| AnlzError::OutputTooLarge { size: output_size })?;
        output.seek(SeekFrom::Start(0))?;
        header.write_be(&mut output)?;
        if let Ok(metadata) = std::fs::metadata(&destination) {
            output.as_file().set_permissions(metadata.permissions())?;
        }
        output.flush()?;
        output.as_file().sync_all()?;
        // Release the source before replacement for Windows filesystems that
        // reject replacing an open file. On failure the source is reopened when
        // next needed; cached edits and original offsets remain intact.
        if replace_source {
            self.source.take();
        }
        let file = output
            .persist(&destination)
            .map_err(|error| AnlzError::Persist {
                path: destination.clone(),
                source: error.error,
            })?;
        if replace_source {
            // Commit offsets only after replacement succeeds. Copies to another
            // path must keep using the original ranges and modification flags.
            self.source = Some(file);
            self.header = header;
            for (entry, range) in self.sections.iter_mut().zip(ranges) {
                entry.original_range = Some(range);
                entry.potentially_modified = false;
                entry.tag = entry.tag().clone();
            }
        }
        Ok(())
    }

    /// Save the current contents to the original ANLZ path.
    ///
    /// Uses [`dump_copy`](Self::dump_copy), including its temporary-file replacement
    /// and error handling. On success, the output becomes the new backing source;
    /// byte ranges are refreshed and modification flags cleared. Decoded sections
    /// remain cached, and previously unread sections can still be loaded lazily.
    pub fn dump(&mut self) -> Result<()> {
        self.dump_copy(self.path.clone())
    }

    // ---- Beat Grid Tags -------------------------------------------------------------------------

    /// Mutably borrow the first PQTZ section (beat grid).
    ///
    /// Uses the same selection and loading behavior as [`get_beat_grid`](Self::get_beat_grid).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_beat_grid(&mut self) -> Result<Option<&mut BeatGrid>> {
        Ok(match self.get_mut_section_by_tag(AnlzTag::BeatGrid)? {
            Some(Section {
                content: Content::BeatGrid(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Borrow the first PQTZ section (beat grid), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_beat_grid(&mut self) -> Result<Option<&BeatGrid>> {
        Ok(match self.get_section_by_tag(AnlzTag::BeatGrid)? {
            Some(Section {
                content: Content::BeatGrid(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Mutably borrow the first PQT2 section (extended beat grid).
    ///
    /// Uses the same selection and loading behavior as [`get_extended_beat_grid`](Self::get_extended_beat_grid).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_extended_beat_grid(&mut self) -> Result<Option<&mut ExtendedBeatGrid>> {
        Ok(
            match self.get_mut_section_by_tag(AnlzTag::ExtendedBeatGrid)? {
                Some(Section {
                    content: Content::ExtendedBeatGrid(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Borrow the first PQT2 section (extended beat grid), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_extended_beat_grid(&mut self) -> Result<Option<&ExtendedBeatGrid>> {
        Ok(match self.get_section_by_tag(AnlzTag::ExtendedBeatGrid)? {
            Some(Section {
                content: Content::ExtendedBeatGrid(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    // ---- Cue List Tags --------------------------------------------------------------------------

    /// Resolve a cue-list subtype without marking inspected candidates modified.
    fn cue_index(&mut self, tag: AnlzTag, list_type: CueListType) -> Result<Option<usize>> {
        for i in 0..self.sections.len() {
            if self.sections[i].tag() != &tag {
                continue;
            }
            let section = self.get_section(i)?.ok_or(AnlzError::InvalidState(
                "an indexed section disappeared during lookup",
            ))?;
            let matches = match &section.content {
                Content::CueList(c) => c.list_type == list_type,
                Content::ExtendedCueList(c) => c.list_type == list_type,
                _ => false,
            };
            if matches {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    /// Mutably borrow the first hot-cue list (PCOB), in file order.
    ///
    /// Uses the same selection and decoding behavior as [`get_hot_cues`](Self::get_hot_cues).
    /// Only the selected list is marked for serialization by this call; inspecting
    /// other candidates leaves their existing modification flags unchanged.
    pub fn get_mut_hot_cues(&mut self) -> Result<Option<&mut CueList>> {
        let Some(i) = self.cue_index(AnlzTag::CueList, CueListType::HotCues)? else {
            return Ok(None);
        };
        Ok(match self.get_mut_section(i)? {
            Some(Section {
                content: Content::CueList(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Borrow the first hot-cue list (PCOB), in file order.
    ///
    /// The same tag can hold memory or hot cues. Matching-tag sections are decoded
    /// in order until their list type matches, so more than one list may be cached.
    /// Returns `Ok(None)` if none matches, or an error if a candidate cannot be decoded.
    /// No modification flags are set.
    pub fn get_hot_cues(&mut self) -> Result<Option<&CueList>> {
        let Some(i) = self.cue_index(AnlzTag::CueList, CueListType::HotCues)? else {
            return Ok(None);
        };
        Ok(match self.get_section(i)? {
            Some(Section {
                content: Content::CueList(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Mutably borrow the first memory-cue list (PCOB), in file order.
    ///
    /// Uses the same selection and decoding behavior as [`get_memory_cues`](Self::get_memory_cues).
    /// Only the selected list is marked for serialization by this call; inspecting
    /// other candidates leaves their existing modification flags unchanged.
    pub fn get_mut_memory_cues(&mut self) -> Result<Option<&mut CueList>> {
        let Some(i) = self.cue_index(AnlzTag::CueList, CueListType::MemoryCues)? else {
            return Ok(None);
        };
        Ok(match self.get_mut_section(i)? {
            Some(Section {
                content: Content::CueList(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Borrow the first memory-cue list (PCOB), in file order.
    ///
    /// The same tag can hold memory or hot cues. Matching-tag sections are decoded
    /// in order until their list type matches, so more than one list may be cached.
    /// Returns `Ok(None)` if none matches, or an error if a candidate cannot be decoded.
    /// No modification flags are set.
    pub fn get_memory_cues(&mut self) -> Result<Option<&CueList>> {
        let Some(i) = self.cue_index(AnlzTag::CueList, CueListType::MemoryCues)? else {
            return Ok(None);
        };
        Ok(match self.get_section(i)? {
            Some(Section {
                content: Content::CueList(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Mutably borrow the first extended hot-cue list (PCO2), in file order.
    ///
    /// Uses the same selection and decoding behavior as [`get_extended_hot_cues`](Self::get_extended_hot_cues).
    /// Only the selected list is marked for serialization by this call; inspecting
    /// other candidates leaves their existing modification flags unchanged.
    pub fn get_mut_extended_hot_cues(&mut self) -> Result<Option<&mut ExtendedCueList>> {
        let Some(i) = self.cue_index(AnlzTag::ExtendedCueList, CueListType::HotCues)? else {
            return Ok(None);
        };
        Ok(match self.get_mut_section(i)? {
            Some(Section {
                content: Content::ExtendedCueList(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Borrow the first extended hot-cue list (PCO2), in file order.
    ///
    /// The same tag can hold memory or hot cues. Matching-tag sections are decoded
    /// in order until their list type matches, so more than one list may be cached.
    /// Returns `Ok(None)` if none matches, or an error if a candidate cannot be decoded.
    /// No modification flags are set.
    pub fn get_extended_hot_cues(&mut self) -> Result<Option<&ExtendedCueList>> {
        let Some(i) = self.cue_index(AnlzTag::ExtendedCueList, CueListType::HotCues)? else {
            return Ok(None);
        };
        Ok(match self.get_section(i)? {
            Some(Section {
                content: Content::ExtendedCueList(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Mutably borrow the first extended memory-cue list (PCO2), in file order.
    ///
    /// Uses the same selection and decoding behavior as [`get_extended_memory_cues`](Self::get_extended_memory_cues).
    /// Only the selected list is marked for serialization by this call; inspecting
    /// other candidates leaves their existing modification flags unchanged.
    pub fn get_mut_extended_memory_cues(&mut self) -> Result<Option<&mut ExtendedCueList>> {
        let Some(i) = self.cue_index(AnlzTag::ExtendedCueList, CueListType::MemoryCues)? else {
            return Ok(None);
        };
        Ok(match self.get_mut_section(i)? {
            Some(Section {
                content: Content::ExtendedCueList(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Borrow the first extended memory-cue list (PCO2), in file order.
    ///
    /// The same tag can hold memory or hot cues. Matching-tag sections are decoded
    /// in order until their list type matches, so more than one list may be cached.
    /// Returns `Ok(None)` if none matches, or an error if a candidate cannot be decoded.
    /// No modification flags are set.
    pub fn get_extended_memory_cues(&mut self) -> Result<Option<&ExtendedCueList>> {
        let Some(i) = self.cue_index(AnlzTag::ExtendedCueList, CueListType::MemoryCues)? else {
            return Ok(None);
        };
        Ok(match self.get_section(i)? {
            Some(Section {
                content: Content::ExtendedCueList(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    // ---- Path Tag -------------------------------------------------------------------------------

    /// Return an owned copy of the audio path in the first PPTH section.
    ///
    /// This is the embedded audio-file path, not the ANLZ file's source path.
    /// Returns `Ok(None)` if absent; reading or parsing PPTH can fail.
    /// Does not mark the section modified.
    pub fn get_path(&mut self) -> Result<Option<String>> {
        Ok(match self.get_section_by_tag(AnlzTag::Path)? {
            Some(Section {
                content: Content::Path(p),
                ..
            }) => Some(p.path.to_string()),
            _ => None,
        })
    }

    /// Set the embedded audio path in the first PPTH section.
    ///
    /// Updates an existing section in place, or inserts a new PPTH as the first
    /// section after PMAI. Insertion shifts existing section indices by one.
    /// Path conversion follows [`Path::set`], including normalizing backslashes to
    /// forward slashes. The ANLZ source path is unchanged.
    ///
    /// Only the affected section is marked for serialization; unrelated payloads
    /// are not decoded. Call [`dump`](Self::dump) or [`dump_copy`](Self::dump_copy)
    /// to persist the edit. Reading an existing PPTH or a tag/content mismatch
    /// can produce an error.
    pub fn set_path<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        if let Some(section) = self.get_mut_section_by_tag(AnlzTag::Path)? {
            match &mut section.content {
                Content::Path(p) => p.set(path),
                _ => {
                    return Err(AnlzError::TagContentMismatch { tag: AnlzTag::Path });
                }
            }
        } else {
            self.sections.insert(
                0,
                SectionEntry {
                    original_range: None,
                    tag: AnlzTag::Path,
                    decoded: Some(Section {
                        tag: AnlzTag::Path,
                        content: Content::Path(Path::new(path)),
                    }),
                    potentially_modified: true,
                },
            );
        }
        Ok(())
    }

    // ---- VBR Tag --------------------------------------------------------------------------------

    /// Mutably borrow the first PVBR section (variable-bitrate seek table).
    ///
    /// Uses the same selection and loading behavior as [`get_vbr`](Self::get_vbr).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_vbr(&mut self) -> Result<Option<&mut VBR>> {
        Ok(match self.get_mut_section_by_tag(AnlzTag::VBR)? {
            Some(Section {
                content: Content::VBR(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Borrow the first PVBR section (variable-bitrate seek table), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_vbr(&mut self) -> Result<Option<&VBR>> {
        Ok(match self.get_section_by_tag(AnlzTag::VBR)? {
            Some(Section {
                content: Content::VBR(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Return an owned big-endian byte representation of the PVBR index and trailer.
    ///
    /// Contains the 400 index values followed by `u2`, excluding the section header
    /// and `u1`. Retained for compatibility; prefer [`get_vbr`](Self::get_vbr)
    /// for typed access without this allocation. Shares that getter's loading,
    /// absence, and error behavior.
    pub fn get_vbr_data(&mut self) -> Result<Option<Vec<u8>>> {
        Ok(self.get_vbr()?.map(|v| {
            v.index
                .iter()
                .chain(std::iter::once(&v.u2))
                .flat_map(|v| v.to_be_bytes())
                .collect()
        }))
    }

    /// Mutably borrow the first PVB2 section (frame seek index).
    ///
    /// Uses the same selection and loading behavior as [`get_seek_index`](Self::get_seek_index).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_seek_index(&mut self) -> Result<Option<&mut SeekIndex>> {
        Ok(match self.get_mut_section_by_tag(AnlzTag::SeekIndex)? {
            Some(Section {
                content: Content::SeekIndex(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Borrow the first PVB2 section (frame seek index), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_seek_index(&mut self) -> Result<Option<&SeekIndex>> {
        Ok(match self.get_section_by_tag(AnlzTag::SeekIndex)? {
            Some(Section {
                content: Content::SeekIndex(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    // ---- Tiny Waveform Preview Tag --------------------------------------------------------------

    /// Mutably borrow the first PWV2 section (small monochrome waveform preview).
    ///
    /// Uses the same selection and loading behavior as [`get_tiny_waveform_preview`](Self::get_tiny_waveform_preview).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_tiny_waveform_preview(&mut self) -> Result<Option<&mut TinyWaveformPreview>> {
        Ok(
            match self.get_mut_section_by_tag(AnlzTag::TinyWaveformPreview)? {
                Some(Section {
                    content: Content::TinyWaveformPreview(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Borrow the first PWV2 section (small monochrome waveform preview), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_tiny_waveform_preview(&mut self) -> Result<Option<&TinyWaveformPreview>> {
        Ok(
            match self.get_section_by_tag(AnlzTag::TinyWaveformPreview)? {
                Some(Section {
                    content: Content::TinyWaveformPreview(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    // ---- Waveform Tags --------------------------------------------------------------------------

    /// Mutably borrow the first PWAV section (monochrome waveform preview).
    ///
    /// Uses the same selection and loading behavior as [`get_waveform_preview`](Self::get_waveform_preview).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_waveform_preview(&mut self) -> Result<Option<&mut WaveformPreview>> {
        Ok(
            match self.get_mut_section_by_tag(AnlzTag::WaveformPreview)? {
                Some(Section {
                    content: Content::WaveformPreview(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Borrow the first PWAV section (monochrome waveform preview), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_waveform_preview(&mut self) -> Result<Option<&WaveformPreview>> {
        Ok(match self.get_section_by_tag(AnlzTag::WaveformPreview)? {
            Some(Section {
                content: Content::WaveformPreview(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    /// Mutably borrow the first PWV3 section (detailed monochrome waveform).
    ///
    /// Uses the same selection and loading behavior as [`get_waveform_detail`](Self::get_waveform_detail).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_waveform_detail(&mut self) -> Result<Option<&mut WaveformDetail>> {
        Ok(
            match self.get_mut_section_by_tag(AnlzTag::WaveformDetail)? {
                Some(Section {
                    content: Content::WaveformDetail(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Borrow the first PWV3 section (detailed monochrome waveform), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_waveform_detail(&mut self) -> Result<Option<&WaveformDetail>> {
        Ok(match self.get_section_by_tag(AnlzTag::WaveformDetail)? {
            Some(Section {
                content: Content::WaveformDetail(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }

    // ---- Waveform Color Tag ---------------------------------------------------------------------

    /// Mutably borrow the first PWV4 section (color waveform preview).
    ///
    /// Uses the same selection and loading behavior as [`get_waveform_color_preview`](Self::get_waveform_color_preview).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_waveform_color_preview(&mut self) -> Result<Option<&mut WaveformColorPreview>> {
        Ok(
            match self.get_mut_section_by_tag(AnlzTag::WaveformColorPreview)? {
                Some(Section {
                    content: Content::WaveformColorPreview(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Borrow the first PWV4 section (color waveform preview), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_waveform_color_preview(&mut self) -> Result<Option<&WaveformColorPreview>> {
        Ok(
            match self.get_section_by_tag(AnlzTag::WaveformColorPreview)? {
                Some(Section {
                    content: Content::WaveformColorPreview(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    // ---- Waveform Color Detail Tag --------------------------------------------------------------

    /// Mutably borrow the first PWV5 section (detailed color waveform).
    ///
    /// Uses the same selection and loading behavior as [`get_waveform_color_detail`](Self::get_waveform_color_detail).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_waveform_color_detail(&mut self) -> Result<Option<&mut WaveformColorDetail>> {
        Ok(
            match self.get_mut_section_by_tag(AnlzTag::WaveformColorDetail)? {
                Some(Section {
                    content: Content::WaveformColorDetail(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Borrow the first PWV5 section (detailed color waveform), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_waveform_color_detail(&mut self) -> Result<Option<&WaveformColorDetail>> {
        Ok(
            match self.get_section_by_tag(AnlzTag::WaveformColorDetail)? {
                Some(Section {
                    content: Content::WaveformColorDetail(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    // ---- Waveform 3-Band Tags -------------------------------------------------------------------

    /// Mutably borrow the first PWV6 section (three-band waveform preview).
    ///
    /// Uses the same selection and loading behavior as [`get_waveform_3band_preview`](Self::get_waveform_3band_preview).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_waveform_3band_preview(&mut self) -> Result<Option<&mut Waveform3BandPreview>> {
        Ok(
            match self.get_mut_section_by_tag(AnlzTag::Waveform3BandPreview)? {
                Some(Section {
                    content: Content::Waveform3BandPreview(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Borrow the first PWV6 section (three-band waveform preview), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_waveform_3band_preview(&mut self) -> Result<Option<&Waveform3BandPreview>> {
        Ok(
            match self.get_section_by_tag(AnlzTag::Waveform3BandPreview)? {
                Some(Section {
                    content: Content::Waveform3BandPreview(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Mutably borrow the first PWV7 section (detailed three-band waveform).
    ///
    /// Uses the same selection and loading behavior as [`get_waveform_3band_detail`](Self::get_waveform_3band_detail).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_waveform_3band_detail(&mut self) -> Result<Option<&mut Waveform3BandDetail>> {
        Ok(
            match self.get_mut_section_by_tag(AnlzTag::Waveform3BandDetail)? {
                Some(Section {
                    content: Content::Waveform3BandDetail(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Borrow the first PWV7 section (detailed three-band waveform), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_waveform_3band_detail(&mut self) -> Result<Option<&Waveform3BandDetail>> {
        Ok(
            match self.get_section_by_tag(AnlzTag::Waveform3BandDetail)? {
                Some(Section {
                    content: Content::Waveform3BandDetail(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    // ---- Waveform 3-Band Calibration Tag --------------------------------------------------------

    /// Mutably borrow the first PWVC section (three-band waveform calibration).
    ///
    /// Uses the same selection and loading behavior as [`get_waveform_3band_calibration`](Self::get_waveform_3band_calibration).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_waveform_3band_calibration(
        &mut self,
    ) -> Result<Option<&mut Waveform3BandCalibration>> {
        Ok(
            match self.get_mut_section_by_tag(AnlzTag::Waveform3BandCalibration)? {
                Some(Section {
                    content: Content::Waveform3BandCalibration(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Borrow the first PWVC section (three-band waveform calibration), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_waveform_3band_calibration(&mut self) -> Result<Option<&Waveform3BandCalibration>> {
        Ok(
            match self.get_section_by_tag(AnlzTag::Waveform3BandCalibration)? {
                Some(Section {
                    content: Content::Waveform3BandCalibration(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    // ---- Song Structure Tag ---------------------------------------------------------------------

    /// Mutably borrow the analysis data in the first PSSI song-structure section.
    ///
    /// Decodes the section on demand, including decryption when needed, and marks
    /// it for serialization. The enclosing section's version and encryption fields
    /// are preserved; use [`get_mut_section_by_tag`](Self::get_mut_section_by_tag)
    /// to edit those fields. Returns `Ok(None)` if absent or an error on decode failure.
    pub fn get_mut_song_structure(&mut self) -> Result<Option<&mut SongStructureData>> {
        Ok(match self.get_mut_section_by_tag(AnlzTag::SongStructure)? {
            Some(Section {
                content: Content::SongStructure(s),
                ..
            }) => Some(&mut s.data),
            _ => None,
        })
    }

    /// Borrow the analysis data in the first PSSI song-structure section.
    ///
    /// Decodes and caches the section, including decryption when needed, without
    /// marking it modified. Returns `Ok(None)` if absent or an error on decode failure.
    /// Call `.cloned()` on the returned option when an owned copy is needed.
    pub fn get_song_structure(&mut self) -> Result<Option<&SongStructureData>> {
        Ok(match self.get_section_by_tag(AnlzTag::SongStructure)? {
            Some(Section {
                content: Content::SongStructure(s),
                ..
            }) => Some(&s.data),
            _ => None,
        })
    }

    // ---- Vocal Detection ------------------------------------------------------------------------

    /// Mutably borrow the first PVDI section (vocal-detection confidence data).
    ///
    /// Uses the same selection and loading behavior as [`get_vocal_detection`](Self::get_vocal_detection).
    /// Marks the section for serialization even if the returned value is not changed.
    pub fn get_mut_vocal_detection(&mut self) -> Result<Option<&mut VocalDetection>> {
        Ok(
            match self.get_mut_section_by_tag(AnlzTag::VocalDetection)? {
                Some(Section {
                    content: Content::VocalDetection(value),
                    ..
                }) => Some(value),
                _ => None,
            },
        )
    }

    /// Borrow the first PVDI section (vocal-detection confidence data), in file order.
    ///
    /// Decodes and caches only that section without marking it modified.
    /// Returns `Ok(None)` if absent; reading or parsing errors are propagated.
    pub fn get_vocal_detection(&mut self) -> Result<Option<&VocalDetection>> {
        Ok(match self.get_section_by_tag(AnlzTag::VocalDetection)? {
            Some(Section {
                content: Content::VocalDetection(value),
                ..
            }) => Some(value),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn failed_source_replacement_reopens_for_retry() {
        use std::os::windows::fs::OpenOptionsExt;
        let bytes = file_bytes(&[path_section("/old"), waveform()]);
        let (_dir, mut file) = open(&bytes);
        file.set_path("/new").unwrap();
        // Allow readers/writers but deny delete sharing, forcing replacement to fail.
        let lock = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(3)
            .open(&file.path)
            .unwrap();
        assert!(file.dump().is_err());
        assert!(file.source.is_none());
        assert_eq!(std::fs::read(&file.path).unwrap(), bytes);
        assert!(file.sections[0].potentially_modified);
        drop(lock);
        // An unloaded read must reopen the old source with its original offsets.
        assert_eq!(
            file.get_waveform_detail().unwrap().unwrap().data.len(),
            100_000
        );
        file.dump().unwrap();
        assert_eq!(
            Anlz::load(&file.path)
                .unwrap()
                .get_path()
                .unwrap()
                .as_deref(),
            Some("/new")
        );
    }

    #[test]
    fn index_reads_only_headers_and_seeks_over_payloads() {
        struct TrackedReader {
            inner: Cursor<Vec<u8>>,
            reads: Vec<Range<u64>>,
        }
        impl Read for TrackedReader {
            fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
                let start = self.inner.position();
                let n = self.inner.read(bytes)?;
                self.reads.push(start..start + n as u64);
                Ok(n)
            }
        }
        impl Seek for TrackedReader {
            fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
                self.inner.seek(pos)
            }
        }
        let wave = waveform();
        let mut reader = TrackedReader {
            inner: Cursor::new(file_bytes(&[wave.clone(), wave])),
            reads: Vec::new(),
        };
        let (_, _, entries) = Anlz::index(&mut reader).unwrap();
        for entry in entries {
            let range = entry.original_range.unwrap();
            let payload = range.start + 12..range.end;
            assert!(reader
                .reads
                .iter()
                .all(|r| r.end <= payload.start || r.start >= payload.end));
        }
        assert_eq!(
            reader.reads.iter().map(|r| r.end - r.start).sum::<u64>(),
            28 + 2 * 12
        );
    }

    #[test]
    fn section_boundary_prevents_consuming_next_section() {
        let mut broken = waveform();
        broken.truncate(24); // claims one sample but has no payload
        broken[8..12].copy_from_slice(&24u32.to_be_bytes());
        broken[16..20].copy_from_slice(&1u32.to_be_bytes());
        let (_dir, mut file) = open(&file_bytes(&[broken, path_section("/still-readable")]));
        assert!(matches!(
            file.get_waveform_detail(),
            Err(AnlzError::SectionDecode {
                index: 0,
                tag: AnlzTag::WaveformDetail,
                offset: 28,
                ..
            })
        ));
        assert_eq!(file.get_path().unwrap().as_deref(), Some("/still-readable"));
    }

    #[test]
    fn persistence_failure_can_be_retried() {
        let bytes = file_bytes(&[path_section("/old"), waveform()]);
        let (dir, mut file) = open(&bytes);
        file.set_path("/new").unwrap();
        let directory_destination = dir.path().join("a-directory");
        std::fs::create_dir(&directory_destination).unwrap();
        assert!(file.dump_copy(&directory_destination).is_err());
        assert_eq!(std::fs::read(&file.path).unwrap(), bytes);
        assert!(file.sections[0].potentially_modified);
        file.dump().unwrap();
        assert_eq!(file.get_path().unwrap().as_deref(), Some("/new"));
    }

    fn section(tag: &[u8; 4], header_size: u32, payload: &[u8]) -> Vec<u8> {
        let mut bytes = tag.to_vec();
        bytes.extend(header_size.to_be_bytes());
        bytes.extend((12 + payload.len() as u32).to_be_bytes());
        bytes.extend(payload);
        bytes
    }

    fn file_bytes(sections: &[Vec<u8>]) -> Vec<u8> {
        let mut bytes = b"PMAI".to_vec();
        bytes.extend(28u32.to_be_bytes());
        bytes.extend((28 + sections.iter().map(Vec::len).sum::<usize>() as u32).to_be_bytes());
        bytes.extend([0x45; 16]); // opaque file-header extension must survive saves
        for s in sections {
            bytes.extend(s);
        }
        bytes
    }

    fn path_section(path: &str) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        Section {
            tag: AnlzTag::Path,
            content: Content::Path(Path::new(path)),
        }
        .write_be(&mut cursor)
        .unwrap();
        cursor.into_inner()
    }

    fn waveform() -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend(1u32.to_be_bytes());
        payload.extend(100_000u32.to_be_bytes());
        payload.extend(0x00960000u32.to_be_bytes());
        payload.extend(vec![0x23; 100_000]);
        section(b"PWV3", 24, &payload)
    }

    fn cue(list_type: u32) -> Vec<u8> {
        let mut payload = list_type.to_be_bytes().to_vec();
        payload.extend([0; 4]);
        payload.extend(u32::MAX.to_be_bytes());
        section(b"PCOB", 24, &payload)
    }

    fn open(bytes: &[u8]) -> (tempfile::TempDir, Anlz) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ANLZ0000.EXT");
        std::fs::write(&path, bytes).unwrap();
        let file = Anlz::load(path).unwrap();
        (dir, file)
    }

    #[test]
    fn indexing_and_path_edits_leave_waveforms_unloaded() {
        let wave = waveform();
        let unknown = section(b"ZZZZ", 12, &[9, 8, 7]);
        let bytes = file_bytes(&[path_section("/old"), wave.clone(), unknown.clone()]);
        let (dir, mut file) = open(&bytes);
        assert_eq!(file.section_count(), 3);
        assert!(file.contains(AnlzTag::WaveformDetail));
        assert_eq!(file.get_tags().unwrap()[0], "Path");
        assert!(file.sections.iter().all(|s| s.decoded.is_none()));
        file.set_path("/a/much/longer/audio/path.flac").unwrap();
        let copy = dir.path().join("copy.EXT");
        file.dump_copy(&copy).unwrap();
        assert!(file.sections[0].potentially_modified);
        assert!(file.sections[1..].iter().all(|s| s.decoded.is_none()));
        assert_eq!(std::fs::read(&file.path).unwrap(), bytes);
        let output = std::fs::read(&copy).unwrap();
        assert!(output.ends_with(&[wave, unknown].concat()));
        assert_eq!(&output[12..28], &bytes[12..28]);
        let eager = AnlzData::read_be(&mut Cursor::new(&output)).unwrap();
        assert_eq!(eager.header.total_size as usize, output.len());
        assert_eq!(file.into_data().unwrap(), eager);
    }

    #[test]
    fn shared_access_caches_without_dirtying_and_mutable_access_dirties() {
        let (_dir, mut file) = open(&file_bytes(&[waveform()]));
        let first = file.get_waveform_detail().unwrap().unwrap().data.as_ptr();
        assert_eq!(
            first,
            file.get_waveform_detail().unwrap().unwrap().data.as_ptr()
        );
        assert!(!file.sections[0].potentially_modified);
        file.get_mut_waveform_detail().unwrap().unwrap().data.pop();
        assert!(file.sections[0].potentially_modified);
    }

    #[test]
    fn repeated_in_place_saves_refresh_unloaded_offsets() {
        let (_dir, mut file) = open(&file_bytes(&[path_section("/old"), waveform()]));
        file.set_path("/new/longer/path.flac").unwrap();
        file.dump().unwrap(); // also exercises Windows replacement of an open source
        assert!(file.sections.iter().all(|s| !s.potentially_modified));
        assert!(file.sections[1].decoded.is_none());
        file.dump_copy(file.path.clone()).unwrap();
        assert_eq!(
            file.get_waveform_detail().unwrap().unwrap().data.len(),
            100_000
        );
        file.get_mut_waveform_detail()
            .unwrap()
            .unwrap()
            .data
            .truncate(17);
        file.dump().unwrap();
        let mut reopened = Anlz::load(&file.path).unwrap();
        assert_eq!(
            reopened.get_path().unwrap().as_deref(),
            Some("/new/longer/path.flac")
        );
        assert_eq!(
            reopened.get_waveform_detail().unwrap().unwrap().data.len(),
            17
        );
        // Parsed structs retain private input-length fields; writers recalculate
        // them. Compare the materialized wire representation after resizing.
        let mut cached = Cursor::new(Vec::new());
        file.into_data().unwrap().write_be(&mut cached).unwrap();
        let mut fresh = Cursor::new(Vec::new());
        reopened.into_data().unwrap().write_be(&mut fresh).unwrap();
        assert_eq!(cached.into_inner(), fresh.into_inner());
    }

    #[test]
    fn unknown_duplicates_and_missing_path_keep_order() {
        let unknown = section(b"ZZZZ", 12, &[7, 6, 5]);
        let bytes = file_bytes(&[unknown.clone(), cue(1), cue(0), unknown.clone()]);
        let (_dir, mut file) = open(&bytes);
        assert_eq!(
            file.get_mut_hot_cues().unwrap().unwrap().list_type,
            CueListType::HotCues
        );
        assert_eq!(
            file.get_mut_memory_cues().unwrap().unwrap().list_type,
            CueListType::MemoryCues
        );
        file.set_path("/inserted").unwrap();
        file.dump().unwrap();
        let mut reopened = Anlz::load(&file.path).unwrap();
        assert_eq!(reopened.get_section(0).unwrap().unwrap().tag, AnlzTag::Path);
        assert_eq!(
            reopened
                .get_mut_sections_by_tag(AnlzTag::CueList)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            reopened.get_section(1).unwrap().unwrap().tag,
            AnlzTag::Unknown(*b"ZZZZ")
        );
        assert_eq!(
            reopened.get_section(4).unwrap().unwrap().tag,
            AnlzTag::Unknown(*b"ZZZZ")
        );
        assert!(reopened.get_section(5).unwrap().is_none());
        assert!(std::fs::read(&file.path).unwrap().ends_with(&unknown));
    }

    #[test]
    fn malformed_payload_is_deferred_and_can_be_copied_untouched() {
        let invalid = section(b"PQTZ", 24, &[0; 12]); // valid outer header, invalid PQTZ header size/data
        let mut invalid = invalid;
        // n_beats = 1 but there are no beat records.
        invalid[20..24].copy_from_slice(&1u32.to_be_bytes());
        let bytes = file_bytes(&[path_section("/old"), invalid.clone(), waveform()]);
        let (dir, mut file) = open(&bytes);
        assert!(file.get_mut_beat_grid().is_err());
        assert!(file.sections[1].decoded.is_none());
        assert!(file.sections.iter().all(|s| !s.potentially_modified));
        file.set_path("/new").unwrap();
        let out = dir.path().join("copy.EXT");
        file.dump_copy(&out).unwrap();
        let mut copied = Anlz::load(&out).unwrap();
        assert_eq!(copied.get_path().unwrap().as_deref(), Some("/new"));
        assert!(copied.get_mut_beat_grid().is_err());
        assert_eq!(
            copied.get_waveform_detail().unwrap().unwrap().data.len(),
            100_000
        );
    }

    #[test]
    fn invalid_outer_bounds_fail_at_open() {
        for (offset, value) in [(4, 11u32), (8, 1), (32, 11), (36, 0), (36, u32::MAX)] {
            let mut bytes = file_bytes(&[waveform()]);
            bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("bad.EXT");
            std::fs::write(&path, bytes).unwrap();
            assert!(
                matches!(Anlz::load(path), Err(AnlzError::InvalidFile { .. })),
                "offset {offset}, value {value}"
            );
        }
    }

    #[test]
    fn unchanged_save_is_byte_exact_even_after_shared_reads() {
        let bytes = file_bytes(&[path_section("/audio.flac"), waveform(), cue(0), cue(1)]);
        let (_dir, mut file) = open(&bytes);
        file.get_waveform_detail().unwrap();
        file.get_memory_cues().unwrap();
        file.dump().unwrap();
        assert_eq!(std::fs::read(&file.path).unwrap(), bytes);
    }

    #[test]
    fn failed_save_preserves_original_and_edits_for_retry() {
        let bytes = file_bytes(&[path_section("/old"), waveform()]);
        let (dir, mut file) = open(&bytes);
        file.set_path("/new").unwrap();
        let destination = dir.path().join("existing.EXT");
        std::fs::write(&destination, b"keep me").unwrap();
        // A mutable section's public tag and content must agree.
        file.get_mut_section(0).unwrap().unwrap().tag = AnlzTag::BeatGrid;
        assert!(file.contains(AnlzTag::BeatGrid)); // queries follow edits, not stale metadata
        assert!(!file.contains(AnlzTag::Path));
        assert!(matches!(
            file.dump_copy(&destination),
            Err(AnlzError::TagContentMismatch {
                tag: AnlzTag::BeatGrid,
            })
        ));
        assert_eq!(std::fs::read(&destination).unwrap(), b"keep me");
        assert_eq!(std::fs::read(&file.path).unwrap(), bytes);
        assert!(file.sections[0].potentially_modified);
        file.get_mut_section(0).unwrap().unwrap().tag = AnlzTag::Path;
        file.dump().unwrap();
        assert_eq!(
            Anlz::load(&file.path)
                .unwrap()
                .get_path()
                .unwrap()
                .as_deref(),
            Some("/new")
        );
    }
}
