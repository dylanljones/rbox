// Author: Dylan Jones
// Date:   2025-07-06

#![allow(non_snake_case)]

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use napi::JsDate;
use napi::{Env, Error, Result, Status};
use napi_derive::napi;
use std::sync::{Arc, RwLock};

use rbox::xml::{
    PlaylistKeyType, PlaylistNode, PlaylistNodeType, PlaylistTrack, PositionMark, RekordboxXml,
    Tempo, Track,
};

pub fn datetime_to_js_date<'a>(env: &'a Env, date: &NaiveDate) -> JsDate<'a> {
    const MIDNIGHT: NaiveTime = NaiveTime::from_hms_opt(0, 0, 0).unwrap();

    let dt = date.and_time(MIDNIGHT).and_local_timezone(Utc).unwrap();
    env.create_date(dt.timestamp_millis() as f64).unwrap()
}

pub fn datetime_opt_to_js_date(env: &Env, date: Option<NaiveDate>) -> Option<JsDate> {
    date.map(|d| datetime_to_js_date(env, &d))
}

pub fn js_date_to_datetime(js_date: JsDate) -> NaiveDate {
    match js_date.value_of() {
        Ok(timestamp_ms) => {
            let timestamp_ms = timestamp_ms as i64;
            DateTime::<Utc>::from_timestamp_millis(timestamp_ms)
                .unwrap_or_else(|| Utc::now())
                .naive_local()
                .date()
        }
        Err(_) => Utc::now().naive_local().date(),
    }
}

pub fn js_date_opt_to_datetime(js_date: Option<JsDate>) -> Option<NaiveDate> {
    js_date.map(|d| js_date_to_datetime(d))
}

pub trait IntoJs<T> {
    fn into_js(self) -> T;
}

pub trait IntoJsDate {
    type Out<'a>;
    fn into_js(self, env: &Env) -> Self::Out<'_>;
}

#[derive(Clone)]
#[napi(object, js_name = "Tempo")]
pub struct JsTempo {
    pub inizio: f64,
    pub bpm: f64,
    pub metro: String,
    pub battito: u16,
}

impl Into<JsTempo> for Tempo {
    fn into(self) -> JsTempo {
        JsTempo {
            inizio: self.inizio,
            bpm: self.bpm,
            metro: self.metro,
            battito: self.battito,
        }
    }
}

impl From<JsTempo> for Tempo {
    fn from(model: JsTempo) -> Self {
        Self {
            inizio: model.inizio,
            bpm: model.bpm,
            metro: model.metro,
            battito: model.battito,
        }
    }
}

#[derive(Clone)]
#[napi(object, js_name = "PositionMark")]
pub struct JsPositionMark {
    pub name: String,
    pub mark_type: u16,
    pub start: f64,
    pub end: Option<f64>,
    pub num: i32,
}

impl Into<JsPositionMark> for PositionMark {
    fn into(self) -> JsPositionMark {
        JsPositionMark {
            name: self.name,
            mark_type: self.mark_type,
            start: self.start,
            end: self.end,
            num: self.num,
        }
    }
}

impl From<JsPositionMark> for PositionMark {
    fn from(model: JsPositionMark) -> Self {
        Self {
            name: model.name,
            mark_type: model.mark_type,
            start: model.start,
            end: model.end,
            num: model.num,
        }
    }
}

#[derive(Clone)]
#[napi(object, js_name = "Track")]
pub struct JsTrack<'a> {
    pub trackid: String,
    pub name: Option<String>,
    pub artist: Option<String>,
    pub composer: Option<String>,
    pub album: Option<String>,
    pub grouping: Option<String>,
    pub genre: Option<String>,
    pub kind: Option<String>,
    pub size: Option<i32>,
    pub totaltime: Option<i32>,
    pub discnumber: Option<i32>,
    pub tracknumber: Option<i32>,
    pub year: Option<i32>,
    pub averagebpm: Option<f64>,
    pub datemodified: Option<JsDate<'a>>,
    pub dateadded: Option<JsDate<'a>>,
    pub bitrate: Option<i32>,
    pub samplerate: Option<f64>,
    pub comments: Option<String>,
    pub playcount: Option<i32>,
    pub lastplayed: Option<JsDate<'a>>,
    pub rating: Option<i32>,
    pub location: String,
    pub remixer: Option<String>,
    pub tonality: Option<String>,
    pub label: Option<String>,
    pub mix: Option<String>,
    pub color: Option<String>,
    pub tempos: Option<Vec<JsTempo>>,
    pub position_marks: Option<Vec<JsPositionMark>>,
}

