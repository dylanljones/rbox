// Author: Dylan Jones
// Date:   2025-05-08

use super::xor::XorStream;
use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian, NullWideString,
};
use modular_bitfield::prelude::*;
use std::ffi::OsStr;

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
    ///
    /// *Note:* Since the release of the Nexus 2 series, there also exists the `ExtendedCueList`
    /// section which can carry additional information.
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
    /*
    /// Unknown Waveform data
    ///
    /// Used in `.2EX` files.
    #[brw(magic = b"PWVC")]
    WaveformUnknown,
    */
    /// Describes the structure of a sond (Intro, Chrous, Verse, etc.).
    ///
    /// Used in `.EXT` files.
    #[brw(magic = b"PSSI")]
    SongStructure,
    /// Unknown Kind.
    ///
    /// This allows handling files that contain unknown section types and allows to access later
    /// sections in the file that have a known type instead of failing to parse the whole file.
    Unknown([u8; 4]),
}

impl AnlzTag {
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
            AnlzTag::WaveformPreview => "WaveformPreview".to_string(),
            AnlzTag::TinyWaveformPreview => "TinyWaveformPreview".to_string(),
            AnlzTag::WaveformDetail => "WaveformDetail".to_string(),
            AnlzTag::WaveformColorPreview => "WaveformColorPreview".to_string(),
            AnlzTag::WaveformColorDetail => "WaveformColorDetail".to_string(),
            AnlzTag::Waveform3BandPreview => "Waveform3BandPreview".to_string(),
            AnlzTag::Waveform3BandDetail => "Waveform3BandDetail".to_string(),
            AnlzTag::SongStructure => "SongStructure".to_string(),
            _ => format!("Unknown({:?})", self),
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
            "WaveformPreview" => AnlzTag::WaveformPreview,
            "TinyWaveformPreview" => AnlzTag::TinyWaveformPreview,
            "WaveformDetail" => AnlzTag::WaveformDetail,
            "WaveformColorPreview" => AnlzTag::WaveformColorPreview,
            "WaveformColorDetail" => AnlzTag::WaveformColorDetail,
            "Waveform3BandPreview" => AnlzTag::Waveform3BandPreview,
            "Waveform3BandDetail" => AnlzTag::Waveform3BandDetail,
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
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
pub struct BeatGrid {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
    /// Unknown field.
    u1: u32,
    /// Unknown field: Seems to always be `0x80000`.
    u2: u32,
    // Number of beats
    #[br(temp)]
    #[bw(calc = beats.len() as u32)]
    n_beats: u32,
    /// Beats in this beatgrid.
    #[br(count = n_beats)]
    pub beats: Vec<Beat>,
}

impl SizedSection for BeatGrid {
    fn size(&self) -> u32 {
        self.total_size
    }
}

/// Single beat grid entry inside a `BeatGridExtended` section.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(big)]
pub struct ExtBeat {
    /// Beat number inside the bar (1-4).
    pub beat_number: u8,
    /// Unknown field.
    u: u8,
}

/// Extended Beat Grid Tag (PQT2) containing all extended beats in the track
///
/// Identifier: PQT2
/// Used in `.DAT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct ExtendedBeatGrid {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 56))]
    #[bw(calc = 56)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
    /// Padding: 4bytes.
    pad1: u32,
    /// Unknown field: Seems to always be `0x01000002`.
    u1: u32,
    /// Padding: 4bytes.
    pad2: u32,
    /// Two 'normal' beats for BPM
    #[br(count = 2)]
    pub bpm: Vec<Beat>,
    /// Number of beats
    #[br(temp)]
    #[bw(calc = beats.len() as u32)]
    n_beats: u32,
    /// Unknow fields
    u2: u32,
    u3: u32,
    u4: u32,
    /// Beats in this beatgrid.
    #[br(count = n_beats)]
    pub beats: Vec<ExtBeat>,
}

impl SizedSection for ExtendedBeatGrid {
    fn size(&self) -> u32 {
        self.total_size
    }
}

// -- Cue List Tags -------------------------------------------------------------------------------

/// Indicates if the cue is point or a loop.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(repr = u8)]
pub enum CueType {
    /// Cue point.
    Point = 1,
    /// Loop.
    Loop = 2,
}

