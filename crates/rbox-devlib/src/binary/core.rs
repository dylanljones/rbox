// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared DeviceSQL constants and separate standard/extended page-type namespaces.

use binrw::{binrw, BinRead, BinResult, BinWrite};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::ops::Range;

use crate::{Error, Result};

/// Observed default page size in bytes; readers use the file's declared size.
pub const PAGE_SIZE: usize = 4096;
/// Full data-page header size in bytes, also the data heap's page-relative start.
pub const PAGE_HEADER_SIZE: usize =
    CommonPageHeader::BINARY_SIZE as usize + DataPageHeader::BINARY_SIZE as usize;
/// Sentinel for an absent page reference.
pub const NO_PAGE: u32 = 0x03ff_ffff;

/// Table numbering differs between the two database kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DatabaseKind {
    /// Standard DeviceSQL database, `export.pdb`.
    #[default]
    Standard,
    /// Extended DeviceSQL database, `exportExt.pdb`.
    Extended,
}

impl DatabaseKind {
    /// Interpret a raw table code using the appropriate page-type enum.
    ///
    /// Supply the kind explicitly: the file header cannot reliably distinguish it.
    ///
    /// ```
    /// use rbox_devlib::binary::core::{DatabaseKind, ExtPageType, PageType, TableKind};
    ///
    /// assert_eq!(DatabaseKind::Standard.table_kind(3),
    ///            TableKind::Standard(PageType::Albums));
    /// assert_eq!(DatabaseKind::Extended.table_kind(3),
    ///            TableKind::Extended(ExtPageType::MyTags));
    /// ```
    pub const fn table_kind(self, code: u32) -> TableKind {
        match self {
            Self::Standard => TableKind::Standard(PageType::from_code(code)),
            Self::Extended => TableKind::Extended(ExtPageType::from_code(code)),
        }
    }

    /// Return the known table name, or `None` for an unidentified code.
    pub const fn table_name(self, code: u32) -> Option<&'static str> {
        self.table_kind(code).name()
    }
}

/// Table/page codes in the standard `export.pdb` namespace.
///
/// This identifies the table's row family, not whether a page is a data or index
/// page. Binary reads and writes consume exactly one little-endian `u32`.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageType {
    /// Table 0: tracks.
    #[brw(magic = 0u32)]
    Tracks,
    /// Table 1: genres.
    #[brw(magic = 1u32)]
    Genres,
    /// Table 2: artists.
    #[brw(magic = 2u32)]
    Artists,
    /// Table 3: albums.
    #[brw(magic = 3u32)]
    Albums,
    /// Table 4: labels.
    #[brw(magic = 4u32)]
    Labels,
    /// Table 5: musical keys.
    #[brw(magic = 5u32)]
    MusicalKeys,
    /// Table 6: color labels.
    #[brw(magic = 6u32)]
    ColorLabels,
    /// Table 7: playlist tree.
    #[brw(magic = 7u32)]
    PlaylistTree,
    /// Table 8: playlist entries.
    #[brw(magic = 8u32)]
    PlaylistEntries,
    /// Table 11: history playlists.
    #[brw(magic = 11u32)]
    HistoryPlaylists,
    /// Table 12: history entries.
    #[brw(magic = 12u32)]
    HistoryEntries,
    /// Table 13: artwork references.
    #[brw(magic = 13u32)]
    ArtworkReferences,
    /// Table 16: browse columns.
    #[brw(magic = 16u32)]
    BrowseColumns,
    /// Table 17: browse category settings.
    #[brw(magic = 17u32)]
    BrowseCategorySettings,
    /// Table 18: browse sort settings.
    #[brw(magic = 18u32)]
    BrowseSortSettings,
    /// Table 19: export properties.
    #[brw(magic = 19u32)]
    ExportProperties,
    /// Unidentified table code, retaining all 32 bits.
    Unknown(u32),
}

impl PageType {
    /// Interpret a raw code in the standard namespace without losing unknown values.
    pub const fn from_code(code: u32) -> Self {
        match code {
            0 => Self::Tracks,
            1 => Self::Genres,
            2 => Self::Artists,
            3 => Self::Albums,
            4 => Self::Labels,
            5 => Self::MusicalKeys,
            6 => Self::ColorLabels,
            7 => Self::PlaylistTree,
            8 => Self::PlaylistEntries,
            11 => Self::HistoryPlaylists,
            12 => Self::HistoryEntries,
            13 => Self::ArtworkReferences,
            16 => Self::BrowseColumns,
            17 => Self::BrowseCategorySettings,
            18 => Self::BrowseSortSettings,
            19 => Self::ExportProperties,
            _ => Self::Unknown(code),
        }
    }

    /// The original four-byte wire code.
    pub const fn code(self) -> u32 {
        match self {
            Self::Tracks => 0,
            Self::Genres => 1,
            Self::Artists => 2,
            Self::Albums => 3,
            Self::Labels => 4,
            Self::MusicalKeys => 5,
            Self::ColorLabels => 6,
            Self::PlaylistTree => 7,
            Self::PlaylistEntries => 8,
            Self::HistoryPlaylists => 11,
            Self::HistoryEntries => 12,
            Self::ArtworkReferences => 13,
            Self::BrowseColumns => 16,
            Self::BrowseCategorySettings => 17,
            Self::BrowseSortSettings => 18,
            Self::ExportProperties => 19,
            Self::Unknown(code) => code,
        }
    }

    /// A known display name, or `None` for an unidentified code.
    pub const fn name(self) -> Option<&'static str> {
        match self {
            Self::Tracks => Some("Tracks"),
            Self::Genres => Some("Genres"),
            Self::Artists => Some("Artists"),
            Self::Albums => Some("Albums"),
            Self::Labels => Some("Labels"),
            Self::MusicalKeys => Some("Musical keys"),
            Self::ColorLabels => Some("Color labels"),
            Self::PlaylistTree => Some("Playlist tree"),
            Self::PlaylistEntries => Some("Playlist entries"),
            Self::HistoryPlaylists => Some("History playlists"),
            Self::HistoryEntries => Some("History entries"),
            Self::ArtworkReferences => Some("Artwork references"),
            Self::BrowseColumns => Some("Browse columns"),
            Self::BrowseCategorySettings => Some("Browse category settings"),
            Self::BrowseSortSettings => Some("Browse sort settings"),
            Self::ExportProperties => Some("Export properties"),
            Self::Unknown(_) => None,
        }
    }
}