impl IntoJsDate for Track {
    type Out<'a> = JsTrack<'a>;

    fn into_js(self, env: &Env) -> Self::Out<'_> {
        JsTrack {
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
            datemodified: datetime_opt_to_js_date(env, self.datemodified),
            dateadded: datetime_opt_to_js_date(env, self.dateadded),
            bitrate: self.bitrate,
            samplerate: self.samplerate,
            comments: self.comments,
            playcount: self.playcount,
            lastplayed: datetime_opt_to_js_date(env, self.lastplayed),
            rating: self.rating,
            location: self.location,
            remixer: self.remixer,
            tonality: self.tonality,
            label: self.label,
            mix: self.mix,
            color: self.color,
            tempos: Some(self.tempos.iter().map(|item| item.clone().into()).collect()),
            position_marks: Some(
                self.position_marks
                    .iter()
                    .map(|item| item.clone().into())
                    .collect(),
            ),
        }
    }
}

impl From<JsTrack<'_>> for Track {
    fn from(model: JsTrack) -> Self {
        let mut tempos = Vec::new();
        let mut position_marks = Vec::new();
        if let Some(t) = model.tempos {
            for item in t {
                tempos.push(Tempo::from(item));
            }
        }
        if let Some(pm) = model.position_marks {
            for item in pm {
                position_marks.push(PositionMark::from(item));
            }
        }

        Self {
            trackid: model.trackid,
            name: model.name,
            artist: model.artist,
            composer: model.composer,
            album: model.album,
            grouping: model.grouping,
            genre: model.genre,
            kind: model.kind,
            size: model.size,
            totaltime: model.totaltime,
            discnumber: model.discnumber,
            tracknumber: model.tracknumber,
            year: model.year,
            averagebpm: model.averagebpm,
            datemodified: js_date_opt_to_datetime(model.datemodified),
            dateadded: js_date_opt_to_datetime(model.dateadded),
            bitrate: model.bitrate,
            samplerate: model.samplerate,
            comments: model.comments,
            playcount: model.playcount,
            lastplayed: js_date_opt_to_datetime(model.lastplayed),
            rating: model.rating,
            location: model.location,
            remixer: model.remixer,
            tonality: model.tonality,
            label: model.label,
            mix: model.mix,
            color: model.color,
            tempos,
            position_marks,
        }
    }
}

#[derive(Clone, PartialEq)]
#[napi(object, js_name = "PlaylistTrack")]
pub struct JsPlaylistTrack {
    pub key: String,
}

impl Into<JsPlaylistTrack> for PlaylistTrack {
    fn into(self) -> JsPlaylistTrack {
        JsPlaylistTrack { key: self.key }
    }
}

impl From<JsPlaylistTrack> for PlaylistTrack {
    fn from(model: JsPlaylistTrack) -> Self {
        Self { key: model.key }
    }
}

#[derive(Clone, PartialEq)]
#[napi(js_name = "PlaylistNodeType")]
pub enum JsPlaylistNodeType {
    Folder = 0,
    Playlist = 1,
}

impl Into<JsPlaylistNodeType> for PlaylistNodeType {
    fn into(self) -> JsPlaylistNodeType {
        match self {
            PlaylistNodeType::Folder => JsPlaylistNodeType::Folder,
            PlaylistNodeType::Playlist => JsPlaylistNodeType::Playlist,
        }
    }
}

impl From<JsPlaylistNodeType> for PlaylistNodeType {
    fn from(js: JsPlaylistNodeType) -> Self {
        match js {
            JsPlaylistNodeType::Folder => PlaylistNodeType::Folder,
            JsPlaylistNodeType::Playlist => PlaylistNodeType::Playlist,
        }
    }
}

