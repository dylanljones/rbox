// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Binary row layouts, bounded live-row loading, and standard/extended decoding.
//!
//! Original row windows remain separate from decoded fields. Windows can contain
//! inactive records and slack; they are not encoded lengths or writable capacity.
//! Unknown fields and string frames are preserved without assigning semantics.
//! These codecs do not perform row editing, allocation, or relationship loading.

use binrw::{binrw, BinRead, BinResult};
use std::io::{Cursor, Read, Seek, SeekFrom};

use super::core::{
    DatabaseKind, ExtPageType, PageIndex, PageType, RowDirectory, RowSlot, TableKind,
};
use super::string::DeviceSqlString;
use crate::{Error, Result};

/// Decoded contents in the explicitly selected database namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// A row in the standard database.
    Standard(StandardRow),
    /// A row in the extended database.
    Extended(ExtendedRow),
}

/// Implemented standard row families; unsupported tables remain opaque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandardRow {
    /// Track scalars and all 21 string frames.
    Track(TrackRow),
    /// Artist with a near/far name reference.
    Artist(ArtistRow),
    /// Album with a near/far name reference.
    Album(AlbumRow),
    /// Musical-key identifiers and name.
    Key(KeyRow),
    /// Color identifiers and display label.
    Color(ColorRow),
    /// Playlist or folder node.
    PlaylistTree(PlaylistTreeRow),
    /// Playlist track occurrence.
    PlaylistEntry(PlaylistEntryRow),
    /// History track occurrence.
    HistoryEntry(HistoryEntryRow),
    /// Browse menu/column label.
    BrowseColumn(BrowseColumnRow),
    /// Browse category configuration.
    BrowseCategory(BrowseCategoryRow),
    /// Browse sort configuration with raw tail.
    BrowseSort(BrowseSortRow),
    /// Export metadata and sequential string fields.
    ExportProperty(ExportPropertyRow),
    /// Genre ID and name.
    Genre(NamedRow),
    /// Label ID and name.
    Label(NamedRow),
    /// History-playlist ID and name.
    HistoryPlaylist(NamedRow),
    /// Artwork ID and image path.
    Artwork(NamedRow),
    /// Unknown or not-yet-implemented layout; bytes remain in [`RowRecord`].
    Opaque,
}

/// Extended row families. Typed extended codecs will be added incrementally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtendedRow {
    /// Native extended table-0 track pair, including uninterpreted surrounding fields.
    RecommendLike(RecommendLikeRow),
    /// My Tag or category, including original near/far offsets and reserved fields.
    MyTag(MyTagRow),
    /// Track/tag association.
    TrackMyTag(TrackMyTagRow),
    /// My Tag database identity and five reserved strings.
    DatabaseProperty(DatabasePropertyRow),
    /// Unknown or not-yet-implemented layout; bytes remain in [`RowRecord`].
    Opaque,
}

/// An immutable live-row window, its table namespace, and original location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowRecord {
    page: PageIndex,
    table: TableKind,
    slot: RowSlot,
    bytes: Vec<u8>,
}

impl RowRecord {
    /// Source page index.
    pub fn page(&self) -> PageIndex {
        self.page
    }

    /// Table code interpreted in the caller-supplied database namespace.
    pub fn table(&self) -> TableKind {
        self.table
    }

    /// Original page-local slot, heap offset, and presence/transaction state.
    pub fn slot(&self) -> RowSlot {
        self.slot
    }

    /// Complete original read window, including any bytes beyond encoded fields.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Decode supported fields without changing or discarding original bytes.
    ///
    /// Unknown tables/subtypes and not-yet-implemented tables yield `Opaque` in
    /// their namespace. Only documented subtypes are decoded; offset width is
    /// never guessed from an unknown subtype's bits.
    /// A malformed supported layout returns an error with page/table/slot context;
    /// it is never silently converted to `Opaque`. String text is not decoded here.
    pub fn decode(&self) -> Result<Row> {
        let context = |source| Error::RowDecode {
            page: self.page,
            table: self.table,
            slot: self.slot.slot,
            source,
        };
        let supported: Option<&[u16]> = match self.table {
            TableKind::Standard(PageType::Tracks) => Some(&[TrackHeader::SUBTYPE]),
            TableKind::Standard(PageType::Artists) => {
                Some(&[ArtistHeader::NEAR_SUBTYPE, ArtistHeader::FAR_SUBTYPE])
            }
            TableKind::Standard(PageType::Albums) => {
                Some(&[AlbumHeader::NEAR_SUBTYPE, AlbumHeader::FAR_SUBTYPE])
            }
            TableKind::Standard(PageType::ExportProperties) => Some(&[ExportPropertyRow::SUBTYPE]),
            TableKind::Extended(ExtPageType::MyTags) => {
                Some(&[MyTagHeader::NEAR_SUBTYPE, MyTagHeader::FAR_SUBTYPE])
            }
            TableKind::Extended(ExtPageType::DatabasePropertyOptions) => {
                Some(&[DatabasePropertyHeader::SUBTYPE])
            }
            _ => None,
        };
        if let Some(supported) = supported {
            let subtype = u16::read_le(&mut Cursor::new(self.as_bytes())).map_err(context)?;
            if !supported.contains(&subtype) {
                return Ok(match self.table {
                    TableKind::Standard(_) => Row::Standard(StandardRow::Opaque),
                    TableKind::Extended(_) => Row::Extended(ExtendedRow::Opaque),
                });
            }
        }
        decode_payload(self.table, self.as_bytes()).map_err(context)
    }
}