/// Table/page codes in the extended `exportExt.pdb` namespace.
///
/// This identifies the table's row family, not whether a page is a data or index
/// page. Binary reads and writes consume exactly one little-endian `u32`.
/// Unknown codes are preserved.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtPageType {
    /// Table 0: recommend like.
    #[brw(magic = 0u32)]
    RecommendLike,
    /// Table 1: history.
    #[brw(magic = 1u32)]
    History,
    /// Table 2: song history.
    #[brw(magic = 2u32)]
    SongHistory,
    /// Table 3: my tags.
    #[brw(magic = 3u32)]
    MyTags,
    /// Table 4: track my tags.
    #[brw(magic = 4u32)]
    TrackMyTags,
    /// Table 5: cue options.
    #[brw(magic = 5u32)]
    CueOptions,
    /// Table 6: content options.
    #[brw(magic = 6u32)]
    ContentOptions,
    /// Table 7: database property options.
    #[brw(magic = 7u32)]
    DatabasePropertyOptions,
    /// Table 8: general options.
    #[brw(magic = 8u32)]
    GeneralOptions,
    /// Unidentified table code, retaining all 32 bits.
    Unknown(u32),
}

impl ExtPageType {
    /// Interpret a raw code in the extended namespace without losing unknown values.
    pub const fn from_code(code: u32) -> Self {
        match code {
            0 => Self::RecommendLike,
            1 => Self::History,
            2 => Self::SongHistory,
            3 => Self::MyTags,
            4 => Self::TrackMyTags,
            5 => Self::CueOptions,
            6 => Self::ContentOptions,
            7 => Self::DatabasePropertyOptions,
            8 => Self::GeneralOptions,
            _ => Self::Unknown(code),
        }
    }

    /// The original four-byte wire code.
    pub const fn code(self) -> u32 {
        match self {
            Self::RecommendLike => 0,
            Self::History => 1,
            Self::SongHistory => 2,
            Self::MyTags => 3,
            Self::TrackMyTags => 4,
            Self::CueOptions => 5,
            Self::ContentOptions => 6,
            Self::DatabasePropertyOptions => 7,
            Self::GeneralOptions => 8,
            Self::Unknown(code) => code,
        }
    }

    /// A known display name, or `None` for an unidentified code.
    pub const fn name(self) -> Option<&'static str> {
        match self {
            Self::RecommendLike => Some("Recommend like"),
            Self::History => Some("History"),
            Self::SongHistory => Some("Song history"),
            Self::MyTags => Some("My Tags"),
            Self::TrackMyTags => Some("Track My Tags"),
            Self::CueOptions => Some("Cue options"),
            Self::ContentOptions => Some("Content options"),
            Self::DatabasePropertyOptions => Some("Database property options"),
            Self::GeneralOptions => Some("General options"),
            Self::Unknown(_) => None,
        }
    }
}

/// A page type together with its database namespace.
///
/// Use [`PageType`] or [`ExtPageType`] directly when the namespace is fixed.
/// This wrapper lets inspection code handle both without conflating equal codes.
/// Raw header fields remain `u32`; resolve them with [`DatabaseKind::table_kind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableKind {
    /// A table in `export.pdb`, including unidentified standard codes.
    Standard(PageType),
    /// A table in `exportExt.pdb`, including unidentified extended codes.
    Extended(ExtPageType),
}

impl TableKind {
    /// The database namespace containing this table.
    pub const fn database_kind(self) -> DatabaseKind {
        match self {
            Self::Standard(_) => DatabaseKind::Standard,
            Self::Extended(_) => DatabaseKind::Extended,
        }
    }

    /// The original on-disk table code, without truncating unknown values.
    pub const fn code(self) -> u32 {
        match self {
            Self::Standard(page_type) => page_type.code(),
            Self::Extended(page_type) => page_type.code(),
        }
    }

    /// A known display name, or `None` when the table is unidentified.
    pub const fn name(self) -> Option<&'static str> {
        match self {
            Self::Standard(page_type) => page_type.name(),
            Self::Extended(page_type) => page_type.name(),
        }
    }
}

/// Raw physical page index. All 32 bits survive reading and writing.
///
/// Allocation candidates may point beyond EOF. [`crate::binary::read_table_chains`]
/// validates reachable chain pages against file length separately.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageIndex(pub u32);

impl PageIndex {
    /// No-page sentinel used by index-page references.
    pub const NONE: Self = Self(NO_PAGE);

    /// Whether this is the observed no-page sentinel.
    pub const fn is_none(self) -> bool {
        self.0 == Self::NONE.0
    }

    /// Byte offset from the start of the file, or `None` for a sentinel/zero size.
    ///
    /// This does not establish that the page exists. Multiplication uses `u64`
    /// and retains the full wire width instead of assuming a 26-bit address space.
    pub const fn file_offset(self, page_size: u32) -> Option<u64> {
        if self.is_none() || page_size == 0 {
            None
        } else {
            Some(self.0 as u64 * page_size as u64)
        }
    }
}

/// A 16-byte table-directory entry, with offsets relative to the entry.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TablePointer {
    /// `0x00`: table code in the standard or extended database namespace.
    pub table_type: u32,
    /// `0x04`: reserved future page, often the last page's successor; may exceed EOF.
    pub empty_candidate: PageIndex,
    /// `0x08`: first linked page, normally an index page even for an empty table.
    pub first_page: PageIndex,
    /// `0x0c`: inclusive last page. Traversal must stop here, not at a zero successor.
    pub last_page: PageIndex,
}

impl TablePointer {
    /// Serialized size in bytes.
    pub const BINARY_SIZE: u32 = 16;
}

/// The 28-byte prefix at the start of page zero.
///
/// This fixed-width model preserves arbitrary field values without allocating
/// from them. [`FileHeader`] validates lengths before reading variable data.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileHeaderPrefix {
    /// `0x00`: unknown signature/reserved word, observed zero; not enforced as magic.
    pub signature: u32,
    /// `0x04`: bytes per page, including headers; observed 4096.
    pub page_size: u32,
    /// `0x08`: number of table pointers following this prefix.
    pub table_count: u32,
    /// `0x0c`: allocator high-water reference; may point beyond physical EOF.
    pub next_unused_page: PageIndex,
    /// `0x10`: unknown compatibility/state word, observed 1 and 5.
    /// This is not a reliable Rekordbox major-version identifier.
    pub unknown_10: u32,
    /// `0x14`: database edit sequence. Its exact relation to page sequences is unresolved.
    pub sequence: u32,
    /// `0x18`: unknown/reserved word, observed zero.
    pub unknown_18: u32,
}

impl FileHeaderPrefix {
    /// Serialized size in bytes, excluding the table directory and slack.
    pub const BINARY_SIZE: u32 = 28;

    fn directory_fits(&self) -> bool {
        u64::from(Self::BINARY_SIZE)
            + u64::from(self.table_count) * u64::from(TablePointer::BINARY_SIZE)
            <= u64::from(self.page_size)
    }

    fn trailing_size(&self) -> usize {
        // Called only after the page size and directory extent have been checked.
        (self.page_size - Self::BINARY_SIZE - self.table_count * TablePointer::BINARY_SIZE) as usize
    }
}

