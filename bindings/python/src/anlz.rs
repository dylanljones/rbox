// Author: Dylan Jones
// Date:   2025-05-10

use pyo3::prelude::*;
use rbox::anlz::anlz::*;

use super::errors::AnlzError;
use super::traits::IntoPy;

#[pyclass(name = "Beat", unsendable, get_all, set_all)]
pub struct PyBeat {
    pub beat_number: u16,
    pub tempo: u16,
    pub time: u32,
}

impl IntoPy<PyBeat> for Beat {
    fn into_py(self, _py: Python) -> PyResult<PyBeat> {
        let model = PyBeat {
            beat_number: self.beat_number,
            tempo: self.tempo,
            time: self.time,
        };
        Ok(model)
    }
}

#[pyclass(name = "ExtBeat", unsendable, get_all, set_all)]
pub struct PyExtBeat {
    pub beat_number: u8,
}

impl IntoPy<PyExtBeat> for ExtBeat {
    fn into_py(self, _py: Python) -> PyResult<PyExtBeat> {
        let model = PyExtBeat {
            beat_number: self.beat_number,
        };
        Ok(model)
    }
}

#[pyclass(name = "BeatGrid", unsendable, get_all, set_all)]
pub struct PyBeatGrid {
    pub beats: Vec<Py<PyBeat>>,
}

