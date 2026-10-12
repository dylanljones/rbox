// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Binary models for ANLZ file headers, sections, and section payloads.

#![allow(unused_parens)] // Suppress warnings coming from binrw macros

use binrw::io::{Read, Seek, SeekFrom, Write};
use binrw::{binrw, BinRead, BinResult, BinWrite, Endian, NullWideString};
use modular_bitfield::prelude::*;

use crate::error::{AnlzError, Result};
use crate::string::AnlzString;
use crate::xor::XorStream;

/// The tag of section.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub enum AnlzTag {
    /// File section that contains all other sections.
    #[brw(magic = b"PMAI")]
    File,
    /// All beats found in the track.
    #[brw(magic = b"PQTZ")]
    BeatGrid,
    /// All extended beats found in the track.
    #[brw(magic = b"PQT2")]
    ExtendedBeatGrid,
    /// Either memory points and loops or hotcues and hot loops of the track.
    #[brw(magic = b"PCOB")]
    CueList,
    /// Extended version of the `CueList` section (since Nexus 2 series).
    #[brw(magic = b"PCO2")]
    ExtendedCueList,
    /// Single cue entry inside a `ExtendedCueList` section.
    #[brw(magic = b"PCP2")]
    ExtendedCue,
    /// Single cue entry inside a `CueList` section.
    #[brw(magic = b"PCPT")]
    Cue,
    /// File path of the audio file.
    #[brw(magic = b"PPTH")]
    Path,
    /// Seek information for variable bitrate files.
    #[brw(magic = b"PVBR")]
    VBR,
    /// Frame seek index for FLAC tracks.
    #[brw(magic = b"PVB2")]
    SeekIndex,
    /// Fixed-width monochrome preview of the track waveform.
    #[brw(magic = b"PWAV")]
    WaveformPreview,
    /// Smaller version of the fixed-width monochrome preview of the track waveform (for the
    /// CDJ-900).
    #[brw(magic = b"PWV2")]
    TinyWaveformPreview,
    /// Variable-width large monochrome version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PWV3")]
    WaveformDetail,
    /// Fixed-width colored version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PWV4")]
    WaveformColorPreview,
    /// Variable-width large colored version of the track waveform.
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PWV5")]
    WaveformColorDetail,
    /// Fixed-width three-band preview of the track waveform.
    ///
    /// Used in `.2EX` files.
    #[brw(magic = b"PWV6")]
    Waveform3BandPreview,
    /// Variable-width three-band rendition of the track waveform.
    ///
    /// Used in `.2EX` files.
    #[brw(magic = b"PWV7")]
    Waveform3BandDetail,
    /// Calibration data accompanying the three-band waveforms.
    ///
    /// Used in `.2EX` files.
    #[brw(magic = b"PWVC")]
    Waveform3BandCalibration,
    /// Describes the structure of a song (intro, chorus, verse, etc.).
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PSSI")]
    SongStructure,
    /// Vocal-detection confidence data.
    #[brw(magic = b"PVDI")]
    VocalDetection,
    /// Unknown Kind.
    ///
    /// This allows handling files that contain unknown section types and allows to access later
    /// sections in the file that have a known type instead of failing to parse the whole file.
    Unknown([u8; 4]),
}

impl AnlzTag {
    /// Four-byte on-disk identifier, including unrecognized identifiers.
    pub fn code(&self) -> [u8; 4] {
        match self {
            Self::BeatGrid => *b"PQTZ",
            Self::ExtendedBeatGrid => *b"PQT2",
            Self::CueList => *b"PCOB",
            Self::ExtendedCueList => *b"PCO2",
            Self::Path => *b"PPTH",
            Self::VBR => *b"PVBR",
            Self::SeekIndex => *b"PVB2",
            Self::TinyWaveformPreview => *b"PWV2",
            Self::WaveformPreview => *b"PWAV",
            Self::WaveformDetail => *b"PWV3",
            Self::WaveformColorPreview => *b"PWV4",
            Self::WaveformColorDetail => *b"PWV5",
            Self::Waveform3BandPreview => *b"PWV6",
            Self::Waveform3BandDetail => *b"PWV7",
            Self::Waveform3BandCalibration => *b"PWVC",
            Self::SongStructure => *b"PSSI",
            Self::VocalDetection => *b"PVDI",
            Self::File => *b"PMAI",
            Self::Cue => *b"PCPT",
            Self::ExtendedCue => *b"PCP2",
            Self::Unknown(code) => *code,
        }
    }

    /// Resolve a four-byte identifier without losing unknown tags.
    pub fn from_code(code: [u8; 4]) -> Self {
        match &code {
            b"PQTZ" => Self::BeatGrid,
            b"PQT2" => Self::ExtendedBeatGrid,
            b"PCOB" => Self::CueList,
            b"PCO2" => Self::ExtendedCueList,
            b"PPTH" => Self::Path,
            b"PVBR" => Self::VBR,
            b"PVB2" => Self::SeekIndex,
            b"PWV2" => Self::TinyWaveformPreview,
            b"PWAV" => Self::WaveformPreview,
            b"PWV3" => Self::WaveformDetail,
            b"PWV4" => Self::WaveformColorPreview,
            b"PWV5" => Self::WaveformColorDetail,
            b"PWV6" => Self::Waveform3BandPreview,
            b"PWV7" => Self::Waveform3BandDetail,
            b"PWVC" => Self::Waveform3BandCalibration,
            b"PSSI" => Self::SongStructure,
            b"PVDI" => Self::VocalDetection,
            b"PMAI" => Self::File,
            b"PCPT" => Self::Cue,
            b"PCP2" => Self::ExtendedCue,
            _ => Self::Unknown(code),
        }
    }

    /// Returns the tag type as a string.
    pub fn to_string(&self) -> String {
        match self {
            AnlzTag::File => "File".to_string(),
            AnlzTag::BeatGrid => "BeatGrid".to_string(),
            AnlzTag::ExtendedBeatGrid => "ExtendedBeatGrid".to_string(),
            AnlzTag::CueList => "CueList".to_string(),
            AnlzTag::ExtendedCueList => "ExtendedCueList".to_string(),
            AnlzTag::ExtendedCue => "ExtendedCue".to_string(),
            AnlzTag::Cue => "Cue".to_string(),
            AnlzTag::Path => "Path".to_string(),
            AnlzTag::VBR => "VBR".to_string(),
            AnlzTag::SeekIndex => "SeekIndex".to_string(),
            AnlzTag::WaveformPreview => "WaveformPreview".to_string(),
            AnlzTag::TinyWaveformPreview => "TinyWaveformPreview".to_string(),
            AnlzTag::WaveformDetail => "WaveformDetail".to_string(),
            AnlzTag::WaveformColorPreview => "WaveformColorPreview".to_string(),
            AnlzTag::WaveformColorDetail => "WaveformColorDetail".to_string(),
            AnlzTag::Waveform3BandPreview => "Waveform3BandPreview".to_string(),
            AnlzTag::Waveform3BandDetail => "Waveform3BandDetail".to_string(),
            AnlzTag::Waveform3BandCalibration => "Waveform3BandCalibration".to_string(),
            AnlzTag::SongStructure => "SongStructure".to_string(),
            AnlzTag::VocalDetection => "VocalDetection".to_string(),
            _ => format!("Unknown({:?})", self),
        }
    }