#[derive(Clone)]
#[napi(js_name = "PlaylistKeyType")]
pub enum JsPlaylistKeyType {
    TrackID = 0,
    Location = 1,
}

impl Into<JsPlaylistKeyType> for PlaylistKeyType {
    fn into(self) -> JsPlaylistKeyType {
        match self {
            PlaylistKeyType::TrackID => JsPlaylistKeyType::TrackID,
            PlaylistKeyType::Location => JsPlaylistKeyType::Location,
        }
    }
}

impl From<JsPlaylistKeyType> for PlaylistKeyType {
    fn from(js: JsPlaylistKeyType) -> Self {
        match js {
            JsPlaylistKeyType::TrackID => PlaylistKeyType::TrackID,
            JsPlaylistKeyType::Location => PlaylistKeyType::Location,
        }
    }
}

#[derive(Clone)]
struct PlaylistNodeInner {
    name: String,
    node_type: JsPlaylistNodeType, // 0=Folder, 1=Playlist (map to your enums at the edges)
    key_type: Option<JsPlaylistKeyType>, // 0=TrackID, 1=Location
    nodes: Vec<Arc<RwLock<PlaylistNodeInner>>>,
    tracks: Vec<JsPlaylistTrack>, // or your real PlaylistTrack; using String keys for clarity
}

#[derive(Clone)]
#[napi(js_name = "PlaylistNode")]
pub struct JsPlaylistNode {
    inner: Arc<RwLock<PlaylistNodeInner>>,
}

impl JsPlaylistNode {
    fn from_inner(inner: Arc<RwLock<PlaylistNodeInner>>) -> Self {
        Self { inner }
    }
}

#[napi]
impl JsPlaylistNode {
    #[napi(constructor)]
    pub fn new(
        name: String,
        node_type: JsPlaylistNodeType,
        key_type: Option<JsPlaylistKeyType>,
    ) -> Self {
        if node_type == JsPlaylistNodeType::Folder && key_type.is_some() {
            panic!("Folder node cannot have a key type");
        }
        let inner = Arc::new(RwLock::new(PlaylistNodeInner {
            name,
            node_type,
            key_type,
            tracks: Vec::new(), // filled below
            nodes: Vec::new(),  // filled below
        }));
        return Self::from_inner(inner);
    }

    #[napi(factory)]
    pub fn playlist(name: String, key_type: JsPlaylistKeyType) -> Self {
        Self::new(name, JsPlaylistNodeType::Playlist, Some(key_type))
    }

    #[napi(factory)]
    pub fn folder(name: String) -> Self {
        Self::new(name, JsPlaylistNodeType::Folder, None)
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.read().unwrap().name.clone()
    }

    #[napi(setter)]
    pub fn set_name(&self, v: String) {
        self.inner.write().unwrap().name = v;
    }

    #[napi(getter)]
    pub fn node_type(&self) -> JsPlaylistNodeType {
        self.inner.read().unwrap().node_type.clone()
    }

    #[napi(getter)]
    pub fn key_type(&self) -> Option<JsPlaylistKeyType> {
        self.inner.read().unwrap().key_type.clone()
    }

    #[napi(getter)]
    pub fn get_nodes(&self) -> Vec<JsPlaylistNode> {
        let guard = self.inner.read().unwrap();
        let items: Vec<JsPlaylistNode> = guard
            .nodes
            .iter()
            .map(|n| JsPlaylistNode::from_inner(n.clone()))
            .collect();
        items
    }

    #[napi(getter)]
    pub fn get_tracks(&self) -> Vec<JsPlaylistTrack> {
        let guard = self.inner.read().unwrap();
        guard.tracks.clone()
    }

    #[napi(getter)]
    pub fn num_nodes(&self) -> u32 {
        self.inner.read().unwrap().nodes.len() as u32
    }

    #[napi(getter)]
    pub fn num_tracks(&self) -> u32 {
        self.inner.read().unwrap().tracks.len() as u32
    }

    #[napi]
    pub fn add_node(&self, node: &JsPlaylistNode) {
        self.inner.write().unwrap().nodes.push(node.inner.clone());
    }

