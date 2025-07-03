// Author: Dylan Jones
// Date:   2025-05-15

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyList, PyType};
use rbox::xml::{
    PlaylistKeyType, PlaylistNode, PlaylistTrack, PositionMark, RekordboxXml, Tempo, Track,
};

use super::errors::XmlError;
use super::traits::{FromPy, IntoPy};

#[pyclass(name = "Tempo", unsendable, get_all, set_all)]
pub struct PyTempo {
    pub inizio: f64,
    pub bpm: f64,
    pub metro: String,
    pub battito: i32,
}

impl IntoPy<PyTempo> for Tempo {
    fn into_py(self, _py: Python) -> PyResult<PyTempo> {
        let model = PyTempo {
            inizio: self.inizio,
            bpm: self.bpm,
            metro: self.metro,
            battito: self.battito,
        };
        Ok(model)
    }
}

impl FromPy<PyTempo> for Tempo {
    fn from_py(_py: Python, model: PyRef<'_, PyTempo>) -> Self {
        Self {
            inizio: model.inizio,
            bpm: model.bpm,
            metro: model.metro.clone(),
            battito: model.battito,
        }
    }
}

/// Position element for storing position markers like cue points of a track
#[pyclass(name = "PositionMark", unsendable, get_all, set_all)]
pub struct PyPositionMark {
    pub name: String,
    pub mark_type: i32,
    pub start: f64,
    pub end: Option<f64>,
    pub num: i32,
}

impl IntoPy<PyPositionMark> for PositionMark {
    fn into_py(self, _py: Python) -> PyResult<PyPositionMark> {
        let model = PyPositionMark {
            name: self.name,
            mark_type: self.mark_type,
            start: self.start,
            end: self.end,
            num: self.num,
        };
        Ok(model)
    }
}

impl FromPy<PyPositionMark> for PositionMark {
    fn from_py(_py: Python, model: PyRef<'_, PyPositionMark>) -> Self {
        Self {
            name: model.name.clone(),
            mark_type: model.mark_type,
            start: model.start,
            end: model.end,
            num: model.num,
        }
    }
}

#[pyclass(name = "Track", unsendable, get_all, set_all)]
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
    pub datemodified: Option<Py<PyDateTime>>,
    pub dateadded: Option<Py<PyDateTime>>,
    pub bitrate: Option<i32>,
    pub samplerate: Option<f64>,
    pub comments: Option<String>,
    pub playcount: Option<i32>,
    pub lastplayed: Option<Py<PyDateTime>>,
    pub rating: Option<i32>,
    pub location: String,
    pub remixer: Option<String>,
    pub tonality: Option<String>,
    pub label: Option<String>,
    pub mix: Option<String>,
    pub color: Option<String>,
    pub tempos: Vec<Py<PyTempo>>,
    pub position_marks: Vec<Py<PyPositionMark>>,
}

impl IntoPy<PyTrack> for Track {
    fn into_py(self, py: Python) -> PyResult<PyTrack> {
        const MIDNIGHT: NaiveTime = NaiveTime::from_hms_opt(0, 0, 0).unwrap();

        let mut tempos = Vec::new();
        let mut position_marks = Vec::new();
        for item in self.tempos {
            let py_item = item.clone().into_py(py)?;
            tempos.push(Py::new(py, py_item)?);
        }
        for item in self.position_marks {
            let py_item = item.clone().into_py(py)?;
            position_marks.push(Py::new(py, py_item)?);
        }

        let datemodified: Option<Py<PyDateTime>> = if let Some(dt) = self.datemodified {
            let date = dt
                .and_time(MIDNIGHT)
                .and_local_timezone(Utc)
                .earliest()
                .unwrap();
            Some(date.into_pyobject(py)?.into())
        } else {
            None
        };
        let dateadded: Option<Py<PyDateTime>> = if let Some(dt) = self.dateadded {
            let date = dt
                .and_time(MIDNIGHT)
                .and_local_timezone(Utc)
                .earliest()
                .unwrap();
            Some(date.into_pyobject(py)?.into())
        } else {
            None
        };
        let lastplayed: Option<Py<PyDateTime>> = if let Some(dt) = self.lastplayed {
            let date = dt
                .and_time(MIDNIGHT)
                .and_local_timezone(Utc)
                .earliest()
                .unwrap();
            Some(date.into_pyobject(py)?.into())
        } else {
            None
        };

        let model = PyTrack {
            trackid: self.trackid,
            name: self.name,
            artist: self.artist,
            composer: self.composer,
            album: self.album,
            grouping: self.grouping,
            genre: self.genre,
            kind: self.kind,
            size: self.size,
            totaltime: self.totaltime,
            discnumber: self.discnumber,
            tracknumber: self.tracknumber,
            year: self.year,
            averagebpm: self.averagebpm,
            datemodified,
            dateadded,
            bitrate: self.bitrate,
            samplerate: self.samplerate,
            comments: self.comments,
            playcount: self.playcount,
            lastplayed,
            rating: self.rating,
            location: self.location,
            remixer: self.remixer,
            tonality: self.tonality,
            label: self.label,
            mix: self.mix,
            color: self.color,
            tempos,
            position_marks,
        };
        Ok(model)
    }
}