fn read_fields<T>(bytes: &[u8]) -> BinResult<T>
where
    T: for<'a> BinRead<Args<'a> = ()>,
{
    T::read_le(&mut Cursor::new(bytes))
}

fn decode_payload(table: TableKind, bytes: &[u8]) -> BinResult<Row> {
    Ok(match table {
        TableKind::Standard(kind) => Row::Standard(match kind {
            PageType::Tracks => StandardRow::Track(TrackRow::decode(bytes)?),
            PageType::Genres => StandardRow::Genre(read_fields(bytes)?),
            PageType::Artists => StandardRow::Artist(ArtistRow::decode(bytes)?),
            PageType::Albums => StandardRow::Album(AlbumRow::decode(bytes)?),
            PageType::Labels => StandardRow::Label(read_fields(bytes)?),
            PageType::MusicalKeys => StandardRow::Key(read_fields(bytes)?),
            PageType::ColorLabels => StandardRow::Color(read_fields(bytes)?),
            PageType::PlaylistTree => StandardRow::PlaylistTree(read_fields(bytes)?),
            PageType::PlaylistEntries => StandardRow::PlaylistEntry(read_fields(bytes)?),
            PageType::HistoryPlaylists => StandardRow::HistoryPlaylist(read_fields(bytes)?),
            PageType::HistoryEntries => StandardRow::HistoryEntry(read_fields(bytes)?),
            PageType::ArtworkReferences => StandardRow::Artwork(read_fields(bytes)?),
            PageType::BrowseColumns => StandardRow::BrowseColumn(read_fields(bytes)?),
            PageType::BrowseCategorySettings => StandardRow::BrowseCategory(read_fields(bytes)?),
            PageType::BrowseSortSettings => StandardRow::BrowseSort(read_fields(bytes)?),
            PageType::ExportProperties => StandardRow::ExportProperty(read_fields(bytes)?),
            PageType::Unknown(_) => StandardRow::Opaque,
        }),
        TableKind::Extended(kind) => Row::Extended(match kind {
            ExtPageType::RecommendLike => ExtendedRow::RecommendLike(read_fields(bytes)?),
            ExtPageType::MyTags => ExtendedRow::MyTag(MyTagRow::decode(bytes)?),
            ExtPageType::TrackMyTags => ExtendedRow::TrackMyTag(read_fields(bytes)?),
            ExtPageType::DatabasePropertyOptions => {
                ExtendedRow::DatabaseProperty(DatabasePropertyRow::decode(bytes)?)
            }
            _ => ExtendedRow::Opaque,
        }),
    })
}

/// Load one live slot's bounded window from the stream used to read its directory.
///
/// The directory retains its validated page location and bounds. The caller must
/// supply the same unchanged file and its explicit database kind. Only the selected
/// window is read; other pages, headers, and footers remain unread. The stream
/// position is not restored. Inactive or absent slots return `None` without I/O.
/// Loading does not decode fields; use [`RowRecord::decode`] separately.
///
/// ```no_run
/// use std::fs::File;
/// use binrw::BinRead;
/// use rbox_devlib::binary::core::DatabaseKind;
/// use rbox_devlib::binary::{FileHeader, read_table_chains, read_row_directory, read_row};
///
/// # fn main() -> rbox_devlib::Result<()> {
/// let mut input = File::open("export.pdb")?;
/// let header = FileHeader::read_le(&mut input)?;
/// for chain in read_table_chains(&mut input, &header)? {
///     for page in chain.pages.iter().filter(|page| !page.kind.is_index()) {
///         let directory = read_row_directory(&mut input, header.prefix.page_size, page)?;
///         for slot in directory.slots().filter(|slot| slot.live) {
///             if let Some(record) = read_row(&mut input, DatabaseKind::Standard, &directory, slot.slot)? {
///                 println!("{:?}: {:?}", record.table(), record.decode()?);
///             }
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn read_row<R: Read + Seek>(
    input: &mut R,
    database_kind: DatabaseKind,
    directory: &RowDirectory,
    slot: u16,
) -> Result<Option<RowRecord>> {
    let Some(window) = directory.live_row_window(slot) else {
        return Ok(None);
    };
    let Some(metadata) = directory.slot(slot) else {
        return Ok(None);
    };
    let table = database_kind.table_kind(directory.table_code);
    let io_error = |source| Error::RowRead {
        page: directory.page,
        table,
        slot,
        source,
    };
    input
        .seek(SeekFrom::Start(
            directory.heap_start + u64::from(window.start),
        ))
        .map_err(io_error)?;
    let mut bytes = vec![0; usize::from(window.end - window.start)];
    input.read_exact(&mut bytes).map_err(io_error)?;
    Ok(Some(RowRecord {
        page: directory.page,
        table,
        slot: metadata,
        bytes,
    }))
}

