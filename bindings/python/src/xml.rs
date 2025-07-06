// Author: Dylan Jones
// Date:   2025-05-15

use super::errors::XmlError;
use super::traits::{FromPy, IntoPy};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyType};
use rbox::xml::{
    PlaylistKeyType, PlaylistNode, PlaylistNodeType, PlaylistTrack, PositionMark, RekordboxXml,
    Tempo, Track,
};

#[derive(Debug, Clone)]
#[pyclass(name = "Tempo", unsendable, get_all, set_all)]
pub struct PyTempo {
    pub inizio: f64,
    pub bpm: f64,
    pub metro: String,
    pub battito: u16,
}

#[pymethods]
impl PyTempo {
    #[new]
    fn new(inizio: f64, bpm: f64, metro: &str, battito: u16) -> Self {
        Self {
            inizio,
            bpm,
            metro: metro.to_string(),
            battito,
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Tempo(inizio: {}, bpm: {}, metro: {}, battito: {:})",
            self.inizio, self.bpm, self.metro, self.battito
        ))
    }
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
#[derive(Clone, Debug)]
#[pyclass(name = "PositionMark", unsendable, get_all, set_all)]
pub struct PyPositionMark {
    pub name: String,
    pub mark_type: u16,
    pub start: f64,
    pub end: Option<f64>,
    pub num: i32,
}

#[pymethods]
impl PyPositionMark {
    #[new]
    fn new(name: &str, mark_type: u16, num: i32, start: f64, end: Option<f64>) -> Self {
        Self {
            name: name.to_string(),
            mark_type,
            start,
            end,
            num,
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "PositionMark(name: {}, type: {}, start: {}, end: {:?}, num: {})",
            self.name, self.mark_type, self.start, self.end, self.num
        ))
    }
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

#[derive(Debug)]
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

impl PyTrack {
    fn clone_py(&self, py: Python) -> Self {
        let mut tempos: Vec<Py<PyTempo>> = Vec::new();
        let mut position_marks: Vec<Py<PyPositionMark>> = Vec::new();
        let tempo_vec: &Vec<Py<PyTempo>> = self.tempos.as_ref();
        let position_vec: &Vec<Py<PyPositionMark>> = self.position_marks.as_ref();
        for item in tempo_vec.iter() {
            let pyitem = Py::new(py, item.borrow(py).clone()).expect("");
            tempos.push(pyitem);
        }
        for item in position_vec.iter() {
            let pyitem = Py::new(py, item.borrow(py).clone()).expect("");
            position_marks.push(pyitem);
        }

        Self {
            tempos,
            position_marks,
            trackid: self.trackid.clone(),
            location: self.location.clone(),
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
            datemodified: None,
            dateadded: None,
            bitrate: self.bitrate,
            samplerate: self.samplerate,
            comments: self.comments.clone(),
            playcount: self.playcount,
            lastplayed: None,
            rating: self.rating,
            remixer: self.remixer.clone(),
            tonality: self.tonality.clone(),
            label: self.label.clone(),
            mix: self.mix.clone(),
            color: self.color.clone(),
        }
    }
}