impl FromPy<PyTrack> for Track {
    fn from_py(py: Python, model: PyRef<'_, PyTrack>) -> Self {
        let mut tempos: Vec<Tempo> = Vec::new();
        let mut position_marks: Vec<PositionMark> = Vec::new();
        for item in &model.tempos {
            tempos.push(Tempo::from_py(py, item.borrow(py)));
        }
        for item in &model.position_marks {
            position_marks.push(PositionMark::from_py(py, item.borrow(py)));
        }

        let datemodified: Option<NaiveDate> = if let Some(dt) = &model.datemodified {
            let date: DateTime<Utc> = dt.extract::<DateTime<Utc>>(py).unwrap();
            Some(date.date_naive())
        } else {
            None
        };
        let dateadded: Option<NaiveDate> = if let Some(dt) = &model.dateadded {
            let date: DateTime<Utc> = dt.extract::<DateTime<Utc>>(py).unwrap();
            Some(date.date_naive())
        } else {
            None
        };
        let lastplayed: Option<NaiveDate> = if let Some(dt) = &model.lastplayed {
            let date: DateTime<Utc> = dt.extract::<DateTime<Utc>>(py).unwrap();
            Some(date.date_naive())
        } else {
            None
        };

        Self {
            trackid: model.trackid.clone(),
            name: model.name.clone(),
            artist: model.artist.clone(),
            composer: model.composer.clone(),
            album: model.album.clone(),
            grouping: model.grouping.clone(),
            genre: model.genre.clone(),
            kind: model.kind.clone(),
            size: model.size,
            totaltime: model.totaltime,
            discnumber: model.discnumber,
            tracknumber: model.tracknumber,
            year: model.year,
            averagebpm: model.averagebpm,
            datemodified,
            dateadded,
            bitrate: model.bitrate,
            samplerate: model.samplerate,
            comments: model.comments.clone(),
            playcount: model.playcount,
            lastplayed,
            rating: model.rating,
            location: model.location.clone(),
            remixer: model.remixer.clone(),
            tonality: model.tonality.clone(),
            label: model.label.clone(),
            mix: model.mix.clone(),
            color: model.color.clone(),
            tempos,
            position_marks,
        }
    }
}

#[pyclass(name = "PlaylistTrack", unsendable, get_all, set_all)]
pub struct PyPlaylistTrack {
    pub key: String,
}

impl IntoPy<PyPlaylistTrack> for PlaylistTrack {
    fn into_py(self, _py: Python) -> PyResult<PyPlaylistTrack> {
        Ok(PyPlaylistTrack { key: self.key })
    }
}

impl FromPy<PyPlaylistTrack> for PlaylistTrack {
    fn from_py(_py: Python, model: PyRef<'_, PyPlaylistTrack>) -> Self {
        Self {
            key: model.key.clone(),
        }
    }
}

