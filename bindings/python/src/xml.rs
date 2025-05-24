// Author: Dylan Jones
// Date:   2025-05-15

use chrono::NaiveDate;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyType};
use rbox::xml::{PlaylistKeyType, PositionMark, RekordboxXml, Tempo, Track};
use serde::{Deserialize, Serialize};

use super::errors::XmlError;
use super::util::{model_to_pydict, models_to_pylist, pydict_to_model};

/// Tempo element representing the beat grid of a track.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PyTempo {
    pub inizio: f64,
    pub bpm: f64,
    pub metro: String,
    pub battito: i32,
}

impl From<&Tempo> for PyTempo {
    fn from(tempo: &Tempo) -> Self {
        Self {
            inizio: tempo.inizio,
            bpm: tempo.bpm,
            metro: tempo.metro.clone(),
            battito: tempo.battito,
        }
    }
}

impl Into<Tempo> for PyTempo {
    fn into(self) -> Tempo {
        Tempo {
            inizio: self.inizio,
            bpm: self.bpm,
            metro: self.metro,
            battito: self.battito,
        }
    }
}

/// Position element for storing position markers like cue points of a track
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PyPositionMark {
    pub name: String,
    pub mark_type: i32,
    pub start: f64,
    pub end: Option<f64>,
    pub num: i32,
}

impl From<&PositionMark> for PyPositionMark {
    fn from(mark: &PositionMark) -> Self {
        Self {
            name: mark.name.clone(),
            mark_type: mark.mark_type,
            start: mark.start,
            end: mark.end,
            num: mark.num,
        }
    }
}