impl CueType {
    /// Returns the tag type as a string.
    pub fn to_int(&self) -> i32 {
        match self {
            CueType::Point => 1,
            CueType::Loop => 2,
        }
    }
}

/// Describes the types of entries found in a Cue List section.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(big, repr = u32)]
pub enum CueListType {
    /// Memory cues or loops.
    MemoryCues = 0,
    /// Hot cues or loops.
    HotCues = 1,
}

/// Indicates if the cue is point or a loop.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(repr = u32)]
pub enum CueStatus {
    /// Cue point diabled.
    Disabled = 0,
    /// Cue point enabled
    Active = 4,
}

/// A memory or hot cue (or loop).
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(big)]
pub struct Cue {
    /// Cue entry header.
    pub header: Header,
    /// Hot cue number.
    pub hot_cue: u32,
    /// Loop status. `4` if this cue is an active loop, `0` otherwise.
    pub status: u32,
    /// Unknown field. Seems to always have the value `0x10000`.
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
    u2: u8,
    /// Unknown field. Seems always have the value `0x03E8` (= decimal 1000).
    u3: u16,
    /// Time in milliseconds after which this cue would occur (at normal playback speed).
    pub time: u32,
    /// Time in milliseconds after which this the loop would jump back to `time` (at normal playback speed).
    pub loop_time: u32,
    /// Unknown field.
    u4: u32,
    /// Unknown field.
    u5: u32,
    /// Unknown field.
    u6: u32,
    /// Unknown field.
    u7: u32,
}

/// A extended memory or hot cue (or loop).
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(big)]
pub struct ExtendedCue {
    /// Cue entry header.
    pub header: Header,
    /// Hot cue number.
    pub hot_cue: u32,
    /// Type of this cue (`2` if this cue is a loop).
    pub cue_type: CueType,
    /// Unknown field. Seems always have the value `0`.
    u1: u8,
    /// Unknown field. Seems always have the value `0x03E8` (= decimal 1000).
    u2: u16,
    /// Time in milliseconds after which this cue would occur (at normal playback speed).
    pub time: u32,
    /// Time in milliseconds after which this the loop would jump back to `time` (at normal playback speed).
    pub loop_time: u32,
    /// Color assigned to this cue.
    ///
    /// Only used by memory cues, hot cues use a different value (see below).
    pub color: u8,
    /// Unknown field.
    u3: u8,
    /// Unknown field.
    u4: u16,
    /// Unknown field.
    u5: u32,
    /// Represents the loop size numerator (if this is a quantized loop).
    pub loop_numerator: u16,
    /// Represents the loop size denominator (if this is a quantized loop).
    pub loop_denominator: u16,
    /// Length of the comment string in bytes.
    #[br(temp)]
    #[bw(calc = (comment.len() as u32 + 1) * 2)]
    len_comment: u32,
    /// An UTF-16BE encoded string, followed by a trailing  `0x0000`.
    #[br(assert((comment.len() as u32 + 1) * 2 == len_comment))]
    pub comment: NullWideString,
    /// Rekordbox hotcue color index.
    pub hot_cue_color_index: u8,
    /// Rekordbot hot cue color RGB value.
    ///
    /// This color is similar but not identical to the color that Rekordbox displays, and possibly
    /// used to illuminate the RGB LEDs in a player that has loaded the cue. If not color is
    /// associated with this hot cue, the value is `(0, 0, 0)`.
    pub hot_cue_color_rgb: (u8, u8, u8),
    /// Unknown field.
    u6: u32,
    /// Unknown field.
    u7: u32,
    /// Unknown field.
    u8: u32,
    /// Unknown field.
    u9: u32,
    /// Unknown field.
    u10: u32,
}

/// List of cue points or loops.
///
/// Identifier: PCOB
/// Used in `.DAT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct CueList {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
    /// The types of cues (memory or hot) that this list contains.
    pub list_type: CueListType,
    /// Unknown field
    unknown: u16,
    /// Number of cues.
    #[br(temp)]
    #[bw(calc = cues.len() as u16)]
    len_cues: u16,
    /// Unknown field.
    memory_count: u32,
    /// Cues
    #[br(count = usize::from(len_cues))]
    pub cues: Vec<Cue>,
}