/// The complete first page: prefix, ordered table directory, and opaque slack.
///
/// Reading consumes exactly `prefix.page_size` bytes. Writing checks the stored
/// count and size against the vectors before emitting bytes; callers changing
/// the directory must update both the count and slack deliberately.
///
/// The 512..=65536 power-of-two size range is a codec resource policy, not a
/// claim of device support. Use [`FileHeaderPrefix`] to inspect other sizes.
/// Unknown words and nonzero slack are preserved rather than regenerated.
#[binrw]
#[brw(little)]
#[bw(assert(supported_page_size(prefix.page_size), "unsupported page size"))]
#[bw(assert(prefix.directory_fits(), "table directory exceeds page zero"))]
#[bw(assert(tables.len() as u64 == u64::from(prefix.table_count), "table count mismatch"))]
#[bw(assert(trailing.len() == prefix.trailing_size(), "header page length mismatch"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeader {
    /// Fixed file metadata at byte zero.
    #[br(assert(supported_page_size(prefix.page_size), "unsupported page size"))]
    #[br(assert(prefix.directory_fits(), "table directory exceeds page zero"))]
    pub prefix: FileHeaderPrefix,
    /// Table descriptors in their original order, beginning at `0x1c`.
    #[br(count = prefix.table_count)]
    pub tables: Vec<TablePointer>,
    /// Every remaining byte of page zero, including any nonzero unknown data.
    #[br(count = prefix.trailing_size())]
    pub trailing: Vec<u8>,
}

pub(crate) fn supported_page_size(size: u32) -> bool {
    (512..=65536).contains(&size) && size.is_power_of_two()
}

/// Three little-endian bytes containing 13 allocated-count and 11 live-count bits.
///
/// Raw decoding preserves all bit patterns, including index-page values. The
/// relationship between counts, page size, and row directories is not validated
/// at this header-only layer.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedRowCounts([u8; 3]);

impl PackedRowCounts {
    /// Largest allocated-slot count representable in the 13-bit field.
    pub const MAX_ALLOCATED: u16 = 0x1fff;
    /// Largest live-row count representable in the 11-bit field.
    pub const MAX_LIVE: u16 = 0x07ff;

    /// Pack counts without silently truncating values outside their bit widths.
    pub fn new(allocated: u16, live: u16) -> Result<Self> {
        if allocated > Self::MAX_ALLOCATED || live > Self::MAX_LIVE {
            return Err(Error::InvalidRowCounts { allocated, live });
        }
        let bytes = (u32::from(allocated) | (u32::from(live) << 13)).to_le_bytes();
        Ok(Self([bytes[0], bytes[1], bytes[2]]))
    }

    /// Retain the exact wire representation, without imposing data-page semantics.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 3]) -> Self {
        Self(bytes)
    }

    /// The exact three wire bytes.
    #[must_use]
    pub const fn into_bytes(self) -> [u8; 3] {
        self.0
    }

    /// Allocated row offsets, including inactive slots (bits 0..=12).
    #[must_use]
    pub const fn allocated(self) -> u16 {
        u16::from_le_bytes([self.0[0], self.0[1] & 0x1f])
    }

    /// Live rows (bits 13..=23); distinct from allocated offsets after deletion.
    #[must_use]
    pub const fn live(self) -> u16 {
        (self.0[1] as u16 >> 5) | ((self.0[2] as u16) << 3)
    }
}

/// All eight page flag bits, including unknown bits.
///
/// Bits 0, 1, 3, and 7 are normally clear; bits 2 and 5 normally set.
/// Observed bytes include `0x24`/`0x34` (data) and `0x64`/`0x44` (index).
/// These observations are not enforced as fixed values.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageFlags(pub u8);

impl PageFlags {
    /// Bit selecting the index-page suffix.
    pub const INDEX: u8 = 0x40;
    /// Transaction/deletion-related bit; it does not establish row liveness.
    pub const TRANSACTION: u8 = 0x10;

    /// Whether bit 6 selects the longer index-page header.
    #[must_use]
    pub const fn is_index(self) -> bool {
        self.0 & Self::INDEX != 0
    }

    /// Whether the deletion/transaction-related bit 4 is set.
    ///
    /// Rekordcrate calls this `contains_deleted`; other sources associate it
    /// with transaction state. It is not a reliable live/deleted-row predicate.
    #[must_use]
    pub const fn has_transaction_flag(self) -> bool {
        self.0 & Self::TRANSACTION != 0
    }

    /// Set or clear the transaction bit, preserving every other bit.
    pub const fn with_transaction_flag(self, set: bool) -> Self {
        Self(if set {
            self.0 | Self::TRANSACTION
        } else {
            self.0 & !Self::TRANSACTION
        })
    }
}

/// The 32-byte common prefix of every table page.
///
/// Field offsets below are relative to the page start. Reading does not follow
/// page links, validate ownership, or load the heap/directory.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommonPageHeader {
    /// `0x00`: unknown signature/reserved word, observed zero.
    pub signature: u32,
    /// `0x04`: physical self index; normally matches the page's file position.
    pub page_index: PageIndex,
    /// `0x08`: raw table code, interpreted in the containing file's namespace.
    pub table_type: u32,
    /// `0x0c`: successor or allocation candidate after the table's declared last page.
    pub next_page: PageIndex,
    /// `0x10`: page edit sequence/generation; no increment rule is imposed.
    pub sequence: u32,
    /// `0x14`: unknown/reserved word, commonly zero.
    pub unknown_14: u32,
    /// `0x18`: allocated and live row counts packed into three bytes.
    pub counts: PackedRowCounts,
    /// `0x1b`: page-kind selector and preserved state/unknown bits.
    pub flags: PageFlags,
    /// `0x1c`: free heap bytes on data pages; not a universal index-page size rule.
    pub free_size: u16,
    /// `0x1e`: data heap extent in bytes, including retired allocations and slack.
    pub used_size: u16,
}

impl CommonPageHeader {
    /// Serialized size in bytes, excluding the page-kind-specific suffix.
    pub const BINARY_SIZE: u32 = 32;
}

/// Eight-byte data-page suffix; follows [`CommonPageHeader`] at page offset `0x20`.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataPageHeader {
    /// `0x20`: count of slots touched by the recorded transaction, or `0x1fff`.
    /// This is not an alternative allocated/live row count.
    pub transaction_row_count: u16,
    /// `0x22`: first touched slot index, or `0x1fff`; not a large-page row count.
    pub transaction_row_index: u16,
    /// `0x24`: unknown/reserved, commonly zero on data pages.
    pub unknown_24: u16,
    /// `0x26`: unknown/reserved; zero is common, some history pages reportedly use 1.
    pub unknown_26: u16,
}

impl DataPageHeader {
    /// Serialized suffix size in bytes; the full data header is 40 bytes.
    pub const BINARY_SIZE: u32 = 8;
    /// Recorded-transaction sentinel; sources describe no/failed transaction.
    pub const NO_TRANSACTION: u16 = 0x1fff;
}