    /// Accept a descriptive tag name or a four-byte ASCII identifier.
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "BeatGrid" => Ok(Self::BeatGrid),
            "ExtendedBeatGrid" => Ok(Self::ExtendedBeatGrid),
            "CueList" => Ok(Self::CueList),
            "ExtendedCueList" => Ok(Self::ExtendedCueList),
            "Path" => Ok(Self::Path),
            "VBR" => Ok(Self::VBR),
            "SeekIndex" => Ok(Self::SeekIndex),
            "TinyWaveformPreview" => Ok(Self::TinyWaveformPreview),
            "WaveformPreview" => Ok(Self::WaveformPreview),
            "WaveformDetail" => Ok(Self::WaveformDetail),
            "WaveformColorPreview" => Ok(Self::WaveformColorPreview),
            "WaveformColorDetail" => Ok(Self::WaveformColorDetail),
            "Waveform3BandPreview" => Ok(Self::Waveform3BandPreview),
            "Waveform3BandDetail" => Ok(Self::Waveform3BandDetail),
            "Waveform3BandCalibration" => Ok(Self::Waveform3BandCalibration),
            "SongStructure" => Ok(Self::SongStructure),
            "VocalDetection" => Ok(Self::VocalDetection),
            "File" => Ok(Self::File),
            "Cue" => Ok(Self::Cue),
            "ExtendedCue" => Ok(Self::ExtendedCue),
            _ if value.is_ascii() && value.len() == 4 => {
                let bytes = value.as_bytes();
                Ok(Self::from_code([bytes[0], bytes[1], bytes[2], bytes[3]]))
            }
            _ => Err(AnlzError::InvalidTag(value.to_owned())),
        }
    }
}

impl From<String> for AnlzTag {
    fn from(tag: String) -> Self {
        match tag.as_str() {
            "File" => AnlzTag::File,
            "BeatGrid" => AnlzTag::BeatGrid,
            "ExtendedBeatGrid" => AnlzTag::ExtendedBeatGrid,
            "CueList" => AnlzTag::CueList,
            "ExtendedCueList" => AnlzTag::ExtendedCueList,
            "ExtendedCue" => AnlzTag::ExtendedCue,
            "Cue" => AnlzTag::Cue,
            "Path" => AnlzTag::Path,
            "VBR" => AnlzTag::VBR,
            "SeekIndex" => AnlzTag::SeekIndex,
            "WaveformPreview" => AnlzTag::WaveformPreview,
            "TinyWaveformPreview" => AnlzTag::TinyWaveformPreview,
            "WaveformDetail" => AnlzTag::WaveformDetail,
            "WaveformColorPreview" => AnlzTag::WaveformColorPreview,
            "WaveformColorDetail" => AnlzTag::WaveformColorDetail,
            "Waveform3BandPreview" => AnlzTag::Waveform3BandPreview,
            "Waveform3BandDetail" => AnlzTag::Waveform3BandDetail,
            "Waveform3BandCalibration" => AnlzTag::Waveform3BandCalibration,
            "SongStructure" => AnlzTag::SongStructure,
            "VocalDetection" => AnlzTag::VocalDetection,
            _ => panic!("Unknown tag type: {}", tag),
        }
    }
}

/// Header struct
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub struct Header {
    /// Kind of content in this item.
    pub tag: AnlzTag,
    /// Length of the header data (including `kind`, `size` and `total_size`).
    pub size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
}

impl Header {
    fn remaining_size(&self) -> u32 {
        self.size - 12
    }

    fn content_size(&self) -> u32 {
        self.total_size - self.size
    }
}

trait SizedSection {
    fn size(&self) -> u32;
}

// -- Beat Grid Tags ------------------------------------------------------------------------------

/// Single beat grid entry inside a `BeatGrid` section.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub struct Beat {
    /// Beat number inside the bar (1-4).
    pub beat_number: u16,
    /// Current tempo in centi-BPM (= 1/100 BPM).
    pub tempo: u16,
    /// Time in milliseconds after which this beat would occur (at normal playback speed).
    pub time: u32,
}

/// Beat Grid Tag containing all beats in the track
///
/// Identifier: PQTZ
/// Used in `.DAT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BeatGrid {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Unknown field: Seems to always be `0`.
    #[br(temp)]
    #[br(assert(u1 == 0))]
    #[bw(calc = 0)]
    u1: u32,
    /// PQTZ flags and signed millisecond grid correction in the low 16 bits.
    /// Preserve the high bits even when their meaning is not known.
    pub header_flags: u32,
    /// Number of beats
    #[br(temp)]
    #[bw(calc = beats.len() as u32)]
    n_beats: u32,
    /// Beats in this beatgrid.
    #[br(count = n_beats)]
    pub beats: Vec<Beat>,
}

impl BeatGrid {
    /// Create a grid with observed zero-offset flags. Times are ms; tempo is centi-BPM.
    pub fn new(beats: Vec<Beat>) -> Self {
        Self {
            total_size: 0,
            header_flags: 0x0008_0000,
            beats,
        }
    }

    /// Signed millisecond correction applied independently of stored beat times.
    pub fn grid_offset(&self) -> i16 {
        self.header_flags as u16 as i16
    }

    /// Change the correction while preserving flags and original beat records.
    pub fn set_grid_offset(&mut self, offset_ms: i16) {
        self.header_flags = (self.header_flags & 0xffff0000) | u32::from(offset_ms as u16);
    }
}

impl SizedSection for BeatGrid {
    fn size(&self) -> u32 {
        24 + self.beats.len() as u32 * 8
    }
}

/// Sub-millisecond precision for the corresponding `PQTZ` beat.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub struct ExtBeat {
    /// Microseconds after the raw whole-millisecond timestamp (normally 0..=999).
    pub microseconds: u16,
}

/// Extended Beat Grid Tag (PQT2) containing all extended beats in the track
///
/// Identifier: PQT2
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtendedBeatGrid {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 56))]
    #[bw(calc = 56)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Padding: 4bytes.
    #[br(temp)]
    #[br(assert(pad1 == 0))]
    #[bw(calc = 0)]
    pad1: u32,
    /// Observed flags include `0x01000002`, `0x02000002`, and `0x03000002`.
    pub version: u32,
    /// Signed millisecond correction, applied once to raw beat times.
    pub offset_ms: i16,
    /// Uninterpreted flags following the saved offset; preserved verbatim.
    pub offset_flags: u16,
    /// First raw beat record from the corresponding `PQTZ` section.
    pub first_beat: Beat,
    /// Last raw beat record from the corresponding `PQTZ` section.
    pub last_beat: Beat,
    /// Number of beats
    #[br(temp)]
    #[bw(calc = beats.len() as u32)]
    n_beats: u32,
    /// Wrapping sum of beat number, centi-BPM and raw milliseconds over all PQTZ beats.
    /// Must be updated alongside changes to the corresponding PQTZ grid.
    pub checksum: u32,
    /// Unknow field: Seems to always be `0`
    #[br(temp)]
    #[br(assert(u3 == 0))]
    #[bw(calc = 0)]
    u3: u32,
    /// Unknow field: Seems to always be `0`
    #[br(temp)]
    #[br(assert(u4 == 0))]
    #[bw(calc = 0)]
    u4: u32,
    /// Beats in this beatgrid.
    #[br(count = n_beats)]
    pub beats: Vec<ExtBeat>,
}

impl ExtendedBeatGrid {
    /// Create an extended grid with zero offset and supplied raw endpoints and checksum.
    pub fn new(version: u32, endpoints: [Beat; 2], beats: Vec<ExtBeat>, checksum: u32) -> Self {
        let [first_beat, last_beat] = endpoints;
        Self {
            total_size: 0,
            version,
            offset_ms: 0,
            offset_flags: 0,
            first_beat,
            last_beat,
            beats,
            checksum,
        }
    }
}