impl SizedSection for CueList {
    fn size(&self) -> u32 {
        self.total_size
    }
}

/// List of cue points or loops PCO2
///
/// Identifier: PCO2
/// Used in `.DAT` files.
/// Variation of the original `CueList` that also adds support for more metadata such as
/// comments and colors. Introduces with the Nexus 2 series players.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct ExtendedCueList {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 20))]
    #[bw(calc = 20)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
    /// The types of cues (memory or hot) that this list contains.
    pub list_type: CueListType,
    /// Number of cues.
    #[br(temp)]
    #[bw(calc = cues.len() as u16)]
    len_cues: u16,
    /// Unknown field
    #[br(assert(unknown == 0))]
    unknown: u16,
    /// Cues
    #[br(count = usize::from(len_cues))]
    pub cues: Vec<ExtendedCue>,
}

impl SizedSection for ExtendedCueList {
    fn size(&self) -> u32 {
        self.total_size
    }
}

// -- Path Tag ------------------------------------------------------------------------------------

/// Path tag containing the file path that this analysis belongs to.
///
/// Identifier: PPTH
/// Used in `.DAT`, `.EXT` and `.2EX` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct Path {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 16))]
    #[bw(calc = 16)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
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
        self.total_size
    }
}

impl Path {
    pub fn new<P: AsRef<std::path::Path> + AsRef<OsStr>>(path: P) -> Self {
        let string = NullWideString::from("");
        let mut item = Self {
            total_size: 0,
            path: string,
        };
        item.set(path);
        item
    }

    pub fn set<P: AsRef<std::path::Path> + AsRef<OsStr>>(&mut self, path: P) {
        let p = std::path::Path::new(&path);
        let string = NullWideString::from(p.as_os_str().to_str().unwrap());
        self.path = string.clone();
        self.total_size = ((string.len() as u32) + 1) * 2 + 16;
    }
}

// -- VBR Tag -------------------------------------------------------------------------------------

/// Variable bitrate tag (PVBR) containing seek information.
///
/// Identifier: PVBR
/// Used in `.DAT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct VBR {
    /// Length of the header data (including `kind`, `size` and `total_size`).
    #[br(assert(header_size == 16))]
    pub header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
    /// Unknown field.
    i2: u32,
    /// Unknown data.
    #[br(count = total_size - header_size)]
    pub data: Vec<u8>,
}

impl SizedSection for VBR {
    fn size(&self) -> u32 {
        self.total_size
    }
}

// -- Tiny Waveform Preview Tag -------------------------------------------------------------------

/// Single Column value in a Tiny Waveform Preview.
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(big, map = Self::from_bytes)]
#[bw(big, map = |x: &TinyWaveformColumn| x.into_bytes())]
pub struct TinyWaveformColumn {
    #[allow(dead_code)]
    unused: B4,
    /// Height of the Column in pixels.
    pub height: B4,
}

impl Default for TinyWaveformColumn {
    fn default() -> Self {
        Self::new()
    }
}

/// Smaller version of the fixed-width monochrome preview of the track waveform.
///
/// Identifier: PWAV
/// Used in `.DAT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct TinyWaveformPreview {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 20))]
    #[bw(calc = 20)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
    /// Unknown field.
    #[br(temp)]
    #[br(assert(len_preview == total_size - header_size))]
    #[bw(calc = data.len() as u32)]
    len_preview: u32,
    /// Unknown field (apparently always `0x00100000`)
    u1: u32,
    /// Waveform preview column data.
    #[br(count = len_preview)]
    pub data: Vec<TinyWaveformColumn>,
}

