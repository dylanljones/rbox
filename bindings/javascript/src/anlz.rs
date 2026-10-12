// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Typed ANLZ snapshots and explicit editing methods for JavaScript.

use napi::Result;
use napi_derive::napi;
use rbox::anlz::sections::*;
use rbox::anlz::{Anlz, AnlzPaths};

fn invalid(message: &str) -> napi::Error {
    napi::Error::new(napi::Status::InvalidArg, message)
}

fn read_number(value: f64, min: f64, max: f64, field: &str) -> Result<f64> {
    if !value.is_finite() || value.fract() != 0.0 || value < min || value > max {
        return Err(invalid(&format!(
            "{field} must be an integer in {min}..={max}"
        )));
    }
    Ok(value)
}

fn read_index(value: f64) -> Result<usize> {
    Ok(read_number(value, 0.0, u32::MAX as f64, "section index")? as usize)
}

fn read_u64(value: &napi::bindgen_prelude::BigInt) -> Result<u64> {
    let (negative, number, lossless) = value.get_u64();
    if negative || !lossless {
        return Err(invalid("value must fit an unsigned 64-bit integer"));
    }
    Ok(number)
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "Beat")]
pub struct JsBeat {
    pub beat_number: f64,
    pub tempo: f64,
    pub time: f64,
}

impl From<&Beat> for JsBeat {
    fn from(value: &Beat) -> Self {
        Self {
            beat_number: f64::from(value.beat_number),
            tempo: f64::from(value.tempo),
            time: f64::from(value.time),
        }
    }
}