/// Four-byte prefix shared by the supported subtype-based row layouts.
/// This prefix is absent from inline ID/name and association rows.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowPrefix {
    /// Raw row subtype; interpretation belongs to the containing row codec.
    pub subtype: u16,
    /// Eleven slot bits above five preserved internal table bits.
    pub index_shift: u16,
}

impl RowPrefix {
    /// Largest page-local slot representable by this prefix.
    pub const MAX_SLOT: u16 = u16::MAX >> 5;

    /// Replace only the slot bits, preserving subtype and internal table bits.
    pub fn set_slot(&mut self, slot: u16) -> Result<()> {
        if slot > Self::MAX_SLOT {
            return Err(Error::MutationOverflow);
        }
        self.index_shift = (slot << 5) | (self.index_shift & 0x1f);
        Ok(())
    }
}

/// A preserved marker followed by `N` row-relative string offsets.
///
/// Reading imports `far`: false selects one-byte fields, true selects two-byte
/// little-endian fields. The containing row's established subtype selects the
/// width. Markers are retained without requiring the usual value of 3. Offset
/// order and aliases are preserved; the array itself does not resolve strings.
#[binrw]
#[brw(little)]
#[br(import(far: bool))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringOffsets<const N: usize> {
    /// One-byte marker and offsets.
    #[br(pre_assert(!far))]
    Near {
        /// Raw marker/reserved inline-string byte.
        marker: u8,
        /// Offsets in their original field order, in bytes from the row start.
        offsets: [u8; N],
    },
    /// Two-byte marker and offsets.
    #[br(pre_assert(far))]
    Far {
        /// Raw marker/reserved word.
        marker: u16,
        /// Offsets in their original field order, in bytes from the row start.
        offsets: [u16; N],
    },
}

impl<const N: usize> StringOffsets<N> {
    /// Serialized bytes for the marker and all offsets, excluding string frames.
    pub fn binary_size(&self) -> usize {
        (N + 1)
            * if self.is_far() {
                size_of::<u16>()
            } else {
                size_of::<u8>()
            }
    }

    /// Whether the array uses two-byte fields.
    pub fn is_far(&self) -> bool {
        matches!(self, Self::Far { .. })
    }

    /// An offset in bytes from the row start, retaining zero and other raw values.
    pub fn offset(&self, index: usize) -> Option<u16> {
        match self {
            Self::Near { offsets, .. } => offsets.get(index).copied().map(u16::from),
            Self::Far { offsets, .. } => offsets.get(index).copied(),
        }
    }

    /// Resolve every frame within a bounded row, retaining array order and aliases.
    pub(crate) fn resolve(
        &self,
        bytes: &[u8],
        data_start: usize,
    ) -> BinResult<[DeviceSqlString; N]> {
        let mut frames = Vec::with_capacity(N);
        match self {
            Self::Near { offsets, .. } => {
                for &offset in offsets {
                    frames.push(read_string_at(bytes, u16::from(offset), data_start)?);
                }
            }
            Self::Far { offsets, .. } => {
                for &offset in offsets {
                    frames.push(read_string_at(bytes, offset, data_start)?);
                }
            }
        }
        frames.try_into().map_err(|_| binrw::Error::AssertFail {
            pos: 0,
            message: "string-array length mismatch".into(),
        })
    }
}

/// Resolve within the supplied row window, excluding its prefix and offset array.
/// Zero is rejected rather than guessed to mean a missing string. Empty strings
/// must have an actual frame. Aliases and nonmonotonic offsets are permitted.
pub(crate) fn read_string_at(
    bytes: &[u8],
    offset: u16,
    data_start: usize,
) -> BinResult<DeviceSqlString> {
    let start = usize::from(offset);
    if start < data_start || start >= bytes.len() {
        return Err(binrw::Error::AssertFail {
            pos: u64::from(offset),
            message: "string offset is outside the row's string area".into(),
        });
    }
    let mut input = Cursor::new(bytes);
    input.set_position(u64::from(offset));
    DeviceSqlString::read_le(&mut input)
}

/// An ID followed immediately by one framed string, without a subtype header.
///
/// The containing table determines the ID namespace and whether `name` is a name
/// or artwork path. The codec consumes only these fields; subsequent window bytes
/// are not treated as padding or included in writes. Unknown string frames remain
/// lossless; text conversion is a separate operation.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedRow {
    /// Record ID within the containing table's namespace.
    pub id: u32,
    /// Original string frame; an image path for artwork-reference rows.
    pub name: DeviceSqlString,
}

/// The eight-byte artist prefix; subtype and slot information remain raw.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtistHeader {
    /// 0x0060 for near name offsets; 0x0064 for far offsets.
    pub subtype: u16,
    /// Raw page-local slot information (observed slot multiplied by 32).
    pub index_shift: u16,
    /// Artist ID.
    pub id: u32,
}