impl SizedSection for TinyWaveformPreview {
    fn size(&self) -> u32 {
        self.total_size
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
/// Identifier: PWV2
/// Used in `.DAT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct WaveformPreview {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 20))]
    #[bw(calc = 20)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
    /// Unknown field.
    #[br(temp)]
    #[br(assert(len_preview == total_size - header_size))]
    #[bw(calc = data.len() as u32)]
    len_preview: u32,
    /// Unknown field (apparently always `0x00100000`)
    u1: u32,
    /// Waveform preview column data.
    #[br(count = len_preview)]
    pub data: Vec<WaveformColumn>,
}

impl SizedSection for WaveformPreview {
    fn size(&self) -> u32 {
        self.total_size
    }
}

/// Variable-width large monochrome version of the track waveform.
///
/// Identifier: PWV3
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct WaveformDetail {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
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
    #[br(assert(u1 == 0x00960000))]
    u1: u32,
    /// Waveform preview column data.
    ///
    /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
    /// so for each second of track audio there are 150 waveform detail entries.
    #[br(count = len_entries)]
    pub data: Vec<WaveformColumn>,
}

impl SizedSection for WaveformDetail {
    fn size(&self) -> u32 {
        self.total_size
    }
}

// -- Waveform Color Tag ------------------------------------------------------------------

/// Single Column value in a Waveform Color Preview.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[brw(big)]
pub struct WaveformColorPreviewColumn {
    /// Unknown field (somehow encodes the "whiteness").
    u1: u8,
    /// Unknown field (somehow encodes the "whiteness").
    u2: u8,
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
#[derive(Debug, PartialEq, Eq)]
pub struct WaveformColorPreview {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
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
    /// Unknown field.
    unknown: u32,
    /// Waveform preview column data.
    ///
    /// Each entry represents one half-frame of audio data, and there are 75 frames per second,
    /// so for each second of track audio there are 150 waveform detail entries.
    #[br(count = len_entries)]
    pub data: Vec<WaveformColorPreviewColumn>,
}

impl SizedSection for WaveformColorPreview {
    fn size(&self) -> u32 {
        self.total_size
    }
}

// -- Waveform Color Detail Tag -------------------------------------------------------------------

/// Single Column value in a Waveform Color Detail section.
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(map = Self::from_bytes)]
#[bw(big, map = |x: &WaveformColorDetailColumn| x.into_bytes())]
pub struct WaveformColorDetailColumn {
    /// Red color component.
    pub red: B3,
    /// Green color component.
    pub green: B3,
    /// Blue color component.
    pub blue: B3,
    /// Height of the column.
    pub height: B5,
    /// Unknown field
    #[allow(dead_code)]
    unknown: B2,
}

/// Variable-width large colored version of the track waveform.
///
/// Identifier: PWV5
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct WaveformColorDetail {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
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
    /// Unknown field.
    unknown: u32,
    /// Waveform detail column data.
    #[br(count = len_entries)]
    pub data: Vec<WaveformColorDetailColumn>,
}

impl SizedSection for WaveformColorDetail {
    fn size(&self) -> u32 {
        self.total_size
    }
}

// -- Waveform 3-Band Tags ------------------------------------------------------------------------

/// Single Column value in a Waveform 3-Band Preview section.
#[bitfield]
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy)]
#[br(map = Self::from_bytes)]
#[bw(big, map = |x: &Waveform3BandColumn| x.into_bytes())]
pub struct Waveform3BandColumn {
    /// Mid-range component (amber).
    pub mid: B8,
    /// High frequencies (white).
    pub high: B8,
    /// Low frequencies (dark blue).
    pub low: B8,
}

/// Fixed-width three-band preview of the track waveform.
///
/// Identifier: PWV6
/// Used in `.2EX` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct Waveform3BandPreview {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 20))]
    #[bw(calc = 20)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
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

impl SizedSection for Waveform3BandPreview {
    fn size(&self) -> u32 {
        self.total_size
    }
}

/// Variable-width three-band rendition of the track waveform, used for CDJ-3000 and in Rekordbox.
///
/// Identifier: PWV7
/// Used in `.2EX` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct Waveform3BandDetail {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 24))]
    #[bw(calc = 24)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
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
    /// Unknown field.
    unknown: u32,
    /// Waveform detail column data.
    #[br(count = len_entries)]
    pub data: Vec<Waveform3BandColumn>,
}