#[pyclass(name = "PlaylistNode", unsendable, get_all, set_all)]
pub struct PyPlaylistNode {
    pub name: String,
    pub node_type: u16,
    pub key_type: Option<u16>,
    pub tracks: Option<Vec<Py<PyPlaylistTrack>>>,
    pub nodes: Option<Vec<Py<PyPlaylistNode>>>,
    pub node_path: Vec<String>,
}

impl IntoPy<PyPlaylistNode> for PlaylistNode {
    fn into_py(self, py: Python) -> PyResult<PyPlaylistNode> {
        let tracks = if let Some(items) = self.tracks {
            let mut vec = Vec::new();
            for item in items {
                let py_item = item.clone().into_py(py)?;
                vec.push(Py::new(py, py_item)?);
            }
            Some(vec)
        } else {
            None
        };
        let nodes = if let Some(items) = self.nodes {
            let mut vec = Vec::new();
            for item in items {
                let py_item = item.clone().into_py(py)?;
                vec.push(Py::new(py, py_item)?);
            }
            Some(vec)
        } else {
            None
        };

        let model = PyPlaylistNode {
            name: self.name,
            node_type: self.node_type.into(),
            key_type: self.key_type.map_or(None, |key| Some(key.into())),
            tracks,
            nodes,
            node_path: self.node_path,
        };
        Ok(model)
    }
}

impl FromPy<PyPlaylistNode> for PlaylistNode {
    fn from_py(py: Python, model: PyRef<'_, PyPlaylistNode>) -> Self {
        let tracks: Option<Vec<PlaylistTrack>> = if let Some(items) = &model.tracks {
            let mut data: Vec<PlaylistTrack> = Vec::new();
            for item in items {
                data.push(PlaylistTrack::from_py(py, item.borrow(py)));
            }
            Some(data)
        } else {
            None
        };

        let nodes: Option<Vec<PlaylistNode>> = if let Some(items) = &model.nodes {
            let mut data: Vec<PlaylistNode> = Vec::new();
            for item in items {
                data.push(PlaylistNode::from_py(py, item.borrow(py)));
            }
            Some(data)
        } else {
            None
        };

        Self {
            name: model.name.clone(),
            node_type: model.node_type.try_into().expect("invalid node type"),
            key_type: model
                .key_type
                .map_or(None, |key| Some(key.try_into().expect("invalid key type"))),
            tracks,
            nodes,
            node_path: model.node_path.clone(),
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
        let data = PyList::empty(py);
        for item in self.xml.get_tracks() {
            let py_item = item.clone().into_py(py)?;
            let _ = data.append(py_item);
        }
        Ok(data.into())
    }

    pub fn get_track_by_id(&mut self, py: Python, track_id: &str) -> PyResult<Option<PyTrack>> {
        let data = self.xml.get_track_by_id(track_id);
        if let Some(data) = data {
            Ok(Some(data.clone().into_py(py)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_track_by_location(
        &mut self,
        py: Python,
        location: &str,
    ) -> PyResult<Option<PyTrack>> {
        let data = self.xml.get_track_by_location(location);
        if let Some(data) = data {
            Ok(Some(data.clone().into_py(py)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_track_by_key(
        &mut self,
        py: Python,
        key: &str,
        key_type: i32,
    ) -> PyResult<Option<PyTrack>> {
        let key_type: PlaylistKeyType = key_type.try_into().expect("invalid key type");
        let data = self.xml.get_track_by_key(key, key_type);
        if let Some(data) = data {
            Ok(Some(data.clone().into_py(py)?))
        } else {
            Ok(None)
        }
    }

    pub fn add_track(&mut self, py: Python, track: PyRef<'_, PyTrack>) -> PyResult<()> {
        self.xml.add_track(Track::from_py(py, track));
        Ok(())
    }

    pub fn update_track(&mut self, py: Python, track: PyRef<'_, PyTrack>) -> PyResult<()> {
        let data = Track::from_py(py, track);
        self.xml
            .update_track(data)
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

    pub fn get_playlist_root(&mut self, py: Python) -> PyResult<PyPlaylistNode> {
        let data = self.xml.get_playlist_root();
        data.clone().into_py(py)
    }
}