impl JsBeat {
    fn to_core(&self) -> Result<Beat> {
        Ok(Beat {
            beat_number: read_number(
                self.beat_number,
                u16::MIN as f64,
                u16::MAX as f64,
                "Beat.beat_number",
            )? as u16,
            tempo: read_number(self.tempo, u16::MIN as f64, u16::MAX as f64, "Beat.tempo")? as u16,
            time: read_number(self.time, u32::MIN as f64, u32::MAX as f64, "Beat.time")? as u32,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "ExtBeat")]
pub struct JsExtBeat {
    pub microseconds: f64,
}

impl From<&ExtBeat> for JsExtBeat {
    fn from(value: &ExtBeat) -> Self {
        Self {
            microseconds: f64::from(value.microseconds),
        }
    }
}

impl JsExtBeat {
    fn to_core(&self) -> Result<ExtBeat> {
        Ok(ExtBeat {
            microseconds: read_number(
                self.microseconds,
                u16::MIN as f64,
                u16::MAX as f64,
                "ExtBeat.microseconds",
            )? as u16,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "BeatGrid")]
pub struct JsBeatGrid {
    pub beats: Vec<JsBeat>,
    pub header_flags: f64,
}

impl From<&BeatGrid> for JsBeatGrid {
    fn from(value: &BeatGrid) -> Self {
        Self {
            beats: value.beats.iter().map(Into::into).collect(),
            header_flags: f64::from(value.header_flags),
        }
    }
}

impl JsBeatGrid {
    fn to_core(&self) -> Result<BeatGrid> {
        let mut value = BeatGrid::new(
            self.beats
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        );
        value.header_flags = read_number(
            self.header_flags,
            u32::MIN as f64,
            u32::MAX as f64,
            "BeatGrid.header_flags",
        )? as u32;
        Ok(value)
    }
}

/// Create a fixed 4/4 grid at 1..=300 BPM. Times are raw integer milliseconds;
/// endMs is inclusive. Apply the returned snapshot with Anlz.setBeatGrid.
#[napi]
pub fn new_static_beat_grid(bpm: f64, first_beat_ms: f64, end_ms: f64) -> Result<JsBeatGrid> {
    let first = read_number(first_beat_ms, 0., u32::MAX as f64, "firstBeatMs")? as u32;
    let end = read_number(end_ms, 0., u32::MAX as f64, "endMs")? as u32;
    let grid = BeatGrid::new_static_bpm(bpm, first, end).map_err(|e| invalid(&e.to_string()))?;
    Ok((&grid).into())
}

/// Return a copy with fixed-tempo beats, preserving flags and grid correction.
/// The input object is unchanged. Apply the result with Anlz.setBeatGrid.
/// BPM is 1..=300; raw millisecond endMs is inclusive. PQT2 is not updated.
#[napi]
pub fn set_static_beat_grid(
    grid: JsBeatGrid,
    bpm: f64,
    first_beat_ms: f64,
    end_ms: f64,
) -> Result<JsBeatGrid> {
    let first = read_number(first_beat_ms, 0., u32::MAX as f64, "firstBeatMs")? as u32;
    let end = read_number(end_ms, 0., u32::MAX as f64, "endMs")? as u32;
    let mut grid = grid.to_core()?;
    grid.set_static_bpm(bpm, first, end)
        .map_err(|e| invalid(&e.to_string()))?;
    Ok((&grid).into())
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "ExtendedBeatGrid")]
pub struct JsExtendedBeatGrid {
    pub first_beat: JsBeat,
    pub last_beat: JsBeat,
    pub offset_ms: f64,
    pub offset_flags: f64,
    pub checksum: f64,
    pub beats: Vec<JsExtBeat>,
    pub version: f64,
}

impl From<&ExtendedBeatGrid> for JsExtendedBeatGrid {
    fn from(value: &ExtendedBeatGrid) -> Self {
        Self {
            first_beat: (&value.first_beat).into(),
            last_beat: (&value.last_beat).into(),
            offset_ms: f64::from(value.offset_ms),
            offset_flags: f64::from(value.offset_flags),
            checksum: f64::from(value.checksum),
            beats: value.beats.iter().map(Into::into).collect(),
            version: f64::from(value.version),
        }
    }
}

impl JsExtendedBeatGrid {
    fn to_core(&self) -> Result<ExtendedBeatGrid> {
        let mut value = ExtendedBeatGrid::new(
            read_number(
                self.version,
                u32::MIN as f64,
                u32::MAX as f64,
                "ExtendedBeatGrid.version",
            )? as u32,
            [self.first_beat.to_core()?, self.last_beat.to_core()?],
            self.beats
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
            read_number(
                self.checksum,
                u32::MIN as f64,
                u32::MAX as f64,
                "ExtendedBeatGrid.checksum",
            )? as u32,
        );
        value.offset_ms = read_number(
            self.offset_ms,
            i16::MIN as f64,
            i16::MAX as f64,
            "ExtendedBeatGrid.offset_ms",
        )? as i16;
        value.offset_flags = read_number(
            self.offset_flags,
            u16::MIN as f64,
            u16::MAX as f64,
            "ExtendedBeatGrid.offset_flags",
        )? as u16;
        Ok(value)
    }
}

#[napi(js_name = "CueType")]
pub enum JsCueType {
    Point = 1,
    Loop = 2,
}

impl Into<JsCueType> for &CueType {
    fn into(self) -> JsCueType {
        match self {
            CueType::Point => JsCueType::Point,
            CueType::Loop => JsCueType::Loop,
        }
    }
}

#[napi(js_name = "CueListType")]
pub enum JsCueListType {
    MemoryCues = 0,
    HotCues = 1,
}

impl Into<JsCueListType> for &CueListType {
    fn into(self) -> JsCueListType {
        match self {
            CueListType::MemoryCues => JsCueListType::MemoryCues,
            CueListType::HotCues => JsCueListType::HotCues,
        }
    }
}

#[napi(js_name = "CueStatus")]
pub enum JsCueStatus {
    Disabled = 0,
    Active = 4,
}

impl Into<JsCueStatus> for &CueStatus {
    fn into(self) -> JsCueStatus {
        match self {
            CueStatus::Disabled => JsCueStatus::Disabled,
            CueStatus::Active => JsCueStatus::Active,
        }
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "AnlzCue")]
pub struct JsCue {
    pub hot_cue: f64,
    pub status: JsCueStatus,
    pub order_first: f64,
    pub order_last: f64,
    pub cue_type: JsCueType,
    pub time: f64,
    pub loop_time: f64,
    pub header_size: f64,
    pub u4: f64,
    pub u5: f64,
    pub u6: f64,
    pub u7: f64,
}

impl From<&Cue> for JsCue {
    fn from(value: &Cue) -> Self {
        Self {
            hot_cue: f64::from(value.hot_cue),
            status: (&value.status).into(),
            order_first: f64::from(value.order_first),
            order_last: f64::from(value.order_last),
            cue_type: (&value.cue_type).into(),
            time: f64::from(value.time),
            loop_time: f64::from(value.loop_time),
            header_size: f64::from(value.header.size),
            u4: f64::from(value.u4),
            u5: f64::from(value.u5),
            u6: f64::from(value.u6),
            u7: f64::from(value.u7),
        }
    }
}

impl JsCue {
    fn to_core(&self) -> Result<Cue> {
        Ok(Cue {
            hot_cue: read_number(
                self.hot_cue,
                u32::MIN as f64,
                u32::MAX as f64,
                "AnlzCue.hot_cue",
            )? as u32,
            status: match &self.status {
                JsCueStatus::Disabled => CueStatus::Disabled,
                JsCueStatus::Active => CueStatus::Active,
            },
            order_first: read_number(
                self.order_first,
                u16::MIN as f64,
                u16::MAX as f64,
                "AnlzCue.order_first",
            )? as u16,
            order_last: read_number(
                self.order_last,
                u16::MIN as f64,
                u16::MAX as f64,
                "AnlzCue.order_last",
            )? as u16,
            cue_type: match &self.cue_type {
                JsCueType::Point => CueType::Point,
                JsCueType::Loop => CueType::Loop,
            },
            time: read_number(self.time, u32::MIN as f64, u32::MAX as f64, "AnlzCue.time")? as u32,
            loop_time: read_number(
                self.loop_time,
                u32::MIN as f64,
                u32::MAX as f64,
                "AnlzCue.loop_time",
            )? as u32,
            u4: read_number(self.u4, u32::MIN as f64, u32::MAX as f64, "AnlzCue.u4")? as u32,
            u5: read_number(self.u5, u32::MIN as f64, u32::MAX as f64, "AnlzCue.u5")? as u32,
            u6: read_number(self.u6, u32::MIN as f64, u32::MAX as f64, "AnlzCue.u6")? as u32,
            u7: read_number(self.u7, u32::MIN as f64, u32::MAX as f64, "AnlzCue.u7")? as u32,
            header: Header {
                tag: AnlzTag::Cue,
                size: read_number(
                    self.header_size,
                    u32::MIN as f64,
                    u32::MAX as f64,
                    "AnlzCue.header_size",
                )? as u32,
                total_size: 56,
            },
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "ExtendedCue")]
pub struct JsExtendedCue {
    pub hot_cue: f64,
    pub cue_type: JsCueType,
    pub time: f64,
    pub loop_time: f64,
    pub color: f64,
    pub loop_numerator: f64,
    pub loop_denominator: f64,
    pub comment: String,
    pub hot_cue_color_index: f64,
    pub hot_cue_color_red: f64,
    pub hot_cue_color_green: f64,
    pub hot_cue_color_blue: f64,
    pub header_size: f64,
    pub u3: f64,
    pub u4: f64,
    pub u5: f64,
    pub comment_trailing_nuls: f64,
    pub trailing_data: Vec<f64>,
}

impl From<&ExtendedCue> for JsExtendedCue {
    fn from(value: &ExtendedCue) -> Self {
        Self {
            hot_cue: f64::from(value.hot_cue),
            cue_type: (&value.cue_type).into(),
            time: f64::from(value.time),
            loop_time: f64::from(value.loop_time),
            color: f64::from(u8::from(value.color)),
            loop_numerator: f64::from(value.loop_numerator),
            loop_denominator: f64::from(value.loop_denominator),
            comment: value.comment.to_string(),
            hot_cue_color_index: f64::from(value.hot_cue_color_index),
            hot_cue_color_red: f64::from(value.hot_cue_color_rgb.0),
            hot_cue_color_green: f64::from(value.hot_cue_color_rgb.1),
            hot_cue_color_blue: f64::from(value.hot_cue_color_rgb.2),
            header_size: f64::from(value.header.size),
            u3: f64::from(value.u3),
            u4: f64::from(value.u4),
            u5: f64::from(value.u5),
            comment_trailing_nuls: f64::from(value.comment.trailing_nuls()),
            trailing_data: value.trailing_data.iter().map(|&v| f64::from(v)).collect(),
        }
    }
}

impl JsExtendedCue {
    fn to_core(&self) -> Result<ExtendedCue> {
        Ok(ExtendedCue {
            hot_cue: read_number(
                self.hot_cue,
                u32::MIN as f64,
                u32::MAX as f64,
                "ExtendedCue.hot_cue",
            )? as u32,
            cue_type: match &self.cue_type {
                JsCueType::Point => CueType::Point,
                JsCueType::Loop => CueType::Loop,
            },
            time: read_number(
                self.time,
                u32::MIN as f64,
                u32::MAX as f64,
                "ExtendedCue.time",
            )? as u32,
            loop_time: read_number(
                self.loop_time,
                u32::MIN as f64,
                u32::MAX as f64,
                "ExtendedCue.loop_time",
            )? as u32,
            color: (read_number(
                self.color,
                u8::MIN as f64,
                u8::MAX as f64,
                "ExtendedCue.color",
            )? as u8)
                .into(),
            loop_numerator: read_number(
                self.loop_numerator,
                u16::MIN as f64,
                u16::MAX as f64,
                "ExtendedCue.loop_numerator",
            )? as u16,
            loop_denominator: read_number(
                self.loop_denominator,
                u16::MIN as f64,
                u16::MAX as f64,
                "ExtendedCue.loop_denominator",
            )? as u16,
            comment: rbox::anlz::AnlzString::with_padding(
                self.comment.clone(),
                read_number(
                    self.comment_trailing_nuls,
                    u32::MIN as f64,
                    u32::MAX as f64,
                    "ExtendedCue.comment_trailing_nuls",
                )? as u32,
            ),
            hot_cue_color_index: read_number(
                self.hot_cue_color_index,
                u8::MIN as f64,
                u8::MAX as f64,
                "ExtendedCue.hot_cue_color_index",
            )? as u8,
            u3: read_number(self.u3, u8::MIN as f64, u8::MAX as f64, "ExtendedCue.u3")? as u8,
            u4: read_number(self.u4, u16::MIN as f64, u16::MAX as f64, "ExtendedCue.u4")? as u16,
            u5: read_number(self.u5, u32::MIN as f64, u32::MAX as f64, "ExtendedCue.u5")? as u32,
            trailing_data: self
                .trailing_data
                .iter()
                .map(|&v| {
                    Ok(read_number(
                        v,
                        u8::MIN as f64,
                        u8::MAX as f64,
                        "ExtendedCue.trailing_data",
                    )? as u8)
                })
                .collect::<Result<Vec<_>>>()?,
            header: Header {
                tag: AnlzTag::ExtendedCue,
                size: read_number(
                    self.header_size,
                    u32::MIN as f64,
                    u32::MAX as f64,
                    "ExtendedCue.header_size",
                )? as u32,
                total_size: 0,
            },
            hot_cue_color_rgb: (
                read_number(
                    self.hot_cue_color_red,
                    u8::MIN as f64,
                    u8::MAX as f64,
                    "ExtendedCue.hot_cue_color_red",
                )? as u8,
                read_number(
                    self.hot_cue_color_green,
                    u8::MIN as f64,
                    u8::MAX as f64,
                    "ExtendedCue.hot_cue_color_green",
                )? as u8,
                read_number(
                    self.hot_cue_color_blue,
                    u8::MIN as f64,
                    u8::MAX as f64,
                    "ExtendedCue.hot_cue_color_blue",
                )? as u8,
            ),
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "CueList")]
pub struct JsCueList {
    pub list_type: JsCueListType,
    pub cues: Vec<JsCue>,
    pub memory_count: f64,
}

impl From<&CueList> for JsCueList {
    fn from(value: &CueList) -> Self {
        Self {
            list_type: (&value.list_type).into(),
            cues: value.cues.iter().map(Into::into).collect(),
            memory_count: f64::from(value.memory_count),
        }
    }
}

impl JsCueList {
    fn to_core(&self) -> Result<CueList> {
        let mut value = CueList::new(
            match &self.list_type {
                JsCueListType::MemoryCues => CueListType::MemoryCues,
                JsCueListType::HotCues => CueListType::HotCues,
            },
            self.cues
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        );
        value.memory_count = read_number(
            self.memory_count,
            i32::MIN as f64,
            i32::MAX as f64,
            "CueList.memory_count",
        )? as i32;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "ExtendedCueList")]
pub struct JsExtendedCueList {
    pub list_type: JsCueListType,
    pub cues: Vec<JsExtendedCue>,
}

impl From<&ExtendedCueList> for JsExtendedCueList {
    fn from(value: &ExtendedCueList) -> Self {
        Self {
            list_type: (&value.list_type).into(),
            cues: value.cues.iter().map(Into::into).collect(),
        }
    }
}

impl JsExtendedCueList {
    fn to_core(&self) -> Result<ExtendedCueList> {
        let value = ExtendedCueList::new(
            match &self.list_type {
                JsCueListType::MemoryCues => CueListType::MemoryCues,
                JsCueListType::HotCues => CueListType::HotCues,
            },
            self.cues
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "TinyWaveformColumn")]
pub struct JsTinyWaveformColumn {
    pub height: f64,
    pub unused: f64,
}

impl From<&TinyWaveformColumn> for JsTinyWaveformColumn {
    fn from(value: &TinyWaveformColumn) -> Self {
        Self {
            height: f64::from(value.height()),
            unused: f64::from(value.unused()),
        }
    }
}

impl JsTinyWaveformColumn {
    fn to_core(&self) -> Result<TinyWaveformColumn> {
        let mut value = TinyWaveformColumn::new();
        value
            .set_height_checked(read_number(
                self.height,
                u8::MIN as f64,
                u8::MAX as f64,
                "TinyWaveformColumn.height",
            )? as u8)
            .map_err(|_| invalid("TinyWaveformColumn.height is out of range"))?;
        value
            .set_unused_checked(read_number(
                self.unused,
                u8::MIN as f64,
                u8::MAX as f64,
                "TinyWaveformColumn.unused",
            )? as u8)
            .map_err(|_| invalid("TinyWaveformColumn.unused is out of range"))?;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "TinyWaveformPreview")]
pub struct JsTinyWaveformPreview {
    pub data: Vec<JsTinyWaveformColumn>,
}

impl From<&TinyWaveformPreview> for JsTinyWaveformPreview {
    fn from(value: &TinyWaveformPreview) -> Self {
        Self {
            data: value.data.iter().map(Into::into).collect(),
        }
    }
}

impl JsTinyWaveformPreview {
    fn to_core(&self) -> Result<TinyWaveformPreview> {
        let value = TinyWaveformPreview::new(
            self.data
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "WaveformColumn")]
pub struct JsWaveformColumn {
    pub height: f64,
    pub whiteness: f64,
}

impl From<&WaveformColumn> for JsWaveformColumn {
    fn from(value: &WaveformColumn) -> Self {
        Self {
            height: f64::from(value.height()),
            whiteness: f64::from(value.whiteness()),
        }
    }
}

impl JsWaveformColumn {
    fn to_core(&self) -> Result<WaveformColumn> {
        let mut value = WaveformColumn::new();
        value
            .set_height_checked(read_number(
                self.height,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColumn.height",
            )? as u8)
            .map_err(|_| invalid("WaveformColumn.height is out of range"))?;
        value
            .set_whiteness_checked(read_number(
                self.whiteness,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColumn.whiteness",
            )? as u8)
            .map_err(|_| invalid("WaveformColumn.whiteness is out of range"))?;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "WaveformPreview")]
pub struct JsWaveformPreview {
    pub data: Vec<JsWaveformColumn>,
}

impl From<&WaveformPreview> for JsWaveformPreview {
    fn from(value: &WaveformPreview) -> Self {
        Self {
            data: value.data.iter().map(Into::into).collect(),
        }
    }
}

impl JsWaveformPreview {
    fn to_core(&self) -> Result<WaveformPreview> {
        let value = WaveformPreview::new(
            self.data
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "WaveformDetail")]
pub struct JsWaveformDetail {
    pub data: Vec<JsWaveformColumn>,
}

impl From<&WaveformDetail> for JsWaveformDetail {
    fn from(value: &WaveformDetail) -> Self {
        Self {
            data: value.data.iter().map(Into::into).collect(),
        }
    }
}

impl JsWaveformDetail {
    fn to_core(&self) -> Result<WaveformDetail> {
        let value = WaveformDetail::new(
            self.data
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "WaveformColorPreviewColumn")]
pub struct JsWaveformColorPreviewColumn {
    pub energy_bottom_half_freq: f64,
    pub energy_bottom_third_freq: f64,
    pub energy_mid_third_freq: f64,
    pub energy_top_third_freq: f64,
    pub u1: f64,
    pub u2: f64,
}

impl From<&WaveformColorPreviewColumn> for JsWaveformColorPreviewColumn {
    fn from(value: &WaveformColorPreviewColumn) -> Self {
        Self {
            energy_bottom_half_freq: f64::from(value.energy_bottom_half_freq),
            energy_bottom_third_freq: f64::from(value.energy_bottom_third_freq),
            energy_mid_third_freq: f64::from(value.energy_mid_third_freq),
            energy_top_third_freq: f64::from(value.energy_top_third_freq),
            u1: f64::from(value.u1),
            u2: f64::from(value.u2),
        }
    }
}

impl JsWaveformColorPreviewColumn {
    fn to_core(&self) -> Result<WaveformColorPreviewColumn> {
        Ok(WaveformColorPreviewColumn {
            energy_bottom_half_freq: read_number(
                self.energy_bottom_half_freq,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorPreviewColumn.energy_bottom_half_freq",
            )? as u8,
            energy_bottom_third_freq: read_number(
                self.energy_bottom_third_freq,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorPreviewColumn.energy_bottom_third_freq",
            )? as u8,
            energy_mid_third_freq: read_number(
                self.energy_mid_third_freq,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorPreviewColumn.energy_mid_third_freq",
            )? as u8,
            energy_top_third_freq: read_number(
                self.energy_top_third_freq,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorPreviewColumn.energy_top_third_freq",
            )? as u8,
            u1: read_number(
                self.u1,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorPreviewColumn.u1",
            )? as u8,
            u2: read_number(
                self.u2,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorPreviewColumn.u2",
            )? as u8,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "WaveformColorPreview")]
pub struct JsWaveformColorPreview {
    pub data: Vec<JsWaveformColorPreviewColumn>,
}

impl From<&WaveformColorPreview> for JsWaveformColorPreview {
    fn from(value: &WaveformColorPreview) -> Self {
        Self {
            data: value.data.iter().map(Into::into).collect(),
        }
    }
}

impl JsWaveformColorPreview {
    fn to_core(&self) -> Result<WaveformColorPreview> {
        let value = WaveformColorPreview::new(
            self.data
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "WaveformColorDetailColumn")]
pub struct JsWaveformColorDetailColumn {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub height: f64,
    pub low_bits: f64,
    pub fine_height_substep: Option<f64>,
    pub full_height: Option<f64>,
}

impl From<&WaveformColorDetailColumn> for JsWaveformColorDetailColumn {
    fn from(value: &WaveformColorDetailColumn) -> Self {
        Self {
            red: f64::from(value.red()),
            green: f64::from(value.green()),
            blue: f64::from(value.blue()),
            height: f64::from(value.height()),
            low_bits: f64::from(value.low_bits()),
            fine_height_substep: Some(f64::from(value.fine_height_substep())),
            full_height: Some(f64::from(value.full_height())),
        }
    }
}

impl JsWaveformColorDetailColumn {
    fn to_core(&self) -> Result<WaveformColorDetailColumn> {
        let mut value = WaveformColorDetailColumn::new();
        value
            .set_low_bits_checked(read_number(
                self.low_bits,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorDetailColumn.low_bits",
            )? as u8)
            .map_err(|_| invalid("WaveformColorDetailColumn.low_bits is out of range"))?;
        value
            .set_height_checked(read_number(
                self.height,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorDetailColumn.height",
            )? as u8)
            .map_err(|_| invalid("WaveformColorDetailColumn.height is out of range"))?;
        value
            .set_red_checked(read_number(
                self.red,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorDetailColumn.red",
            )? as u8)
            .map_err(|_| invalid("WaveformColorDetailColumn.red is out of range"))?;
        value
            .set_green_checked(read_number(
                self.green,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorDetailColumn.green",
            )? as u8)
            .map_err(|_| invalid("WaveformColorDetailColumn.green is out of range"))?;
        value
            .set_blue_checked(read_number(
                self.blue,
                u8::MIN as f64,
                u8::MAX as f64,
                "WaveformColorDetailColumn.blue",
            )? as u8)
            .map_err(|_| invalid("WaveformColorDetailColumn.blue is out of range"))?;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "WaveformColorDetail")]
pub struct JsWaveformColorDetail {
    pub data: Vec<JsWaveformColorDetailColumn>,
    pub unknown: f64,
}

impl From<&WaveformColorDetail> for JsWaveformColorDetail {
    fn from(value: &WaveformColorDetail) -> Self {
        Self {
            data: value.data.iter().map(Into::into).collect(),
            unknown: f64::from(value.unknown),
        }
    }
}

impl JsWaveformColorDetail {
    fn to_core(&self) -> Result<WaveformColorDetail> {
        let mut value = WaveformColorDetail::new(
            self.data
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        );
        value.unknown = read_number(
            self.unknown,
            u32::MIN as f64,
            u32::MAX as f64,
            "WaveformColorDetail.unknown",
        )? as u32;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "Waveform3BandColumn")]
pub struct JsWaveform3BandColumn {
    pub mid: f64,
    pub high: f64,
    pub low: f64,
}

impl From<&Waveform3BandColumn> for JsWaveform3BandColumn {
    fn from(value: &Waveform3BandColumn) -> Self {
        Self {
            mid: f64::from(value.mid()),
            high: f64::from(value.high()),
            low: f64::from(value.low()),
        }
    }
}

impl JsWaveform3BandColumn {
    fn to_core(&self) -> Result<Waveform3BandColumn> {
        let mut value = Waveform3BandColumn::new();
        value
            .set_low_checked(read_number(
                self.low,
                u8::MIN as f64,
                u8::MAX as f64,
                "Waveform3BandColumn.low",
            )? as u8)
            .map_err(|_| invalid("Waveform3BandColumn.low is out of range"))?;
        value
            .set_mid_checked(read_number(
                self.mid,
                u8::MIN as f64,
                u8::MAX as f64,
                "Waveform3BandColumn.mid",
            )? as u8)
            .map_err(|_| invalid("Waveform3BandColumn.mid is out of range"))?;
        value
            .set_high_checked(read_number(
                self.high,
                u8::MIN as f64,
                u8::MAX as f64,
                "Waveform3BandColumn.high",
            )? as u8)
            .map_err(|_| invalid("Waveform3BandColumn.high is out of range"))?;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "Waveform3BandPreview")]
pub struct JsWaveform3BandPreview {
    pub data: Vec<JsWaveform3BandColumn>,
}

impl From<&Waveform3BandPreview> for JsWaveform3BandPreview {
    fn from(value: &Waveform3BandPreview) -> Self {
        Self {
            data: value.data.iter().map(Into::into).collect(),
        }
    }
}

impl JsWaveform3BandPreview {
    fn to_core(&self) -> Result<Waveform3BandPreview> {
        let value = Waveform3BandPreview::new(
            self.data
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "Waveform3BandDetail")]
pub struct JsWaveform3BandDetail {
    pub data: Vec<JsWaveform3BandColumn>,
    pub unknown: f64,
}

impl From<&Waveform3BandDetail> for JsWaveform3BandDetail {
    fn from(value: &Waveform3BandDetail) -> Self {
        Self {
            data: value.data.iter().map(Into::into).collect(),
            unknown: f64::from(value.unknown),
        }
    }
}

impl JsWaveform3BandDetail {
    fn to_core(&self) -> Result<Waveform3BandDetail> {
        let mut value = Waveform3BandDetail::new(
            self.data
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        );
        value.unknown = read_number(
            self.unknown,
            u32::MIN as f64,
            u32::MAX as f64,
            "Waveform3BandDetail.unknown",
        )? as u32;
        Ok(value)
    }
}

#[napi(js_name = "Mood")]
pub enum JsMood {
    High = 1,
    Mid,
    Low,
}

impl Into<JsMood> for &Mood {
    fn into(self) -> JsMood {
        match self {
            Mood::High => JsMood::High,
            Mood::Mid => JsMood::Mid,
            Mood::Low => JsMood::Low,
        }
    }
}

#[napi(js_name = "Bank")]
pub enum JsBank {
    Default = 0,
    Cool,
    Natural,
    Hot,
    Subtle,
    Warm,
    Vivid,
    Club1,
    Club2,
}

impl Into<JsBank> for &Bank {
    fn into(self) -> JsBank {
        match self {
            Bank::Default => JsBank::Default,
            Bank::Cool => JsBank::Cool,
            Bank::Natural => JsBank::Natural,
            Bank::Hot => JsBank::Hot,
            Bank::Subtle => JsBank::Subtle,
            Bank::Warm => JsBank::Warm,
            Bank::Vivid => JsBank::Vivid,
            Bank::Club1 => JsBank::Club1,
            Bank::Club2 => JsBank::Club2,
        }
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "Phrase")]
pub struct JsPhrase {
    pub index: f64,
    pub beat: f64,
    pub kind: f64,
    pub k1: f64,
    pub k2: f64,
    pub b: f64,
    pub beat2: f64,
    pub beat3: f64,
    pub beat4: f64,
    pub k3: f64,
    pub fill: f64,
    pub beat_fill: f64,
    pub u1: f64,
    pub u2: f64,
    pub u3: f64,
    pub u4: f64,
    pub u5: f64,
}

impl From<&Phrase> for JsPhrase {
    fn from(value: &Phrase) -> Self {
        Self {
            index: f64::from(value.index),
            beat: f64::from(value.beat),
            kind: f64::from(value.kind),
            k1: f64::from(value.k1),
            k2: f64::from(value.k2),
            b: f64::from(value.b),
            beat2: f64::from(value.beat2),
            beat3: f64::from(value.beat3),
            beat4: f64::from(value.beat4),
            k3: f64::from(value.k3),
            fill: f64::from(value.fill),
            beat_fill: f64::from(value.beat_fill),
            u1: f64::from(value.u1),
            u2: f64::from(value.u2),
            u3: f64::from(value.u3),
            u4: f64::from(value.u4),
            u5: f64::from(value.u5),
        }
    }
}

impl JsPhrase {
    fn to_core(&self) -> Result<Phrase> {
        Ok(Phrase {
            index: read_number(self.index, u16::MIN as f64, u16::MAX as f64, "Phrase.index")?
                as u16,
            beat: read_number(self.beat, u16::MIN as f64, u16::MAX as f64, "Phrase.beat")? as u16,
            kind: read_number(self.kind, u16::MIN as f64, u16::MAX as f64, "Phrase.kind")? as u16,
            k1: read_number(self.k1, u8::MIN as f64, u8::MAX as f64, "Phrase.k1")? as u8,
            k2: read_number(self.k2, u8::MIN as f64, u8::MAX as f64, "Phrase.k2")? as u8,
            b: read_number(self.b, u8::MIN as f64, u8::MAX as f64, "Phrase.b")? as u8,
            beat2: read_number(self.beat2, u16::MIN as f64, u16::MAX as f64, "Phrase.beat2")?
                as u16,
            beat3: read_number(self.beat3, u16::MIN as f64, u16::MAX as f64, "Phrase.beat3")?
                as u16,
            beat4: read_number(self.beat4, u16::MIN as f64, u16::MAX as f64, "Phrase.beat4")?
                as u16,
            k3: read_number(self.k3, u8::MIN as f64, u8::MAX as f64, "Phrase.k3")? as u8,
            fill: read_number(self.fill, u8::MIN as f64, u8::MAX as f64, "Phrase.fill")? as u8,
            beat_fill: read_number(
                self.beat_fill,
                u16::MIN as f64,
                u16::MAX as f64,
                "Phrase.beat_fill",
            )? as u16,
            u1: read_number(self.u1, u8::MIN as f64, u8::MAX as f64, "Phrase.u1")? as u8,
            u2: read_number(self.u2, u8::MIN as f64, u8::MAX as f64, "Phrase.u2")? as u8,
            u3: read_number(self.u3, u8::MIN as f64, u8::MAX as f64, "Phrase.u3")? as u8,
            u4: read_number(self.u4, u8::MIN as f64, u8::MAX as f64, "Phrase.u4")? as u8,
            u5: read_number(self.u5, u8::MIN as f64, u8::MAX as f64, "Phrase.u5")? as u8,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "SongStructureData")]
pub struct JsSongStructureData {
    pub mood: JsMood,
    pub end_beat: f64,
    pub bank: JsBank,
    pub phrases: Vec<JsPhrase>,
    pub u1: f64,
    pub u2: f64,
    pub u3: f64,
    pub u4: f64,
}

impl From<&SongStructureData> for JsSongStructureData {
    fn from(value: &SongStructureData) -> Self {
        Self {
            mood: (&value.mood).into(),
            end_beat: f64::from(value.end_beat),
            bank: (&value.bank).into(),
            phrases: value.phrases.iter().map(Into::into).collect(),
            u1: f64::from(value.u1),
            u2: f64::from(value.u2),
            u3: f64::from(value.u3),
            u4: f64::from(value.u4),
        }
    }
}

impl JsSongStructureData {
    fn to_core(&self) -> Result<SongStructureData> {
        Ok(SongStructureData {
            mood: match &self.mood {
                JsMood::High => Mood::High,
                JsMood::Mid => Mood::Mid,
                JsMood::Low => Mood::Low,
            },
            end_beat: read_number(
                self.end_beat,
                u16::MIN as f64,
                u16::MAX as f64,
                "SongStructureData.end_beat",
            )? as u16,
            bank: match &self.bank {
                JsBank::Default => Bank::Default,
                JsBank::Cool => Bank::Cool,
                JsBank::Natural => Bank::Natural,
                JsBank::Hot => Bank::Hot,
                JsBank::Subtle => Bank::Subtle,
                JsBank::Warm => Bank::Warm,
                JsBank::Vivid => Bank::Vivid,
                JsBank::Club1 => Bank::Club1,
                JsBank::Club2 => Bank::Club2,
            },
            phrases: self
                .phrases
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
            u1: read_number(
                self.u1,
                u32::MIN as f64,
                u32::MAX as f64,
                "SongStructureData.u1",
            )? as u32,
            u2: read_number(
                self.u2,
                u16::MIN as f64,
                u16::MAX as f64,
                "SongStructureData.u2",
            )? as u16,
            u3: read_number(
                self.u3,
                u16::MIN as f64,
                u16::MAX as f64,
                "SongStructureData.u3",
            )? as u16,
            u4: read_number(
                self.u4,
                u8::MIN as f64,
                u8::MAX as f64,
                "SongStructureData.u4",
            )? as u8,
        })
    }
}

#[napi(object, js_name = "AnlzPaths")]
pub struct JsAnlzPaths {
    pub dat: String,
    pub ext: Option<String>,
    pub ex2: Option<String>,
}

impl From<AnlzPaths> for JsAnlzPaths {
    fn from(model: AnlzPaths) -> Self {
        JsAnlzPaths {
            dat: model.dat.as_os_str().to_str().unwrap().to_string(),
            ext: model.ext.as_ref().map(|s| s.to_str().unwrap().to_string()),
            ex2: model.ex2.as_ref().map(|s| s.to_str().unwrap().to_string()),
        }
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "VBR")]
pub struct JsVBR {
    pub u1: f64,
    pub index: Vec<f64>,
    pub u2: f64,
}

impl From<&VBR> for JsVBR {
    fn from(value: &VBR) -> Self {
        Self {
            u1: f64::from(value.u1),
            index: value.index.iter().map(|&v| f64::from(v)).collect(),
            u2: f64::from(value.u2),
        }
    }
}

impl JsVBR {
    fn to_core(&self) -> Result<VBR> {
        Ok(VBR {
            u1: read_number(self.u1, u32::MIN as f64, u32::MAX as f64, "VBR.u1")? as u32,
            index: self
                .index
                .iter()
                .map(|&v| Ok(read_number(v, u32::MIN as f64, u32::MAX as f64, "VBR.index")? as u32))
                .collect::<Result<Vec<_>>>()?
                .try_into()
                .map_err(|_| invalid("VBR index must contain exactly 400 values"))?,
            u2: read_number(self.u2, u32::MIN as f64, u32::MAX as f64, "VBR.u2")? as u32,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "SeekPoint")]
pub struct JsSeekPoint {
    pub sample: napi::bindgen_prelude::BigInt,
    pub offset: napi::bindgen_prelude::BigInt,
    pub frame_samples: f64,
}

impl From<&SeekPoint> for JsSeekPoint {
    fn from(value: &SeekPoint) -> Self {
        Self {
            sample: value.sample.into(),
            offset: value.offset.into(),
            frame_samples: f64::from(value.frame_samples),
        }
    }
}

impl JsSeekPoint {
    fn to_core(&self) -> Result<SeekPoint> {
        Ok(SeekPoint {
            sample: read_u64(&self.sample)?,
            offset: read_u64(&self.offset)?,
            frame_samples: read_number(
                self.frame_samples,
                u32::MIN as f64,
                u32::MAX as f64,
                "SeekPoint.frame_samples",
            )? as u32,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "SeekIndex")]
pub struct JsSeekIndex {
    pub u1: f64,
    pub total_samples: napi::bindgen_prelude::BigInt,
    pub entries: Vec<JsSeekPoint>,
}

impl From<&SeekIndex> for JsSeekIndex {
    fn from(value: &SeekIndex) -> Self {
        Self {
            u1: f64::from(value.u1),
            total_samples: value.total_samples.into(),
            entries: value.entries.iter().map(Into::into).collect(),
        }
    }
}

impl JsSeekIndex {
    fn to_core(&self) -> Result<SeekIndex> {
        Ok(SeekIndex {
            u1: read_number(self.u1, u32::MIN as f64, u32::MAX as f64, "SeekIndex.u1")? as u32,
            total_samples: read_u64(&self.total_samples)?,
            entries: self
                .entries
                .iter()
                .map(|v| v.to_core())
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "Waveform3BandCalibration")]
pub struct JsWaveform3BandCalibration {
    pub low_gain: f64,
    pub mid_gain: f64,
    pub high_gain: f64,
}

impl From<&Waveform3BandCalibration> for JsWaveform3BandCalibration {
    fn from(value: &Waveform3BandCalibration) -> Self {
        Self {
            low_gain: f64::from(value.low_gain),
            mid_gain: f64::from(value.mid_gain),
            high_gain: f64::from(value.high_gain),
        }
    }
}

impl JsWaveform3BandCalibration {
    fn to_core(&self) -> Result<Waveform3BandCalibration> {
        Ok(Waveform3BandCalibration {
            low_gain: read_number(
                self.low_gain,
                u16::MIN as f64,
                u16::MAX as f64,
                "Waveform3BandCalibration.low_gain",
            )? as u16,
            mid_gain: read_number(
                self.mid_gain,
                u16::MIN as f64,
                u16::MAX as f64,
                "Waveform3BandCalibration.mid_gain",
            )? as u16,
            high_gain: read_number(
                self.high_gain,
                u16::MIN as f64,
                u16::MAX as f64,
                "Waveform3BandCalibration.high_gain",
            )? as u16,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "VocalDetection")]
pub struct JsVocalDetection {
    pub u1: f64,
    pub u2: f64,
    pub confidence: Vec<f64>,
}

impl From<&VocalDetection> for JsVocalDetection {
    fn from(value: &VocalDetection) -> Self {
        Self {
            u1: f64::from(value.u1),
            u2: f64::from(value.u2),
            confidence: value.confidence.iter().map(|&v| f64::from(v)).collect(),
        }
    }
}

impl JsVocalDetection {
    fn to_core(&self) -> Result<VocalDetection> {
        Ok(VocalDetection {
            u1: read_number(
                self.u1,
                u32::MIN as f64,
                u32::MAX as f64,
                "VocalDetection.u1",
            )? as u32,
            u2: read_number(
                self.u2,
                u32::MIN as f64,
                u32::MAX as f64,
                "VocalDetection.u2",
            )? as u32,
            confidence: self
                .confidence
                .iter()
                .map(|&v| {
                    Ok(read_number(
                        v,
                        u8::MIN as f64,
                        u8::MAX as f64,
                        "VocalDetection.confidence",
                    )? as u8)
                })
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[napi(object, js_name = "SongStructure")]
pub struct JsSongStructure {
    pub version: f64,
    pub is_encrypted: bool,
    pub data: JsSongStructureData,
}

impl From<&SongStructure> for JsSongStructure {
    fn from(value: &SongStructure) -> Self {
        Self {
            version: f64::from(value.version),
            is_encrypted: value.is_encrypted,
            data: (&value.data).into(),
        }
    }
}

impl JsSongStructure {
    fn to_core(&self) -> Result<SongStructure> {
        let value = SongStructure::new(
            read_number(
                self.version,
                u16::MIN as f64,
                u16::MAX as f64,
                "SongStructure.version",
            )? as u16,
            self.is_encrypted,
            self.data.to_core()?,
        );

        Ok(value)
    }
}

/// Opaque section bytes, including the exact four-byte identifier.
#[napi(object, js_name = "UnknownSection")]
pub struct JsUnknownSection {
    pub code: Vec<f64>,
    pub header_data: Vec<f64>,
    pub content_data: Vec<f64>,
}

impl JsUnknownSection {
    fn to_core(&self) -> Result<Section> {
        let bytes = |values: &[f64]| -> Result<Vec<u8>> {
            values
                .iter()
                .map(|&v| Ok(read_number(v, 0.0, 255.0, "unknown section byte")? as u8))
                .collect()
        };
        let code: [u8; 4] = bytes(&self.code)?
            .try_into()
            .map_err(|_| invalid("unknown tag code must contain four bytes"))?;
        let tag = AnlzTag::from_code(code);
        if !matches!(tag, AnlzTag::Unknown(_)) {
            return Err(invalid("known tags require typed content"));
        }
        Ok(Section {
            tag,
            content: Content::Unknown(
                Unknown::new(bytes(&self.header_data)?, bytes(&self.content_data)?)
                    .map_err(crate::util::binding_error)?,
            ),
        })
    }
}

/// Tagged, editable snapshot of one section. Unknown tags retain their original code and bytes.
#[napi(discriminant = "tag", js_name = "AnlzSection")]
pub enum JsSection {
    PQTZ { content: JsBeatGrid },
    PQT2 { content: JsExtendedBeatGrid },
    PCOB { content: JsCueList },
    PCO2 { content: JsExtendedCueList },
    PPTH { content: String },
    PVBR { content: JsVBR },
    PVB2 { content: JsSeekIndex },
    PWV2 { content: JsTinyWaveformPreview },
    PWAV { content: JsWaveformPreview },
    PWV3 { content: JsWaveformDetail },
    PWV4 { content: JsWaveformColorPreview },
    PWV5 { content: JsWaveformColorDetail },
    PWV6 { content: JsWaveform3BandPreview },
    PWV7 { content: JsWaveform3BandDetail },
    PWVC { content: JsWaveform3BandCalibration },
    PSSI { content: JsSongStructure },
    PVDI { content: JsVocalDetection },
    Unknown { content: JsUnknownSection },
}

impl JsSection {
    fn from_core(section: &Section) -> Result<Self> {
        Ok(match &section.content {
            Content::BeatGrid(value) => Self::PQTZ {
                content: value.into(),
            },
            Content::ExtendedBeatGrid(value) => Self::PQT2 {
                content: value.into(),
            },
            Content::CueList(value) => Self::PCOB {
                content: value.into(),
            },
            Content::ExtendedCueList(value) => Self::PCO2 {
                content: value.into(),
            },
            Content::Path(value) => Self::PPTH {
                content: value.path.to_string(),
            },
            Content::VBR(value) => Self::PVBR {
                content: value.into(),
            },
            Content::SeekIndex(value) => Self::PVB2 {
                content: value.into(),
            },
            Content::TinyWaveformPreview(value) => Self::PWV2 {
                content: value.into(),
            },
            Content::WaveformPreview(value) => Self::PWAV {
                content: value.into(),
            },
            Content::WaveformDetail(value) => Self::PWV3 {
                content: value.into(),
            },
            Content::WaveformColorPreview(value) => Self::PWV4 {
                content: value.into(),
            },
            Content::WaveformColorDetail(value) => Self::PWV5 {
                content: value.into(),
            },
            Content::Waveform3BandPreview(value) => Self::PWV6 {
                content: value.into(),
            },
            Content::Waveform3BandDetail(value) => Self::PWV7 {
                content: value.into(),
            },
            Content::Waveform3BandCalibration(value) => Self::PWVC {
                content: value.into(),
            },
            Content::SongStructure(value) => Self::PSSI {
                content: value.into(),
            },
            Content::VocalDetection(value) => Self::PVDI {
                content: value.into(),
            },
            Content::Unknown(value) => Self::Unknown {
                content: JsUnknownSection {
                    code: section.tag.code().iter().map(|&v| f64::from(v)).collect(),
                    header_data: value.header_data().iter().map(|&v| f64::from(v)).collect(),
                    content_data: value.content_data().iter().map(|&v| f64::from(v)).collect(),
                },
            },
        })
    }

    fn to_core(&self) -> Result<Section> {
        let section = match self {
            Self::PQTZ { content } => Section {
                tag: AnlzTag::BeatGrid,
                content: Content::BeatGrid(content.to_core()?),
            },
            Self::PQT2 { content } => Section {
                tag: AnlzTag::ExtendedBeatGrid,
                content: Content::ExtendedBeatGrid(content.to_core()?),
            },
            Self::PCOB { content } => Section {
                tag: AnlzTag::CueList,
                content: Content::CueList(content.to_core()?),
            },
            Self::PCO2 { content } => Section {
                tag: AnlzTag::ExtendedCueList,
                content: Content::ExtendedCueList(content.to_core()?),
            },
            Self::PPTH { content } => Section {
                tag: AnlzTag::Path,
                content: Content::Path(Path::new(content)),
            },
            Self::PVBR { content } => Section {
                tag: AnlzTag::VBR,
                content: Content::VBR(content.to_core()?),
            },
            Self::PVB2 { content } => Section {
                tag: AnlzTag::SeekIndex,
                content: Content::SeekIndex(content.to_core()?),
            },
            Self::PWV2 { content } => Section {
                tag: AnlzTag::TinyWaveformPreview,
                content: Content::TinyWaveformPreview(content.to_core()?),
            },
            Self::PWAV { content } => Section {
                tag: AnlzTag::WaveformPreview,
                content: Content::WaveformPreview(content.to_core()?),
            },
            Self::PWV3 { content } => Section {
                tag: AnlzTag::WaveformDetail,
                content: Content::WaveformDetail(content.to_core()?),
            },
            Self::PWV4 { content } => Section {
                tag: AnlzTag::WaveformColorPreview,
                content: Content::WaveformColorPreview(content.to_core()?),
            },
            Self::PWV5 { content } => Section {
                tag: AnlzTag::WaveformColorDetail,
                content: Content::WaveformColorDetail(content.to_core()?),
            },
            Self::PWV6 { content } => Section {
                tag: AnlzTag::Waveform3BandPreview,
                content: Content::Waveform3BandPreview(content.to_core()?),
            },
            Self::PWV7 { content } => Section {
                tag: AnlzTag::Waveform3BandDetail,
                content: Content::Waveform3BandDetail(content.to_core()?),
            },
            Self::PWVC { content } => Section {
                tag: AnlzTag::Waveform3BandCalibration,
                content: Content::Waveform3BandCalibration(content.to_core()?),
            },
            Self::PSSI { content } => Section {
                tag: AnlzTag::SongStructure,
                content: Content::SongStructure(content.to_core()?),
            },
            Self::PVDI { content } => Section {
                tag: AnlzTag::VocalDetection,
                content: Content::VocalDetection(content.to_core()?),
            },
            Self::Unknown { content } => content.to_core()?,
        };
        section.validate().map_err(crate::util::binding_error)?;
        Ok(section)
    }
}

/// Computed PMAI header information in a complete snapshot.
#[napi(object, js_name = "AnlzHeader")]
pub struct JsAnlzHeader {
    pub code: Vec<u8>,
    pub size: u32,
    pub total_size: u32,
}

/// Complete eager snapshot. Calling getData explicitly loads all sections.
#[napi(object, js_name = "AnlzData")]
pub struct JsAnlzData {
    pub header: JsAnlzHeader,
    pub header_data: Vec<u8>,
    pub sections: Vec<JsSection>,
}

#[napi(js_name = "Anlz")]
pub struct JsAnlz {
    anlz: Anlz,
}

#[napi]
impl JsAnlz {
    /// Owned snapshot equivalent of Rust's into_data; the binding handler remains usable.
    #[napi]
    pub fn into_data(&mut self) -> Result<JsAnlzData> {
        self.get_data()
    }

    /// Include version and encoding settings when editing song structure.
    #[napi]
    pub fn get_song_structure_section(&mut self) -> Result<Option<JsSongStructure>> {
        Ok(self
            .anlz
            .get_section_by_tag(AnlzTag::SongStructure)
            .map_err(crate::util::binding_error)?
            .and_then(|section| match &section.content {
                Content::SongStructure(value) => Some(value.into()),
                _ => None,
            }))
    }

    /// Update phrase data while retaining the existing version and encoding.
    #[napi]
    pub fn set_song_structure_data(&mut self, value: JsSongStructureData) -> Result<()> {
        let data = value.to_core()?;
        let section = self
            .anlz
            .get_section_by_tag(AnlzTag::SongStructure)
            .map_err(crate::util::binding_error)?
            .ok_or_else(|| {
                invalid("song structure is absent; use setSongStructure to create it")
            })?;
        let Content::SongStructure(mut value) = section.content.clone() else {
            return Err(invalid("song structure content mismatch"));
        };
        value.data = data;
        Section {
            tag: AnlzTag::SongStructure,
            content: Content::SongStructure(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_song_structure(value)
            .map_err(crate::util::binding_error)
    }
    /// Create an empty file in memory; dump writes it.
    #[napi(factory, js_name = "newDat")]
    pub fn new_dat(path: String) -> Result<Self> {
        Ok(Self {
            anlz: Anlz::new_dat(path).map_err(crate::util::binding_error)?,
        })
    }

    /// Create an empty file in memory; dump writes it.
    #[napi(factory, js_name = "newExt")]
    pub fn new_ext(path: String) -> Result<Self> {
        Ok(Self {
            anlz: Anlz::new_ext(path).map_err(crate::util::binding_error)?,
        })
    }

    /// Create an empty file in memory; dump writes it.
    #[napi(factory, js_name = "new2ex")]
    pub fn new_2ex(path: String) -> Result<Self> {
        Ok(Self {
            anlz: Anlz::new_2ex(path).map_err(crate::util::binding_error)?,
        })
    }

    #[napi(factory)]
    pub fn load(path: String) -> Result<Self> {
        Self::new(path)
    }

    #[napi]
    pub fn section_count(&self) -> u32 {
        self.anlz.section_count() as u32
    }

    #[napi]
    pub fn contains(&self, tag: String) -> Result<bool> {
        Ok(self
            .anlz
            .contains(AnlzTag::parse(&tag).map_err(crate::util::binding_error)?))
    }

    #[napi]
    pub fn get_section(&mut self, index: f64) -> Result<Option<JsSection>> {
        self.anlz
            .get_section(read_index(index)?)
            .map_err(crate::util::binding_error)?
            .map(JsSection::from_core)
            .transpose()
    }

    #[napi]
    pub fn get_section_by_tag(&mut self, tag: String) -> Result<Option<JsSection>> {
        let tag = AnlzTag::parse(&tag).map_err(crate::util::binding_error)?;
        self.anlz
            .get_section_by_tag(tag)
            .map_err(crate::util::binding_error)?
            .map(JsSection::from_core)
            .transpose()
    }

    #[napi]
    pub fn get_sections_by_tag(&mut self, tag: String) -> Result<Vec<JsSection>> {
        let tag = AnlzTag::parse(&tag).map_err(crate::util::binding_error)?;
        self.anlz
            .get_sections_by_tag(tag)
            .map_err(crate::util::binding_error)?
            .into_iter()
            .map(JsSection::from_core)
            .collect()
    }

    /// Read the complete file without consuming or modifying the handler.
    #[napi]
    pub fn get_data(&mut self) -> Result<JsAnlzData> {
        let value = self.anlz.get_data().map_err(crate::util::binding_error)?;
        Ok(JsAnlzData {
            header: JsAnlzHeader {
                code: value.header.tag.code().to_vec(),
                size: value.header.size,
                total_size: value.header.total_size,
            },
            header_data: value.header_data,
            sections: value
                .sections
                .iter()
                .map(JsSection::from_core)
                .collect::<Result<_>>()?,
        })
    }

    #[napi]
    pub fn insert_section(&mut self, index: f64, section: JsSection) -> Result<()> {
        self.anlz
            .insert_section(read_index(index)?, section.to_core()?)
            .map_err(crate::util::binding_error)
    }

    #[napi]
    pub fn set_section_at(&mut self, index: f64, section: JsSection) -> Result<()> {
        self.anlz
            .set_section_at(read_index(index)?, section.to_core()?)
            .map_err(crate::util::binding_error)
    }

    #[napi]
    pub fn push_section(&mut self, section: JsSection) -> Result<()> {
        self.anlz
            .push_section(section.to_core()?)
            .map_err(crate::util::binding_error)
    }

    #[napi]
    pub fn set_section(&mut self, section: JsSection) -> Result<()> {
        self.anlz
            .set_section(section.to_core()?)
            .map_err(crate::util::binding_error)
    }

    #[napi]
    pub fn remove_section(&mut self, index: f64) -> Result<()> {
        self.anlz
            .remove_section(read_index(index)?)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_section_by_tag(&mut self, tag: String) -> Result<bool> {
        Ok(self
            .anlz
            .remove_section_by_tag(AnlzTag::parse(&tag).map_err(crate::util::binding_error)?))
    }
    #[napi]
    pub fn remove_sections_by_tag(&mut self, tag: String) -> Result<u32> {
        Ok(self
            .anlz
            .remove_sections_by_tag(AnlzTag::parse(&tag).map_err(crate::util::binding_error)?)
            as u32)
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_beat_grid(&mut self, value: JsBeatGrid) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::BeatGrid,
            content: Content::BeatGrid(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_beat_grid(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_beat_grid(&mut self) -> bool {
        self.anlz.remove_beat_grid()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_extended_beat_grid(&mut self, value: JsExtendedBeatGrid) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::ExtendedBeatGrid,
            content: Content::ExtendedBeatGrid(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_extended_beat_grid(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_extended_beat_grid(&mut self) -> bool {
        self.anlz.remove_extended_beat_grid()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_cue_list(&mut self, value: JsCueList) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::CueList,
            content: Content::CueList(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_cue_list(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_cue_list(&mut self) -> bool {
        self.anlz.remove_cue_list()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_extended_cue_list(&mut self, value: JsExtendedCueList) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::ExtendedCueList,
            content: Content::ExtendedCueList(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_extended_cue_list(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_extended_cue_list(&mut self) -> bool {
        self.anlz.remove_extended_cue_list()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_vbr(&mut self, value: JsVBR) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::VBR,
            content: Content::VBR(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz.set_vbr(value).map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_vbr(&mut self) -> bool {
        self.anlz.remove_vbr()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_seek_index(&mut self, value: JsSeekIndex) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::SeekIndex,
            content: Content::SeekIndex(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_seek_index(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_seek_index(&mut self) -> bool {
        self.anlz.remove_seek_index()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_tiny_waveform_preview(&mut self, value: JsTinyWaveformPreview) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::TinyWaveformPreview,
            content: Content::TinyWaveformPreview(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_tiny_waveform_preview(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_tiny_waveform_preview(&mut self) -> bool {
        self.anlz.remove_tiny_waveform_preview()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_waveform_preview(&mut self, value: JsWaveformPreview) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::WaveformPreview,
            content: Content::WaveformPreview(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_waveform_preview(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_waveform_preview(&mut self) -> bool {
        self.anlz.remove_waveform_preview()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_waveform_detail(&mut self, value: JsWaveformDetail) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::WaveformDetail,
            content: Content::WaveformDetail(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_waveform_detail(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_waveform_detail(&mut self) -> bool {
        self.anlz.remove_waveform_detail()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_waveform_color_preview(&mut self, value: JsWaveformColorPreview) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::WaveformColorPreview,
            content: Content::WaveformColorPreview(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_waveform_color_preview(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_waveform_color_preview(&mut self) -> bool {
        self.anlz.remove_waveform_color_preview()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_waveform_color_detail(&mut self, value: JsWaveformColorDetail) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::WaveformColorDetail,
            content: Content::WaveformColorDetail(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_waveform_color_detail(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_waveform_color_detail(&mut self) -> bool {
        self.anlz.remove_waveform_color_detail()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_waveform_3band_preview(&mut self, value: JsWaveform3BandPreview) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::Waveform3BandPreview,
            content: Content::Waveform3BandPreview(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_waveform_3band_preview(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_waveform_3band_preview(&mut self) -> bool {
        self.anlz.remove_waveform_3band_preview()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_waveform_3band_detail(&mut self, value: JsWaveform3BandDetail) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::Waveform3BandDetail,
            content: Content::Waveform3BandDetail(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_waveform_3band_detail(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_waveform_3band_detail(&mut self) -> bool {
        self.anlz.remove_waveform_3band_detail()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_waveform_3band_calibration(
        &mut self,
        value: JsWaveform3BandCalibration,
    ) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::Waveform3BandCalibration,
            content: Content::Waveform3BandCalibration(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_waveform_3band_calibration(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_waveform_3band_calibration(&mut self) -> bool {
        self.anlz.remove_waveform_3band_calibration()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_song_structure(&mut self, value: JsSongStructure) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::SongStructure,
            content: Content::SongStructure(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_song_structure(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_song_structure(&mut self) -> bool {
        self.anlz.remove_song_structure()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_vocal_detection(&mut self, value: JsVocalDetection) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::VocalDetection,
            content: Content::VocalDetection(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_vocal_detection(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_vocal_detection(&mut self) -> bool {
        self.anlz.remove_vocal_detection()
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_hot_cues(&mut self, value: JsCueList) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::CueList,
            content: Content::CueList(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_hot_cues(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_hot_cues(&mut self) -> Result<bool> {
        self.anlz
            .remove_hot_cues()
            .map_err(crate::util::binding_error)
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_memory_cues(&mut self, value: JsCueList) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::CueList,
            content: Content::CueList(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_memory_cues(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_memory_cues(&mut self) -> Result<bool> {
        self.anlz
            .remove_memory_cues()
            .map_err(crate::util::binding_error)
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_extended_hot_cues(&mut self, value: JsExtendedCueList) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::ExtendedCueList,
            content: Content::ExtendedCueList(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_extended_hot_cues(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_extended_hot_cues(&mut self) -> Result<bool> {
        self.anlz
            .remove_extended_hot_cues()
            .map_err(crate::util::binding_error)
    }

    /// Replace or insert this section using Rust's section ordering.
    #[napi]
    pub fn set_extended_memory_cues(&mut self, value: JsExtendedCueList) -> Result<()> {
        let value = value.to_core()?;
        Section {
            tag: AnlzTag::ExtendedCueList,
            content: Content::ExtendedCueList(value.clone()),
        }
        .validate()
        .map_err(crate::util::binding_error)?;
        self.anlz
            .set_extended_memory_cues(value)
            .map_err(crate::util::binding_error)
    }
    #[napi]
    pub fn remove_extended_memory_cues(&mut self) -> Result<bool> {
        self.anlz
            .remove_extended_memory_cues()
            .map_err(crate::util::binding_error)
    }

    #[napi]
    pub fn remove_path(&mut self) -> bool {
        self.anlz.remove_path()
    }

    #[napi]
    pub fn get_vbr(&mut self) -> Result<Option<JsVBR>> {
        Ok(self
            .anlz
            .get_vbr()
            .map_err(crate::util::binding_error)?
            .map(Into::into))
    }

    #[napi]
    pub fn get_seek_index(&mut self) -> Result<Option<JsSeekIndex>> {
        Ok(self
            .anlz
            .get_seek_index()
            .map_err(crate::util::binding_error)?
            .map(Into::into))
    }

    #[napi]
    pub fn get_waveform_3band_calibration(&mut self) -> Result<Option<JsWaveform3BandCalibration>> {
        Ok(self
            .anlz
            .get_waveform_3band_calibration()
            .map_err(crate::util::binding_error)?
            .map(Into::into))
    }

    #[napi]
    pub fn get_vocal_detection(&mut self) -> Result<Option<JsVocalDetection>> {
        Ok(self
            .anlz
            .get_vocal_detection()
            .map_err(crate::util::binding_error)?
            .map(Into::into))
    }

    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        let anlz = Anlz::load(path).map_err(crate::util::binding_error)?;
        Ok(JsAnlz { anlz })
    }

    #[napi]
    pub fn dump_copy(&mut self, path: String) -> Result<()> {
        self.anlz
            .dump_copy(path)
            .map_err(crate::util::binding_error)?;
        Ok(())
    }

    #[napi]
    pub fn dump(&mut self) -> Result<()> {
        self.anlz.dump().map_err(crate::util::binding_error)?;
        Ok(())
    }

    #[napi]
    pub fn get_tags(&mut self) -> Result<Vec<String>> {
        let tags = self.anlz.get_tags().map_err(crate::util::binding_error)?;
        Ok(tags)
    }

    #[napi]
    pub fn get_beat_grid(&mut self) -> Result<Option<JsBeatGrid>> {
        let data = self
            .anlz
            .get_beat_grid()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_extended_beat_grid(&mut self) -> Result<Option<JsExtendedBeatGrid>> {
        let data = self
            .anlz
            .get_extended_beat_grid()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_hot_cues(&mut self) -> Result<Option<JsCueList>> {
        let data = self
            .anlz
            .get_hot_cues()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_memory_cues(&mut self) -> Result<Option<JsCueList>> {
        let data = self
            .anlz
            .get_memory_cues()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_extended_hot_cues(&mut self) -> Result<Option<JsExtendedCueList>> {
        let data = self
            .anlz
            .get_extended_hot_cues()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_extended_memory_cues(&mut self) -> Result<Option<JsExtendedCueList>> {
        let data = self
            .anlz
            .get_extended_memory_cues()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_path(&mut self) -> Result<Option<String>> {
        self.anlz.get_path().map_err(crate::util::binding_error)
    }

    #[napi]
    pub fn set_path(&mut self, path: String) -> Result<()> {
        self.anlz
            .set_path(path)
            .map_err(crate::util::binding_error)?;
        Ok(())
    }

    #[napi]
    pub fn get_vbr_data(&mut self) -> Result<Option<Vec<u8>>> {
        Ok(self
            .anlz
            .get_vbr_data()
            .map_err(crate::util::binding_error)?)
    }

    #[napi]
    pub fn get_tiny_waveform_preview(&mut self) -> Result<Option<JsTinyWaveformPreview>> {
        let data = self
            .anlz
            .get_tiny_waveform_preview()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_waveform_preview(&mut self) -> Result<Option<JsWaveformPreview>> {
        let data = self
            .anlz
            .get_waveform_preview()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_waveform_detail(&mut self) -> Result<Option<JsWaveformDetail>> {
        let data = self
            .anlz
            .get_waveform_detail()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_waveform_color_preview(&mut self) -> Result<Option<JsWaveformColorPreview>> {
        let data = self
            .anlz
            .get_waveform_color_preview()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_waveform_color_detail(&mut self) -> Result<Option<JsWaveformColorDetail>> {
        let data = self
            .anlz
            .get_waveform_color_detail()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_waveform_3band_preview(&mut self) -> Result<Option<JsWaveform3BandPreview>> {
        let data = self
            .anlz
            .get_waveform_3band_preview()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_waveform_3band_detail(&mut self) -> Result<Option<JsWaveform3BandDetail>> {
        let data = self
            .anlz
            .get_waveform_3band_detail()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_song_structure(&mut self) -> Result<Option<JsSongStructureData>> {
        let data = self
            .anlz
            .get_song_structure()
            .map_err(crate::util::binding_error)?;
        if let Some(data) = data {
            return Ok(Some(data.into()));
        }
        Ok(None)
    }
}