/// 28-byte index-page suffix, beginning at page offset `0x20`.
///
/// The entry array starts at `0x3c` and is intentionally not read here. Several
/// names remain tentative: in particular, capacity and reference fields reflect
/// observations, not a complete index allocation algorithm.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexPageHeader {
    /// `0x20`: transaction-like/unknown word, observed `0x1fff` or 1.
    pub unknown_20: u16,
    /// `0x22`: transaction-like/unknown word, observed `0x1fff` or 0.
    pub unknown_22: u16,
    /// `0x24`: apparent entry capacity, observed 1004 (`0x03ec`) for 4096-byte pages.
    /// That agrees with `(4096 - 60 - 20) / 4`; other profiles are not established.
    pub entry_capacity: u16,
    /// `0x26`: next entry position, measured in entries from `0x3c`, not bytes.
    /// Can differ from `entry_count`; retained without normalizing it.
    pub next_entry: u16,
    /// `0x28`: repeated self/reference index, matches the common self index in samples.
    pub page_index: PageIndex,
    /// `0x2c`: first/next data-page reference, or [`PageIndex::NONE`] when empty.
    /// Can differ from the common successor, which may be an allocation candidate.
    pub data_page: PageIndex,
    /// `0x30`: unknown reference/sentinel word, observed `0x03ffffff`.
    pub unknown_30: u32,
    /// `0x34`: unknown/reserved word, observed zero; some codecs combine it with `0x30`.
    pub unknown_34: u32,
    /// `0x38`: declared number of packed entries following this header.
    pub entry_count: u16,
    /// `0x3a`: first empty entry slot; may equal `entry_count`, lie below it, or be `0x1fff`.
    pub first_empty: u16,
}

impl IndexPageHeader {
    /// Serialized suffix size in bytes; the full index header is 60 bytes.
    pub const BINARY_SIZE: u32 = 28;
    /// Sentinel observed in the first-empty-slot field.
    pub const NO_EMPTY_SLOT: u16 = 0x1fff;
}

/// Page-kind-specific suffix selected using bit `0x40` of the common flags.
#[binrw]
#[brw(little)]
#[br(import(flags: PageFlags))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageHeaderKind {
    /// The heap starts immediately after this suffix at page offset `0x28`.
    #[br(pre_assert(!flags.is_index()))]
    Data(DataPageHeader),
    /// Index entries start immediately after this suffix at page offset `0x3c`.
    #[br(pre_assert(flags.is_index()))]
    Index(IndexPageHeader),
}

impl PageHeaderKind {
    /// Whether this is the index-page suffix.
    #[must_use]
    pub const fn is_index(&self) -> bool {
        matches!(self, Self::Index(_))
    }
}

/// A complete table-page header, consuming exactly 40 (data) or 60 (index) bytes.
///
/// Reading stops at the payload start, regardless of stream position, with no
/// seeks to rows or subsequent pages. Writing rejects mismatched flags/suffixes
/// before emitting any bytes, and never rewrites payload bytes or index filler.
/// Structural/header serialization alone does not implement a database edit:
/// row directories, allocator state, and transactions remain separate concerns.
#[binrw]
#[brw(little)]
#[bw(assert(common.flags.is_index() == kind.is_index(), "page flags and header kind disagree"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageHeader {
    /// The shared 32-byte prefix.
    pub common: CommonPageHeader,
    /// Data- or index-specific metadata at page offset `0x20`.
    #[br(args(common.flags))]
    pub kind: PageHeaderKind,
}

impl PageHeader {
    /// Serialized header length/payload start, in bytes relative to the page.
    #[must_use]
    pub const fn binary_size(&self) -> u32 {
        CommonPageHeader::BINARY_SIZE
            + match self.kind {
                PageHeaderKind::Data(_) => DataPageHeader::BINARY_SIZE,
                PageHeaderKind::Index(_) => IndexPageHeader::BINARY_SIZE,
            }
    }
}

/// One little-endian packed index entry, preserving every raw bit.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexEntry(pub u32);

impl IndexEntry {
    /// Serialized size in bytes.
    pub const BINARY_SIZE: u32 = 4;
    /// Observed empty-entry encoding (`f8 ff ff 1f` on disk).
    pub const EMPTY: Self = Self(0x1fff_fff8);

    /// Whether the raw word exactly matches the observed empty-entry encoding.
    /// Other flag combinations with a no-page reference are retained distinctly.
    pub const fn is_empty(self) -> bool {
        self.0 == Self::EMPTY.0
    }

    /// The full 29-bit reference (`raw >> 3`), including sentinel/unknown values.
    /// This does not establish that the referenced page exists or is in the table.
    pub const fn page_index(self) -> PageIndex {
        PageIndex(self.0 >> 3)
    }

    /// All three low flag bits. Their individual meanings remain unresolved.
    pub const fn flags(self) -> u8 {
        (self.0 & 7) as u8
    }
}

/// Read exactly the declared index entries from one page in a complete file stream.
///
/// Supply a header from this stream and the file page size. The page's self-index
/// locates it; entries begin at byte 60 relative to that page. Works for both
/// standard and extended databases. Only `entry_count * 4` bytes are read;
/// unused entry space and trailing bytes are left unread. Position is not restored.
///
/// Rejects data pages, invalid page sizes/indices, counts extending beyond the
/// page, and pages beyond EOF. Counts are bounded before allocation. The apparent
/// capacity, next-entry position, first-empty marker, and observed 20-byte tail
/// convention are not treated as universal layout constraints. Reference validity
/// and completeness relative to table chains are not assumed or enforced here.
///
/// ```no_run
/// use std::fs::File;
/// use binrw::BinRead;
/// use rbox_devlib::binary::{read_index_entries, read_table_chains, FileHeader};
///
/// # fn main() -> rbox_devlib::Result<()> {
/// let mut input = File::open("export.pdb")?;
/// let header = FileHeader::read_le(&mut input)?;
/// for chain in read_table_chains(&mut input, &header)? {
///     for page in chain.pages {
///         if page.kind.is_index() {
///             for entry in read_index_entries(&mut input, header.prefix.page_size, &page)? {
///                 println!("raw={:#010x}, page={:?}, flags={:#x}",
///                          entry.0, entry.page_index(), entry.flags());
///             }
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn read_index_entries<R: Read + Seek>(
    input: &mut R,
    page_size: u32,
    header: &PageHeader,
) -> Result<Vec<IndexEntry>> {
    let page = header.common.page_index;
    let invalid = |reason| Error::InvalidIndexEntries { page, reason };
    let PageHeaderKind::Index(index) = header.kind else {
        return Err(invalid("expected an index page"));
    };
    if !header.common.flags.is_index() {
        return Err(invalid("page flags disagree with index header"));
    }
    if !supported_page_size(page_size) || page.0 == 0 || page.is_none() {
        return Err(invalid("invalid page size or index"));
    }
    let entries_size = u32::from(index.entry_count) * IndexEntry::BINARY_SIZE;
    if header.binary_size() + entries_size > page_size {
        return Err(invalid("declared entries extend beyond page boundary"));
    }
    let page_start = u64::from(page.0) * u64::from(page_size);
    let io_error = |source| Error::IndexEntriesRead {
        page,
        source: binrw::Error::Io(source),
    };
    let file_size = input.seek(SeekFrom::End(0)).map_err(io_error)?;
    if page_start + u64::from(page_size) > file_size {
        return Err(invalid("page extends beyond EOF"));
    }
    input
        .seek(SeekFrom::Start(
            page_start + u64::from(header.binary_size()),
        ))
        .map_err(io_error)?;
    let mut entries = Vec::with_capacity(usize::from(index.entry_count));
    for _ in 0..index.entry_count {
        entries.push(
            IndexEntry::read_le(input)
                .map_err(|source| Error::IndexEntriesRead { page, source })?,
        );
    }
    Ok(entries)
}