impl ArtistHeader {
    /// Fixed scalar prefix size in bytes, excluding the string-offset array.
    pub const BINARY_SIZE: usize = 8;
    /// Subtype using one-byte name offsets.
    pub const NEAR_SUBTYPE: u16 = 0x0060;
    /// Subtype using two-byte name offsets.
    pub const FAR_SUBTYPE: u16 = 0x0064;
}

/// The 20-byte album prefix, including both reserved words.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumHeader {
    /// 0x0080 for near name offsets; 0x0084 for far offsets.
    pub subtype: u16,
    /// Raw page-local slot information.
    pub index_shift: u16,
    /// Unknown/reserved word at offset 0x04.
    pub unknown_04: u32,
    /// Album artist ID; independent of a track's main artist.
    pub artist_id: u32,
    /// Album ID.
    pub id: u32,
    /// Unknown/reserved word at offset 0x10.
    pub unknown_10: u32,
}

impl AlbumHeader {
    /// Fixed scalar prefix size in bytes, excluding the string-offset array.
    pub const BINARY_SIZE: usize = 20;
    /// Subtype using one-byte name offsets.
    pub const NEAR_SUBTYPE: u16 = 0x0080;
    /// Subtype using two-byte name offsets.
    pub const FAR_SUBTYPE: u16 = 0x0084;
}

/// Musical-key IDs and inline name; subsequent window bytes are not consumed.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRow {
    /// Key ID.
    pub id: u32,
    /// Second stored ID; its role is unknown and it need not equal id.
    pub secondary_id: u32,
    /// Original key-name frame.
    pub name: DeviceSqlString,
}

/// Color identifier and inline label, preserving the independent reserved byte.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorRow {
    /// Unknown/reserved word.
    pub unknown_00: u32,
    /// Unknown byte at 0x04: zero in export1, duplicates the ID in the RB7 demo.
    pub unknown_04: u8,
    /// Color ID at unaligned offset 0x05, following Crate Digger's u16 layout.
    /// Rekordcrate treats only its low byte as the ID; the high byte is preserved.
    pub id: u16,
    /// Unknown/reserved byte.
    pub unknown_07: u8,
    /// Original display-name frame; labels may be customized.
    pub name: DeviceSqlString,
}

/// Decoded artist fields; original gaps and tail remain in RowRecord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtistRow {
    /// Original fixed prefix.
    pub header: ArtistHeader,
    /// Original marker and row-relative name offset.
    pub strings: StringOffsets<1>,
    /// Original name frame.
    pub name: DeviceSqlString,
}

impl ArtistRow {
    /// Decode a bounded row window using the documented near/far subtype.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Ok(Self::decode(bytes)?)
    }

    pub(crate) fn decode(bytes: &[u8]) -> BinResult<Self> {
        let mut input = Cursor::new(bytes);
        let header = ArtistHeader::read_le(&mut input)?;
        let far = match header.subtype {
            ArtistHeader::NEAR_SUBTYPE => false,
            ArtistHeader::FAR_SUBTYPE => true,
            _ => {
                return Err(binrw::Error::AssertFail {
                    pos: 0,
                    message: "unsupported artist subtype".into(),
                })
            }
        };
        let strings = StringOffsets::<1>::read_le_args(&mut input, (far,))?;
        let [name] = strings.resolve(bytes, input.position() as usize)?;
        Ok(Self {
            header,
            strings,
            name,
        })
    }
}

/// Decoded album fields; original gaps and tail remain in RowRecord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumRow {
    /// Original fixed prefix.
    pub header: AlbumHeader,
    /// Original marker and row-relative name offset.
    pub strings: StringOffsets<1>,
    /// Original name frame.
    pub name: DeviceSqlString,
}

impl AlbumRow {
    /// Decode a bounded row window using the documented near/far subtype.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Ok(Self::decode(bytes)?)
    }

    pub(crate) fn decode(bytes: &[u8]) -> BinResult<Self> {
        let mut input = Cursor::new(bytes);
        let header = AlbumHeader::read_le(&mut input)?;
        let far = match header.subtype {
            AlbumHeader::NEAR_SUBTYPE => false,
            AlbumHeader::FAR_SUBTYPE => true,
            _ => {
                return Err(binrw::Error::AssertFail {
                    pos: 0,
                    message: "unsupported album subtype".into(),
                })
            }
        };
        let strings = StringOffsets::<1>::read_le_args(&mut input, (far,))?;
        let [name] = strings.resolve(bytes, input.position() as usize)?;
        Ok(Self {
            header,
            strings,
            name,
        })
    }
}