impl Into<PositionMark> for PyPositionMark {
    fn into(self) -> PositionMark {
        PositionMark {
            name: self.name,
            mark_type: self.mark_type,
            start: self.start,
            end: self.end,
            num: self.num,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PyTrack {
    pub trackid: String,
    pub name: Option<String>,
    pub artist: Option<String>,
    pub composer: Option<String>,
    pub album: Option<String>,
    pub grouping: Option<String>,
    pub genre: Option<String>,
    pub kind: Option<String>,
    pub size: Option<i64>,
    pub totaltime: Option<f64>,
    pub discnumber: Option<i32>,
    pub tracknumber: Option<i32>,
    pub year: Option<i32>,
    pub averagebpm: Option<f64>,
    pub datemodified: Option<NaiveDate>,
    pub dateadded: Option<NaiveDate>,
    pub bitrate: Option<i32>,
    pub samplerate: Option<f64>,
    pub comments: Option<String>,
    pub playcount: Option<i32>,
    pub lastplayed: Option<NaiveDate>,
    pub rating: Option<i32>,
    pub location: String,
    pub remixer: Option<String>,
    pub tonality: Option<String>,
    pub label: Option<String>,
    pub mix: Option<String>,
    pub colour: Option<String>,
    pub tempos: Vec<PyTempo>,
    pub position_marks: Vec<PyPositionMark>,
}

impl From<&Track> for PyTrack {
    fn from(track: &Track) -> Self {
        let tempos: Vec<PyTempo> = track.tempos.iter().map(|tempo| tempo.into()).collect();
        let marks: Vec<PyPositionMark> = track
            .position_marks
            .iter()
            .map(|mark| mark.into())
            .collect();
        Self {
            trackid: track.trackid.clone(),
            name: track.name.clone(),
            artist: track.artist.clone(),
            composer: track.composer.clone(),
            album: track.album.clone(),
            grouping: track.grouping.clone(),
            genre: track.genre.clone(),
            kind: track.kind.clone(),
            size: track.size,
            totaltime: track.totaltime,
            discnumber: track.discnumber,
            tracknumber: track.tracknumber,
            year: track.year,
            averagebpm: track.averagebpm,
            datemodified: track.datemodified,
            dateadded: track.dateadded,
            bitrate: track.bitrate,
            samplerate: track.samplerate,
            comments: track.comments.clone(),
            playcount: track.playcount,
            lastplayed: track.lastplayed,
            rating: track.rating,
            location: track.location.clone(),
            remixer: track.remixer.clone(),
            tonality: track.tonality.clone(),
            label: track.label.clone(),
            mix: track.mix.clone(),
            colour: track.colour.clone(),
            tempos: tempos,
            position_marks: marks,
        }
    }
}

impl Into<Track> for PyTrack {
    fn into(self) -> Track {
        let tempos: Vec<Tempo> = self
            .tempos
            .iter()
            .map(|tempo| tempo.clone().into())
            .collect();
        let marks: Vec<PositionMark> = self
            .position_marks
            .iter()
            .map(|mark| mark.clone().into())
            .collect();
        Track {
            trackid: self.trackid.clone(),
            name: self.name.clone(),
            artist: self.artist.clone(),
            composer: self.composer.clone(),
            album: self.album.clone(),
            grouping: self.grouping.clone(),
            genre: self.genre.clone(),
            kind: self.kind.clone(),
            size: self.size,
            totaltime: self.totaltime,
            discnumber: self.discnumber,
            tracknumber: self.tracknumber,
            year: self.year,
            averagebpm: self.averagebpm,
            datemodified: self.datemodified,
            dateadded: self.dateadded,
            bitrate: self.bitrate,
            samplerate: self.samplerate,
            comments: self.comments.clone(),
            playcount: self.playcount,
            lastplayed: self.lastplayed,
            rating: self.rating,
            location: self.location.clone(),
            remixer: self.remixer.clone(),
            tonality: self.tonality.clone(),
            label: self.label.clone(),
            mix: self.mix.clone(),
            colour: self.colour.clone(),
            tempos: tempos,
            position_marks: marks,
        }
    }
}

#[pyclass(unsendable)]
pub struct PyRekordboxXml {
    xml: RekordboxXml,
}

#[pymethods]
impl PyRekordboxXml {
    #[new]
    pub fn new(path: &str) -> PyResult<Self> {
        let xml = RekordboxXml::load(path);
        Ok(Self { xml })
    }

    #[classmethod]
    fn load(_cls: &Bound<'_, PyType>, path: &str) -> PyResult<Self> {
        let xml = RekordboxXml::load(path);
        Ok(PyRekordboxXml { xml })
    }

    pub fn dump_copy(&mut self, path: &str) -> PyResult<()> {
        self.xml
            .dump_copy(path)
            .map_err(|e| PyErr::new::<XmlError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn dump(&mut self) -> PyResult<()> {
        self.xml
            .dump()
            .map_err(|e| PyErr::new::<XmlError, _>(e.to_string()))?;
        Ok(())
    }

    // -- Track handling --------------------------------------------------------------------------

    pub fn get_tracks(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models: Vec<PyTrack> = self
            .xml
            .get_tracks()
            .iter()
            .map(|track| track.into())
            .collect();
        let result = models_to_pylist(py, models.to_vec())?;
        Ok(result.into())
    }

    pub fn get_track_by_id(&mut self, py: Python, track_id: &str) -> PyResult<Option<Py<PyDict>>> {
        let model = self.xml.get_track_by_id(track_id);
        if let Some(model) = model {
            let item: PyTrack = (&model).into();
            let result = model_to_pydict(py, item)?;
            Ok(Some(result.into()))
        } else {
            Ok(None)
        }
    }

    pub fn get_track_by_location(
        &mut self,
        py: Python,
        location: &str,
    ) -> PyResult<Option<Py<PyDict>>> {
        let model = self.xml.get_track_by_location(location);
        if let Some(model) = model {
            let item: PyTrack = (&model).into();
            let result = model_to_pydict(py, item)?;
            Ok(Some(result.into()))
        } else {
            Ok(None)
        }
    }

    pub fn get_track_by_key(
        &mut self,
        py: Python,
        key: &str,
        key_type: i32,
    ) -> PyResult<Option<Py<PyDict>>> {
        let key_type: PlaylistKeyType = key_type.try_into().expect("invalid key type");
        let model = self.xml.get_track_by_key(key, key_type);
        if let Some(model) = model {
            let item: PyTrack = (&model).into();
            let result = model_to_pydict(py, item)?;
            Ok(Some(result.into()))
        } else {
            Ok(None)
        }
    }

    pub fn add_track(&mut self, py: Python, track: &Bound<'_, PyDict>) -> PyResult<()> {
        let track: PyTrack = pydict_to_model(py, track)?;
        let track_data: Track = track.into();
        self.xml.add_track(track_data);
        Ok(())
    }

    pub fn update_track(&mut self, py: Python, track: &Bound<'_, PyDict>) -> PyResult<()> {
        let track: PyTrack = pydict_to_model(py, track)?;
        let track_data: Track = track.into();
        self.xml
            .update_track(track_data)
            .map_err(|e| PyErr::new::<XmlError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn remove_track(&mut self, track_id: &str) -> PyResult<()> {
        self.xml
            .remove_track(track_id)
            .map_err(|e| PyErr::new::<XmlError, _>(e.to_string()))?;
        Ok(())
    }

    // -- Playlist handling -----------------------------------------------------------------------
}