impl SizedSection for ExtendedBeatGrid {
    fn size(&self) -> u32 {
        56 + self.beats.len() as u32 * 2
    }
}

// -- Cue List Tags -------------------------------------------------------------------------------

/// Indicates if the cue is point or a loop.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(repr = u8)]
pub enum CueType {
    /// Cue point.
    Point = 1,
    /// Loop.
    Loop = 2,
}

impl Into<u16> for CueType {
    fn into(self) -> u16 {
        match self {
            CueType::Point => 1,
            CueType::Loop => 2,
        }
    }
}

/// Describes the types of entries found in a Cue List section.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big, repr = u32)]
pub enum CueListType {
    /// Memory cues or loops.
    MemoryCues = 0,
    /// Hot cues or loops.
    HotCues = 1,
}

impl Into<u16> for CueListType {
    fn into(self) -> u16 {
        match self {
            CueListType::MemoryCues => 0,
            CueListType::HotCues => 1,
        }
    }
}

/// Indicates if the cue is point or a loop.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(repr = u32)]
pub enum CueStatus {
    /// Cue point diabled.
    Disabled = 0,
    /// Cue point enabled
    Active = 4,
}

impl Into<u16> for CueStatus {
    fn into(self) -> u16 {
        match self {
            CueStatus::Disabled => 0,
            CueStatus::Active => 4,
        }
    }
}

/// A memory or hot cue (or loop).
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub struct Cue {
    /// Cue entry header.
    #[br(assert(header.tag == AnlzTag::Cue && header.total_size == 56))]
    pub header: Header,
    /// Hot cue number.
    pub hot_cue: u32,
    /// Loop status. `4` if this cue is an active loop, `0` otherwise.
    pub status: CueStatus,
    /// Unknown field. Seems to always have the value `0x10000`.
    #[br(temp)]
    #[br(assert(u1 == 0x10000))]
    #[bw(calc = 0x10000)]
    u1: u32,
    /// Somehow used for sorting cues.
    /// 0xffff for first cue, 0,1,2 for next
    pub order_first: u16,
    /// Somehow used for sorting cues.
    /// 1,2,3 for first, second, third cue, 0xffff for last
    pub order_last: u16,
    /// Type of this cue
    pub cue_type: CueType,
    /// Unknown field. Seems always have the value `0`.
    #[br(temp)]
    #[br(assert(u2 == 0))]
    #[bw(calc = 0)]
    u2: u8,
    /// Unknown field. Seems always have the value `0x03E8` (= decimal 1000).
    #[br(temp)]
    #[br(assert(u3 == 1000))]
    #[bw(calc = 1000)]
    u3: u16,
    /// Time in milliseconds after which this cue would occur (at normal playback speed).
    pub time: u32,
    /// Time in milliseconds after which this the loop would jump back to `time` (at normal playback speed).
    pub loop_time: u32,
    /// Unknown field.
    pub u4: u32,
    /// Unknown field.
    pub u5: u32,
    /// Unknown field.
    pub u6: u32,
    /// Unknown field.
    pub u7: u32,
}

/// Indexed memory-cue colors. Unrecognized values are retained.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ColorIndex {
    #[brw(magic = 0u8)]
    None,
    #[brw(magic = 1u8)]
    Pink,
    #[brw(magic = 2u8)]
    Red,
    #[brw(magic = 3u8)]
    Orange,
    #[brw(magic = 4u8)]
    Yellow,
    #[brw(magic = 5u8)]
    Green,
    #[brw(magic = 6u8)]
    Aqua,
    #[brw(magic = 7u8)]
    Blue,
    #[brw(magic = 8u8)]
    Purple,
    Other(u8),
}

impl From<u8> for ColorIndex {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Pink,
            2 => Self::Red,
            3 => Self::Orange,
            4 => Self::Yellow,
            5 => Self::Green,
            6 => Self::Aqua,
            7 => Self::Blue,
            8 => Self::Purple,
            value => Self::Other(value),
        }
    }
}

impl From<ColorIndex> for u8 {
    fn from(value: ColorIndex) -> Self {
        match value {
            ColorIndex::None => 0,
            ColorIndex::Pink => 1,
            ColorIndex::Red => 2,
            ColorIndex::Orange => 3,
            ColorIndex::Yellow => 4,
            ColorIndex::Green => 5,
            ColorIndex::Aqua => 6,
            ColorIndex::Blue => 7,
            ColorIndex::Purple => 8,
            ColorIndex::Other(value) => value,
        }
    }
}

/// An extended memory or hot cue (or loop).
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub struct ExtendedCue {
    /// Cue entry header.
    #[br(assert(header.tag == AnlzTag::ExtendedCue && header.total_size >= 48))]
    #[bw(map = |header: &Header| { let mut h = header.clone(); h.total_size = self.size(); h })]
    pub header: Header,
    /// Hot cue number.
    pub hot_cue: u32,
    /// Type of this cue (`2` if this cue is a loop).
    pub cue_type: CueType,
    /// Unknown field. Seems always have the value `0`.
    #[br(temp)]
    #[br(assert(u1 == 0))]
    #[bw(calc = 0)]
    u1: u8,
    /// Unknown field. Seems always have the value `0x03E8` (= decimal 1000).
    #[br(temp)]
    #[br(assert(u2 == 1000))]
    #[bw(calc = 1000)]
    u2: u16,
    /// Time in milliseconds after which this cue would occur (at normal playback speed).
    pub time: u32,
    /// Time in milliseconds after which this the loop would jump back to `time` (at normal playback speed).
    pub loop_time: u32,
    /// Color assigned to this cue.
    ///
    /// Only used by memory cues, hot cues use a different value (see below).
    pub color: ColorIndex,
    /// Unknown field.
    pub u3: u8,
    /// Unknown field.
    pub u4: u16,
    /// Unknown field.
    pub u5: u32,
    /// Represents the loop size numerator (if this is a quantized loop).
    pub loop_numerator: u16,
    /// Represents the loop size denominator (if this is a quantized loop).
    pub loop_denominator: u16,
    /// A byte-length-prefixed UTF-16BE comment, retaining its original padding.
    #[br(args(header.total_size - 48), parse_with = AnlzString::read_bounded)]
    pub comment: AnlzString,
    /// Rekordbox hotcue color index.
    pub hot_cue_color_index: u8,
    /// Rekordbot hot cue color RGB value.
    ///
    /// This color is similar but not identical to the color that Rekordbox displays, and possibly
    /// used to illuminate the RGB LEDs in a player that has loaded the cue. If not color is
    /// associated with this hot cue, the value is `(0, 0, 0)`.
    pub hot_cue_color_rgb: (u8, u8, u8),
    /// Preserve the variable-length remainder of this cue entry.
    #[br(count = header.total_size - 48 - comment.payload_byte_len())]
    pub trailing_data: Vec<u8>,
}

impl ExtendedCue {
    fn size(&self) -> u32 {
        48 + self.comment.payload_byte_len() + self.trailing_data.len() as u32
    }
}