/// The 92-byte track scalar prefix; unknown values and source/export IDs remain distinct.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackHeader {
    /// Established track subtype 0x0024 (far offsets).
    pub subtype: u16,
    /// Raw page-local slot information.
    pub index_shift: u16,
    /// CONTENTLINK; bit meanings and generation rules remain unknown.
    pub content_link: u32,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Composer artist ID.
    pub composer_id: u32,
    /// Audio file size in bytes.
    pub file_size: u32,
    /// Content identity in the source/master library, not the export ID.
    pub master_content_id: u32,
    /// Source/master library identity.
    pub master_db_id: u32,
    /// Artwork-reference ID.
    pub artwork_id: u32,
    /// Musical-key ID.
    pub key_id: u32,
    /// Original artist ID.
    pub original_artist_id: u32,
    /// Label ID.
    pub label_id: u32,
    /// Remixer artist ID.
    pub remixer_id: u32,
    /// Stored bitrate, normally in kb/s.
    pub bitrate: u32,
    /// Track number within the release.
    pub track_number: u32,
    /// Tempo in BPM multiplied by 100.
    pub tempo: u32,
    /// Genre ID.
    pub genre_id: u32,
    /// Album ID.
    pub album_id: u32,
    /// Main artist ID.
    pub artist_id: u32,
    /// Export track ID referenced by playlists and tag associations.
    pub id: u32,
    /// Disc number.
    pub disc_number: u16,
    /// Stored play count.
    pub play_count: u16,
    /// Release year.
    pub year: u16,
    /// Sample depth in bits per sample.
    pub sample_depth: u16,
    /// Track duration in seconds.
    pub duration: u16,
    /// Raw word at 0x56; analysis-bit interpretation is incomplete.
    pub raw_analysis: u16,
    /// Raw color ID; conventionally 0 or 1 through 8.
    pub color_id: u8,
    /// Raw star rating; values outside 0 through 5 are retained.
    pub rating: u8,
    /// Raw container/format code (e.g. 11 WAV); unknown codes are retained.
    pub audio_format: u16,
}

impl TrackHeader {
    /// Fixed scalar prefix size in bytes, excluding the string-offset array.
    pub const BINARY_SIZE: usize = 92;
    /// Established track subtype, using two-byte string offsets.
    pub const SUBTYPE: u16 = 0x0024;
}

/// All 21 track string frames in their documented offset-array order.
/// No text conversion, date parsing, path normalization, or ISRC guessing occurs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackStrings {
    /// Slot 0: original ISRC frame; use isrc_text() explicitly for the special encoding.
    pub isrc: DeviceSqlString,
    /// Slot 1: lyricist.
    pub lyricist: DeviceSqlString,
    /// Slot 2: unknown numeric string; no counter semantics are assumed.
    pub unknown_02: DeviceSqlString,
    /// Slot 3: unknown numeric string.
    pub unknown_03: DeviceSqlString,
    /// Slot 4: unknown string, which need not be empty.
    pub unknown_04: DeviceSqlString,
    /// Slot 5: track message, separate from comment.
    pub message: DeviceSqlString,
    /// Slot 6: publish track information, often ON or empty.
    pub publish_track_info: DeviceSqlString,
    /// Slot 7: hot-cue auto-load setting as original text.
    pub hot_cue_auto_load: DeviceSqlString,
    /// Slot 8: unknown string.
    pub unknown_08: DeviceSqlString,
    /// Slot 9: unknown string.
    pub unknown_09: DeviceSqlString,
    /// Slot 10: date added to the collection as original text.
    pub date_added: DeviceSqlString,
    /// Slot 11: release date as original text.
    pub release_date: DeviceSqlString,
    /// Slot 12: mix/remix name.
    pub mix_name: DeviceSqlString,
    /// Slot 13: unknown string.
    pub unknown_13: DeviceSqlString,
    /// Slot 14: analysis-file path, without separator normalization.
    pub analysis_path: DeviceSqlString,
    /// Slot 15: analysis date as original text.
    pub analysis_date: DeviceSqlString,
    /// Slot 16: DJ comment.
    pub comment: DeviceSqlString,
    /// Slot 17: track title.
    pub title: DeviceSqlString,
    /// Slot 18: unknown string.
    pub unknown_18: DeviceSqlString,
    /// Slot 19: audio basename.
    pub filename: DeviceSqlString,
    /// Slot 20: audio media path, without separator normalization.
    pub file_path: DeviceSqlString,
}

/// Decoded track fields. Original gaps, unused strings and tail stay in RowRecord.
/// This view has no row-rebuilding writer; only the prefix and offset-array codecs
/// serialize their own fixed fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackRow {
    /// Original 92-byte scalar prefix.
    pub header: TrackHeader,
    /// Original two-byte marker and 21 row-relative offsets.
    pub strings: StringOffsets<21>,
    /// All referenced string frames with their original encodings.
    pub text: TrackStrings,
}