impl SizedSection for Waveform3BandDetail {
    fn size(&self) -> u32 {
        self.total_size
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
    u1: u8,
    /// Flag byte used for numbered variations (in case of the `High` mood).
    ///
    /// See the documentation for details:
    /// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#high-phrase-variants>
    pub k1: u8,
    /// Unknown field.
    #[allow(dead_code)]
    u2: u8,
    /// Flag byte used for numbered variations (in case of the `High` mood).
    ///
    /// See the documentation for details:
    /// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#high-phrase-variants>
    pub k2: u8,
    /// Unknown field.
    #[allow(dead_code)]
    u3: u8,
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
    u4: u8,
    /// Flag byte used for numbered variations (in case of the `High` mood).
    ///
    /// See the documentation for details:
    /// <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/anlz.html#high-phrase-variants>
    pub k3: u8,
    /// Unknown field.
    #[allow(dead_code)]
    u5: u8,
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
    u1: u32,
    /// Unknown field.
    u2: u16,
    /// Number of the beat at which the last recognized phrase ends.
    pub end_beat: u16,
    /// Unknown field.
    u3: u16,
    /// Stylistic bank assigned in Lightning Mode.
    pub bank: Bank,
    /// Unknown field.
    u4: u8,
    /// Phrase entry data.
    #[br(count = usize::from(len_entries))]
    pub phrases: Vec<Phrase>,
}

impl SongStructureData {
    const KEY_DATA: [u8; 19] = [
        0xCB, 0xE1, 0xEE, 0xFA, 0xE5, 0xEE, 0xAD, 0xEE, 0xE9, 0xD2, 0xE9, 0xEB, 0xE1, 0xE9, 0xF3,
        0xE8, 0xE9, 0xF4, 0xE1,
    ];

    /// Returns an iterator over the key bytes (RB6+).
    fn get_key(len_entries: u16) -> impl Iterator<Item = u8> {
        Self::KEY_DATA.into_iter().map(move |x: u8| -> u8 {
            let value = u16::from(x) + len_entries;
            (value % 256) as u8
        })
    }

    /// Returns `true` if the [`SongStructureData`] is encrypted.
    ///
    /// The method tries to decrypt the `raw_mood` field and checking if the result is valid.
    fn check_if_encrypted(raw_mood: [u8; 2], len_entries: u16) -> bool {
        let buffer: Vec<u8> = raw_mood
            .iter()
            .zip(Self::get_key(len_entries).take(2))
            .map(|(byte, key)| byte ^ key)
            .collect();
        let mut reader = binrw::io::Cursor::new(buffer);
        Mood::read(&mut reader).is_ok()
    }

    /// Read a [`SongStructureData`] section that may be encrypted, depending on the `is_encrypted`
    /// value.
    fn read_encrypted<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (is_encrypted, len_entries): (bool, u16),
    ) -> BinResult<Self> {
        if is_encrypted {
            let key: Vec<u8> = Self::get_key(len_entries).collect();
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
        (is_encrypted, len_entries): (bool, u16),
    ) -> BinResult<()> {
        if is_encrypted {
            let key: Vec<u8> = Self::get_key(len_entries).collect();
            let mut xor_writer = XorStream::with_key(writer, key);
            self.write_options(&mut xor_writer, endian, ())
        } else {
            self.write_options(writer, endian, ())
        }
    }
}

/// Describes the structure of a song (Intro, Chrous, Verse, etc.).
///
/// Identifier: PSSI
/// Used in `.EXT` files.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct SongStructure {
    /// Length of the header data (including `kind`, `header_size` and `total_size`).
    #[br(temp)]
    #[br(assert(header_size == 32))]
    #[bw(calc = 32)]
    header_size: u32,
    /// Length of the section (including the header).
    pub total_size: u32,
    /// Size of a single entry, always 24.
    #[br(temp)]
    #[br(assert(len_entry_bytes == 24))]
    #[bw(calc = 24u32)]
    len_entry_bytes: u32,
    /// Number of entries in this section.
    #[br(temp)]
    #[br(assert((len_entry_bytes * (len_entries as u32)) == total_size - header_size))]
    #[bw(calc = data.phrases.len() as u16)]
    len_entries: u16,
    /// Indicates if the remaining parts of the song structure section are encrypted.
    ///
    /// This is a virtual field and not actually present in the file.
    #[br(restore_position, map = |raw_mood: [u8; 2]| SongStructureData::check_if_encrypted(raw_mood, len_entries))]
    #[bw(ignore)]
    is_encrypted: bool,
    /// Song structure data.
    #[br(args(is_encrypted, len_entries), parse_with = SongStructureData::read_encrypted)]
    #[bw(args(*is_encrypted, len_entries), write_with = SongStructureData::write_encrypted)]
    data: SongStructureData,
}