/// List of cue points or loops.
///
/// Identifier: PCOB
/// Used in `.DAT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CueList {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// The types of cues (memory or hot) that this list contains.
    pub list_type: CueListType,
    /// Unknown field: Seems to always be `0`.
    #[br(temp)]
    #[br(assert(u1 == 0))]
    #[bw(calc = 0)]
    u1: u16,
    /// Number of cues.
    #[br(temp)]
    #[bw(calc = cues.len() as u16)]
    len_cues: u16,
    /// Number of memory cues, or -1 when unspecified.
    pub memory_count: i32,
    /// Cues
    #[br(count = usize::from(len_cues))]
    pub cues: Vec<Cue>,
}

impl CueList {
    /// Create a cue list with an unspecified memory count (-1).
    pub fn new(list_type: CueListType, cues: Vec<Cue>) -> Self {
        Self {
            total_size: 0,
            list_type,
            memory_count: -1,
            cues,
        }
    }
}

impl SizedSection for CueList {
    fn size(&self) -> u32 {
        24 + self.cues.len() as u32 * 56
    }
}

/// List of cue points or loops PCO2
///
/// Identifier: PCO2
/// Used in `.DAT` files.
/// Variation of the original `CueList` that also adds support for more metadata such as
/// comments and colors. Introduces with the Nexus 2 series players.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtendedCueList {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 20))]
    #[bw(calc = 20)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// The types of cues (memory or hot) that this list contains.
    pub list_type: CueListType,
    /// Number of cues.
    #[br(temp)]
    #[bw(calc = cues.len() as u16)]
    len_cues: u16,
    /// Unknown field (apparently always `0`)
    #[br(temp)]
    #[br(assert(u1 == 0))]
    #[bw(calc = 0)]
    u1: u16,
    /// Cues
    #[br(count = usize::from(len_cues))]
    pub cues: Vec<ExtendedCue>,
}

impl ExtendedCueList {
    /// Create an extended cue list from supplied cues.
    pub fn new(list_type: CueListType, cues: Vec<ExtendedCue>) -> Self {
        Self {
            total_size: 0,
            list_type,
            cues,
        }
    }
}

impl SizedSection for ExtendedCueList {
    fn size(&self) -> u32 {
        20 + self.cues.iter().map(ExtendedCue::size).sum::<u32>()
    }
}

// -- Path Tag ------------------------------------------------------------------------------------

/// Path tag containing the file path that this analysis belongs to.
///
/// Identifier: PPTH
/// Used in `.DAT`, `.EXT` and `.2EX` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Path {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 16))]
    #[bw(calc = 16)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Length of the path field in bytes.
    #[br(temp)]
    #[br(assert(len_path == total_size - header_size))]
    #[bw(calc = ((path.len() as u32) + 1) * 2)]
    len_path: u32,
    /// Path of the audio file.
    #[br(assert(len_path == total_size - header_size))]
    #[br(assert((path.len() as u32 + 1) * 2 == len_path))]
    pub path: NullWideString,
}

impl SizedSection for Path {
    fn size(&self) -> u32 {
        16 + (self.path.len() as u32 + 1) * 2
    }
}

impl Path {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        let string = NullWideString::from("");
        let mut item = Self {
            total_size: 0,
            path: string,
        };
        item.set(path);
        item
    }

    pub fn set<P: AsRef<std::path::Path>>(&mut self, path: P) {
        let path = path.as_ref();
        let string = NullWideString::from(path.to_string_lossy().replace('\\', "/"));
        self.path = string.clone();
        self.total_size = ((string.len() as u32) + 1) * 2 + 16;
    }
}

// -- VBR Tag -------------------------------------------------------------------------------------

/// Variable bitrate seek index (PVBR), used in `.DAT` files.
///
/// The index contains exactly 400 big-endian values. The surrounding fields
/// are retained because their meaning is not fully understood.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub struct VBR {
    #[br(temp, assert(header_size == 16))]
    #[bw(calc = 16u32)]
    header_size: u32,
    #[br(temp, assert(total_size == 1620))]
    #[bw(calc = 1620u32)]
    total_size: u32,
    /// Unknown header value.
    pub u1: u32,
    /// Seek-index values in file order.
    pub index: [u32; 400],
    /// Unknown value following the index.
    pub u2: u32,
}

impl SizedSection for VBR {
    fn size(&self) -> u32 {
        1620
    }
}

/// A frame entry in a PVB2 seek index.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[brw(big)]
pub struct SeekPoint {
    /// Sample position at which the encoded audio frame starts.
    pub sample: u64,
    /// Byte offset of the encoded audio frame in the audio file.
    pub offset: u64,
    /// Number of samples in the frame.
    pub frame_samples: u32,
}

/// Frame seek index for FLAC tracks (PVB2).
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub struct SeekIndex {
    #[br(temp, assert(header_size == 32))]
    #[bw(calc = 32u32)]
    header_size: u32,
    #[br(temp, assert(total_size >= header_size))]
    #[bw(calc = 32u32 + entries.len() as u32 * 20)]
    total_size: u32,
    /// Unknown header value.
    pub u1: u32,
    /// Total number of audio samples.
    pub total_samples: u64,
    #[br(temp, assert(u64::from(entry_count) * 20 == u64::from(total_size - header_size)))]
    #[bw(calc = entries.len() as u32)]
    entry_count: u32,
    #[br(temp, assert(len_entry_bytes == 20))]
    #[bw(calc = 20u32)]
    len_entry_bytes: u32,
    #[br(count = entry_count)]
    pub entries: Vec<SeekPoint>,
}

impl SizedSection for SeekIndex {
    fn size(&self) -> u32 {
        32 + self.entries.len() as u32 * 20
    }
}

// -- Tiny Waveform Preview Tag -------------------------------------------------------------------

/// Single Column value in a Tiny Waveform Preview.
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(big, map = Self::from_bytes)]
#[bw(big, map = |x: &TinyWaveformColumn| x.into_bytes())]
pub struct TinyWaveformColumn {
    /// Height of the column in pixels (0..=15).
    pub height: B4,
    /// Unused upper nibble, preserved verbatim.
    pub unused: B4,
}

impl Default for TinyWaveformColumn {
    fn default() -> Self {
        Self::new()
    }
}

/// Smaller version of the fixed-width monochrome preview of the track waveform.
///
/// Identifier: PWV2
/// Used in `.DAT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TinyWaveformPreview {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 20))]
    #[bw(calc = 20)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Unknown field.
    #[br(temp)]
    #[br(assert(len_preview == total_size - header_size))]
    #[bw(calc = data.len() as u32)]
    len_preview: u32,
    /// Unknown field (apparently always `0x000100000`)
    #[br(temp)]
    #[br(assert(unknown == 0x00010000))]
    #[bw(calc = 0x00010000)]
    unknown: u32,
    /// Waveform preview column data.
    #[br(count = len_preview)]
    pub data: Vec<TinyWaveformColumn>,
}

impl TinyWaveformPreview {
    /// Create a section from supplied columns; lengths are calculated on write.
    pub fn new(data: Vec<TinyWaveformColumn>) -> Self {
        Self {
            total_size: 0,
            data,
        }
    }
}

impl SizedSection for TinyWaveformPreview {
    fn size(&self) -> u32 {
        20 + self.data.len() as u32
    }
}

// -- Waveform Tags -------------------------------------------------------------------------------

/// Single Column value in a Waveform Preview.
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(big, map = Self::from_bytes)]
#[bw(big, map = |x: &WaveformColumn| x.into_bytes())]
pub struct WaveformColumn {
    /// Height of the Column in pixels.
    pub height: B5,
    /// Shade of white.
    pub whiteness: B3,
}