impl TrackRow {
    /// Decode a bounded subtype-0x0024 track, resolving all 21 string frames.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Ok(Self::decode(bytes)?)
    }

    pub(crate) fn decode(bytes: &[u8]) -> BinResult<Self> {
        let mut input = Cursor::new(bytes);
        let header = TrackHeader::read_le(&mut input)?;
        if header.subtype != TrackHeader::SUBTYPE {
            return Err(binrw::Error::AssertFail {
                pos: 0,
                message: "unsupported track subtype".into(),
            });
        }
        let strings = StringOffsets::<21>::read_le_args(&mut input, (true,))?;
        let [isrc, lyricist, unknown_02, unknown_03, unknown_04, message, publish_track_info, hot_cue_auto_load, unknown_08, unknown_09, date_added, release_date, mix_name, unknown_13, analysis_path, analysis_date, comment, title, unknown_18, filename, file_path] =
            strings.resolve(bytes, input.position() as usize)?;
        let text = TrackStrings {
            isrc,
            lyricist,
            unknown_02,
            unknown_03,
            unknown_04,
            message,
            publish_track_info,
            hot_cue_auto_load,
            unknown_08,
            unknown_09,
            date_added,
            release_date,
            mix_name,
            unknown_13,
            analysis_path,
            analysis_date,
            comment,
            title,
            unknown_18,
            filename,
            file_path,
        };
        Ok(Self {
            header,
            strings,
            text,
        })
    }
}

/// Playlist/folder node with an inline name; no subtype header.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistTreeRow {
    /// Parent playlist/folder ID; zero for root.
    pub parent_id: u32,
    /// Unknown/reserved word at offset 4.
    pub unknown_04: u32,
    /// Sibling sort order under the parent; retained without renumbering.
    pub position: u32,
    /// Playlist/folder node ID.
    pub id: u32,
    /// Raw folder flag; conventionally zero for playlist and nonzero for folder.
    pub folder_flag: u32,
    /// Original node-name frame.
    pub name: DeviceSqlString,
}

/// A 12-byte playlist occurrence; repeated tracks and position gaps are retained.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistEntryRow {
    /// Entry position within the playlist; commonly one-based.
    pub position: u32,
    /// Export track ID, not the source/master content ID.
    pub track_id: u32,
    /// Playlist ID.
    pub playlist_id: u32,
}

/// A 12-byte history occurrence, with field order distinct from playlist entries.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntryRow {
    /// Export track ID.
    pub track_id: u32,
    /// History-playlist ID from standard table 11.
    pub playlist_id: u32,
    /// Stored position within the history playlist.
    pub position: u32,
}

/// Column/menu label with an inline string; Unicode framing characters are retained.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowseColumnRow {
    /// Column/menu-label ID.
    pub id: u16,
    /// Column code; unknown codes remain raw.
    pub code: u16,
    /// Original label frame, including any U+FFFA/U+FFFB characters.
    pub name: DeviceSqlString,
}

/// Eight-byte category settings; state values are not coerced to a boolean.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowseCategoryRow {
    /// Column/menu-label ID from standard table 16.
    pub column_id: u16,
    /// Content/menu selector, not a PDB table ID.
    pub selector: u16,
    /// Raw selector/attribute byte.
    pub unknown_04: u8,
    /// Raw visibility/state byte; values beyond 0 and 1 are not qualified.
    pub state: u8,
    /// Display order.
    pub position: u16,
}

/// Eight-byte sort settings; tail semantics are not shared with category settings.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowseSortRow {
    /// Column/menu-label ID.
    pub column_id: u16,
    /// Sort-item selector/index.
    pub selector: u16,
    /// All four tail bytes, without guessing flag/position widths.
    pub raw_tail: u32,
}

/// The 28-byte fixed prefix of an extended table-3 tag or category row.
///
/// All fields are preserved verbatim. The prefix codec alone does not validate a
/// subtype or follow strings; [`MyTagRow::parse`] supports 0x0680 and 0x0684.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyTagHeader {
    /// 0x0680 selects near offsets; 0x0684 selects far offsets.
    pub subtype: u16,
    /// Raw page-local slot information; observed as slot number multiplied by 32.
    pub index_shift: u16,
    /// Unknown/reserved word at row offset 0x04.
    pub unknown_04: u32,
    /// Unknown/reserved word at row offset 0x08.
    pub unknown_08: u32,
    /// Parent category ID; native category rows use zero.
    pub parent_id: u32,
    /// Zero-based position within the parent/category list.
    pub position: u32,
    /// Tag or category ID in the extended My Tags table.
    pub id: u32,
    /// Unknown/reserved word at row offset 0x18, separate from the attribute.
    pub unknown_18: u16,
    /// Unknown/reserved byte at row offset 0x1a.
    pub unknown_1a: u8,
    /// Raw attribute at row offset 0x1b: observed 0 for tag and 1 for category.
    pub attribute: u8,
}

impl MyTagHeader {
    /// Fixed scalar prefix size in bytes, excluding the string-offset array.
    pub const BINARY_SIZE: usize = 28;
    /// Subtype using one-byte string offsets.
    pub const NEAR_SUBTYPE: u16 = 0x0680;
    /// Subtype using two-byte string offsets.
    pub const FAR_SUBTYPE: u16 = 0x0684;