impl IntoPy<PyBeatGrid> for BeatGrid {
    fn into_py(self, py: Python) -> PyResult<PyBeatGrid> {
        let mut beats = Vec::new();
        for item in self.beats {
            beats.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PyBeatGrid { beats };
        Ok(model)
    }
}

#[pyclass(name = "ExtendedBeatGrid", unsendable, get_all, set_all)]
pub struct PyExtendedBeatGrid {
    pub bpm: Vec<Py<PyBeat>>,
    pub beats: Vec<Py<PyExtBeat>>,
}

impl IntoPy<PyExtendedBeatGrid> for ExtendedBeatGrid {
    fn into_py(self, py: Python) -> PyResult<PyExtendedBeatGrid> {
        let mut bpm = Vec::new();
        let mut beats = Vec::new();
        for item in self.bpm {
            bpm.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        for item in self.beats {
            beats.push(Py::new(py, item.clone().into_py(py)?)?)
        }

        let model = PyExtendedBeatGrid { bpm, beats };
        Ok(model)
    }
}

#[pyclass(name = "Cue", unsendable, get_all, set_all)]
pub struct PyCue {
    pub hot_cue: u32,
    pub status: u16,
    pub order_first: u16,
    pub order_last: u16,
    pub cue_type: u16,
    pub time: u32,
    pub loop_time: u32,
}

impl IntoPy<PyCue> for Cue {
    fn into_py(self, _py: Python) -> PyResult<PyCue> {
        let model = PyCue {
            hot_cue: self.hot_cue,
            status: self.status.into(),
            order_first: self.order_first,
            order_last: self.order_last,
            cue_type: self.cue_type.into(),
            time: self.time,
            loop_time: self.loop_time,
        };
        Ok(model)
    }
}

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
}

impl IntoPy<PyExtendedCue> for ExtendedCue {
    fn into_py(self, _py: Python) -> PyResult<PyExtendedCue> {
        let model = PyExtendedCue {
            hot_cue: self.hot_cue,
            cue_type: self.cue_type.into(),
            time: self.time,
            loop_time: self.loop_time,
            color: self.color,
            loop_numerator: self.loop_numerator,
            loop_denominator: self.loop_denominator,
            comment: self.comment.to_string(),
            hot_cue_color_index: self.hot_cue_color_index,
            hot_cue_color_red: self.hot_cue_color_rgb.0,
            hot_cue_color_green: self.hot_cue_color_rgb.1,
            hot_cue_color_blue: self.hot_cue_color_rgb.2,
        };
        Ok(model)
    }
}

#[pyclass(name = "CueList", unsendable, get_all, set_all)]
pub struct PyCueList {
    pub list_type: u16,
    pub cues: Vec<Py<PyCue>>,
}

impl IntoPy<PyCueList> for CueList {
    fn into_py(self, py: Python) -> PyResult<PyCueList> {
        let mut cues = Vec::new();
        for item in self.cues {
            cues.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PyCueList {
            list_type: self.list_type.into(),
            cues,
        };
        Ok(model)
    }
}

#[pyclass(name = "ExtendedCueList", unsendable, get_all, set_all)]
pub struct PyExtendedCueList {
    pub list_type: u16,
    pub cues: Vec<Py<PyExtendedCue>>,
}

impl IntoPy<PyExtendedCueList> for ExtendedCueList {
    fn into_py(self, py: Python) -> PyResult<PyExtendedCueList> {
        let mut cues = Vec::new();
        for item in self.cues {
            cues.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PyExtendedCueList {
            list_type: self.list_type.into(),
            cues,
        };
        Ok(model)
    }
}

#[pyclass(name = "TinyWaveformColumn", unsendable, get_all, set_all)]
pub struct PyTinyWaveformColumn {
    pub height: u8,
}

impl IntoPy<PyTinyWaveformColumn> for TinyWaveformColumn {
    fn into_py(self, _py: Python) -> PyResult<PyTinyWaveformColumn> {
        let model = PyTinyWaveformColumn {
            height: self.height(),
        };
        Ok(model)
    }
}

#[pyclass(name = "TinyWaveformPreview", unsendable, get_all, set_all)]
pub struct PyTinyWaveformPreview {
    pub data: Vec<Py<PyTinyWaveformColumn>>,
}

impl IntoPy<PyTinyWaveformPreview> for TinyWaveformPreview {
    fn into_py(self, py: Python) -> PyResult<PyTinyWaveformPreview> {
        let mut data = Vec::new();
        for item in self.data {
            data.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PyTinyWaveformPreview { data };
        Ok(model)
    }
}

#[pyclass(name = "WaveformColumn", unsendable, get_all, set_all)]
pub struct PyWaveformColumn {
    pub height: u8,
    pub whiteness: u8,
}

impl IntoPy<PyWaveformColumn> for WaveformColumn {
    fn into_py(self, _py: Python) -> PyResult<PyWaveformColumn> {
        let model = PyWaveformColumn {
            height: self.height(),
            whiteness: self.whiteness(),
        };
        Ok(model)
    }
}

#[pyclass(name = "WaveformPreview", unsendable, get_all, set_all)]
pub struct PyWaveformPreview {
    pub data: Vec<Py<PyWaveformColumn>>,
}

impl IntoPy<PyWaveformPreview> for WaveformPreview {
    fn into_py(self, py: Python) -> PyResult<PyWaveformPreview> {
        let mut data = Vec::new();
        for item in self.data {
            data.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PyWaveformPreview { data };
        Ok(model)
    }
}

#[pyclass(name = "WaveformDetail", unsendable, get_all, set_all)]
pub struct PyWaveformDetail {
    pub data: Vec<Py<PyWaveformColumn>>,
}

impl IntoPy<PyWaveformDetail> for WaveformDetail {
    fn into_py(self, py: Python) -> PyResult<PyWaveformDetail> {
        let mut data = Vec::new();
        for item in self.data {
            data.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PyWaveformDetail { data };
        Ok(model)
    }
}

#[pyclass(name = "WaveformColorPreviewColumn", unsendable, get_all, set_all)]
pub struct PyWaveformColorPreviewColumn {
    pub energy_bottom_half_freq: u8,
    pub energy_bottom_third_freq: u8,
    pub energy_mid_third_freq: u8,
    pub energy_top_third_freq: u8,
}

impl IntoPy<PyWaveformColorPreviewColumn> for WaveformColorPreviewColumn {
    fn into_py(self, _py: Python) -> PyResult<PyWaveformColorPreviewColumn> {
        let model = PyWaveformColorPreviewColumn {
            energy_bottom_half_freq: self.energy_bottom_half_freq,
            energy_bottom_third_freq: self.energy_bottom_third_freq,
            energy_mid_third_freq: self.energy_mid_third_freq,
            energy_top_third_freq: self.energy_top_third_freq,
        };
        Ok(model)
    }
}

#[pyclass(name = "WaveformColorPreview", unsendable, get_all, set_all)]
pub struct PyWaveformColorPreview {
    pub data: Vec<Py<PyWaveformColorPreviewColumn>>,
}

impl IntoPy<PyWaveformColorPreview> for WaveformColorPreview {
    fn into_py(self, py: Python) -> PyResult<PyWaveformColorPreview> {
        let mut data = Vec::new();
        for item in self.data {
            data.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PyWaveformColorPreview { data };
        Ok(model)
    }
}

#[pyclass(name = "WaveformColorDetailColumn", unsendable, get_all, set_all)]
pub struct PyWaveformColorDetailColumn {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub height: u8,
}

impl IntoPy<PyWaveformColorDetailColumn> for WaveformColorDetailColumn {
    fn into_py(self, _py: Python) -> PyResult<PyWaveformColorDetailColumn> {
        let model = PyWaveformColorDetailColumn {
            red: self.red(),
            green: self.green(),
            blue: self.blue(),
            height: self.height(),
        };
        Ok(model)
    }
}

#[pyclass(name = "WaveformColorDetail", unsendable, get_all, set_all)]
pub struct PyWaveformColorDetail {
    pub data: Vec<Py<PyWaveformColorDetailColumn>>,
}

impl IntoPy<PyWaveformColorDetail> for WaveformColorDetail {
    fn into_py(self, py: Python) -> PyResult<PyWaveformColorDetail> {
        let mut data = Vec::new();
        for item in self.data {
            data.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PyWaveformColorDetail { data };
        Ok(model)
    }
}

#[pyclass(name = "Waveform3BandColumn", unsendable, get_all, set_all)]
pub struct PyWaveform3BandColumn {
    pub mid: u8,
    pub high: u8,
    pub low: u8,
}

impl IntoPy<PyWaveform3BandColumn> for Waveform3BandColumn {
    fn into_py(self, _py: Python) -> PyResult<PyWaveform3BandColumn> {
        let model = PyWaveform3BandColumn {
            mid: self.mid(),
            high: self.high(),
            low: self.low(),
        };
        Ok(model)
    }
}

#[pyclass(name = "Waveform3BandPreview", unsendable, get_all, set_all)]
pub struct PyWaveform3BandPreview {
    pub data: Vec<Py<PyWaveform3BandColumn>>,
}

impl IntoPy<PyWaveform3BandPreview> for Waveform3BandPreview {
    fn into_py(self, py: Python) -> PyResult<PyWaveform3BandPreview> {
        let mut data = Vec::new();
        for item in self.data {
            data.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PyWaveform3BandPreview { data };
        Ok(model)
    }
}

#[pyclass(name = "Waveform3BandDetail", unsendable, get_all, set_all)]
pub struct PyWaveform3BandDetail {
    pub data: Vec<Py<PyWaveform3BandColumn>>,
}

impl IntoPy<PyWaveform3BandDetail> for Waveform3BandDetail {
    fn into_py(self, py: Python) -> PyResult<PyWaveform3BandDetail> {
        let mut data = Vec::new();
        for item in self.data {
            data.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PyWaveform3BandDetail { data };
        Ok(model)
    }
}

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
}

impl IntoPy<PyPhrase> for Phrase {
    fn into_py(self, _py: Python) -> PyResult<PyPhrase> {
        let model = PyPhrase {
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
        };
        Ok(model)
    }
}

#[pyclass(name = "SongStructureData", unsendable, get_all, set_all)]
pub struct PySongStructureData {
    pub mood: u16,
    pub end_beat: u16,
    pub bank: u16,
    pub phrases: Vec<Py<PyPhrase>>,
}

impl IntoPy<PySongStructureData> for SongStructureData {
    fn into_py(self, py: Python) -> PyResult<PySongStructureData> {
        let mut phrases = Vec::new();
        for item in self.phrases {
            phrases.push(Py::new(py, item.clone().into_py(py)?)?)
        }
        let model = PySongStructureData {
            mood: self.mood.into(),
            end_beat: self.end_beat,
            bank: self.bank.into(),
            phrases,
        };
        Ok(model)
    }
}

#[pyclass(unsendable)]
pub struct PyAnlz {
    anlz: Anlz,
}

#[pymethods]
impl PyAnlz {
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