/// Fixed-width monochrome preview of the track waveform.
///
/// Identifier: PWAV
/// Used in `.DAT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WaveformPreview {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 20))]
    #[bw(calc = 20)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Unknown field.
    #[br(temp)]
    #[br(assert(len_preview == total_size - header_size))]
    #[bw(calc = data.len() as u32)]
    len_preview: u32,
    /// Unknown field (apparently always `0x00010000`)
    #[br(temp)]
    #[br(assert(unknown == 0x00010000))]
    #[bw(calc = 0x00010000)]
    unknown: u32,
    /// Waveform preview column data.
    #[br(count = len_preview)]
    pub data: Vec<WaveformColumn>,
}

impl WaveformPreview {
    /// Create a section from supplied columns; lengths are calculated on write.
    pub fn new(data: Vec<WaveformColumn>) -> Self {
        Self {
            total_size: 0,
            data,
        }
    }
}

impl SizedSection for WaveformPreview {
    fn size(&self) -> u32 {
        20 + self.data.len() as u32
    }
}

/// Variable-width large monochrome version of the track waveform.
///
/// Identifier: PWV3
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WaveformDetail {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Size of a single entry, always 1.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 1))]
    #[bw(calc = 1u32)]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    #[br(assert((len_entry_bytes * len_entries) == total_size - header_size))]
    len_entries: u32,
    /// Unknown field (apparently always `0x00960000`)
    #[br(temp)]
    #[br(assert(unknown == 0x00960000))]
    #[bw(calc = 0x00960000)]
    unknown: u32,
    /// Waveform preview column data.
    ///
    /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
    /// so for each second of track audio there are 150 waveform detail entries.
    #[br(count = len_entries)]
    pub data: Vec<WaveformColumn>,
}

impl WaveformDetail {
    /// Create a section from supplied columns; lengths are calculated on write.
    pub fn new(data: Vec<WaveformColumn>) -> Self {
        Self {
            total_size: 0,
            data,
        }
    }
}

impl SizedSection for WaveformDetail {
    fn size(&self) -> u32 {
        24 + self.data.len() as u32
    }
}

// -- Waveform Color Tag ------------------------------------------------------------------

/// Single Column value in a Waveform Color Preview.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[brw(big)]
pub struct WaveformColorPreviewColumn {
    /// Unknown field (somehow encodes the "whiteness").
    pub u1: u8,
    /// Unknown field (somehow encodes the "whiteness").
    pub u2: u8,
    /// Sound energy in the bottom half of the frequency range (<10 KHz).
    pub energy_bottom_half_freq: u8,
    /// Sound energy in the bottom third of the frequency range.
    pub energy_bottom_third_freq: u8,
    /// Sound energy in the mid of the frequency range.
    pub energy_mid_third_freq: u8,
    /// Sound energy in the top of the frequency range.
    pub energy_top_third_freq: u8,
}

/// Variable-width large monochrome version of the track waveform.
///
/// Identifier: PWV4
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WaveformColorPreview {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Size of a single entry, always 6.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 6))]
    #[bw(calc = 6u32)]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    #[br(assert((len_entry_bytes * len_entries) == total_size - header_size))]
    len_entries: u32,
    /// Unknown field (apparently always `0`)
    #[br(temp)]
    #[br(assert(unknown == 0))]
    #[bw(calc = 0)]
    unknown: u32,
    /// Waveform preview column data.
    ///
    /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
    /// so for each second of track audio there are 150 waveform detail entries.
    #[br(count = len_entries)]
    pub data: Vec<WaveformColorPreviewColumn>,
}

impl WaveformColorPreview {
    /// Create a section from supplied columns; lengths are calculated on write.
    pub fn new(data: Vec<WaveformColorPreviewColumn>) -> Self {
        Self {
            total_size: 0,
            data,
        }
    }
}

impl SizedSection for WaveformColorPreview {
    fn size(&self) -> u32 {
        24 + self.data.len() as u32 * 6
    }
}

// -- Waveform Color Detail Tag -------------------------------------------------------------------

/// Single Column value in a Waveform Color Detail section.
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(big, map = |value: u16| Self::from_bytes(value.to_le_bytes()))]
#[bw(big, map = |x: &WaveformColorDetailColumn| u16::from_le_bytes(x.into_bytes()))]
pub struct WaveformColorDetailColumn {
    /// Fine-height bits in their original on-disk order.
    pub low_bits: B2,
    /// Height of the column.
    pub height: B5,
    /// Blue color component.
    pub blue: B3,
    /// Green color component.
    pub green: B3,
    /// Red color component.
    pub red: B3,
}

impl WaveformColorDetailColumn {
    /// Fine-height substep using the significance ordering reported by rekordcrate.
    ///
    /// The two low wire bits are reversed before combining with the coarse height.
    pub fn fine_height_substep(&self) -> u8 {
        let bits = self.low_bits();
        ((bits & 1) << 1) | ((bits >> 1) & 1)
    }

    /// Seven-bit height combining the coarse height and fine substep.
    pub fn full_height(&self) -> u8 {
        (self.height() << 2) | self.fine_height_substep()
    }
}

/// Variable-width large colored version of the track waveform.
///
/// Identifier: PWV5
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WaveformColorDetail {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Size of a single entry, always 2.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 2))]
    #[bw(calc = 2u32)]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    #[br(assert((len_entry_bytes * len_entries) == total_size - header_size))]
    len_entries: u32,
    /// Unknown field (apparently always `0x00960305`)
    pub unknown: u32,
    /// Waveform detail column data.
    #[br(count = len_entries)]
    pub data: Vec<WaveformColorDetailColumn>,
}

impl WaveformColorDetail {
    /// Create a section from supplied columns; lengths are calculated on write.
    pub fn new(data: Vec<WaveformColorDetailColumn>) -> Self {
        Self {
            total_size: 0,
            data,
            unknown: 0x00960305,
        }
    }
}

impl SizedSection for WaveformColorDetail {
    fn size(&self) -> u32 {
        24 + self.data.len() as u32 * 2
    }
}

// -- Waveform 3-Band Tags ------------------------------------------------------------------------

/// Single Column value in a Waveform 3-Band Preview section.
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(map = Self::from_bytes)]
#[bw(big, map = |x: &Waveform3BandColumn| x.into_bytes())]
pub struct Waveform3BandColumn {
    /// Low frequencies (dark blue), first byte on disk.
    pub low: B8,
    /// Mid-range component (amber), second byte on disk.
    pub mid: B8,
    /// High frequencies (white), third byte on disk.
    pub high: B8,
}

/// Fixed-width three-band preview of the track waveform.
///
/// Identifier: PWV6
/// Used in `.2EX` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Waveform3BandPreview {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 20))]
    #[bw(calc = 20)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Size of a single entry, always 3.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 3))]
    #[bw(calc = 3u32)]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    #[br(assert((len_entry_bytes * len_entries) == total_size - header_size))]
    len_entries: u32,
    /// Waveform detail column data.
    #[br(count = len_entries)]
    pub data: Vec<Waveform3BandColumn>,
}

impl Waveform3BandPreview {
    /// Create a section from supplied columns; lengths are calculated on write.
    pub fn new(data: Vec<Waveform3BandColumn>) -> Self {
        Self {
            total_size: 0,
            data,
        }
    }
}

impl SizedSection for Waveform3BandPreview {
    fn size(&self) -> u32 {
        20 + self.data.len() as u32 * 3
    }
}