impl SizedSection for SongStructure {
    fn size(&self) -> u32 {
        self.total_size
    }
}

// -- Unknown -------------------------------------------------------------------------------------

/// Unknown content.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct Unknown {
    /// Length of the header data (including `kind`, `size` and `total_size`).
    header_size: u32,
    /// Length of the section (including the header).
    total_size: u32,
    /// Unknown header data.
    #[br(count = header_size - 12)]
    header_data: Vec<u8>,
    /// Unknown content data.
    #[br(count = total_size - header_size)]
    content_data: Vec<u8>,
}

impl SizedSection for Unknown {
    fn size(&self) -> u32 {
        self.total_size
    }
}

// -- Anlz data -----------------------------------------------------------------------------------

/// Section content which differs depending on the section type.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
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
    /// Describes the structure of a song (Intro, Chrous, Verse, etc.).
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
#[derive(Debug, PartialEq, Eq)]
pub struct Section {
    /// Kind of content in this item.
    pub tag: AnlzTag,
    /// The section content.
    #[br(args(tag.clone()))]
    pub content: Content,
}

/// ANLZ file contents
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(big)]
pub struct AnlzData {
    /// The file header.
    #[br(assert(header.tag == AnlzTag::File))]
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

    pub fn update_total_size(&mut self) -> anyhow::Result<()> {
        let mut total = self.header.size;
        for section in self.sections.iter_mut() {
            total += section.content.size();
        }
        self.header.total_size = total;
        Ok(())
    }
}

// -- Anlz file handler ---------------------------------------------------------------------------

/// Rekordbox ANLZ file handler.
pub struct Anlz {
    /// Path to the XML file
    path: std::path::PathBuf,
    /// XML document
    pub data: AnlzData,
}

impl Anlz {
    /// Read a Rekordbox masterPlaylist6 XML file.
    pub fn load<P: AsRef<std::path::Path> + AsRef<OsStr>>(path: P) -> anyhow::Result<Self> {
        let p = std::path::Path::new(&path).to_path_buf();
        let mut file = std::fs::File::open(&p).expect("File not found");
        let data = AnlzData::read(&mut file).expect("Can't read ANLZ");
        Ok(Anlz { path: p, data })
    }

