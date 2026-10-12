// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Typed ANLZ snapshots and explicit editing methods for Python.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rbox::anlz::sections::*;
use rbox::anlz::Anlz;

use super::errors::AnlzError;
use super::traits::IntoPy;

fn anlz_error(error: rbox::anlz::AnlzError) -> PyErr {
    AnlzError::new_err(error.to_string())
}

fn invalid(message: &str) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(message.to_owned())
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "Beat", unsendable, get_all, set_all)]
pub struct PyBeat {
    pub beat_number: u16,
    pub tempo: u16,
    pub time: u32,
}

#[pymethods]
impl PyBeat {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(beat_number: u16, tempo: u16, time: u32) -> Self {
        Self {
            beat_number,
            tempo,
            time,
        }
    }
}

impl IntoPy<PyBeat> for &Beat {
    fn into_py(self, _py: Python) -> PyResult<PyBeat> {
        let value = self;
        Ok(PyBeat {
            beat_number: value.beat_number,
            tempo: value.tempo,
            time: value.time,
        })
    }
}

impl PyBeat {
    fn to_core(&self, _py: Python) -> PyResult<Beat> {
        Ok(Beat {
            beat_number: self.beat_number,
            tempo: self.tempo,
            time: self.time,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "ExtBeat", unsendable, get_all, set_all)]
pub struct PyExtBeat {
    pub microseconds: u16,
}

#[pymethods]
impl PyExtBeat {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(microseconds: u16) -> Self {
        Self { microseconds }
    }
}

impl IntoPy<PyExtBeat> for &ExtBeat {
    fn into_py(self, _py: Python) -> PyResult<PyExtBeat> {
        let value = self;
        Ok(PyExtBeat {
            microseconds: value.microseconds,
        })
    }
}

impl PyExtBeat {
    fn to_core(&self, _py: Python) -> PyResult<ExtBeat> {
        Ok(ExtBeat {
            microseconds: self.microseconds,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "BeatGrid", unsendable, get_all, set_all)]
pub struct PyBeatGrid {
    pub beats: Py<PyList>,
    pub header_flags: u32,
}

#[pymethods]
impl PyBeatGrid {
    /// Create a fixed 4/4 grid at 1..=300 BPM. Timestamps are raw milliseconds;
    /// end_ms is inclusive. Apply the snapshot with Anlz.set_beat_grid.
    #[staticmethod]
    fn new_static_bpm(py: Python, bpm: f64, first_beat_ms: u32, end_ms: u32) -> PyResult<Self> {
        let grid = BeatGrid::new_static_bpm(bpm, first_beat_ms, end_ms).map_err(anlz_error)?;
        (&grid).into_py(py)
    }

    /// Replace beats, preserving flags and correction. Errors leave this snapshot
    /// unchanged. Apply with Anlz.set_beat_grid; PQT2 is not updated implicitly.
    fn set_static_bpm(
        &mut self,
        py: Python,
        bpm: f64,
        first_beat_ms: u32,
        end_ms: u32,
    ) -> PyResult<()> {
        let mut grid = self.to_core(py)?;
        grid.set_static_bpm(bpm, first_beat_ms, end_ms)
            .map_err(anlz_error)?;
        *self = (&grid).into_py(py)?;
        Ok(())
    }

    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, beats: Vec<Py<PyBeat>>, header_flags: u32) -> PyResult<Self> {
        Ok(Self {
            beats: PyList::new(py, beats)?.unbind(),
            header_flags,
        })
    }
}

impl IntoPy<PyBeatGrid> for &BeatGrid {
    fn into_py(self, py: Python) -> PyResult<PyBeatGrid> {
        let value = self;
        Ok(PyBeatGrid {
            beats: PyList::new(
                py,
                value
                    .beats
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
            header_flags: value.header_flags,
        })
    }
}

impl PyBeatGrid {
    fn to_core(&self, py: Python) -> PyResult<BeatGrid> {
        let mut value = BeatGrid::new(
            self.beats
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PyBeat>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
        );
        value.header_flags = self.header_flags;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "ExtendedBeatGrid", unsendable, get_all, set_all)]
pub struct PyExtendedBeatGrid {
    pub first_beat: Py<PyBeat>,
    pub last_beat: Py<PyBeat>,
    pub offset_ms: i16,
    pub offset_flags: u16,
    pub checksum: u32,
    pub beats: Py<PyList>,
    pub version: u32,
}

#[pymethods]
impl PyExtendedBeatGrid {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python,
        first_beat: Py<PyBeat>,
        last_beat: Py<PyBeat>,
        offset_ms: i16,
        offset_flags: u16,
        checksum: u32,
        beats: Vec<Py<PyExtBeat>>,
        version: u32,
    ) -> PyResult<Self> {
        Ok(Self {
            first_beat,
            last_beat,
            offset_ms,
            offset_flags,
            checksum,
            beats: PyList::new(py, beats)?.unbind(),
            version,
        })
    }
}

impl IntoPy<PyExtendedBeatGrid> for &ExtendedBeatGrid {
    fn into_py(self, py: Python) -> PyResult<PyExtendedBeatGrid> {
        let value = self;
        Ok(PyExtendedBeatGrid {
            first_beat: Py::new(py, (&value.first_beat).into_py(py)?)?,
            last_beat: Py::new(py, (&value.last_beat).into_py(py)?)?,
            offset_ms: value.offset_ms,
            offset_flags: value.offset_flags,
            checksum: value.checksum,
            beats: PyList::new(
                py,
                value
                    .beats
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
            version: value.version,
        })
    }
}

impl PyExtendedBeatGrid {
    fn to_core(&self, py: Python) -> PyResult<ExtendedBeatGrid> {
        let mut value = ExtendedBeatGrid::new(
            self.version,
            [
                self.first_beat.try_borrow(py)?.to_core(py)?,
                self.last_beat.try_borrow(py)?.to_core(py)?,
            ],
            self.beats
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PyExtBeat>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
            self.checksum,
        );
        value.offset_ms = self.offset_ms;
        value.offset_flags = self.offset_flags;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "AnlzCue", unsendable, get_all, set_all)]
pub struct PyCue {
    pub hot_cue: u32,
    pub status: u16,
    pub order_first: u16,
    pub order_last: u16,
    pub cue_type: u16,
    pub time: u32,
    pub loop_time: u32,
    pub header_size: u32,
    pub u4: u32,
    pub u5: u32,
    pub u6: u32,
    pub u7: u32,
}

#[pymethods]
impl PyCue {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        hot_cue: u32,
        status: u16,
        order_first: u16,
        order_last: u16,
        cue_type: u16,
        time: u32,
        loop_time: u32,
        header_size: u32,
        u4: u32,
        u5: u32,
        u6: u32,
        u7: u32,
    ) -> Self {
        Self {
            hot_cue,
            status,
            order_first,
            order_last,
            cue_type,
            time,
            loop_time,
            header_size,
            u4,
            u5,
            u6,
            u7,
        }
    }
}

impl IntoPy<PyCue> for &Cue {
    fn into_py(self, _py: Python) -> PyResult<PyCue> {
        let value = self;
        Ok(PyCue {
            hot_cue: value.hot_cue,
            status: value.status.clone().into(),
            order_first: value.order_first,
            order_last: value.order_last,
            cue_type: value.cue_type.clone().into(),
            time: value.time,
            loop_time: value.loop_time,
            header_size: value.header.size,
            u4: value.u4,
            u5: value.u5,
            u6: value.u6,
            u7: value.u7,
        })
    }
}

impl PyCue {
    fn to_core(&self, _py: Python) -> PyResult<Cue> {
        Ok(Cue {
            hot_cue: self.hot_cue,
            status: match self.status {
                0 => CueStatus::Disabled,
                4 => CueStatus::Active,
                _ => return Err(invalid("CueStatus value is invalid")),
            },
            order_first: self.order_first,
            order_last: self.order_last,
            cue_type: match self.cue_type {
                1 => CueType::Point,
                2 => CueType::Loop,
                _ => return Err(invalid("CueType value is invalid")),
            },
            time: self.time,
            loop_time: self.loop_time,
            u4: self.u4,
            u5: self.u5,
            u6: self.u6,
            u7: self.u7,
            header: Header {
                tag: AnlzTag::Cue,
                size: self.header_size,
                total_size: 56,
            },
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "ExtendedCue", unsendable, get_all, set_all)]
pub struct PyExtendedCue {
    pub hot_cue: u32,
    pub cue_type: u16,
    pub time: u32,
    pub loop_time: u32,
    pub color: u8,
    pub loop_numerator: u16,
    pub loop_denominator: u16,
    pub comment: String,
    pub hot_cue_color_index: u8,
    pub hot_cue_color_red: u8,
    pub hot_cue_color_green: u8,
    pub hot_cue_color_blue: u8,
    pub header_size: u32,
    pub u3: u8,
    pub u4: u16,
    pub u5: u32,
    pub comment_trailing_nuls: u32,
    pub trailing_data: Vec<u8>,
}

#[pymethods]
impl PyExtendedCue {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        hot_cue: u32,
        cue_type: u16,
        time: u32,
        loop_time: u32,
        color: u8,
        loop_numerator: u16,
        loop_denominator: u16,
        comment: String,
        hot_cue_color_index: u8,
        hot_cue_color_red: u8,
        hot_cue_color_green: u8,
        hot_cue_color_blue: u8,
        header_size: u32,
        u3: u8,
        u4: u16,
        u5: u32,
        comment_trailing_nuls: u32,
        trailing_data: Vec<u8>,
    ) -> Self {
        Self {
            hot_cue,
            cue_type,
            time,
            loop_time,
            color,
            loop_numerator,
            loop_denominator,
            comment,
            hot_cue_color_index,
            hot_cue_color_red,
            hot_cue_color_green,
            hot_cue_color_blue,
            header_size,
            u3,
            u4,
            u5,
            comment_trailing_nuls,
            trailing_data,
        }
    }
}

impl IntoPy<PyExtendedCue> for &ExtendedCue {
    fn into_py(self, _py: Python) -> PyResult<PyExtendedCue> {
        let value = self;
        Ok(PyExtendedCue {
            hot_cue: value.hot_cue,
            cue_type: value.cue_type.clone().into(),
            time: value.time,
            loop_time: value.loop_time,
            color: value.color.into(),
            loop_numerator: value.loop_numerator,
            loop_denominator: value.loop_denominator,
            comment: value.comment.to_string(),
            hot_cue_color_index: value.hot_cue_color_index,
            hot_cue_color_red: value.hot_cue_color_rgb.0,
            hot_cue_color_green: value.hot_cue_color_rgb.1,
            hot_cue_color_blue: value.hot_cue_color_rgb.2,
            header_size: value.header.size,
            u3: value.u3,
            u4: value.u4,
            u5: value.u5,
            comment_trailing_nuls: value.comment.trailing_nuls(),
            trailing_data: value.trailing_data.clone(),
        })
    }
}

impl PyExtendedCue {
    fn to_core(&self, _py: Python) -> PyResult<ExtendedCue> {
        Ok(ExtendedCue {
            hot_cue: self.hot_cue,
            cue_type: match self.cue_type {
                1 => CueType::Point,
                2 => CueType::Loop,
                _ => return Err(invalid("CueType value is invalid")),
            },
            time: self.time,
            loop_time: self.loop_time,
            color: self.color.into(),
            loop_numerator: self.loop_numerator,
            loop_denominator: self.loop_denominator,
            comment: rbox::anlz::AnlzString::with_padding(
                self.comment.clone(),
                self.comment_trailing_nuls,
            ),
            hot_cue_color_index: self.hot_cue_color_index,
            u3: self.u3,
            u4: self.u4,
            u5: self.u5,
            trailing_data: self.trailing_data.clone(),
            header: Header {
                tag: AnlzTag::ExtendedCue,
                size: self.header_size,
                total_size: 0,
            },
            hot_cue_color_rgb: (
                self.hot_cue_color_red,
                self.hot_cue_color_green,
                self.hot_cue_color_blue,
            ),
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "CueList", unsendable, get_all, set_all)]
pub struct PyCueList {
    pub list_type: u16,
    pub cues: Py<PyList>,
    pub memory_count: i32,
}

#[pymethods]
impl PyCueList {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, list_type: u16, cues: Vec<Py<PyCue>>, memory_count: i32) -> PyResult<Self> {
        Ok(Self {
            list_type,
            cues: PyList::new(py, cues)?.unbind(),
            memory_count,
        })
    }
}

impl IntoPy<PyCueList> for &CueList {
    fn into_py(self, py: Python) -> PyResult<PyCueList> {
        let value = self;
        Ok(PyCueList {
            list_type: value.list_type.clone().into(),
            cues: PyList::new(
                py,
                value
                    .cues
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
            memory_count: value.memory_count,
        })
    }
}

impl PyCueList {
    fn to_core(&self, py: Python) -> PyResult<CueList> {
        let mut value = CueList::new(
            match self.list_type {
                0 => CueListType::MemoryCues,
                1 => CueListType::HotCues,
                _ => return Err(invalid("CueListType value is invalid")),
            },
            self.cues
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PyCue>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
        );
        value.memory_count = self.memory_count;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "ExtendedCueList", unsendable, get_all, set_all)]
pub struct PyExtendedCueList {
    pub list_type: u16,
    pub cues: Py<PyList>,
}

#[pymethods]
impl PyExtendedCueList {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, list_type: u16, cues: Vec<Py<PyExtendedCue>>) -> PyResult<Self> {
        Ok(Self {
            list_type,
            cues: PyList::new(py, cues)?.unbind(),
        })
    }
}

impl IntoPy<PyExtendedCueList> for &ExtendedCueList {
    fn into_py(self, py: Python) -> PyResult<PyExtendedCueList> {
        let value = self;
        Ok(PyExtendedCueList {
            list_type: value.list_type.clone().into(),
            cues: PyList::new(
                py,
                value
                    .cues
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
        })
    }
}

impl PyExtendedCueList {
    fn to_core(&self, py: Python) -> PyResult<ExtendedCueList> {
        let value = ExtendedCueList::new(
            match self.list_type {
                0 => CueListType::MemoryCues,
                1 => CueListType::HotCues,
                _ => return Err(invalid("CueListType value is invalid")),
            },
            self.cues
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PyExtendedCue>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "TinyWaveformColumn", unsendable, get_all, set_all)]
pub struct PyTinyWaveformColumn {
    pub height: u8,
    pub unused: u8,
}

#[pymethods]
impl PyTinyWaveformColumn {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(height: u8, unused: u8) -> Self {
        Self { height, unused }
    }
}

impl IntoPy<PyTinyWaveformColumn> for &TinyWaveformColumn {
    fn into_py(self, _py: Python) -> PyResult<PyTinyWaveformColumn> {
        let value = self;
        Ok(PyTinyWaveformColumn {
            height: value.height(),
            unused: value.unused(),
        })
    }
}

impl PyTinyWaveformColumn {
    fn to_core(&self, _py: Python) -> PyResult<TinyWaveformColumn> {
        let mut value = TinyWaveformColumn::new();
        value
            .set_height_checked(self.height)
            .map_err(|_| invalid("TinyWaveformColumn.height is out of range"))?;
        value
            .set_unused_checked(self.unused)
            .map_err(|_| invalid("TinyWaveformColumn.unused is out of range"))?;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "TinyWaveformPreview", unsendable, get_all, set_all)]
pub struct PyTinyWaveformPreview {
    pub data: Py<PyList>,
}

#[pymethods]
impl PyTinyWaveformPreview {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, data: Vec<Py<PyTinyWaveformColumn>>) -> PyResult<Self> {
        Ok(Self {
            data: PyList::new(py, data)?.unbind(),
        })
    }
}

impl IntoPy<PyTinyWaveformPreview> for &TinyWaveformPreview {
    fn into_py(self, py: Python) -> PyResult<PyTinyWaveformPreview> {
        let value = self;
        Ok(PyTinyWaveformPreview {
            data: PyList::new(
                py,
                value
                    .data
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
        })
    }
}

impl PyTinyWaveformPreview {
    fn to_core(&self, py: Python) -> PyResult<TinyWaveformPreview> {
        let value = TinyWaveformPreview::new(
            self.data
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PyTinyWaveformColumn>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "WaveformColumn", unsendable, get_all, set_all)]
pub struct PyWaveformColumn {
    pub height: u8,
    pub whiteness: u8,
}

#[pymethods]
impl PyWaveformColumn {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(height: u8, whiteness: u8) -> Self {
        Self { height, whiteness }
    }
}

impl IntoPy<PyWaveformColumn> for &WaveformColumn {
    fn into_py(self, _py: Python) -> PyResult<PyWaveformColumn> {
        let value = self;
        Ok(PyWaveformColumn {
            height: value.height(),
            whiteness: value.whiteness(),
        })
    }
}

impl PyWaveformColumn {
    fn to_core(&self, _py: Python) -> PyResult<WaveformColumn> {
        let mut value = WaveformColumn::new();
        value
            .set_height_checked(self.height)
            .map_err(|_| invalid("WaveformColumn.height is out of range"))?;
        value
            .set_whiteness_checked(self.whiteness)
            .map_err(|_| invalid("WaveformColumn.whiteness is out of range"))?;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "WaveformPreview", unsendable, get_all, set_all)]
pub struct PyWaveformPreview {
    pub data: Py<PyList>,
}

#[pymethods]
impl PyWaveformPreview {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, data: Vec<Py<PyWaveformColumn>>) -> PyResult<Self> {
        Ok(Self {
            data: PyList::new(py, data)?.unbind(),
        })
    }
}

impl IntoPy<PyWaveformPreview> for &WaveformPreview {
    fn into_py(self, py: Python) -> PyResult<PyWaveformPreview> {
        let value = self;
        Ok(PyWaveformPreview {
            data: PyList::new(
                py,
                value
                    .data
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
        })
    }
}

impl PyWaveformPreview {
    fn to_core(&self, py: Python) -> PyResult<WaveformPreview> {
        let value = WaveformPreview::new(
            self.data
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PyWaveformColumn>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "WaveformDetail", unsendable, get_all, set_all)]
pub struct PyWaveformDetail {
    pub data: Py<PyList>,
}

#[pymethods]
impl PyWaveformDetail {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, data: Vec<Py<PyWaveformColumn>>) -> PyResult<Self> {
        Ok(Self {
            data: PyList::new(py, data)?.unbind(),
        })
    }
}

impl IntoPy<PyWaveformDetail> for &WaveformDetail {
    fn into_py(self, py: Python) -> PyResult<PyWaveformDetail> {
        let value = self;
        Ok(PyWaveformDetail {
            data: PyList::new(
                py,
                value
                    .data
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
        })
    }
}

impl PyWaveformDetail {
    fn to_core(&self, py: Python) -> PyResult<WaveformDetail> {
        let value = WaveformDetail::new(
            self.data
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PyWaveformColumn>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "WaveformColorPreviewColumn", unsendable, get_all, set_all)]
pub struct PyWaveformColorPreviewColumn {
    pub energy_bottom_half_freq: u8,
    pub energy_bottom_third_freq: u8,
    pub energy_mid_third_freq: u8,
    pub energy_top_third_freq: u8,
    pub u1: u8,
    pub u2: u8,
}

#[pymethods]
impl PyWaveformColorPreviewColumn {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        energy_bottom_half_freq: u8,
        energy_bottom_third_freq: u8,
        energy_mid_third_freq: u8,
        energy_top_third_freq: u8,
        u1: u8,
        u2: u8,
    ) -> Self {
        Self {
            energy_bottom_half_freq,
            energy_bottom_third_freq,
            energy_mid_third_freq,
            energy_top_third_freq,
            u1,
            u2,
        }
    }
}

impl IntoPy<PyWaveformColorPreviewColumn> for &WaveformColorPreviewColumn {
    fn into_py(self, _py: Python) -> PyResult<PyWaveformColorPreviewColumn> {
        let value = self;
        Ok(PyWaveformColorPreviewColumn {
            energy_bottom_half_freq: value.energy_bottom_half_freq,
            energy_bottom_third_freq: value.energy_bottom_third_freq,
            energy_mid_third_freq: value.energy_mid_third_freq,
            energy_top_third_freq: value.energy_top_third_freq,
            u1: value.u1,
            u2: value.u2,
        })
    }
}

impl PyWaveformColorPreviewColumn {
    fn to_core(&self, _py: Python) -> PyResult<WaveformColorPreviewColumn> {
        Ok(WaveformColorPreviewColumn {
            energy_bottom_half_freq: self.energy_bottom_half_freq,
            energy_bottom_third_freq: self.energy_bottom_third_freq,
            energy_mid_third_freq: self.energy_mid_third_freq,
            energy_top_third_freq: self.energy_top_third_freq,
            u1: self.u1,
            u2: self.u2,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "WaveformColorPreview", unsendable, get_all, set_all)]
pub struct PyWaveformColorPreview {
    pub data: Py<PyList>,
}

#[pymethods]
impl PyWaveformColorPreview {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, data: Vec<Py<PyWaveformColorPreviewColumn>>) -> PyResult<Self> {
        Ok(Self {
            data: PyList::new(py, data)?.unbind(),
        })
    }
}

impl IntoPy<PyWaveformColorPreview> for &WaveformColorPreview {
    fn into_py(self, py: Python) -> PyResult<PyWaveformColorPreview> {
        let value = self;
        Ok(PyWaveformColorPreview {
            data: PyList::new(
                py,
                value
                    .data
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
        })
    }
}

impl PyWaveformColorPreview {
    fn to_core(&self, py: Python) -> PyResult<WaveformColorPreview> {
        let value = WaveformColorPreview::new(
            self.data
                .bind(py)
                .iter()
                .map(|v| {
                    v.extract::<PyRef<'_, PyWaveformColorPreviewColumn>>()?
                        .to_core(py)
                })
                .collect::<PyResult<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "WaveformColorDetailColumn", unsendable, get_all, set_all)]
pub struct PyWaveformColorDetailColumn {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub height: u8,
    pub low_bits: u8,
    pub fine_height_substep: u8,
    pub full_height: u8,
}

#[pymethods]
impl PyWaveformColorDetailColumn {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(red: u8, green: u8, blue: u8, height: u8, low_bits: u8) -> Self {
        Self {
            red,
            green,
            blue,
            height,
            low_bits,
            fine_height_substep: ((low_bits & 1) << 1) | ((low_bits >> 1) & 1),
            full_height: (height << 2) | ((low_bits & 1) << 1) | ((low_bits >> 1) & 1),
        }
    }
}

impl IntoPy<PyWaveformColorDetailColumn> for &WaveformColorDetailColumn {
    fn into_py(self, _py: Python) -> PyResult<PyWaveformColorDetailColumn> {
        let value = self;
        Ok(PyWaveformColorDetailColumn {
            red: value.red(),
            green: value.green(),
            blue: value.blue(),
            height: value.height(),
            low_bits: value.low_bits(),
            fine_height_substep: value.fine_height_substep(),
            full_height: value.full_height(),
        })
    }
}

impl PyWaveformColorDetailColumn {
    fn to_core(&self, _py: Python) -> PyResult<WaveformColorDetailColumn> {
        let mut value = WaveformColorDetailColumn::new();
        value
            .set_low_bits_checked(self.low_bits)
            .map_err(|_| invalid("WaveformColorDetailColumn.low_bits is out of range"))?;
        value
            .set_height_checked(self.height)
            .map_err(|_| invalid("WaveformColorDetailColumn.height is out of range"))?;
        value
            .set_red_checked(self.red)
            .map_err(|_| invalid("WaveformColorDetailColumn.red is out of range"))?;
        value
            .set_green_checked(self.green)
            .map_err(|_| invalid("WaveformColorDetailColumn.green is out of range"))?;
        value
            .set_blue_checked(self.blue)
            .map_err(|_| invalid("WaveformColorDetailColumn.blue is out of range"))?;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "WaveformColorDetail", unsendable, get_all, set_all)]
pub struct PyWaveformColorDetail {
    pub data: Py<PyList>,
    pub unknown: u32,
}

#[pymethods]
impl PyWaveformColorDetail {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, data: Vec<Py<PyWaveformColorDetailColumn>>, unknown: u32) -> PyResult<Self> {
        Ok(Self {
            data: PyList::new(py, data)?.unbind(),
            unknown,
        })
    }
}

impl IntoPy<PyWaveformColorDetail> for &WaveformColorDetail {
    fn into_py(self, py: Python) -> PyResult<PyWaveformColorDetail> {
        let value = self;
        Ok(PyWaveformColorDetail {
            data: PyList::new(
                py,
                value
                    .data
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
            unknown: value.unknown,
        })
    }
}

impl PyWaveformColorDetail {
    fn to_core(&self, py: Python) -> PyResult<WaveformColorDetail> {
        let mut value = WaveformColorDetail::new(
            self.data
                .bind(py)
                .iter()
                .map(|v| {
                    v.extract::<PyRef<'_, PyWaveformColorDetailColumn>>()?
                        .to_core(py)
                })
                .collect::<PyResult<Vec<_>>>()?,
        );
        value.unknown = self.unknown;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "Waveform3BandColumn", unsendable, get_all, set_all)]
pub struct PyWaveform3BandColumn {
    pub mid: u8,
    pub high: u8,
    pub low: u8,
}

#[pymethods]
impl PyWaveform3BandColumn {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(mid: u8, high: u8, low: u8) -> Self {
        Self { mid, high, low }
    }
}

impl IntoPy<PyWaveform3BandColumn> for &Waveform3BandColumn {
    fn into_py(self, _py: Python) -> PyResult<PyWaveform3BandColumn> {
        let value = self;
        Ok(PyWaveform3BandColumn {
            mid: value.mid(),
            high: value.high(),
            low: value.low(),
        })
    }
}

impl PyWaveform3BandColumn {
    fn to_core(&self, _py: Python) -> PyResult<Waveform3BandColumn> {
        let mut value = Waveform3BandColumn::new();
        value
            .set_low_checked(self.low)
            .map_err(|_| invalid("Waveform3BandColumn.low is out of range"))?;
        value
            .set_mid_checked(self.mid)
            .map_err(|_| invalid("Waveform3BandColumn.mid is out of range"))?;
        value
            .set_high_checked(self.high)
            .map_err(|_| invalid("Waveform3BandColumn.high is out of range"))?;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "Waveform3BandPreview", unsendable, get_all, set_all)]
pub struct PyWaveform3BandPreview {
    pub data: Py<PyList>,
}

#[pymethods]
impl PyWaveform3BandPreview {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, data: Vec<Py<PyWaveform3BandColumn>>) -> PyResult<Self> {
        Ok(Self {
            data: PyList::new(py, data)?.unbind(),
        })
    }
}

impl IntoPy<PyWaveform3BandPreview> for &Waveform3BandPreview {
    fn into_py(self, py: Python) -> PyResult<PyWaveform3BandPreview> {
        let value = self;
        Ok(PyWaveform3BandPreview {
            data: PyList::new(
                py,
                value
                    .data
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
        })
    }
}

impl PyWaveform3BandPreview {
    fn to_core(&self, py: Python) -> PyResult<Waveform3BandPreview> {
        let value = Waveform3BandPreview::new(
            self.data
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PyWaveform3BandColumn>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
        );

        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "Waveform3BandDetail", unsendable, get_all, set_all)]
pub struct PyWaveform3BandDetail {
    pub data: Py<PyList>,
    pub unknown: u32,
}

#[pymethods]
impl PyWaveform3BandDetail {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, data: Vec<Py<PyWaveform3BandColumn>>, unknown: u32) -> PyResult<Self> {
        Ok(Self {
            data: PyList::new(py, data)?.unbind(),
            unknown,
        })
    }
}

impl IntoPy<PyWaveform3BandDetail> for &Waveform3BandDetail {
    fn into_py(self, py: Python) -> PyResult<PyWaveform3BandDetail> {
        let value = self;
        Ok(PyWaveform3BandDetail {
            data: PyList::new(
                py,
                value
                    .data
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
            unknown: value.unknown,
        })
    }
}

impl PyWaveform3BandDetail {
    fn to_core(&self, py: Python) -> PyResult<Waveform3BandDetail> {
        let mut value = Waveform3BandDetail::new(
            self.data
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PyWaveform3BandColumn>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
        );
        value.unknown = self.unknown;
        Ok(value)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "Phrase", unsendable, get_all, set_all)]
pub struct PyPhrase {
    pub index: u16,
    pub beat: u16,
    pub kind: u16,
    pub k1: u8,
    pub k2: u8,
    pub b: u8,
    pub beat2: u16,
    pub beat3: u16,
    pub beat4: u16,
    pub k3: u8,
    pub fill: u8,
    pub beat_fill: u16,
    pub u1: u8,
    pub u2: u8,
    pub u3: u8,
    pub u4: u8,
    pub u5: u8,
}

#[pymethods]
impl PyPhrase {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        index: u16,
        beat: u16,
        kind: u16,
        k1: u8,
        k2: u8,
        b: u8,
        beat2: u16,
        beat3: u16,
        beat4: u16,
        k3: u8,
        fill: u8,
        beat_fill: u16,
        u1: u8,
        u2: u8,
        u3: u8,
        u4: u8,
        u5: u8,
    ) -> Self {
        Self {
            index,
            beat,
            kind,
            k1,
            k2,
            b,
            beat2,
            beat3,
            beat4,
            k3,
            fill,
            beat_fill,
            u1,
            u2,
            u3,
            u4,
            u5,
        }
    }
}

impl IntoPy<PyPhrase> for &Phrase {
    fn into_py(self, _py: Python) -> PyResult<PyPhrase> {
        let value = self;
        Ok(PyPhrase {
            index: value.index,
            beat: value.beat,
            kind: value.kind,
            k1: value.k1,
            k2: value.k2,
            b: value.b,
            beat2: value.beat2,
            beat3: value.beat3,
            beat4: value.beat4,
            k3: value.k3,
            fill: value.fill,
            beat_fill: value.beat_fill,
            u1: value.u1,
            u2: value.u2,
            u3: value.u3,
            u4: value.u4,
            u5: value.u5,
        })
    }
}

impl PyPhrase {
    fn to_core(&self, _py: Python) -> PyResult<Phrase> {
        Ok(Phrase {
            index: self.index,
            beat: self.beat,
            kind: self.kind,
            k1: self.k1,
            k2: self.k2,
            b: self.b,
            beat2: self.beat2,
            beat3: self.beat3,
            beat4: self.beat4,
            k3: self.k3,
            fill: self.fill,
            beat_fill: self.beat_fill,
            u1: self.u1,
            u2: self.u2,
            u3: self.u3,
            u4: self.u4,
            u5: self.u5,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "SongStructureData", unsendable, get_all, set_all)]
pub struct PySongStructureData {
    pub mood: u16,
    pub end_beat: u16,
    pub bank: u16,
    pub phrases: Py<PyList>,
    pub u1: u32,
    pub u2: u16,
    pub u3: u16,
    pub u4: u8,
}

#[pymethods]
impl PySongStructureData {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python,
        mood: u16,
        end_beat: u16,
        bank: u16,
        phrases: Vec<Py<PyPhrase>>,
        u1: u32,
        u2: u16,
        u3: u16,
        u4: u8,
    ) -> PyResult<Self> {
        Ok(Self {
            mood,
            end_beat,
            bank,
            phrases: PyList::new(py, phrases)?.unbind(),
            u1,
            u2,
            u3,
            u4,
        })
    }
}

impl IntoPy<PySongStructureData> for &SongStructureData {
    fn into_py(self, py: Python) -> PyResult<PySongStructureData> {
        let value = self;
        Ok(PySongStructureData {
            mood: value.mood.clone().into(),
            end_beat: value.end_beat,
            bank: value.bank.clone().into(),
            phrases: PyList::new(
                py,
                value
                    .phrases
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
            u1: value.u1,
            u2: value.u2,
            u3: value.u3,
            u4: value.u4,
        })
    }
}

impl PySongStructureData {
    fn to_core(&self, py: Python) -> PyResult<SongStructureData> {
        Ok(SongStructureData {
            mood: match self.mood {
                1 => Mood::High,
                2 => Mood::Mid,
                3 => Mood::Low,
                _ => return Err(invalid("Mood value is invalid")),
            },
            end_beat: self.end_beat,
            bank: match self.bank {
                0 => Bank::Default,
                1 => Bank::Cool,
                2 => Bank::Natural,
                3 => Bank::Hot,
                4 => Bank::Subtle,
                5 => Bank::Warm,
                6 => Bank::Vivid,
                7 => Bank::Club1,
                8 => Bank::Club2,
                _ => return Err(invalid("Bank value is invalid")),
            },
            phrases: self
                .phrases
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PyPhrase>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
            u1: self.u1,
            u2: self.u2,
            u3: self.u3,
            u4: self.u4,
        })
    }
}

#[pyclass(unsendable)]
pub struct PyAnlz {
    anlz: Anlz,
}

#[pymethods]
impl PyAnlz {
    pub fn into_data<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.get_data(py)
    }

    pub fn get_song_structure_section(&mut self, py: Python) -> PyResult<Option<PySongStructure>> {
        self.anlz
            .get_section_by_tag(AnlzTag::SongStructure)
            .map_err(anlz_error)?
            .and_then(|section| match &section.content {
                Content::SongStructure(value) => Some(value.into_py(py)),
                _ => None,
            })
            .transpose()
    }

    pub fn set_song_structure_data(
        &mut self,
        py: Python,
        value: &PySongStructureData,
    ) -> PyResult<()> {
        let data = value.to_core(py)?;
        let section = self
            .anlz
            .get_section_by_tag(AnlzTag::SongStructure)
            .map_err(anlz_error)?
            .ok_or_else(|| {
                invalid("song structure is absent; use set_song_structure to create it")
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
        .map_err(anlz_error)?;
        self.anlz.set_song_structure(value).map_err(anlz_error)?;
        Ok(())
    }
    #[staticmethod]
    pub fn new_dat(path: &str) -> PyResult<Self> {
        Ok(Self {
            anlz: Anlz::new_dat(path).map_err(anlz_error)?,
        })
    }

    #[staticmethod]
    pub fn new_ext(path: &str) -> PyResult<Self> {
        Ok(Self {
            anlz: Anlz::new_ext(path).map_err(anlz_error)?,
        })
    }

    #[staticmethod]
    pub fn new_2ex(path: &str) -> PyResult<Self> {
        Ok(Self {
            anlz: Anlz::new_2ex(path).map_err(anlz_error)?,
        })
    }

    #[staticmethod]
    pub fn load(path: &str) -> PyResult<Self> {
        Self::new(path)
    }

    pub fn section_count(&self) -> usize {
        self.anlz.section_count()
    }

    pub fn get_section<'py>(
        &mut self,
        py: Python<'py>,
        index: usize,
    ) -> PyResult<Option<Bound<'py, PyDict>>> {
        self.anlz
            .get_section(index)
            .map_err(anlz_error)?
            .map(|v| section_to_dict(py, v))
            .transpose()
    }

    pub fn get_section_by_tag<'py>(
        &mut self,
        py: Python<'py>,
        tag: &str,
    ) -> PyResult<Option<Bound<'py, PyDict>>> {
        self.anlz
            .get_section_by_tag(AnlzTag::parse(tag).map_err(anlz_error)?)
            .map_err(anlz_error)?
            .map(|v| section_to_dict(py, v))
            .transpose()
    }

    pub fn get_sections_by_tag(&mut self, py: Python, tag: &str) -> PyResult<Vec<Py<PyDict>>> {
        self.anlz
            .get_sections_by_tag(AnlzTag::parse(tag).map_err(anlz_error)?)
            .map_err(anlz_error)?
            .into_iter()
            .map(|v| Ok(section_to_dict(py, v)?.unbind()))
            .collect()
    }

    pub fn get_data<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let value = self.anlz.get_data().map_err(anlz_error)?;
        let result = PyDict::new(py);
        result.set_item("header_data", value.header_data)?;
        let header = PyDict::new(py);
        header.set_item("code", value.header.tag.code().to_vec())?;
        header.set_item("size", value.header.size)?;
        header.set_item("total_size", value.header.total_size)?;
        result.set_item("header", header)?;
        let sections = value
            .sections
            .iter()
            .map(|v| Ok(section_to_dict(py, v)?.unbind()))
            .collect::<PyResult<Vec<_>>>()?;
        result.set_item("sections", sections)?;
        Ok(result)
    }

    pub fn insert_section(&mut self, index: usize, section: &Bound<'_, PyDict>) -> PyResult<()> {
        self.anlz
            .insert_section(index, section_from_dict(section)?)
            .map_err(anlz_error)?;
        Ok(())
    }

    pub fn set_section_at(&mut self, index: usize, section: &Bound<'_, PyDict>) -> PyResult<()> {
        self.anlz
            .set_section_at(index, section_from_dict(section)?)
            .map_err(anlz_error)?;
        Ok(())
    }

    pub fn push_section(&mut self, section: &Bound<'_, PyDict>) -> PyResult<()> {
        self.anlz
            .push_section(section_from_dict(section)?)
            .map_err(anlz_error)?;
        Ok(())
    }

    pub fn set_section(&mut self, section: &Bound<'_, PyDict>) -> PyResult<()> {
        self.anlz
            .set_section(section_from_dict(section)?)
            .map_err(anlz_error)?;
        Ok(())
    }

    pub fn remove_section(&mut self, index: usize) -> PyResult<()> {
        self.anlz.remove_section(index).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_section_by_tag(&mut self, tag: &str) -> PyResult<bool> {
        Ok(self
            .anlz
            .remove_section_by_tag(AnlzTag::parse(tag).map_err(anlz_error)?))
    }
    pub fn remove_sections_by_tag(&mut self, tag: &str) -> PyResult<usize> {
        Ok(self
            .anlz
            .remove_sections_by_tag(AnlzTag::parse(tag).map_err(anlz_error)?))
    }

    pub fn set_beat_grid(&mut self, py: Python, value: &PyBeatGrid) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::BeatGrid,
            content: Content::BeatGrid(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_beat_grid(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_beat_grid(&mut self) -> bool {
        self.anlz.remove_beat_grid()
    }

    pub fn set_extended_beat_grid(
        &mut self,
        py: Python,
        value: &PyExtendedBeatGrid,
    ) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::ExtendedBeatGrid,
            content: Content::ExtendedBeatGrid(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz
            .set_extended_beat_grid(value)
            .map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_extended_beat_grid(&mut self) -> bool {
        self.anlz.remove_extended_beat_grid()
    }

    pub fn set_cue_list(&mut self, py: Python, value: &PyCueList) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::CueList,
            content: Content::CueList(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_cue_list(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_cue_list(&mut self) -> bool {
        self.anlz.remove_cue_list()
    }

    pub fn set_extended_cue_list(&mut self, py: Python, value: &PyExtendedCueList) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::ExtendedCueList,
            content: Content::ExtendedCueList(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_extended_cue_list(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_extended_cue_list(&mut self) -> bool {
        self.anlz.remove_extended_cue_list()
    }

    pub fn set_vbr(&mut self, py: Python, value: &PyVBR) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::VBR,
            content: Content::VBR(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_vbr(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_vbr(&mut self) -> bool {
        self.anlz.remove_vbr()
    }

    pub fn set_seek_index(&mut self, py: Python, value: &PySeekIndex) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::SeekIndex,
            content: Content::SeekIndex(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_seek_index(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_seek_index(&mut self) -> bool {
        self.anlz.remove_seek_index()
    }

    pub fn set_tiny_waveform_preview(
        &mut self,
        py: Python,
        value: &PyTinyWaveformPreview,
    ) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::TinyWaveformPreview,
            content: Content::TinyWaveformPreview(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz
            .set_tiny_waveform_preview(value)
            .map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_tiny_waveform_preview(&mut self) -> bool {
        self.anlz.remove_tiny_waveform_preview()
    }

    pub fn set_waveform_preview(&mut self, py: Python, value: &PyWaveformPreview) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::WaveformPreview,
            content: Content::WaveformPreview(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_waveform_preview(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_waveform_preview(&mut self) -> bool {
        self.anlz.remove_waveform_preview()
    }

    pub fn set_waveform_detail(&mut self, py: Python, value: &PyWaveformDetail) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::WaveformDetail,
            content: Content::WaveformDetail(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_waveform_detail(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_waveform_detail(&mut self) -> bool {
        self.anlz.remove_waveform_detail()
    }

    pub fn set_waveform_color_preview(
        &mut self,
        py: Python,
        value: &PyWaveformColorPreview,
    ) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::WaveformColorPreview,
            content: Content::WaveformColorPreview(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz
            .set_waveform_color_preview(value)
            .map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_waveform_color_preview(&mut self) -> bool {
        self.anlz.remove_waveform_color_preview()
    }

    pub fn set_waveform_color_detail(
        &mut self,
        py: Python,
        value: &PyWaveformColorDetail,
    ) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::WaveformColorDetail,
            content: Content::WaveformColorDetail(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz
            .set_waveform_color_detail(value)
            .map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_waveform_color_detail(&mut self) -> bool {
        self.anlz.remove_waveform_color_detail()
    }

    pub fn set_waveform_3band_preview(
        &mut self,
        py: Python,
        value: &PyWaveform3BandPreview,
    ) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::Waveform3BandPreview,
            content: Content::Waveform3BandPreview(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz
            .set_waveform_3band_preview(value)
            .map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_waveform_3band_preview(&mut self) -> bool {
        self.anlz.remove_waveform_3band_preview()
    }

    pub fn set_waveform_3band_detail(
        &mut self,
        py: Python,
        value: &PyWaveform3BandDetail,
    ) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::Waveform3BandDetail,
            content: Content::Waveform3BandDetail(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz
            .set_waveform_3band_detail(value)
            .map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_waveform_3band_detail(&mut self) -> bool {
        self.anlz.remove_waveform_3band_detail()
    }

    pub fn set_waveform_3band_calibration(
        &mut self,
        py: Python,
        value: &PyWaveform3BandCalibration,
    ) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::Waveform3BandCalibration,
            content: Content::Waveform3BandCalibration(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz
            .set_waveform_3band_calibration(value)
            .map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_waveform_3band_calibration(&mut self) -> bool {
        self.anlz.remove_waveform_3band_calibration()
    }

    pub fn set_song_structure(&mut self, py: Python, value: &PySongStructure) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::SongStructure,
            content: Content::SongStructure(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_song_structure(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_song_structure(&mut self) -> bool {
        self.anlz.remove_song_structure()
    }

    pub fn set_vocal_detection(&mut self, py: Python, value: &PyVocalDetection) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::VocalDetection,
            content: Content::VocalDetection(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_vocal_detection(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_vocal_detection(&mut self) -> bool {
        self.anlz.remove_vocal_detection()
    }

    pub fn set_hot_cues(&mut self, py: Python, value: &PyCueList) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::CueList,
            content: Content::CueList(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_hot_cues(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_hot_cues(&mut self) -> PyResult<bool> {
        Ok(self.anlz.remove_hot_cues().map_err(anlz_error)?)
    }

    pub fn set_memory_cues(&mut self, py: Python, value: &PyCueList) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::CueList,
            content: Content::CueList(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_memory_cues(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_memory_cues(&mut self) -> PyResult<bool> {
        Ok(self.anlz.remove_memory_cues().map_err(anlz_error)?)
    }

    pub fn set_extended_hot_cues(&mut self, py: Python, value: &PyExtendedCueList) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::ExtendedCueList,
            content: Content::ExtendedCueList(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz.set_extended_hot_cues(value).map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_extended_hot_cues(&mut self) -> PyResult<bool> {
        Ok(self.anlz.remove_extended_hot_cues().map_err(anlz_error)?)
    }

    pub fn set_extended_memory_cues(
        &mut self,
        py: Python,
        value: &PyExtendedCueList,
    ) -> PyResult<()> {
        let value = value.to_core(py)?;
        Section {
            tag: AnlzTag::ExtendedCueList,
            content: Content::ExtendedCueList(value.clone()),
        }
        .validate()
        .map_err(anlz_error)?;
        self.anlz
            .set_extended_memory_cues(value)
            .map_err(anlz_error)?;
        Ok(())
    }
    pub fn remove_extended_memory_cues(&mut self) -> PyResult<bool> {
        Ok(self
            .anlz
            .remove_extended_memory_cues()
            .map_err(anlz_error)?)
    }

    pub fn remove_path(&mut self) -> bool {
        self.anlz.remove_path()
    }

    pub fn get_vbr(&mut self, py: Python) -> PyResult<Option<PyVBR>> {
        self.anlz
            .get_vbr()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?
            .map(|value| value.into_py(py))
            .transpose()
    }

    pub fn get_seek_index(&mut self, py: Python) -> PyResult<Option<PySeekIndex>> {
        self.anlz
            .get_seek_index()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?
            .map(|value| value.into_py(py))
            .transpose()
    }

    pub fn get_waveform_3band_calibration(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyWaveform3BandCalibration>> {
        self.anlz
            .get_waveform_3band_calibration()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?
            .map(|value| value.into_py(py))
            .transpose()
    }

    pub fn get_vocal_detection(&mut self, py: Python) -> PyResult<Option<PyVocalDetection>> {
        self.anlz
            .get_vocal_detection()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?
            .map(|value| value.into_py(py))
            .transpose()
    }

    #[new]
    pub fn new(path: &str) -> PyResult<Self> {
        let anlz = Anlz::load(path).map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        Ok(PyAnlz { anlz })
    }

    pub fn dump_copy(&mut self, path: &str) -> PyResult<()> {
        self.anlz
            .dump_copy(path)
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn dump(&mut self) -> PyResult<()> {
        self.anlz
            .dump()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn contains(&self, tag: &str) -> PyResult<bool> {
        Ok(self.anlz.contains(AnlzTag::parse(tag).map_err(anlz_error)?))
    }

    pub fn get_tags(&mut self) -> PyResult<Vec<String>> {
        let tags = self
            .anlz
            .get_tags()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        Ok(tags.clone())
    }

    pub fn get_beat_grid(&mut self, py: Python) -> PyResult<Option<PyBeatGrid>> {
        let data = self
            .anlz
            .get_beat_grid()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_extended_beat_grid(&mut self, py: Python) -> PyResult<Option<PyExtendedBeatGrid>> {
        let data = self
            .anlz
            .get_extended_beat_grid()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_hot_cues(&mut self, py: Python) -> PyResult<Option<PyCueList>> {
        let data = self
            .anlz
            .get_hot_cues()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_memory_cues(&mut self, py: Python) -> PyResult<Option<PyCueList>> {
        let data = self
            .anlz
            .get_memory_cues()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_extended_hot_cues(&mut self, py: Python) -> PyResult<Option<PyExtendedCueList>> {
        let data = self
            .anlz
            .get_extended_hot_cues()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_extended_memory_cues(&mut self, py: Python) -> PyResult<Option<PyExtendedCueList>> {
        let data = self
            .anlz
            .get_extended_memory_cues()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_path(&mut self) -> PyResult<Option<String>> {
        let path = self
            .anlz
            .get_path()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(path) = path {
            return Ok(Some(path));
        }
        Ok(None)
    }

    pub fn set_path(&mut self, path: &str) -> PyResult<()> {
        self.anlz
            .set_path(path)
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn get_vbr_data(&mut self) -> PyResult<Option<Vec<u8>>> {
        let vbr = self
            .anlz
            .get_vbr_data()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        Ok(vbr)
    }

    pub fn get_tiny_waveform_preview(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyTinyWaveformPreview>> {
        let data = self
            .anlz
            .get_tiny_waveform_preview()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_preview(&mut self, py: Python) -> PyResult<Option<PyWaveformPreview>> {
        let data = self
            .anlz
            .get_waveform_preview()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_detail(&mut self, py: Python) -> PyResult<Option<PyWaveformDetail>> {
        let data = self
            .anlz
            .get_waveform_detail()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_color_preview(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyWaveformColorPreview>> {
        let data = self
            .anlz
            .get_waveform_color_preview()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_color_detail(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyWaveformColorDetail>> {
        let data = self
            .anlz
            .get_waveform_color_detail()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_3band_preview(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyWaveform3BandPreview>> {
        let data = self
            .anlz
            .get_waveform_3band_preview()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_3band_detail(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyWaveform3BandDetail>> {
        let data = self
            .anlz
            .get_waveform_3band_detail()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_song_structure(&mut self, py: Python) -> PyResult<Option<PySongStructureData>> {
        let data = self
            .anlz
            .get_song_structure()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        if let Some(data) = data {
            return Ok(Some(data.into_py(py)?));
        }
        Ok(None)
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "VBR", unsendable, get_all, set_all)]
pub struct PyVBR {
    pub u1: u32,
    pub index: Py<PyList>,
    pub u2: u32,
}

#[pymethods]
impl PyVBR {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python, u1: u32, index: Vec<u32>, u2: u32) -> PyResult<Self> {
        Ok(Self {
            u1,
            index: PyList::new(py, index)?.unbind(),
            u2,
        })
    }
}

impl IntoPy<PyVBR> for &VBR {
    fn into_py(self, py: Python) -> PyResult<PyVBR> {
        let value = self;
        Ok(PyVBR {
            u1: value.u1,
            index: PyList::new(py, value.index.to_vec())?.unbind(),
            u2: value.u2,
        })
    }
}

impl PyVBR {
    fn to_core(&self, py: Python) -> PyResult<VBR> {
        Ok(VBR {
            u1: self.u1,
            index: self
                .index
                .bind(py)
                .extract::<Vec<u32>>()?
                .try_into()
                .map_err(|_| invalid("VBR index must contain exactly 400 values"))?,
            u2: self.u2,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "SeekPoint", unsendable, get_all, set_all)]
pub struct PySeekPoint {
    pub sample: u64,
    pub offset: u64,
    pub frame_samples: u32,
}

#[pymethods]
impl PySeekPoint {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(sample: u64, offset: u64, frame_samples: u32) -> Self {
        Self {
            sample,
            offset,
            frame_samples,
        }
    }
}

impl IntoPy<PySeekPoint> for &SeekPoint {
    fn into_py(self, _py: Python) -> PyResult<PySeekPoint> {
        let value = self;
        Ok(PySeekPoint {
            sample: value.sample,
            offset: value.offset,
            frame_samples: value.frame_samples,
        })
    }
}

impl PySeekPoint {
    fn to_core(&self, _py: Python) -> PyResult<SeekPoint> {
        Ok(SeekPoint {
            sample: self.sample,
            offset: self.offset,
            frame_samples: self.frame_samples,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "SeekIndex", unsendable, get_all, set_all)]
pub struct PySeekIndex {
    pub u1: u32,
    pub total_samples: u64,
    pub entries: Py<PyList>,
}

#[pymethods]
impl PySeekIndex {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python,
        u1: u32,
        total_samples: u64,
        entries: Vec<Py<PySeekPoint>>,
    ) -> PyResult<Self> {
        Ok(Self {
            u1,
            total_samples,
            entries: PyList::new(py, entries)?.unbind(),
        })
    }
}

impl IntoPy<PySeekIndex> for &SeekIndex {
    fn into_py(self, py: Python) -> PyResult<PySeekIndex> {
        let value = self;
        Ok(PySeekIndex {
            u1: value.u1,
            total_samples: value.total_samples,
            entries: PyList::new(
                py,
                value
                    .entries
                    .iter()
                    .map(|v| Py::new(py, v.into_py(py)?))
                    .collect::<PyResult<Vec<_>>>()?,
            )?
            .unbind(),
        })
    }
}

impl PySeekIndex {
    fn to_core(&self, py: Python) -> PyResult<SeekIndex> {
        Ok(SeekIndex {
            u1: self.u1,
            total_samples: self.total_samples,
            entries: self
                .entries
                .bind(py)
                .iter()
                .map(|v| v.extract::<PyRef<'_, PySeekPoint>>()?.to_core(py))
                .collect::<PyResult<Vec<_>>>()?,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "Waveform3BandCalibration", unsendable, get_all, set_all)]
pub struct PyWaveform3BandCalibration {
    pub low_gain: u16,
    pub mid_gain: u16,
    pub high_gain: u16,
}

#[pymethods]
impl PyWaveform3BandCalibration {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(low_gain: u16, mid_gain: u16, high_gain: u16) -> Self {
        Self {
            low_gain,
            mid_gain,
            high_gain,
        }
    }
}

impl IntoPy<PyWaveform3BandCalibration> for &Waveform3BandCalibration {
    fn into_py(self, _py: Python) -> PyResult<PyWaveform3BandCalibration> {
        let value = self;
        Ok(PyWaveform3BandCalibration {
            low_gain: value.low_gain,
            mid_gain: value.mid_gain,
            high_gain: value.high_gain,
        })
    }
}

impl PyWaveform3BandCalibration {
    fn to_core(&self, _py: Python) -> PyResult<Waveform3BandCalibration> {
        Ok(Waveform3BandCalibration {
            low_gain: self.low_gain,
            mid_gain: self.mid_gain,
            high_gain: self.high_gain,
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "VocalDetection", unsendable, get_all, set_all)]
pub struct PyVocalDetection {
    pub u1: u32,
    pub u2: u32,
    pub confidence: Vec<u8>,
}

#[pymethods]
impl PyVocalDetection {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(u1: u32, u2: u32, confidence: Vec<u8>) -> Self {
        Self { u1, u2, confidence }
    }
}

impl IntoPy<PyVocalDetection> for &VocalDetection {
    fn into_py(self, _py: Python) -> PyResult<PyVocalDetection> {
        let value = self;
        Ok(PyVocalDetection {
            u1: value.u1,
            u2: value.u2,
            confidence: value.confidence.clone(),
        })
    }
}

impl PyVocalDetection {
    fn to_core(&self, _py: Python) -> PyResult<VocalDetection> {
        Ok(VocalDetection {
            u1: self.u1,
            u2: self.u2,
            confidence: self.confidence.clone(),
        })
    }
}

/// Editable snapshot; changes take effect only through an Anlz setter.
#[pyclass(name = "SongStructure", unsendable, get_all, set_all)]
pub struct PySongStructure {
    pub version: u16,
    pub is_encrypted: bool,
    pub data: Py<PySongStructureData>,
}

#[pymethods]
impl PySongStructure {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(version: u16, is_encrypted: bool, data: Py<PySongStructureData>) -> Self {
        Self {
            version,
            is_encrypted,
            data,
        }
    }
}

impl IntoPy<PySongStructure> for &SongStructure {
    fn into_py(self, py: Python) -> PyResult<PySongStructure> {
        let value = self;
        Ok(PySongStructure {
            version: value.version,
            is_encrypted: value.is_encrypted,
            data: Py::new(py, (&value.data).into_py(py)?)?,
        })
    }
}

impl PySongStructure {
    fn to_core(&self, py: Python) -> PyResult<SongStructure> {
        let value = SongStructure::new(
            self.version,
            self.is_encrypted,
            self.data.try_borrow(py)?.to_core(py)?,
        );

        Ok(value)
    }
}

#[pyclass(name = "UnknownSection", unsendable, get_all, set_all)]
pub struct PyUnknownSection {
    pub code: Vec<u8>,
    pub header_data: Vec<u8>,
    pub content_data: Vec<u8>,
}

#[pymethods]
impl PyUnknownSection {
    #[new]
    fn new(code: Vec<u8>, header_data: Vec<u8>, content_data: Vec<u8>) -> Self {
        Self {
            code,
            header_data,
            content_data,
        }
    }
}

fn section_from_dict(value: &Bound<'_, PyDict>) -> PyResult<Section> {
    let tag: String = value
        .get_item("tag")?
        .ok_or_else(|| invalid("section requires tag"))?
        .extract()?;
    let data = value
        .get_item("content")?
        .ok_or_else(|| invalid("section requires content"))?;
    let py = value.py();
    let section = match tag.as_str() {
        "PQTZ" => Section {
            tag: AnlzTag::BeatGrid,
            content: Content::BeatGrid(data.extract::<PyRef<'_, PyBeatGrid>>()?.to_core(py)?),
        },
        "PQT2" => Section {
            tag: AnlzTag::ExtendedBeatGrid,
            content: Content::ExtendedBeatGrid(
                data.extract::<PyRef<'_, PyExtendedBeatGrid>>()?
                    .to_core(py)?,
            ),
        },
        "PCOB" => Section {
            tag: AnlzTag::CueList,
            content: Content::CueList(data.extract::<PyRef<'_, PyCueList>>()?.to_core(py)?),
        },
        "PCO2" => Section {
            tag: AnlzTag::ExtendedCueList,
            content: Content::ExtendedCueList(
                data.extract::<PyRef<'_, PyExtendedCueList>>()?
                    .to_core(py)?,
            ),
        },
        "PPTH" => Section {
            tag: AnlzTag::Path,
            content: Content::Path(Path::new(data.extract::<String>()?)),
        },
        "PVBR" => Section {
            tag: AnlzTag::VBR,
            content: Content::VBR(data.extract::<PyRef<'_, PyVBR>>()?.to_core(py)?),
        },
        "PVB2" => Section {
            tag: AnlzTag::SeekIndex,
            content: Content::SeekIndex(data.extract::<PyRef<'_, PySeekIndex>>()?.to_core(py)?),
        },
        "PWV2" => Section {
            tag: AnlzTag::TinyWaveformPreview,
            content: Content::TinyWaveformPreview(
                data.extract::<PyRef<'_, PyTinyWaveformPreview>>()?
                    .to_core(py)?,
            ),
        },
        "PWAV" => Section {
            tag: AnlzTag::WaveformPreview,
            content: Content::WaveformPreview(
                data.extract::<PyRef<'_, PyWaveformPreview>>()?
                    .to_core(py)?,
            ),
        },
        "PWV3" => Section {
            tag: AnlzTag::WaveformDetail,
            content: Content::WaveformDetail(
                data.extract::<PyRef<'_, PyWaveformDetail>>()?.to_core(py)?,
            ),
        },
        "PWV4" => Section {
            tag: AnlzTag::WaveformColorPreview,
            content: Content::WaveformColorPreview(
                data.extract::<PyRef<'_, PyWaveformColorPreview>>()?
                    .to_core(py)?,
            ),
        },
        "PWV5" => Section {
            tag: AnlzTag::WaveformColorDetail,
            content: Content::WaveformColorDetail(
                data.extract::<PyRef<'_, PyWaveformColorDetail>>()?
                    .to_core(py)?,
            ),
        },
        "PWV6" => Section {
            tag: AnlzTag::Waveform3BandPreview,
            content: Content::Waveform3BandPreview(
                data.extract::<PyRef<'_, PyWaveform3BandPreview>>()?
                    .to_core(py)?,
            ),
        },
        "PWV7" => Section {
            tag: AnlzTag::Waveform3BandDetail,
            content: Content::Waveform3BandDetail(
                data.extract::<PyRef<'_, PyWaveform3BandDetail>>()?
                    .to_core(py)?,
            ),
        },
        "PWVC" => Section {
            tag: AnlzTag::Waveform3BandCalibration,
            content: Content::Waveform3BandCalibration(
                data.extract::<PyRef<'_, PyWaveform3BandCalibration>>()?
                    .to_core(py)?,
            ),
        },
        "PSSI" => Section {
            tag: AnlzTag::SongStructure,
            content: Content::SongStructure(
                data.extract::<PyRef<'_, PySongStructure>>()?.to_core(py)?,
            ),
        },
        "PVDI" => Section {
            tag: AnlzTag::VocalDetection,
            content: Content::VocalDetection(
                data.extract::<PyRef<'_, PyVocalDetection>>()?.to_core(py)?,
            ),
        },
        "Unknown" => {
            let value = data.extract::<PyRef<'_, PyUnknownSection>>()?;
            let code: [u8; 4] = value
                .code
                .clone()
                .try_into()
                .map_err(|_| invalid("unknown tag code must contain four bytes"))?;
            let tag = AnlzTag::from_code(code);
            if !matches!(tag, AnlzTag::Unknown(_)) {
                return Err(invalid("known tags require typed content"));
            }
            Section {
                tag,
                content: Content::Unknown(
                    Unknown::new(value.header_data.clone(), value.content_data.clone())
                        .map_err(anlz_error)?,
                ),
            }
        }
        _ => return Err(invalid("unknown section discriminator")),
    };
    section.validate().map_err(anlz_error)?;
    Ok(section)
}

fn section_to_dict<'py>(py: Python<'py>, value: &Section) -> PyResult<Bound<'py, PyDict>> {
    let result = PyDict::new(py);
    match &value.content {
        Content::BeatGrid(content) => {
            result.set_item("tag", "PQTZ")?;
            result.set_item(
                "content",
                Py::new(py, <&BeatGrid as IntoPy<PyBeatGrid>>::into_py(content, py)?)?,
            )?;
        }
        Content::ExtendedBeatGrid(content) => {
            result.set_item("tag", "PQT2")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&ExtendedBeatGrid as IntoPy<PyExtendedBeatGrid>>::into_py(content, py)?,
                )?,
            )?;
        }
        Content::CueList(content) => {
            result.set_item("tag", "PCOB")?;
            result.set_item(
                "content",
                Py::new(py, <&CueList as IntoPy<PyCueList>>::into_py(content, py)?)?,
            )?;
        }
        Content::ExtendedCueList(content) => {
            result.set_item("tag", "PCO2")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&ExtendedCueList as IntoPy<PyExtendedCueList>>::into_py(content, py)?,
                )?,
            )?;
        }
        Content::Path(content) => {
            result.set_item("tag", "PPTH")?;
            result.set_item("content", content.path.to_string())?;
        }
        Content::VBR(content) => {
            result.set_item("tag", "PVBR")?;
            result.set_item(
                "content",
                Py::new(py, <&VBR as IntoPy<PyVBR>>::into_py(content, py)?)?,
            )?;
        }
        Content::SeekIndex(content) => {
            result.set_item("tag", "PVB2")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&SeekIndex as IntoPy<PySeekIndex>>::into_py(content, py)?,
                )?,
            )?;
        }
        Content::TinyWaveformPreview(content) => {
            result.set_item("tag", "PWV2")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&TinyWaveformPreview as IntoPy<PyTinyWaveformPreview>>::into_py(content, py)?,
                )?,
            )?;
        }
        Content::WaveformPreview(content) => {
            result.set_item("tag", "PWAV")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&WaveformPreview as IntoPy<PyWaveformPreview>>::into_py(content, py)?,
                )?,
            )?;
        }
        Content::WaveformDetail(content) => {
            result.set_item("tag", "PWV3")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&WaveformDetail as IntoPy<PyWaveformDetail>>::into_py(content, py)?,
                )?,
            )?;
        }
        Content::WaveformColorPreview(content) => {
            result.set_item("tag", "PWV4")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&WaveformColorPreview as IntoPy<PyWaveformColorPreview>>::into_py(
                        content, py,
                    )?,
                )?,
            )?;
        }
        Content::WaveformColorDetail(content) => {
            result.set_item("tag", "PWV5")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&WaveformColorDetail as IntoPy<PyWaveformColorDetail>>::into_py(content, py)?,
                )?,
            )?;
        }
        Content::Waveform3BandPreview(content) => {
            result.set_item("tag", "PWV6")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&Waveform3BandPreview as IntoPy<PyWaveform3BandPreview>>::into_py(
                        content, py,
                    )?,
                )?,
            )?;
        }
        Content::Waveform3BandDetail(content) => {
            result.set_item("tag", "PWV7")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&Waveform3BandDetail as IntoPy<PyWaveform3BandDetail>>::into_py(content, py)?,
                )?,
            )?;
        }
        Content::Waveform3BandCalibration(content) => {
            result.set_item("tag", "PWVC")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&Waveform3BandCalibration as IntoPy<PyWaveform3BandCalibration>>::into_py(
                        content, py,
                    )?,
                )?,
            )?;
        }
        Content::SongStructure(content) => {
            result.set_item("tag", "PSSI")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&SongStructure as IntoPy<PySongStructure>>::into_py(content, py)?,
                )?,
            )?;
        }
        Content::VocalDetection(content) => {
            result.set_item("tag", "PVDI")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    <&VocalDetection as IntoPy<PyVocalDetection>>::into_py(content, py)?,
                )?,
            )?;
        }
        Content::Unknown(content) => {
            result.set_item("tag", "Unknown")?;
            result.set_item(
                "content",
                Py::new(
                    py,
                    PyUnknownSection {
                        code: value.tag.code().to_vec(),
                        header_data: content.header_data().to_vec(),
                        content_data: content.content_data().to_vec(),
                    },
                )?,
            )?;
        }
    }
    Ok(result)
}