/// Variable-width three-band rendition of the track waveform, used for CDJ-3000 and in Rekordbox.
///
/// Identifier: PWV7
/// Used in `.2EX` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Waveform3BandDetail {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Size of a single entry, always 2.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 3))]
    #[bw(calc = 3u32)]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    #[br(assert((len_entry_bytes * len_entries) == total_size - header_size))]
    len_entries: u32,
    /// Unknown field (apparently always `0x00960000`)
    pub unknown: u32,
    /// Waveform detail column data.
    #[br(count = len_entries)]
    pub data: Vec<Waveform3BandColumn>,
}

impl Waveform3BandDetail {
    /// Create a section from supplied columns; lengths are calculated on write.
    pub fn new(data: Vec<Waveform3BandColumn>) -> Self {
        Self {
            total_size: 0,
            data,
            unknown: 0x00960000,
        }
    }
}

impl SizedSection for Waveform3BandDetail {
    fn size(&self) -> u32 {
        24 + self.data.len() as u32 * 3
    }
}

// -- Waveform 3-Band Calibration Tag ----------------------------------------------------------------

/// Calibration data accompanying the three-band waveforms.
///
/// The three raw gain values apply to the low, mid, and high bands respectively.
/// The numeric scaling is not specified.
///
/// Identifier: PWVC
/// Used in `.2EX` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub struct Waveform3BandCalibration {
    #[br(temp, assert(header_size == 14))]
    #[bw(calc = 14u32)]
    header_size: u32,
    #[br(temp, assert(total_size == 20))]
    #[bw(calc = 20u32)]
    total_size: u32,
    /// Reserved header value, observed as zero.
    #[br(temp)]
    #[br(assert(unknown == 0))]
    #[bw(calc = 0)]
    unknown: u16,
    /// Gain of the low (blue) band.
    pub low_gain: u16,
    /// Gain of the mid (yellow) band.
    pub mid_gain: u16,
    /// Gain of the high (white) band.
    pub high_gain: u16,
}

impl SizedSection for Waveform3BandCalibration {
    fn size(&self) -> u32 {
        20
    }
}

// -- Song Structure Tag --------------------------------------------------------------------------

/// Music classification that is used for Lightnight mode and based on rhythm, tempo kick drum and
/// sound density.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[brw(big, repr = u16)]
pub enum Mood {
    /// Phrase types consist of "Intro", "Up", "Down", "Chorus", and "Outro". Other values in each
    /// phrase entry cause the intro, chorus, and outro phrases to have their labels subdivided
    /// into styles "1" or "2" (for example, "Intro 1"), and "up" is subdivided into style "Up 1",
    /// "Up 2", or "Up 3".
    High = 1,
    /// Phrase types are labeled "Intro", "Verse 1" through "Verse 6", "Chorus", "Bridge", and
    /// "Outro".
    Mid,
    /// Phrase types are labeled "Intro", "Verse 1", "Verse 2", "Chorus", "Bridge", and "Outro".
    /// There are three different phrase type values for each of "Verse 1" and "Verse 2", but
    /// rekordbox makes no distinction between them.
    Low,
}

impl Into<u16> for Mood {
    fn into(self) -> u16 {
        match self {
            Mood::High => 1,
            Mood::Mid => 2,
            Mood::Low => 3,
        }
    }
}

/// Stylistic track bank for Lightning mode.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[brw(repr = u8)]
pub enum Bank {
    /// Default bank variant, treated as `Cool`.
    Default = 0,
    /// "Cool" bank variant.
    Cool,
    /// "Natural" bank variant.
    Natural,
    /// "Hot" bank variant.
    Hot,
    /// "Subtle" bank variant.
    Subtle,
    /// "Warm" bank variant.
    Warm,
    /// "Vivid" bank variant.
    Vivid,
    /// "Club 1" bank variant.
    Club1,
    /// "Club 2" bank variant.
    Club2,
}

impl Into<u16> for Bank {
    fn into(self) -> u16 {
        match self {
            Bank::Default => 0,
            Bank::Cool => 1,
            Bank::Natural => 2,
            Bank::Hot => 3,
            Bank::Subtle => 4,
            Bank::Warm => 5,
            Bank::Vivid => 6,
            Bank::Club1 => 7,
            Bank::Club2 => 8,
        }
    }
}
/// A song structure entry that represents a phrase in the track.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[brw(big)]
pub struct Phrase {
    /// Phrase number (starting at 1).
    pub index: u16,
    /// Beat number where this phrase begins.
    pub beat: u16,
    /// Kind of phrase that rekordbox has identified (?).
    pub kind: u16,
    /// Unknown field.
    #[allow(dead_code)]
    pub u1: u8,
    /// Flag byte used for numbered variations (in case of the `High` mood).
    ///
    /// See the documentation for details:
    /// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#high-phrase-variants>
    pub k1: u8,
    /// Unknown field.
    #[allow(dead_code)]
    pub u2: u8,
    /// Flag byte used for numbered variations (in case of the `High` mood).
    ///
    /// See the documentation for details:
    /// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#high-phrase-variants>
    pub k2: u8,
    /// Unknown field.
    #[allow(dead_code)]
    pub u3: u8,
    /// Flag that determined if only `beat2` is used (0), or if `beat2`, `beat3` and `beat4` are
    /// used (1).
    pub b: u8,
    /// Beat number.
    pub beat2: u16,
    /// Beat number.
    pub beat3: u16,
    /// Beat number.
    pub beat4: u16,
    /// Unknown field.
    #[allow(dead_code)]
    pub u4: u8,
    /// Flag byte used for numbered variations (in case of the `High` mood).
    ///
    /// See the documentation for details:
    /// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#high-phrase-variants>
    pub k3: u8,
    /// Unknown field.
    #[allow(dead_code)]
    pub u5: u8,
    /// Indicates if there are fill (non-phrase) beats at the end of the phrase.
    pub fill: u8,
    /// Beat number where the fill begins (if `fill` is non-zero).
    pub beat_fill: u16,
}

/// The data part of the [`SongStructure`] section that may be encrypted (RB6+).
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[br(import(len_entries: u16))]
pub struct SongStructureData {
    /// Overall type of phrase structure.
    pub mood: Mood,
    /// Unknown field.
    pub u1: u32,
    /// Unknown field.
    pub u2: u16,
    /// Number of the beat at which the last recognized phrase ends.
    pub end_beat: u16,
    /// Unknown field.
    pub u3: u16,
    /// Stylistic bank assigned in Lightning Mode.
    pub bank: Bank,
    /// Unknown field.
    pub u4: u8,
    /// Phrase entry data.
    #[br(count = usize::from(len_entries))]
    pub phrases: Vec<Phrase>,
}

impl SongStructureData {
    /// The key data used for the XOR encryption of the [`SongStructureData`] section.
    /// Equivalent seed: each byte of ASCII "Kanzen-niRikaishita" plus 0x80.
    /// Exported version 1 tags start the mask at index ``version % len(XOR_MASK)``.
    const KEY_DATA: [u8; 19] = [
        0xCB, 0xE1, 0xEE, 0xFA, 0xE5, 0xEE, 0xAD, 0xEE, 0xE9, 0xD2, 0xE9, 0xEB, 0xE1, 0xE9, 0xF3,
        0xE8, 0xE9, 0xF4, 0xE1,
    ];

    /// The export mask starts at version % 19 and adds the entry count modulo 256.
    fn get_key(len_entries: u16, version: u16) -> impl Iterator<Item = u8> {
        Self::KEY_DATA
            .into_iter()
            .cycle()
            .skip(usize::from(version) % Self::KEY_DATA.len())
            .take(Self::KEY_DATA.len())
            .map(move |byte| byte.wrapping_add(len_entries as u8))
    }