    /// Write the XML document to a file.
    pub fn dump_copy<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        self.data.update_total_size()?;
        let mut file = std::fs::File::create(path).expect("Failed to create file");
        self.data.write(&mut file)?;
        Ok(())
    }

    /// Write the XML document to the original file.
    pub fn dump(&mut self) -> anyhow::Result<()> {
        let path = &self.path.clone();
        self.dump_copy(path)?;
        Ok(())
    }

    /// Return a reference to the *first* section with the given tag type.
    pub fn find_section_by_tag(&mut self, tag: AnlzTag) -> Option<&mut Section> {
        self.data.sections.iter_mut().find(|s| s.tag == tag)
    }

    /// Return all references to the sections with the given tag type.
    pub fn find_sections_by_tag(&mut self, tag: AnlzTag) -> Vec<&mut Section> {
        // self.data.sections.iter_mut().find(|s| s.tag == tag)
        self.data
            .sections
            .iter_mut()
            .filter(|s| s.tag == tag)
            .collect()
    }

    /// Check if the file contains a section
    pub fn contains(&self, tag: AnlzTag) -> bool {
        self.data.sections.iter().any(|s| s.tag == tag)
    }

    /// Return a list of all tag types present in the file
    pub fn get_tags(&self) -> anyhow::Result<Vec<String>> {
        let mut names = Vec::new();
        for section in &self.data.sections {
            let name = &section.tag;
            names.push(name.to_string());
        }
        Ok(names)
    }

    /// Return the beat grid entries (PQTZ) if present
    pub fn get_beat_grid(&mut self) -> Option<Vec<&mut Beat>> {
        let section = self.find_section_by_tag(AnlzTag::BeatGrid);
        if let Some(section) = section {
            if let Content::BeatGrid(content) = &mut section.content {
                return Some(content.beats.iter_mut().collect());
            }
        }
        None
    }

    /// Return the extended beat grid entries (PQT2) if present
    pub fn get_extended_beat_grid(&mut self) -> Option<Vec<&mut ExtBeat>> {
        let section = self.find_section_by_tag(AnlzTag::ExtendedBeatGrid);
        if let Some(section) = section {
            if let Content::ExtendedBeatGrid(content) = &mut section.content {
                return Some(content.beats.iter_mut().collect());
            }
        }
        None
    }

    /// Return the hot cue list entries (PCOB) if present
    pub fn get_hot_cues(&mut self) -> Option<Vec<&mut Cue>> {
        let sections = self.find_sections_by_tag(AnlzTag::CueList);
        for section in sections {
            if let Content::CueList(content) = &mut section.content {
                if content.list_type == CueListType::HotCues {
                    return Some(content.cues.iter_mut().collect());
                }
            }
        }
        None
    }

    /// Return the memory cue list entries (PCOB) if present
    pub fn get_memory_cues(&mut self) -> Option<Vec<&mut Cue>> {
        let sections = self.find_sections_by_tag(AnlzTag::CueList);
        for section in sections {
            if let Content::CueList(content) = &mut section.content {
                if content.list_type == CueListType::MemoryCues {
                    return Some(content.cues.iter_mut().collect());
                }
            }
        }
        None
    }

    /// Return the extended hot cue list entries (PCOB) if present
    pub fn get_extended_hot_cues(&mut self) -> Option<Vec<&mut ExtendedCue>> {
        let sections = self.find_sections_by_tag(AnlzTag::ExtendedCueList);
        for section in sections {
            if let Content::ExtendedCueList(content) = &mut section.content {
                if content.list_type == CueListType::HotCues {
                    return Some(content.cues.iter_mut().collect());
                }
            }
        }
        None
    }

    /// Return the extended memory cue list entries (PCOB) if present
    pub fn get_extended_memory_cues(&mut self) -> Option<Vec<&mut ExtendedCue>> {
        let sections = self.find_sections_by_tag(AnlzTag::ExtendedCueList);
        for section in sections {
            if let Content::ExtendedCueList(content) = &mut section.content {
                if content.list_type == CueListType::MemoryCues {
                    return Some(content.cues.iter_mut().collect());
                }
            }
        }
        None
    }

    /// Returns the path value (PPTH) if present
    pub fn get_path(&mut self) -> Option<String> {
        let section = self.find_section_by_tag(AnlzTag::Path);
        if let Some(section) = section {
            if let Content::Path(content) = &section.content {
                return Some(content.path.to_string());
            }
        }
        None
    }

    /// Sets the path value (PPTH)
    ///
    /// If no PPTH tag is present, a new one is created
    pub fn set_path<P: AsRef<std::path::Path> + AsRef<OsStr>>(
        &mut self,
        path: P,
    ) -> anyhow::Result<()> {
        let section = self.find_section_by_tag(AnlzTag::Path);
        if let Some(section) = section {
            // Found PPTH tag
            if let Content::Path(content) = &mut section.content {
                content.set(path);
            } else {
                return Err(anyhow::anyhow!("Path not found"));
            }
        } else {
            // No PPTH tag, create a new one
            let content = Path::new(path);
            let section = Section {
                tag: AnlzTag::Path,
                content: Content::Path(content),
            };
            self.data.sections.push(section);
        }
        self.data.update_total_size()?;
        Ok(())
    }

    /// Returns the (unknown) VBR data (PVBR) if present
    pub fn get_vbr_data(&mut self) -> Option<Vec<u8>> {
        let section = self.find_section_by_tag(AnlzTag::VBR);
        if let Some(section) = section {
            if let Content::VBR(content) = &section.content {
                return Some(content.data.clone());
            }
        }
        None
    }

    /// Returns the tiny waveform columns (PWAV) if present
    pub fn get_tiny_waveform_preview(&mut self) -> Option<Vec<TinyWaveformColumn>> {
        let section = self.find_section_by_tag(AnlzTag::TinyWaveformPreview);
        if let Some(section) = section {
            if let Content::TinyWaveformPreview(content) = &section.content {
                return Some(content.data.clone());
            }
        }
        None
    }

    /// Returns the fixed-width monochrome waveform preview columns (PWV2) if present
    pub fn get_waveform_preview(&mut self) -> Option<Vec<WaveformColumn>> {
        let section = self.find_section_by_tag(AnlzTag::WaveformPreview);
        if let Some(section) = section {
            if let Content::WaveformPreview(content) = &section.content {
                return Some(content.data.clone());
            }
        }
        None
    }

    /// Returns the variable-width monochrome waveform columns (PWV3) if present
    pub fn get_waveform_detail(&mut self) -> Option<Vec<WaveformColumn>> {
        let section = self.find_section_by_tag(AnlzTag::WaveformDetail);
        if let Some(section) = section {
            if let Content::WaveformDetail(content) = &section.content {
                return Some(content.data.clone());
            }
        }
        None
    }

    /// Returns the variable-width color waveform preview columns (PWV4) if present
    pub fn get_waveform_color_preview(&mut self) -> Option<Vec<WaveformColorPreviewColumn>> {
        let section = self.find_section_by_tag(AnlzTag::WaveformColorPreview);
        if let Some(section) = section {
            if let Content::WaveformColorPreview(content) = &section.content {
                return Some(content.data.clone());
            }
        }
        None
    }

    /// Returns the variable-width color waveform columns (PWV5) if present
    pub fn get_waveform_color_detail(&mut self) -> Option<Vec<WaveformColorDetailColumn>> {
        let section = self.find_section_by_tag(AnlzTag::WaveformColorDetail);
        if let Some(section) = section {
            if let Content::WaveformColorDetail(content) = &section.content {
                return Some(content.data.clone());
            }
        }
        None
    }

    /// Returns the fixed-width three-band waveform preview columns (PWV6) if present
    pub fn get_waveform_3band_preview(&mut self) -> Option<Vec<Waveform3BandColumn>> {
        let section = self.find_section_by_tag(AnlzTag::Waveform3BandPreview);
        if let Some(section) = section {
            if let Content::Waveform3BandPreview(content) = &section.content {
                return Some(content.data.clone());
            }
        }
        None
    }

    /// Returns the fixed-width three-band waveform columns (PWV7) if present
    pub fn get_waveform_3band_detail(&mut self) -> Option<Vec<Waveform3BandColumn>> {
        let section = self.find_section_by_tag(AnlzTag::Waveform3BandDetail);
        if let Some(section) = section {
            if let Content::Waveform3BandDetail(content) = &section.content {
                return Some(content.data.clone());
            }
        }
        None
    }

    /// Returns the song structure data (PSSI) if present
    pub fn get_song_structure(&mut self) -> Option<SongStructureData> {
        let section = self.find_section_by_tag(AnlzTag::SongStructure);
        if let Some(section) = section {
            if let Content::SongStructure(content) = &section.content {
                return Some(content.data.clone());
            }
        }
        None
    }
}