/// One physical footer group, including all raw mask bits.
///
/// Reading takes the allocated slot count (1..=16). Offsets are stored in wire
/// order, from the highest logical slot to the lowest. Reading/writing consumes
/// `2 * offsets.len() + 4` bytes; partial groups have no implicit padding.
#[binrw]
#[brw(little)]
#[br(import(slot_count: u16))]
#[br(pre_assert((1..=16).contains(&slot_count), "invalid row group slot count"))]
#[bw(assert((1..=16).contains(&offsets.len()), "invalid row group slot count"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowGroup {
    /// Heap-relative offsets in reverse logical slot order, including inactive slots.
    #[br(count = slot_count)]
    pub offsets: Vec<u16>,
    /// Bit `i` marks logical slot `i` of this group as live.
    pub presence_flags: u16,
    /// Bit `i` marks a slot touched by the recorded transaction; independent of liveness.
    pub transaction_flags: u16,
}

impl RowGroup {
    /// Maximum slots described by one pair of 16-bit masks.
    pub const MAX_SLOTS: u16 = u16::BITS as u16;

    /// Serialized bytes for this group's offsets and two masks.
    pub fn binary_size(&self) -> usize {
        (self.offsets.len() + 2) * size_of::<u16>()
    }

    /// Footer size in bytes, including a partial final group without padding.
    pub fn footer_size(allocated: u16) -> usize {
        (usize::from(allocated) + 2 * usize::from(allocated.div_ceil(Self::MAX_SLOTS)))
            * size_of::<u16>()
    }

    /// Read raw groups from a bounded page's end, returning logical group order.
    /// Only the footer is read; masks and inactive offsets are not interpreted.
    /// Use [`read_row_directory`] for page/count/live-offset validation.
    pub fn read_footer(page: &[u8], allocated: u16) -> BinResult<Vec<Self>> {
        let start = page
            .len()
            .checked_sub(Self::footer_size(allocated))
            .ok_or_else(|| binrw::Error::AssertFail {
                pos: 0,
                message: "row footer exceeds page".into(),
            })?;
        let mut input = Cursor::new(&page[start..]);
        let mut groups = Vec::new();
        for index in (0..allocated.div_ceil(Self::MAX_SLOTS)).rev() {
            let count = (allocated - index * Self::MAX_SLOTS).min(Self::MAX_SLOTS);
            groups.push(Self::read_le_args(&mut input, (count,))?);
        }
        groups.reverse();
        Ok(groups)
    }

    /// Write logical groups in wire order at a bounded page's end.
    /// Validates group lengths and available space before changing any bytes.
    pub fn write_footer(page: &mut [u8], groups: &[Self]) -> BinResult<()> {
        let invalid = |message: &str| binrw::Error::AssertFail {
            pos: 0,
            message: message.into(),
        };
        let size = groups.iter().try_fold(0usize, |size, group| {
            if !(1..=usize::from(Self::MAX_SLOTS)).contains(&group.offsets.len()) {
                return Err(invalid("invalid row group slot count"));
            }
            size.checked_add(group.binary_size())
                .ok_or_else(|| invalid("row footer size overflow"))
        })?;
        if groups
            .iter()
            .rev()
            .skip(1)
            .any(|g| g.offsets.len() != usize::from(Self::MAX_SLOTS))
        {
            return Err(invalid("only the last logical row group may be partial"));
        }
        let start = page
            .len()
            .checked_sub(size)
            .ok_or_else(|| invalid("row footer exceeds page"))?;
        let mut output = Cursor::new(&mut page[start..]);
        for group in groups.iter().rev() {
            group.write_le(&mut output)?;
        }
        Ok(())
    }
}

/// One allocated directory slot; its number is local to the data page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowSlot {
    /// Zero-based logical slot number within the page.
    pub slot: u16,
    /// Offset in bytes from the data heap start (page offset 40).
    /// Inactive offsets are retained without assuming they are valid addresses.
    pub heap_offset: u16,
    /// Whether the presence mask marks this row as live.
    pub live: bool,
    /// Whether the transaction mask marks this slot as touched.
    pub touched: bool,
}

/// A validated data-page footer without any loaded row payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowDirectory {
    pub(crate) page: PageIndex,
    pub(crate) table_code: u32,
    pub(crate) heap_start: u64,
    groups: Vec<RowGroup>,
    live_offsets: Vec<u16>,
    heap_size: u16,
}

impl RowDirectory {
    /// Groups in logical order: group zero is nearest the physical page end.
    /// To reproduce the footer bytes, write these groups in reverse order.
    pub fn groups(&self) -> &[RowGroup] {
        &self.groups
    }

    /// All allocated slots in logical order, including inactive slots.
    pub fn slots(&self) -> impl Iterator<Item = RowSlot> + '_ {
        self.groups
            .iter()
            .enumerate()
            .flat_map(|(group_index, group)| {
                group
                    .offsets
                    .iter()
                    .rev()
                    .enumerate()
                    .map(move |(index, &offset)| RowSlot {
                        slot: (group_index * usize::from(RowGroup::MAX_SLOTS) + index) as u16,
                        heap_offset: offset,
                        live: group.presence_flags & (1 << index) != 0,
                        touched: group.transaction_flags & (1 << index) != 0,
                    })
            })
    }

    /// Look up one allocated slot without reading its payload, including inactive slots.
    pub fn slot(&self, slot: u16) -> Option<RowSlot> {
        let group = self.groups.get(usize::from(slot / RowGroup::MAX_SLOTS))?;
        let index = usize::from(slot % RowGroup::MAX_SLOTS);
        if index >= group.offsets.len() {
            return None;
        }
        Some(RowSlot {
            slot,
            heap_offset: group.offsets[group.offsets.len() - 1 - index],
            live: group.presence_flags & (1 << index) != 0,
            touched: group.transaction_flags & (1 << index) != 0,
        })
    }

    /// A live slot's conservative heap-relative byte range, or `None` if inactive/absent.
    ///
    /// Ends at the next greater live offset or the declared heap end. May include
    /// inactive records and slack: this is neither an encoded row length nor
    /// writable allocation capacity. No row bytes are read by this method.
    pub fn live_row_window(&self, slot: u16) -> Option<Range<u16>> {
        let metadata = self.slot(slot)?;
        if !metadata.live {
            return None;
        }
        let start = metadata.heap_offset;
        let position = self.live_offsets.binary_search(&start).ok()?;
        let end = self
            .live_offsets
            .get(position + 1)
            .copied()
            .unwrap_or(self.heap_size);
        Some(start..end)
    }
}