    #[napi]
    pub fn new_playlist(&self, name: String, key_type: JsPlaylistKeyType) -> Self {
        let node = Self::playlist(name, key_type);
        self.add_node(&node);
        node
    }

    #[napi]
    pub fn new_folder(&self, name: String) -> Self {
        let node = Self::folder(name);
        self.add_node(&node);
        node
    }

    #[napi]
    pub fn remove_node(&self, index: u32) -> Result<()> {
        let mut guard = self.inner.write().unwrap();
        if (index as usize) < guard.nodes.len() {
            guard.nodes.remove(index as usize);
            Ok(())
        } else {
            Err(Error::new(Status::InvalidArg, "child index out of bounds"))
        }
    }

    #[napi]
    pub fn clear_nodes(&self) {
        self.inner.write().unwrap().nodes.clear();
    }

    #[napi]
    pub fn get_child(&self, index: u32) -> Result<JsPlaylistNode> {
        let guard = self.inner.read().unwrap();
        let child: Arc<RwLock<PlaylistNodeInner>> = guard
            .nodes
            .get(index as usize)
            .ok_or_else(|| Error::new(Status::InvalidArg, "child index out of bounds"))?
            .clone();
        Ok(JsPlaylistNode::from_inner(child))
    }

    #[napi]
    pub fn add_track(&self, track: JsPlaylistTrack) {
        self.inner.write().unwrap().tracks.push(track);
    }

    #[napi]
    pub fn new_track(&self, key: String) {
        let track = JsPlaylistTrack { key };
        self.add_track(track);
    }

    #[napi]
    pub fn remove_track(&self, index: u32) -> Result<()> {
        let mut guard = self.inner.write().unwrap();
        if (index as usize) < guard.tracks.len() {
            guard.tracks.remove(index as usize);
            Ok(())
        } else {
            Err(Error::new(Status::InvalidArg, "track index out of bounds"))
        }
    }

    #[napi]
    pub fn clear_tracks(&self) {
        self.inner.write().unwrap().tracks.clear();
    }
}

#[napi(js_name = "RekordboxXml")]
pub struct JsRekordboxXml {
    xml: RekordboxXml,
    playlist_root: JsPlaylistNode,
    //  tracks: Vec<JsTrack>
}

impl IntoJs<Arc<RwLock<PlaylistNodeInner>>> for PlaylistNode {
    fn into_js(self) -> Arc<RwLock<PlaylistNodeInner>> {
        let inner = Arc::new(RwLock::new(PlaylistNodeInner {
            name: self.name.clone(),
            node_type: self.node_type.into(),
            key_type: self.key_type.map(|kt| kt.into()),
            tracks: Vec::new(), // filled below
            nodes: Vec::new(),  // filled below
        }));

        if let Some(tracks) = &self.tracks {
            let mut js_tracks = Vec::new();
            for track in tracks {
                let js_track: JsPlaylistTrack = track.clone().into();
                js_tracks.push(js_track);
            }
            inner.write().unwrap().tracks = js_tracks;
        }

        if let Some(children) = &self.nodes {
            let mut js_nodes = Vec::new();
            for child in children {
                let js_node = child.clone().into_js();
                js_nodes.push(js_node);
            }
            inner.write().unwrap().nodes = js_nodes;
        }

        inner
    }
}

impl From<PlaylistNodeInner> for PlaylistNode {
    fn from(inner: PlaylistNodeInner) -> Self {
        let nodes: Option<Vec<PlaylistNode>> = if inner.nodes.is_empty() {
            None
        } else {
            let items = inner
                .nodes
                .iter()
                .map(|n| PlaylistNode::from(n.read().unwrap().clone()))
                .collect();
            Some(items)
        };
        let tracks: Option<Vec<PlaylistTrack>> = if inner.tracks.is_empty() {
            None
        } else {
            let items = inner
                .tracks
                .iter()
                .map(|t| PlaylistTrack::from(t.clone()))
                .collect();
            Some(items)
        };
        Self {
            name: inner.name.clone(),
            node_type: inner.node_type.clone().into(),
            key_type: inner.key_type.clone().map(|kt| kt.into()),
            nodes,
            tracks,
        }
    }
}