#[pymethods]
impl PyTrack {
    #[new]
    fn new(trackid: &str, location: &str) -> PyResult<Self> {
        let track = Self {
            trackid: trackid.to_string(),
            location: location.to_string(),
            name: None,
            artist: None,
            composer: None,
            album: None,
            grouping: None,
            genre: None,
            kind: None,
            size: None,
            totaltime: None,
            discnumber: None,
            tracknumber: None,
            year: None,
            averagebpm: None,
            datemodified: None,
            dateadded: None,
            bitrate: None,
            samplerate: None,
            comments: None,
            playcount: None,
            lastplayed: None,
            rating: None,
            remixer: None,
            tonality: None,
            label: None,
            mix: None,
            color: None,
            tempos: Vec::new(),
            position_marks: Vec::new(),
        };
        Ok(track)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Track({:10} {})", self.trackid, self.location))
    }
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

#[derive(Clone, Debug)]
#[pyclass(name = "PlaylistTrack", unsendable, get_all, set_all)]
pub struct PyPlaylistTrack {
    pub key: String,
}

#[pymethods]
impl PyPlaylistTrack {
    #[new]
    pub fn new(key: &str) -> PyResult<Self> {
        Ok(Self {
            key: key.to_string(),
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
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
#[derive(Debug)]
#[pyclass(name = "PlaylistNode", unsendable, get_all, set_all)]
pub struct PyPlaylistNode {
    pub name: String,
    pub node_type: u16,
    pub key_type: Option<u16>,
    pub tracks: Option<Vec<Py<PyPlaylistTrack>>>,
    pub nodes: Option<Vec<Py<PyPlaylistNode>>>,
    pub node_path: Vec<String>,
}

impl PyPlaylistNode {
    fn clone_py(&self, py: Python) -> Self {
        let mut tracks: Option<Vec<Py<PyPlaylistTrack>>> = None;
        if self.tracks.is_some() {
            let mut items = Vec::new();
            for track in self.tracks.as_ref().unwrap().iter() {
                let item = Py::new(py, track.borrow(py).clone()).expect("");
                items.push(item);
            }
            tracks = Some(items);
        }

        let mut nodes: Option<Vec<Py<PyPlaylistNode>>> = None;
        if self.nodes.is_some() {
            let mut items = Vec::new();
            for node in self.nodes.as_ref().unwrap().iter() {
                let item = Py::new(py, node.borrow(py).clone_py(py)).expect("");
                items.push(item);
            }
            nodes = Some(items);
        }
        Self {
            name: self.name.clone(),
            node_type: self.node_type.clone(),
            key_type: self.key_type.clone(),
            tracks: tracks,
            nodes: nodes,
            node_path: self.node_path.clone(),
        }
    }
}

#[pymethods]
impl PyPlaylistNode {
    #[new]
    pub fn new(name: &str, node_type: u16) -> PyResult<Self> {
        Ok(Self {
            name: name.to_string(),
            node_type,
            key_type: None,
            tracks: Some(Vec::new()),
            nodes: Some(Vec::new()),
            node_path: Vec::new(),
        })
    }

    #[classmethod]
    fn playlist(_cls: &Bound<'_, PyType>, name: &str, key_type: u16) -> PyResult<Self> {
        Ok(Self {
            name: name.to_string(),
            node_type: PlaylistNodeType::Playlist.into(),
            key_type: Some(key_type),
            tracks: Some(Vec::new()),
            nodes: Some(Vec::new()),
            node_path: Vec::new(),
        })
    }

    #[classmethod]
    fn folder(_cls: &Bound<'_, PyType>, name: &str) -> PyResult<Self> {
        Ok(Self {
            name: name.to_string(),
            node_type: PlaylistNodeType::Folder.into(),
            key_type: None,
            tracks: Some(Vec::new()),
            nodes: Some(Vec::new()),
            node_path: Vec::new(),
        })
    }

    pub fn add_node(&mut self, py: Python, node: PyRef<'_, PyPlaylistNode>) -> PyResult<()> {
        let mut node = node.clone_py(py);
        node.node_path.extend_from_slice(self.node_path.as_slice());
        node.node_path.push(self.name.clone());
        let pynode = Py::new(py, node)?;
        self.nodes.as_mut().unwrap().push(pynode);
        Ok(())
    }

    pub fn new_playlist(
        &mut self,
        py: Python,
        name: &str,
        key_type: u16,
    ) -> PyResult<&Py<PyPlaylistNode>> {
        if self.node_type != 0 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot add child node to playlist node"
            )));
        }
        let node = Self {
            name: name.to_string(),
            node_type: PlaylistNodeType::Playlist.into(),
            key_type: Some(key_type),
            tracks: Some(Vec::new()),
            nodes: Some(Vec::new()),
            node_path: Vec::new(),
        };
        let index = self.nodes.as_ref().unwrap().len();

        let mut node = node.clone_py(py);
        node.node_path.extend_from_slice(self.node_path.as_slice());
        node.node_path.push(self.name.clone());
        let pynode = Py::new(py, node)?;
        self.nodes.as_mut().unwrap().push(pynode);

        self.get_child(index)
    }