    fn check_if_encrypted<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (len_entries, version): (u16, u16),
    ) -> BinResult<bool> {
        let start = reader.stream_position()?;
        // At counts such as 53 the masked mood can also be 1..=3. Check the
        // complete phrase structure instead of deciding from those two bytes.
        for encrypted in [false, true] {
            reader.seek(SeekFrom::Start(start))?;
            if let Ok(data) =
                Self::read_encrypted(reader, endian, (encrypted, len_entries, version))
            {
                let valid = data.phrases.iter().enumerate().all(|(i, phrase)| {
                    usize::from(phrase.index) == i + 1
                        && phrase.beat > 0
                        && phrase.beat <= data.end_beat
                        && (i == 0 || data.phrases[i - 1].beat < phrase.beat)
                });
                if valid {
                    return Ok(encrypted);
                }
            }
        }
        Err(binrw::Error::AssertFail {
            pos: start,
            message: "invalid plain or masked PSSI phrase structure".into(),
        })
    }

    /// Read a [`SongStructureData`] section that may be encrypted, depending on the `is_encrypted`
    /// value.
    fn read_encrypted<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (is_encrypted, len_entries, version): (bool, u16, u16),
    ) -> BinResult<Self> {
        if is_encrypted {
            let key: Vec<u8> = Self::get_key(len_entries, version).collect();
            let mut xor_reader = XorStream::with_key(reader, key);
            Self::read_options(&mut xor_reader, endian, (len_entries,))
        } else {
            Self::read_options(reader, endian, (len_entries,))
        }
    }

    /// Write a [`SongStructureData`] section that may be encrypted, depending on the
    /// `is_encrypted` value.
    fn write_encrypted<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        (is_encrypted, len_entries, version): (bool, u16, u16),
    ) -> BinResult<()> {
        if is_encrypted {
            let key: Vec<u8> = Self::get_key(len_entries, version).collect();
            let mut xor_writer = XorStream::with_key(writer, key);
            self.write_options(&mut xor_writer, endian, ())
        } else {
            self.write_options(writer, endian, ())
        }
    }
}

/// Describes the structure of a song (intro, chorus, verse, etc.).
///
/// Identifier: PSSI
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SongStructure {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 32))]
    #[bw(calc = 32)]
    header_size: u32,
    /// Length of the section (including the header).
    #[bw(map = |_: &u32| self.size())]
    total_size: u32,
    /// Song structure format version (0 or 1 in known files).
    pub version: u16,
    /// Size of a single entry, always 24.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 24))]
    #[bw(calc = 24u16)]
    len_entry_bytes: u16,
    /// Number of entries in this section.
    #[br(temp)]
    #[br(assert((u32::from(len_entry_bytes) * u32::from(len_entries)) == total_size - header_size))]
    #[bw(calc = data.phrases.len() as u16)]
    len_entries: u16,
    /// Indicates if the remaining parts of the song structure section are encrypted.
    ///
    /// This is a virtual field and not actually present in the file.
    #[br(restore_position, args(len_entries, version), parse_with = SongStructureData::check_if_encrypted)]
    #[bw(ignore)]
    pub is_encrypted: bool,
    /// Song structure data.
    #[br(args(is_encrypted, len_entries, version), parse_with = SongStructureData::read_encrypted)]
    #[bw(args(*is_encrypted, len_entries, *version), write_with = SongStructureData::write_encrypted)]
    pub data: SongStructureData,
}

impl SongStructure {
    /// Create a song-structure section from supplied data and wire-format settings.
    pub fn new(version: u16, is_encrypted: bool, data: SongStructureData) -> Self {
        Self {
            total_size: 0,
            version,
            is_encrypted,
            data,
        }
    }
}

impl SizedSection for SongStructure {
    fn size(&self) -> u32 {
        32 + self.data.phrases.len() as u32 * 24
    }
}

// -- Vocal Detection ------------------------------------------------------------------------------

/// Vocal-detection confidence data.
///
/// Confidence values are stored as bytes, with a variable number of entries.
/// No time scale or normalization is imposed on these raw values.
///
/// Identifier: PVDI
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub struct VocalDetection {
    #[br(temp, assert(header_size == 24))]
    #[bw(calc = 24u32)]
    header_size: u32,
    #[br(temp, assert(total_size >= header_size))]
    #[bw(calc = 24u32 + confidence.len() as u32)]
    total_size: u32,
    /// Unknown header value (observed: 0x400).
    pub u1: u32,
    /// Unknown header value (observed: 0x56220001).
    pub u2: u32,
    #[br(temp, assert(len_confidence == total_size - header_size))]
    #[bw(calc = confidence.len() as u32)]
    len_confidence: u32,
    #[br(count = len_confidence)]
    pub confidence: Vec<u8>,
}

impl SizedSection for VocalDetection {
    fn size(&self) -> u32 {
        24 + self.confidence.len() as u32
    }
}

// -- Unknown -------------------------------------------------------------------------------------

/// Unknown content.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Unknown {
    /// Length of the header data (including `kind`, `size` and `total_size`).
    #[br(assert(header_size >= 12))]
    header_size: u32,
    /// Length of the section (including the header).
    #[br(assert(total_size >= header_size))]
    total_size: u32,
    /// Unknown header data.
    #[br(count = header_size - 12)]
    header_data: Vec<u8>,
    /// Unknown content data.
    #[br(count = total_size - header_size)]
    content_data: Vec<u8>,
}

impl Unknown {
    /// Create an opaque section payload; lengths are derived from the supplied bytes.
    pub fn new(header_data: Vec<u8>, content_data: Vec<u8>) -> Result<Self> {
        let header_size = 12u64 + header_data.len() as u64;
        let total_size = header_size + content_data.len() as u64;
        if total_size > u64::from(u32::MAX) {
            return Err(AnlzError::OutputTooLarge { size: total_size });
        }
        Ok(Self {
            header_size: header_size as u32,
            total_size: total_size as u32,
            header_data,
            content_data,
        })
    }

    /// Opaque bytes between the standard section header and payload.
    pub fn header_data(&self) -> &[u8] {
        &self.header_data
    }

    /// Opaque payload bytes.
    pub fn content_data(&self) -> &[u8] {
        &self.content_data
    }
}

impl SizedSection for Unknown {
    fn size(&self) -> u32 {
        self.total_size
    }
}

// -- Anlz data -----------------------------------------------------------------------------------