/// Read a single data-page directory from a complete file stream.
///
/// Supply a header from this stream, normally obtained by `read_table_chains`,
/// and the file's page size. The stored page index supplies the absolute offset.
/// This shared format works for standard and extended databases.
/// Reads only footer bytes; headers, heap contents and free/slack bytes remain
/// unread. The stream position is not restored. Index pages are rejected.
///
/// Checks page/file bounds, heap/footer separation, allocated/live counts, and
/// live offsets (distinct and inside the heap). Inactive offsets and mask bits
/// beyond the allocated slots are preserved without assigning them semantics.
/// The header's `free_size` is not used to infer heap or record boundaries.
///
/// ```no_run
/// use std::fs::File;
/// use binrw::BinRead;
/// use rbox_devlib::binary::{read_row_directory, read_table_chains, FileHeader};
///
/// # fn main() -> rbox_devlib::Result<()> {
/// let mut input = File::open("exportExt.pdb")?;
/// let header = FileHeader::read_le(&mut input)?;
/// for chain in read_table_chains(&mut input, &header)? {
///     for page in chain.pages {
///         if !page.kind.is_index() {
///             let directory = read_row_directory(&mut input, header.prefix.page_size, &page)?;
///             for slot in directory.slots() {
///                 println!("slot {}: offset {}, live {}", slot.slot, slot.heap_offset, slot.live);
///             }
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn read_row_directory<R: Read + Seek>(
    input: &mut R,
    page_size: u32,
    header: &PageHeader,
) -> Result<RowDirectory> {
    let page = header.common.page_index;
    let invalid = |reason| Error::InvalidRowDirectory { page, reason };
    if header.kind.is_index() || header.common.flags.is_index() {
        return Err(invalid("expected a data page"));
    }
    if !supported_page_size(page_size) || page.0 == 0 || page.is_none() {
        return Err(invalid("invalid page size or index"));
    }
    let allocated = header.common.counts.allocated();
    let live = header.common.counts.live();
    if live > allocated {
        return Err(invalid("live count exceeds allocated count"));
    }
    let directory_size = RowGroup::footer_size(allocated) as u32;
    let heap_end = header.binary_size() + u32::from(header.common.used_size);
    if directory_size + heap_end > page_size {
        return Err(invalid("heap and directory overlap"));
    }
    let page_end = (u64::from(page.0) + 1) * u64::from(page_size);
    let io_error = |source| Error::RowDirectoryRead { page, source };
    let file_size = input.seek(SeekFrom::End(0)).map_err(io_error)?;
    if page_end > file_size {
        return Err(invalid("page extends beyond EOF"));
    }
    input
        .seek(SeekFrom::Start(page_end - u64::from(directory_size)))
        .map_err(io_error)?;
    let mut bytes = vec![0; directory_size as usize];
    input.read_exact(&mut bytes).map_err(io_error)?;
    let groups = RowGroup::read_footer(&bytes, allocated)?;
    let mut directory = RowDirectory {
        page,
        table_code: header.common.table_type,
        heap_start: u64::from(page.0) * u64::from(page_size) + u64::from(header.binary_size()),
        groups,
        live_offsets: Vec::new(),
        heap_size: header.common.used_size,
    };
    let mut offsets: Vec<_> = directory
        .slots()
        .filter(|slot| slot.live)
        .map(|slot| slot.heap_offset)
        .collect();
    if offsets.len() != usize::from(live) {
        return Err(invalid("presence masks disagree with live count"));
    }
    if offsets.iter().any(|&offset| offset >= directory.heap_size) {
        return Err(invalid("live offset is outside the heap"));
    }
    offsets.sort_unstable();
    if offsets.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid("duplicate live offsets"));
    }
    directory.live_offsets = offsets;
    Ok(directory)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite, Endian};

    use super::*;

    // Independent wire bytes: two table descriptors, including an unknown type.
    // Nonzero unknown words/slack exercise preservation, not native defaults.
    fn header_bytes() -> Vec<u8> {
        let mut bytes = vec![
            0x78, 0x56, 0x34, 0x12, 0x00, 0x02, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x36, 0x00,
            0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x2f, 0x00, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde,
            0x03, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x31, 0x00,
            0x00, 0x00, 0xef, 0xbe, 0xad, 0xde, 0x04, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
        ];
        bytes.resize(512, 0xa5);
        bytes
    }

    #[test]
    fn header_preserves_every_byte_and_stops_at_page_boundary() {
        let bytes = header_bytes();
        let mut framed = vec![0xcc; 7];
        framed.extend_from_slice(&bytes);
        framed.extend_from_slice(&[0xdd; 16]);
        let mut input = Cursor::new(framed.clone());
        input.set_position(7);
        // On-disk endian is intrinsic, even when a caller requests big endian.
        let header = FileHeader::read_options(&mut input, Endian::Big, ()).unwrap();
        assert_eq!(input.position(), 519);
        assert_eq!(header.prefix.signature, 0x1234_5678);
        assert_eq!(header.prefix.next_unused_page, PageIndex(54));
        assert_eq!(header.prefix.unknown_10, 5);
        assert_eq!(header.prefix.sequence, 47);
        assert_eq!(header.prefix.unknown_18, 0xdead_beef);
        assert_eq!(header.tables.len(), 2);
        assert_eq!(header.tables[0].table_type, 3);
        assert_eq!(header.tables[0].empty_candidate, PageIndex(51));
        assert_eq!(header.tables[0].first_page, PageIndex(1));
        assert_eq!(header.tables[0].last_page, PageIndex(49));
        assert_eq!(header.tables[1].table_type, 0xdead_beef);
        assert_eq!(header.trailing, vec![0xa5; 452]);
        let mut output = Cursor::new(framed.clone());
        output.set_position(7);
        header.write_options(&mut output, Endian::Big, ()).unwrap();
        assert_eq!(output.position(), 519);
        assert_eq!(output.into_inner(), framed);
    }

    #[test]
    fn native_prefix_layout_is_28_bytes() {
        // Prefix from the native RB7 demo export.pdb fixture.
        let bytes = [
            0, 0, 0, 0, 0, 0x10, 0, 0, 20, 0, 0, 0, 54, 0, 0, 0, 5, 0, 0, 0, 47, 0, 0, 0, 0, 0, 0,
            0,
        ];
        let mut input = Cursor::new(bytes);
        let prefix = FileHeaderPrefix::read_le(&mut input).unwrap();
        assert_eq!(prefix.page_size, 4096);
        assert_eq!(prefix.table_count, 20);
        assert_eq!(input.position(), u64::from(FileHeaderPrefix::BINARY_SIZE));
        let mut output = Cursor::new(Vec::new());
        prefix.write_le(&mut output).unwrap();
        assert_eq!(output.into_inner(), bytes);
    }

    #[test]
    fn editing_fields_changes_only_their_wire_bytes() {
        let bytes = header_bytes();
        let mut header = FileHeader::read_le(&mut Cursor::new(&bytes)).unwrap();
        header.prefix.sequence = 0x1234_5678;
        header.tables[1].empty_candidate = PageIndex(0x1122_3344);
        let mut expected = bytes;
        expected[20..24].copy_from_slice(&[0x78, 0x56, 0x34, 0x12]);
        expected[48..52].copy_from_slice(&[0x44, 0x33, 0x22, 0x11]);
        let mut output = Cursor::new(Vec::new());
        header.write_le(&mut output).unwrap();
        assert_eq!(output.into_inner(), expected);
    }

    #[test]
    fn invalid_lengths_fail_before_variable_reads() {
        for size in [0u32, 27, 256, 513, 131072, u32::MAX] {
            let mut bytes = header_bytes();
            bytes[4..8].copy_from_slice(&size.to_le_bytes());
            let error = FileHeader::read_le(&mut Cursor::new(&bytes[..28])).unwrap_err();
            assert!(error.to_string().contains("unsupported page size"));
        }
        for count in [31u32, u32::MAX] {
            let mut bytes = header_bytes();
            bytes[8..12].copy_from_slice(&count.to_le_bytes());
            let error = FileHeader::read_le(&mut Cursor::new(&bytes[..28])).unwrap_err();
            assert!(error.to_string().contains("table directory exceeds"));
        }
    }

    #[test]
    fn truncated_headers_fail() {
        let bytes = header_bytes();
        for end in 0..bytes.len() {
            assert!(
                FileHeader::read_le(&mut Cursor::new(&bytes[..end])).is_err(),
                "{end}"
            );
        }
    }

    #[test]
    fn inconsistent_writes_fail_without_emitting_bytes() {
        let original = FileHeader::read_le(&mut Cursor::new(header_bytes())).unwrap();
        for case in 0..5 {
            let mut header = original.clone();
            match case {
                0 => header.prefix.table_count = u32::MAX,
                1 => header.prefix.page_size = 0,
                2 => header.tables.pop().map(|_| ()).unwrap(),
                3 => header.trailing.push(0),
                _ => {
                    header.trailing.pop();
                }
            }
            let mut output = Cursor::new(Vec::new());
            assert!(header.write_le(&mut output).is_err());
            assert!(output.into_inner().is_empty());
        }
    }

    #[test]
    fn supported_size_boundaries_and_empty_directory() {
        for size in [512u32, 4096, 65536] {
            let mut bytes = vec![0; size as usize];
            bytes[4..8].copy_from_slice(&size.to_le_bytes());
            let header = FileHeader::read_le(&mut Cursor::new(&bytes)).unwrap();
            assert!(header.tables.is_empty());
            let mut output = Cursor::new(Vec::new());
            header.write_le(&mut output).unwrap();
            assert_eq!(output.into_inner(), bytes);
        }
    }

    #[test]
    fn page_offsets_preserve_wire_width_and_handle_sentinel() {
        assert_eq!(PageIndex(3).file_offset(4096), Some(12288));
        assert_eq!(PageIndex(0).file_offset(4096), Some(0));
        assert_eq!(PageIndex::NONE.file_offset(4096), None);
        assert_eq!(PageIndex(1).file_offset(0), None);
        assert_eq!(
            PageIndex(u32::MAX).file_offset(u32::MAX),
            Some(0xffff_fffe_0000_0001)
        );
        let mut output = Cursor::new(Vec::new());
        PageIndex::NONE.write_le(&mut output).unwrap();
        assert_eq!(output.into_inner(), [0xff, 0xff, 0xff, 0x03]);
    }

    // Independently specified wire layouts with distinct unknown words. Counts
    // 284/284 match the published RX/RK example; no rows or fixture files needed.
    const DATA: [u8; 40] = [
        0x78, 0x56, 0x34, 0x12, 2, 0, 0, 0, 19, 0, 0, 0, 54, 0, 0, 0, 47, 0, 0, 0, 0xef, 0xbe,
        0xad, 0xde, 0x1c, 0x81, 0x23, 0x34, 4, 0, 0xb8, 0x0d, 1, 0, 0x1b, 1, 0x34, 0x12, 0x78,
        0x56,
    ];
    const INDEX: [u8; 60] = [
        0, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 2, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x64, 0,
        0, 0, 0, 0xff, 0x1f, 0xff, 0x1f, 0xec, 3, 4, 0, 1, 0, 0, 0, 0xff, 0xff, 0xff, 3, 0xff,
        0xff, 0xff, 3, 0, 0, 0, 0, 2, 0, 0xff, 0x1f,
    ];

    #[test]
    fn data_header_decodes_full_width_counts_and_transaction_fields() {
        let header = PageHeader::read_le(&mut Cursor::new(DATA)).unwrap();
        assert_eq!(header.common.signature, 0x1234_5678);
        assert_eq!(header.common.page_index, PageIndex(2));
        assert_eq!(header.common.table_type, 19);
        assert_eq!(header.common.next_page, PageIndex(54));
        assert_eq!(header.common.sequence, 47);
        assert_eq!(header.common.unknown_14, 0xdead_beef);
        assert_eq!(header.common.counts.allocated(), 284);
        assert_eq!(header.common.counts.live(), 284);
        assert_eq!(header.common.free_size, 4);
        assert_eq!(header.common.used_size, 3512);
        assert!(header.common.flags.has_transaction_flag());
        let PageHeaderKind::Data(data) = header.kind else {
            panic!("expected data");
        };
        assert_eq!(data.transaction_row_count, 1);
        assert_eq!(data.transaction_row_index, 283);
        assert_eq!(data.unknown_24, 0x1234);
        assert_eq!(data.unknown_26, 0x5678);
    }

    #[test]
    fn index_header_keeps_independent_reference_and_entry_fields() {
        let header = PageHeader::read_le(&mut Cursor::new(INDEX)).unwrap();
        let PageHeaderKind::Index(index) = header.kind else {
            panic!("expected index");
        };
        assert_eq!(index.unknown_20, 0x1fff);
        assert_eq!(index.unknown_22, 0x1fff);
        assert_eq!(index.entry_capacity, 1004);
        assert_eq!(index.next_entry, 4);
        assert_eq!(index.page_index, PageIndex(1));
        assert_eq!(index.data_page, PageIndex::NONE);
        assert_eq!(header.common.next_page, PageIndex(2));
        assert_eq!(index.unknown_30, 0x03ff_ffff);
        assert_eq!(index.unknown_34, 0);
        assert_eq!(index.entry_count, 2);
        assert_eq!(index.first_empty, 0x1fff);
    }

    #[test]
    fn headers_roundtrip_without_reading_or_overwriting_payload() {
        for bytes in [&DATA[..], &INDEX[..]] {
            let mut framed = vec![0xee; 9];
            framed.extend_from_slice(bytes);
            // Any interpretation of these bytes as rows/index entries is out of scope.
            framed.extend_from_slice(&[0xa5; 80]);
            let mut input = Cursor::new(framed.clone());
            input.set_position(9);
            let header = PageHeader::read_options(&mut input, Endian::Big, ()).unwrap();
            assert_eq!(input.position(), 9 + bytes.len() as u64);
            assert_eq!(header.binary_size() as usize, bytes.len());
            let mut output = Cursor::new(framed.clone());
            output.set_position(9);
            header.write_options(&mut output, Endian::Big, ()).unwrap();
            assert_eq!(output.position(), input.position());
            assert_eq!(output.into_inner(), framed);
        }
    }

    #[test]
    fn all_flag_patterns_select_only_by_index_bit_and_roundtrip() {
        for flags in 0..=u8::MAX {
            let mut bytes = if flags & 0x40 == 0 {
                DATA.to_vec()
            } else {
                INDEX.to_vec()
            };
            bytes[27] = flags;
            let header = PageHeader::read_le(&mut Cursor::new(&bytes)).unwrap();
            assert_eq!(header.common.flags.0, flags);
            assert_eq!(header.kind.is_index(), flags & 0x40 != 0);
            let mut output = Cursor::new(Vec::new());
            header.write_le(&mut output).unwrap();
            assert_eq!(output.into_inner(), bytes);
        }
    }

    #[test]
    fn editing_headers_preserves_other_fields_and_payload_bytes() {
        for bytes in [&DATA[..], &INDEX[..]] {
            let mut frame = bytes.to_vec();
            frame.extend_from_slice(&[0xa5; 80]);
            let mut header = PageHeader::read_le(&mut Cursor::new(&frame)).unwrap();
            header.common.sequence = 0x1234_5678;
            match &mut header.kind {
                PageHeaderKind::Data(data) => data.transaction_row_index = 0x1122,
                PageHeaderKind::Index(index) => index.next_entry = 0x3344,
            }
            let mut expected = frame.clone();
            expected[16..20].copy_from_slice(&[0x78, 0x56, 0x34, 0x12]);
            if header.kind.is_index() {
                expected[38..40].copy_from_slice(&[0x44, 0x33]);
            } else {
                expected[34..36].copy_from_slice(&[0x22, 0x11]);
            }
            let mut output = Cursor::new(frame);
            header.write_le(&mut output).unwrap();
            assert_eq!(output.into_inner(), expected);
        }
    }

    #[test]
    fn unknown_index_words_and_data_sentinels_are_not_magic() {
        let mut index = INDEX;
        index[36..38].copy_from_slice(&[0x34, 0x12]);
        index[48..56].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let mut data = DATA;
        data[32..36].copy_from_slice(&[0xff, 0x1f, 0xff, 0x1f]);
        for bytes in [&index[..], &data[..]] {
            let header = PageHeader::read_le(&mut Cursor::new(bytes)).unwrap();
            let mut output = Cursor::new(Vec::new());
            header.write_le(&mut output).unwrap();
            assert_eq!(output.into_inner(), bytes);
        }
    }

    #[test]
    fn truncated_headers_do_not_fall_back_to_the_other_kind() {
        for bytes in [&DATA[..], &INDEX[..]] {
            for end in 0..bytes.len() {
                assert!(
                    PageHeader::read_le(&mut Cursor::new(&bytes[..end])).is_err(),
                    "{end}"
                );
            }
        }
    }

    #[test]
    fn mismatched_flags_fail_before_writing() {
        for bytes in [&DATA[..], &INDEX[..]] {
            let mut header = PageHeader::read_le(&mut Cursor::new(bytes)).unwrap();
            header.common.flags.0 ^= 0x40;
            let mut output = Cursor::new(Vec::new());
            let error: Error = header.write_le(&mut output).unwrap_err().into();
            assert!(matches!(error, Error::Binary(_)));
            assert!(output.into_inner().is_empty());
        }
    }

    #[test]
    fn counts_cover_each_bit_and_reject_overflow() {
        for bit in 0..24 {
            let raw = (1u32 << bit).to_le_bytes();
            let bytes = [raw[0], raw[1], raw[2]];
            let counts = PackedRowCounts::read_le(&mut Cursor::new(bytes)).unwrap();
            let expected_allocated = if bit < 13 { 1 << bit } else { 0 };
            let expected_live = if bit >= 13 { 1 << (bit - 13) } else { 0 };
            assert_eq!(counts.allocated(), expected_allocated);
            assert_eq!(counts.live(), expected_live);
            assert_eq!(
                PackedRowCounts::new(expected_allocated, expected_live)
                    .unwrap()
                    .into_bytes(),
                bytes
            );
        }
        assert_eq!(
            PackedRowCounts::new(284, 284).unwrap().into_bytes(),
            [0x1c, 0x81, 0x23]
        );
        assert_eq!(
            PackedRowCounts::new(3, 1).unwrap().into_bytes(),
            [3, 0x20, 0]
        );
        assert_eq!(
            PackedRowCounts::new(8191, 2047).unwrap().into_bytes(),
            [0xff; 3]
        );
        for (allocated, live) in [(8192, 0), (0, 2048), (u16::MAX, u16::MAX)] {
            assert!(matches!(
                PackedRowCounts::new(allocated, live),
                Err(Error::InvalidRowCounts { .. })
            ));
        }
    }

    #[test]
    fn footer_codec_preserves_wire_order_masks_and_heap() {
        let groups = vec![
            RowGroup {
                offsets: (0..16).rev().collect(),
                presence_flags: 0x8001,
                transaction_flags: 0x0020,
            },
            RowGroup {
                offsets: vec![0x1234],
                presence_flags: 0x0001,
                transaction_flags: 0x8001,
            },
        ];
        let expected = [
            0x34, 0x12, 1, 0, 1, 0x80, 15, 0, 14, 0, 13, 0, 12, 0, 11, 0, 10, 0, 9, 0, 8, 0, 7, 0,
            6, 0, 5, 0, 4, 0, 3, 0, 2, 0, 1, 0, 0, 0, 1, 0x80, 0x20, 0,
        ];
        let mut page = vec![0xa5; 64];
        RowGroup::write_footer(&mut page, &groups).unwrap();
        assert_eq!(RowGroup::footer_size(17), expected.len());
        assert_eq!(&page[..22], &[0xa5; 22]);
        assert_eq!(&page[22..], &expected);
        assert_eq!(RowGroup::read_footer(&page, 17).unwrap(), groups);
    }

    #[test]
    fn footer_codec_rejects_invalid_layout_without_writing() {
        let group = RowGroup {
            offsets: vec![0],
            presence_flags: 1,
            transaction_flags: 0,
        };
        let mut short = [0xa5; 5];
        assert!(RowGroup::write_footer(&mut short, std::slice::from_ref(&group)).is_err());
        assert_eq!(short, [0xa5; 5]);
        assert!(RowGroup::read_footer(&short, 1).is_err());
        let mut page = [0xa5; 64];
        assert!(RowGroup::write_footer(&mut page, &[group.clone(), group]).is_err());
        assert_eq!(page, [0xa5; 64]);
        RowGroup::write_footer(&mut page, &[]).unwrap();
        assert_eq!(page, [0xa5; 64]);
        assert!(RowGroup::read_footer(&page, 0).unwrap().is_empty());
    }
}