    pub fn new_folder(&mut self, py: Python, name: &str) -> PyResult<&Py<PyPlaylistNode>> {
        if self.node_type != 0 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot add child node to playlist node"
            )));
        }
        let node = Self {
            name: name.to_string(),
            node_type: PlaylistNodeType::Folder.into(),
            key_type: None,
            tracks: Some(Vec::new()),
            nodes: Some(Vec::new()),
            node_path: Vec::new(),
        };
        let index = self.nodes.as_ref().unwrap().len();

        let mut node = node.clone_py(py);
        node.node_path.extend_from_slice(self.node_path.as_slice());
        node.node_path.push(self.name.clone());
        let pynode = Py::new(py, node)?;
        self.nodes.as_mut().unwrap().push(pynode);

        self.get_child(index)
    }

    pub fn remove_node(&mut self, index: usize) -> PyResult<()> {
        if self.node_type == 1 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot add child node to playlist node"
            )));
        };
        self.nodes.as_mut().unwrap().remove(index);
        Ok(())
    }
    pub fn clear_nodes(&mut self) -> PyResult<()> {
        if self.node_type == 1 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot clear children of playlist node"
            )));
        };
        self.nodes.as_mut().unwrap().clear();
        Ok(())
    }

    pub fn get_child(&mut self, index: usize) -> PyResult<&Py<PyPlaylistNode>> {
        if self.node_type != 0 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot get child of playlist node"
            )));
        }
        let item = self.nodes.as_ref().unwrap().get(index).unwrap();
        Ok(item)
    }

    pub fn find_child(&mut self, py: Python, name: &str) -> PyResult<Option<&Py<PyPlaylistNode>>> {
        if self.node_type != 0 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot get child of playlist node"
            )));
        }
        let nodes = self.nodes.as_mut().unwrap();
        let item = nodes.iter().find(|n| n.borrow(py).name == name);
        Ok(item)
    }

    pub fn get_track(&mut self, index: usize) -> PyResult<&Py<PyPlaylistTrack>> {
        if self.node_type != 0 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot get track of folder node"
            )));
        }
        let item = self.tracks.as_ref().unwrap().get(index).unwrap();
        Ok(item)
    }

    pub fn new_track(&mut self, py: Python, key: &str) -> PyResult<()> {
        if self.node_type != 1 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot add track to folder node"
            )));
        }
        let track_item = PyPlaylistTrack::new(key)?;
        let pyitem = Py::new(py, track_item)?;
        self.tracks.as_mut().unwrap().push(pyitem);
        Ok(())
    }

    pub fn add_track(&mut self, py: Python, track: PyRef<'_, PyPlaylistTrack>) -> PyResult<()> {
        if self.node_type != 1 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot add track to folder node"
            )));
        }
        let track_item = track.clone();
        let pyitem = Py::new(py, track_item)?;
        self.tracks.as_mut().unwrap().push(pyitem);
        Ok(())
    }

    pub fn remove_track(&mut self, py: Python, key: &str) -> PyResult<()> {
        if self.node_type != 0 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot remove track from playlist node"
            )));
        }
        let mut index: Option<usize> = None;
        for (i, track) in self.tracks.as_mut().unwrap().iter_mut().enumerate() {
            if track.borrow(py).key == key {
                index = Some(i);
            }
        }
        if index.is_none() {
            return Err(PyErr::new::<XmlError, _>(format!("Track not found")));
        }
        self.tracks.as_mut().unwrap().remove(index.unwrap());
        Ok(())
    }

    pub fn clear_tracks(&mut self) -> PyResult<()> {
        if self.node_type != 0 {
            return Err(PyErr::new::<XmlError, _>(format!(
                "Cannot clear tracks from playlist node"
            )));
        }
        self.tracks.as_mut().unwrap().clear();
        Ok(())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
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
    // #[pyo3(get, set)]
    // tracks: Vec<Py<PyTrack>>,
    #[pyo3(get, set)]
    pub root_playlist: Py<PyPlaylistNode>,
}