/// Collection of ANLZ file paths.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnlzPaths {
    pub dat: std::path::PathBuf,
    pub ext: Option<std::path::PathBuf>,
    pub ex2: Option<std::path::PathBuf>,
}

/// Collection of ANLZ files.
pub struct AnlzFiles {
    pub dat: Anlz,
    pub ext: Option<Anlz>,
    pub ex2: Option<Anlz>,
}

/// Scan a directory for ANLZ files.
pub fn find_anlz_files<P: AsRef<std::path::Path> + AsRef<OsStr>>(
    root: P,
) -> anyhow::Result<Option<AnlzPaths>> {
    let root = std::path::Path::new(&root).to_path_buf();
    let mut dat: Option<std::path::PathBuf> = None;
    let mut ext: Option<std::path::PathBuf> = None;
    let mut ex2: Option<std::path::PathBuf> = None;

    let paths = std::fs::read_dir(root)?;
    for path in paths {
        let file = path?.path();
        let extension = file.extension();
        if let Some(extension) = extension {
            match extension.to_ascii_uppercase().to_str().unwrap() {
                "DAT" => dat = Some(file.clone()),
                "EXT" => ext = Some(file.clone()),
                "2EX" => ex2 = Some(file.clone()),
                &_ => {}
            }
        }
    }
    if dat.is_none() {
        return Ok(None);
    }
    let dat = dat.unwrap();
    let files = AnlzPaths { dat, ext, ex2 };
    Ok(Some(files))
}