    pub fn contains(&mut self, tag: &str) -> PyResult<bool> {
        let tag_type = AnlzTag::from(tag.to_string());
        let contains = self.anlz.contains(tag_type);
        Ok(contains)
    }

    pub fn get_tags(&mut self) -> PyResult<Vec<String>> {
        let tags = self
            .anlz
            .get_tags()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        Ok(tags.clone())
    }

    pub fn get_beat_grid(&mut self, py: Python) -> PyResult<Option<PyBeatGrid>> {
        let data = self.anlz.get_beat_grid();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_extended_beat_grid(&mut self, py: Python) -> PyResult<Option<PyExtendedBeatGrid>> {
        let data = self.anlz.get_extended_beat_grid();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_hot_cues(&mut self, py: Python) -> PyResult<Option<PyCueList>> {
        let data = self.anlz.get_hot_cues();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_memory_cues(&mut self, py: Python) -> PyResult<Option<PyCueList>> {
        let data = self.anlz.get_memory_cues();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_extended_hot_cues(&mut self, py: Python) -> PyResult<Option<PyExtendedCueList>> {
        let data = self.anlz.get_extended_hot_cues();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_extended_memory_cues(&mut self, py: Python) -> PyResult<Option<PyExtendedCueList>> {
        let data = self.anlz.get_extended_memory_cues();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_path(&mut self) -> PyResult<Option<String>> {
        let path = self.anlz.get_path();
        if let Some(path) = path {
            return Ok(Some(path));
        }
        Ok(None)
    }

    pub fn set_path(&mut self, path: &str) -> PyResult<()> {
        self.anlz.set_path(path)?;
        Ok(())
    }

    pub fn get_vbr_data(&mut self) -> PyResult<Option<Vec<u8>>> {
        let vbr = self.anlz.get_vbr_data();
        Ok(vbr)
    }

    pub fn get_tiny_waveform_preview(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyTinyWaveformPreview>> {
        let data = self.anlz.get_tiny_waveform_preview();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_preview(&mut self, py: Python) -> PyResult<Option<PyWaveformPreview>> {
        let data = self.anlz.get_waveform_preview();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_detail(&mut self, py: Python) -> PyResult<Option<PyWaveformDetail>> {
        let data = self.anlz.get_waveform_detail();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_color_preview(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyWaveformColorPreview>> {
        let data = self.anlz.get_waveform_color_preview();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_color_detail(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyWaveformColorDetail>> {
        let data = self.anlz.get_waveform_color_detail();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_3band_preview(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyWaveform3BandPreview>> {
        let data = self.anlz.get_waveform_3band_preview();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_waveform_3band_detail(
        &mut self,
        py: Python,
    ) -> PyResult<Option<PyWaveform3BandDetail>> {
        let data = self.anlz.get_waveform_3band_detail();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }

    pub fn get_song_structure(&mut self, py: Python) -> PyResult<Option<PySongStructureData>> {
        let data = self.anlz.get_song_structure();
        if let Some(data) = data {
            return Ok(Some(data.clone().into_py(py)?));
        }
        Ok(None)
    }
}