impl PyRekordboxXml {
    fn update_xml(&mut self, py: Python) {
        let root: PlaylistNode = PlaylistNode::from_py(py, self.root_playlist.borrow(py).into());
        self.xml.set_playlist_root(root);
        // let tracks: Vec<Track> = self.tracks.iter().map(|t| Track::from_py(py, t.borrow(py))).collect();
        // self.xml.set_tracks(tracks);
    }
}

#[pymethods]
impl PyRekordboxXml {
    #[new]
    pub fn new(py: Python, path: &str) -> PyResult<Self> {
        let mut xml = RekordboxXml::load(path);
        let root_node = xml.get_playlist_root();
        let node_ref = root_node.clone().into_py(py)?;
        // let tracks = xml.get_tracks();
        // let track_ref: Vec<Py<PyTrack>> = tracks.iter().map(|t| Py::new(py, t.clone().into_py(py).expect("")).expect("")).collect();
        Ok(Self {
            xml,
            root_playlist: Py::new(py, node_ref)?,
        })
    }

    #[classmethod]
    pub fn load(_cls: &Bound<'_, PyType>, py: Python, path: &str) -> PyResult<Self> {
        Ok(PyRekordboxXml::new(py, path)?)
    }

    pub fn to_string(&mut self, py: Python) -> PyResult<String> {
        self.update_xml(py);
        let s = self.xml.to_string()?;
        Ok(s)
    }

    pub fn dump_copy(&mut self, py: Python, path: &str) -> PyResult<()> {
        self.update_xml(py);
        self.xml
            .dump_copy(path)
            .map_err(|e| PyErr::new::<XmlError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn dump(&mut self, py: Python) -> PyResult<()> {
        self.update_xml(py);
        self.xml
            .dump()
            .map_err(|e| PyErr::new::<XmlError, _>(e.to_string()))?;
        Ok(())
    }

    // -- Track handling --------------------------------------------------------------------------

    pub fn get_tracks(&mut self, py: Python) -> PyResult<Vec<Py<PyTrack>>> {
        let mut data = Vec::new();
        for item in self.xml.get_tracks() {
            let py_item = item.clone().into_py(py)?;
            data.push(Py::new(py, py_item)?);
        }
        Ok(data)
    }

    pub fn get_track(&mut self, py: Python, index: usize) -> PyResult<Option<PyTrack>> {
        let data = self.xml.get_track(index);
        if let Some(data) = data {
            Ok(Some(data.clone().into_py(py)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_track_by_id(&mut self, py: Python, track_id: &str) -> PyResult<Option<PyTrack>> {
        //let item = self
        //    .tracks
        //    .iter_mut()
        //    .find(|t| &t.borrow(py).trackid == track_id);
        //Ok(item)

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

    pub fn new_track(&mut self, py: Python, trackid: &str, location: &str) -> PyResult<PyTrack> {
        let track = self.xml.new_track(trackid, location);
        let pyitem = track.clone().into_py(py)?;
        Ok(pyitem)
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

    pub fn clear_tracks(&mut self) -> PyResult<()> {
        self.xml.clear_tracks();
        Ok(())
    }

    // -- Playlist handling -----------------------------------------------------------------------

    pub fn get_playlist_root(&mut self, py: Python) -> PyResult<PyPlaylistNode> {
        let data = self.xml.get_playlist_root();
        data.clone().into_py(py)
    }
}