/// Section content which differs depending on the section type.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[br(import(tag: AnlzTag))]
pub enum Content {
    /// All beats in the track.
    #[br(pre_assert(tag == AnlzTag::BeatGrid))]
    BeatGrid(BeatGrid),
    /// Extended beats in the track
    #[br(pre_assert(tag == AnlzTag::ExtendedBeatGrid))]
    ExtendedBeatGrid(ExtendedBeatGrid),
    /// List of cue points or loops (either hot cues or memory cues).
    #[br(pre_assert(tag == AnlzTag::CueList))]
    CueList(CueList),
    /// List of cue points or loops (either hot cues or memory cues, extended version).
    #[br(pre_assert(tag == AnlzTag::ExtendedCueList))]
    ExtendedCueList(ExtendedCueList),
    /// Path of the audio file that this analysis belongs to.
    #[br(pre_assert(tag == AnlzTag::Path))]
    Path(Path),
    /// Seek information for variable bitrate files (probably).
    #[br(pre_assert(tag == AnlzTag::VBR))]
    VBR(VBR),
    #[br(pre_assert(tag == AnlzTag::Waveform3BandCalibration))]
    Waveform3BandCalibration(Waveform3BandCalibration),
    #[br(pre_assert(tag == AnlzTag::SeekIndex))]
    SeekIndex(SeekIndex),
    #[br(pre_assert(tag == AnlzTag::VocalDetection))]
    VocalDetection(VocalDetection),
    /// Fixed-width monochrome preview of the track waveform.
    #[br(pre_assert(tag == AnlzTag::WaveformPreview))]
    WaveformPreview(WaveformPreview),
    /// Smaller version of the fixed-width monochrome preview of the track waveform.
    #[br(pre_assert(tag == AnlzTag::TinyWaveformPreview))]
    TinyWaveformPreview(TinyWaveformPreview),
    /// Variable-width large monochrome version of the track waveform.
    #[br(pre_assert(tag == AnlzTag::WaveformDetail))]
    WaveformDetail(WaveformDetail),
    /// Variable-width large monochrome version of the track waveform.
    #[br(pre_assert(tag == AnlzTag::WaveformColorPreview))]
    WaveformColorPreview(WaveformColorPreview),
    /// Variable-width large colored version of the track waveform.
    #[br(pre_assert(tag == AnlzTag::WaveformColorDetail))]
    WaveformColorDetail(WaveformColorDetail),
    /// Fixed-width three-band preview of the track waveform.
    #[br(pre_assert(tag == AnlzTag::Waveform3BandPreview))]
    Waveform3BandPreview(Waveform3BandPreview),
    /// Variable-width three-band rendition of the track waveform.
    #[br(pre_assert(tag == AnlzTag::Waveform3BandDetail))]
    Waveform3BandDetail(Waveform3BandDetail),
    /// Describes the structure of a song (intro, chorus, verse, etc.).
    #[br(pre_assert(tag == AnlzTag::SongStructure))]
    SongStructure(SongStructure),
    /// Unknown content.
    ///
    /// This allows handling files that contain unknown section types and allows to access later
    /// sections in the file that have a known type instead of failing to parse the whole file.
    #[br(pre_assert(matches!(tag, AnlzTag::Unknown(_))))]
    Unknown(Unknown),
}

impl SizedSection for Content {
    fn size(&self) -> u32 {
        match *self {
            Content::BeatGrid(ref x) => x.size(),
            Content::ExtendedBeatGrid(ref x) => x.size(),
            Content::CueList(ref x) => x.size(),
            Content::ExtendedCueList(ref x) => x.size(),
            Content::Path(ref x) => x.size(),
            Content::VBR(ref x) => x.size(),
            Content::Waveform3BandCalibration(ref x) => x.size(),
            Content::SeekIndex(ref x) => x.size(),
            Content::VocalDetection(ref x) => x.size(),
            Content::WaveformPreview(ref x) => x.size(),
            Content::TinyWaveformPreview(ref x) => x.size(),
            Content::WaveformDetail(ref x) => x.size(),
            Content::WaveformColorPreview(ref x) => x.size(),
            Content::WaveformColorDetail(ref x) => x.size(),
            Content::Waveform3BandPreview(ref x) => x.size(),
            Content::Waveform3BandDetail(ref x) => x.size(),
            Content::SongStructure(ref x) => x.size(),
            Content::Unknown(ref x) => x.size(),
        }
    }
}

/// ANLZ Section.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section {
    /// Kind of content in this item.
    pub tag: AnlzTag,
    /// The section content.
    #[br(args(tag.clone()))]
    pub content: Content,
}

impl Section {
    /// Validate an authored snapshot before installing it in a handler.
    /// Includes tag/content agreement, format widths, nested records and lengths.
    pub fn validate(&self) -> Result<()> {
        if !self.tag_matches_content() {
            return Err(AnlzError::TagContentMismatch {
                tag: self.tag.clone(),
            });
        }
        let mut bytes = std::io::Cursor::new(Vec::new());
        self.write_be(&mut bytes)?;
        let length = bytes.get_ref().len() as u64;
        bytes.set_position(0);
        Self::read_be(&mut bytes)?;
        if bytes.position() != length {
            return Err(AnlzError::InvalidSectionValue(
                "section contains unrepresentable counts or trailing bytes",
            ));
        }
        Ok(())
    }

    /// Guard against mismatched tag/content edits made through mutable sections.
    pub(crate) fn tag_matches_content(&self) -> bool {
        match &self.content {
            Content::BeatGrid(..) => self.tag == AnlzTag::BeatGrid,
            Content::ExtendedBeatGrid(..) => self.tag == AnlzTag::ExtendedBeatGrid,
            Content::VBR(..) => self.tag == AnlzTag::VBR,
            Content::SeekIndex(..) => self.tag == AnlzTag::SeekIndex,
            Content::TinyWaveformPreview(..) => self.tag == AnlzTag::TinyWaveformPreview,
            Content::WaveformPreview(..) => self.tag == AnlzTag::WaveformPreview,
            Content::WaveformDetail(..) => self.tag == AnlzTag::WaveformDetail,
            Content::WaveformColorPreview(..) => self.tag == AnlzTag::WaveformColorPreview,
            Content::WaveformColorDetail(..) => self.tag == AnlzTag::WaveformColorDetail,
            Content::Waveform3BandPreview(..) => self.tag == AnlzTag::Waveform3BandPreview,
            Content::Waveform3BandDetail(..) => self.tag == AnlzTag::Waveform3BandDetail,
            Content::Waveform3BandCalibration(..) => self.tag == AnlzTag::Waveform3BandCalibration,
            Content::VocalDetection(..) => self.tag == AnlzTag::VocalDetection,
            Content::CueList(..) => self.tag == AnlzTag::CueList,
            Content::ExtendedCueList(..) => self.tag == AnlzTag::ExtendedCueList,
            Content::Path(..) => self.tag == AnlzTag::Path,
            Content::SongStructure(..) => self.tag == AnlzTag::SongStructure,
            Content::Unknown(..) => matches!(self.tag, AnlzTag::Unknown(_)),
        }
    }
}

/// ANLZ file contents
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(big)]
pub struct AnlzData {
    /// The file header.
    #[br(assert(header.tag == AnlzTag::File && header.size >= 12 && header.total_size >= header.size))]
    #[bw(map = |header: &Header| { let mut h = header.clone(); h.size = 12 + header_data.len() as u32; h.total_size = h.size + sections.iter().map(|s| s.content.size()).sum::<u32>(); h })]
    pub header: Header,
    /// The header data.
    #[br(count = header.remaining_size())]
    pub header_data: Vec<u8>,
    /// The content sections.
    #[br(parse_with = Self::parse_sections, args(header.content_size()))]
    pub sections: Vec<Section>,
}

impl AnlzData {
    fn parse_sections<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: (u32,),
    ) -> BinResult<Vec<Section>> {
        let (content_size,) = args;
        let final_position = reader.stream_position()? + u64::from(content_size);

        let mut sections: Vec<Section> = vec![];
        while reader.stream_position()? < final_position {
            let section = Section::read_options(reader, endian, ())?;
            sections.push(section);
        }
        Ok(sections)
    }

    pub fn update_total_size(&mut self) {
        self.header.size = 12 + self.header_data.len() as u32;
        let mut total = self.header.size;
        for section in self.sections.iter_mut() {
            total += section.content.size();
        }
        self.header.total_size = total;
    }
}