impl JsRekordboxXml {
    fn from_xml(mut xml: RekordboxXml) -> Self {
        let playlist_root_inner = xml.get_playlist_root().clone().into_js();
        let playlist_root = JsPlaylistNode::from_inner(playlist_root_inner);
        Self { xml, playlist_root }
    }

    fn update_xml(&mut self) {
        let guard = self.playlist_root.inner.read().unwrap();
        let root: PlaylistNode = PlaylistNode::from(guard.clone());
        self.xml.set_playlist_root(root);
    }
}

#[napi]
impl JsRekordboxXml {
    #[napi(constructor)]
    pub fn new(path: String) -> Self {
        let xml = RekordboxXml::new(path);
        Self::from_xml(xml)
    }

    #[napi(factory)]
    pub fn load(path: String) -> Self {
        let xml = RekordboxXml::load(path);
        Self::from_xml(xml)
    }

    #[napi]
    pub fn to_string(&mut self) -> Result<String> {
        self.update_xml();
        Ok(self.xml.to_string()?)
    }

    #[napi]
    pub fn dump_copy(&mut self, path: String) -> Result<()> {
        self.update_xml();
        self.xml.dump_copy(path)?;
        Ok(())
    }

    #[napi]
    pub fn dump(&mut self) -> Result<()> {
        self.update_xml();
        self.xml.dump()?;
        Ok(())
    }

    #[napi(getter)]
    pub fn get_playlist_root(&self) -> JsPlaylistNode {
        self.playlist_root.clone()
    }

    #[napi]
    pub fn get_playlist_roots(&self) -> JsPlaylistNode {
        self.playlist_root.clone()
    }

    #[napi]
    pub fn get_tracks<'a>(&mut self, env: &'a Env) -> Vec<JsTrack<'a>> {
        let tracks = self.xml.get_tracks();
        let items: Vec<JsTrack> = tracks.into_iter().map(|t| t.clone().into_js(env)).collect();
        items
    }

    #[napi]
    pub fn get_track<'a>(&mut self, env: &'a Env, index: u32) -> Option<JsTrack<'a>> {
        let idx: usize = index as usize;
        let item: Option<&mut Track> = self.xml.get_track(idx);
        item.map(|track| track.clone().into_js(env))
    }

    #[napi]
    pub fn get_track_by_id<'a>(&mut self, env: &'a Env, track_id: String) -> Option<JsTrack<'a>> {
        let item: Option<&mut Track> = self.xml.get_track_by_id(&track_id);
        item.map(|track| track.clone().into_js(env))
    }

    #[napi]
    pub fn get_track_by_location<'a>(
        &mut self,
        env: &'a Env,
        location: String,
    ) -> Option<JsTrack<'a>> {
        let item: Option<&mut Track> = self.xml.get_track_by_location(&location);
        item.map(|track| track.clone().into_js(env))
    }

    #[napi]
    pub fn get_track_by_key<'a>(
        &mut self,
        key: String,
        env: &'a Env,
        key_type: JsPlaylistKeyType,
    ) -> Option<JsTrack<'a>> {
        match key_type {
            JsPlaylistKeyType::TrackID => self.get_track_by_id(env, key),
            JsPlaylistKeyType::Location => self.get_track_by_location(env, key),
        }
    }

    #[napi]
    pub fn add_track(&mut self, track: JsTrack) {
        let track = Track::from(track);
        self.xml.add_track(track);
    }

    #[napi]
    pub fn new_track<'a>(
        &mut self,
        env: &'a Env,
        trackid: String,
        location: String,
    ) -> JsTrack<'a> {
        let track = self.xml.new_track(&trackid, &location);
        track.clone().into_js(env)
    }

    #[napi]
    pub fn update_track(&mut self, track: JsTrack) -> Result<()> {
        let track = Track::from(track);
        self.xml.update_track(track)?;
        Ok(())
    }

    #[napi]
    pub fn remove_track(&mut self, track_id: String) -> Result<()> {
        self.xml.remove_track(&track_id)?;
        Ok(())
    }

    #[napi]
    pub fn clear_tracks(&mut self) {
        self.xml.clear_tracks();
    }
}