    /// Category status for established attribute values, or `None` if unknown.
    /// Nonzero reserved fields do not affect this interpretation.
    pub fn is_category(&self) -> Option<bool> {
        match self.attribute {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}

/// Decoded fields from an extended My Tag/category row.
///
/// This is a decoded view, not a sequential write layout: strings can have gaps,
/// aliases, and arbitrary order. The original complete window stays in
/// [`crate::binary::RowRecord`]. Prefix/array codecs can reproduce their own bytes, but
/// rebuilding or editing a row's string storage is not implemented here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyTagRow {
    /// Original fixed fields, including unknown values and the raw attribute.
    pub header: MyTagHeader,
    /// Original marker and offsets: name first, reserved string second.
    pub strings: StringOffsets<2>,
    /// Tag/category name as an unchanged string frame.
    pub name: DeviceSqlString,
    /// Second string frame, normally empty; meaning remains unknown.
    pub unknown_string: DeviceSqlString,
}

impl MyTagRow {
    /// Decode one bounded row window, with byte zero at the start of its prefix.
    ///
    /// Supports near (0x0680) and far (0x0684) subtypes. Both string offsets and
    /// frame lengths must fit the window, with offsets after the prefix/array.
    /// Zero offsets are rejected, not converted to empty strings. Original text
    /// encodings and malformed Unicode remain available for explicit conversion.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Ok(Self::decode(bytes)?)
    }

    pub(crate) fn decode(bytes: &[u8]) -> BinResult<Self> {
        let mut input = Cursor::new(bytes);
        let header = MyTagHeader::read_le(&mut input)?;
        let far = match header.subtype {
            MyTagHeader::NEAR_SUBTYPE => false,
            MyTagHeader::FAR_SUBTYPE => true,
            _ => {
                return Err(binrw::Error::AssertFail {
                    pos: 0,
                    message: "unsupported My Tag row subtype".into(),
                })
            }
        };
        let strings = StringOffsets::<2>::read_le_args(&mut input, (far,))?;
        let [name, unknown_string] = strings.resolve(bytes, input.position() as usize)?;
        Ok(Self {
            header,
            strings,
            name,
            unknown_string,
        })
    }
}

/// A 16-byte track/tag association, without a subtype header.
/// Trailing window bytes are not consumed. Nonstandard reserved values are kept.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackMyTagRow {
    /// Unknown/reserved word at offset 0.
    pub unknown_00: u32,
    /// Track ID in the standard export's track table.
    pub track_id: u32,
    /// Tag ID in the extended My Tags table.
    pub tag_id: u32,
    /// Raw byte at offset 12; normally 3, an empty short-string frame.
    pub reserved_string_marker: u8,
    /// Three original trailing reserved/alignment bytes.
    pub reserved_tail: [u8; 3],
}

/// A 24-byte RecommendLike row, without a subtype or page-local slot header.
///
/// The native reference contains fourteen rows. Pair direction, uniqueness and
/// creation rules are not established. The other fields remain raw; in particular
/// the first word pair is not interpreted as a timestamp
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendLikeRow {
    /// Unknown eight bytes at 0x00, retained as a little-endian word.
    pub unknown_00: u64,
    /// First export track ID, at 0x08; no source/target direction is inferred.
    pub track_id_1: u32,
    /// Second export track ID, at 0x0c.
    pub track_id_2: u32,
    /// Unknown word at 0x10, zero in the reference.
    pub unknown_10: u32,
    /// Unknown word at 0x14, retained without interpreting flags or padding.
    pub unknown_14: u32,
}

/// Standard subtype-0x0280 export properties with framed strings.
///
/// The two-byte separator is opaque; it is not interpreted as string offsets.
/// This codec consumes only documented fields, leaving trailing window bytes unread.
/// The raw subtype is checked by RowRecord dispatch, not by this field codec.
#[binrw]
#[brw(little)]
#[br(stream = reader)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPropertyRow {
    #[br(temp, try_calc = reader.stream_position())]
    #[bw(ignore)]
    row_start: u64,
    /// Established subtype 0x0280.
    pub subtype: u16,
    /// Raw page-local slot information.
    pub index_shift: u16,
    /// Number of contents/tracks after the export update.
    pub track_count: u32,
    /// Unknown/reserved byte.
    pub unknown_08: u8,
    /// Device Library background-color value, retained without normalization.
    pub background_color: u8,
    /// Unknown/reserved word.
    pub unknown_0a: u16,
    /// Export/creation date as original text.
    pub date: DeviceSqlString,
    /// Opaque bytes between date and version; commonly 0x1e19.
    pub separator: u16,
    /// Original version frame, commonly 1000.
    pub version: DeviceSqlString,
    /// Optional zero padding before a row-aligned UTF-16 device name.
    /// Baken writes this form; native non-ASCII device-name coverage is pending.
    /// Retained verbatim, not regenerated by the field writer.
    #[br(parse_with = read_name_padding, args(row_start))]
    pub name_padding: Vec<u8>,
    /// Original device/export-name frame.
    pub name: DeviceSqlString,
}

impl ExportPropertyRow {
    /// Scalar prefix size in bytes, before the date frame.
    pub const PREFIX_SIZE: usize = 12;
    /// Established export-property subtype.
    pub const SUBTYPE: u16 = 0x0280;
}

// Recognize only the documented writer's zero-to-four-byte-boundary + UTF-16
// pattern. Otherwise leave the stream untouched for ordinary frame decoding.
fn read_name_padding<R: Read + Seek>(
    reader: &mut R,
    _: binrw::Endian,
    (row_start,): (u64,),
) -> BinResult<Vec<u8>> {
    let start = reader.stream_position()?;
    let padding = ((4 - (start - row_start) % 4) % 4) as usize;
    if padding == 0 {
        return Ok(Vec::new());
    }
    let mut probe = [0; 4];
    let read = reader.read_exact(&mut probe[..padding + 1]);
    reader.seek(SeekFrom::Start(start))?;
    if read.is_ok() && probe[..padding].iter().all(|&b| b == 0) && probe[padding] == 0x90 {
        reader.seek(SeekFrom::Start(start + padding as u64))?;
        Ok(probe[..padding].to_vec())
    } else {
        if let Err(error) = read {
            if error.kind() != std::io::ErrorKind::UnexpectedEof {
                return Err(error.into());
            }
        }
        Ok(Vec::new())
    }
}
/// The 28-byte prefix of extended database property subtype 0x0700.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabasePropertyHeader {
    /// Established subtype 0x0700; a far variant is not established.
    pub subtype: u16,
    /// Raw page-local slot information.
    pub index_shift: u16,
    /// Five unknown/reserved words at offsets 0x04 through 0x14.
    pub unknown_words: [u32; 5],
    /// My Tag master database identity; not assumed equal to track master_db_id.
    pub my_tag_master_db_id: u32,
}

impl DatabasePropertyHeader {
    /// Fixed scalar prefix size in bytes, excluding the string-offset array.
    pub const BINARY_SIZE: usize = 28;
    /// Established subtype, using one-byte string offsets.
    pub const SUBTYPE: u16 = 0x0700;
}

/// Decoded extended database property fields; reserved string meanings are unknown.
/// Original gaps and tail remain in RowRecord; no row-rebuilding writer is provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabasePropertyRow {
    /// Original fixed prefix.
    pub header: DatabasePropertyHeader,
    /// Original one-byte marker and all five offsets.
    pub strings: StringOffsets<5>,
    /// Five original reserved string frames in offset-array order.
    pub unknown_strings: [DeviceSqlString; 5],
}

impl DatabasePropertyRow {
    /// Decode a bounded subtype-0x0700 row, following its five stored offsets.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Ok(Self::decode(bytes)?)
    }

    pub(crate) fn decode(bytes: &[u8]) -> BinResult<Self> {
        let mut input = Cursor::new(bytes);
        let header = DatabasePropertyHeader::read_le(&mut input)?;
        if header.subtype != DatabasePropertyHeader::SUBTYPE {
            return Err(binrw::Error::AssertFail {
                pos: 0,
                message: "unsupported database property subtype".into(),
            });
        }
        let strings = StringOffsets::<5>::read_le_args(&mut input, (false,))?;
        let unknown_strings = strings.resolve(bytes, input.position() as usize)?;
        Ok(Self {
            header,
            strings,
            unknown_strings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinWrite;

    #[test]
    fn row_slot_codec_preserves_subtype_low_bits_and_payload() {
        let mut bytes = [0x60, 0, 0xf7, 0x12, 0xaa];
        let mut prefix = RowPrefix::read_le(&mut Cursor::new(bytes)).unwrap();
        prefix.set_slot(17).unwrap();
        prefix.write_le(&mut Cursor::new(&mut bytes[..])).unwrap();
        assert_eq!(bytes, [0x60, 0, 0x37, 0x02, 0xaa]);
        prefix.set_slot(2047).unwrap();
        assert_eq!(prefix.index_shift, 0xfff7);
        let before = prefix;
        assert!(prefix.set_slot(2048).is_err());
        assert_eq!(prefix, before);
        assert!(RowPrefix::read_le(&mut Cursor::new([0; 3])).is_err());
    }

    #[test]
    fn offset_codecs_preserve_markers_and_match_declared_sizes() {
        for (offsets, expected) in [
            (
                StringOffsets::Near {
                    marker: 0x5a,
                    offsets: [0x12, 0x34],
                },
                vec![0x5a, 0x12, 0x34],
            ),
            (
                StringOffsets::Far {
                    marker: 0xabcd,
                    offsets: [0x1234, 0x5678],
                },
                vec![0xcd, 0xab, 0x34, 0x12, 0x78, 0x56],
            ),
        ] {
            let mut output = Cursor::new(Vec::new());
            offsets.write_le(&mut output).unwrap();
            assert_eq!(offsets.binary_size(), expected.len());
            assert_eq!(output.into_inner(), expected);
            assert_eq!(
                StringOffsets::<2>::read_le_args(&mut Cursor::new(expected), (offsets.is_far(),))
                    .unwrap(),
                offsets
            );
        }
    }
}
