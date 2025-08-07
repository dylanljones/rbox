// Author: Dylan Jones
// Date:   2025-07-06

#![allow(non_snake_case)]

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use napi::Env;
use napi::JsDate;
use napi_derive::napi;

use rbox::xml::{
    PlaylistKeyType, PlaylistNodeType, PlaylistTrack, PositionMark, RekordboxXml, Tempo, Track,
};

pub fn datetime_to_js_date(env: &Env, date: &NaiveDate) -> JsDate {
    const MIDNIGHT: NaiveTime = NaiveTime::from_hms_opt(0, 0, 0).unwrap();

    let dt = date.and_time(MIDNIGHT).and_local_timezone(Utc).unwrap();
    env.create_date(dt.timestamp_millis() as f64).unwrap()
}

pub fn datetime_opt_to_js_date(env: &Env, date: Option<NaiveDate>) -> Option<JsDate> {
    if let Some(date) = date {
        Some(datetime_to_js_date(env, &date))
    } else {
        None
    }
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
    if let Some(js_date) = js_date {
        Some(js_date_to_datetime(js_date))
    } else {
        None
    }
}

pub trait IntoJs<T> {
    fn into_js(self, env: &Env) -> T;
}

#[derive(Clone)]
#[napi(object, js_name = "Tempo")]
pub struct JsTempo {
    pub inizio: f64,
    pub bpm: f64,
    pub metro: String,
    pub battito: u16,
}

impl IntoJs<JsTempo> for Tempo {
    fn into_js(self, _env: &Env) -> JsTempo {
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

impl IntoJs<JsPositionMark> for PositionMark {
    fn into_js(self, _env: &Env) -> JsPositionMark {
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

#[napi(object, js_name = "Track")]
pub struct JsTrack {
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
    pub datemodified: Option<JsDate>,
    pub dateadded: Option<JsDate>,
    pub bitrate: Option<i32>,
    pub samplerate: Option<f64>,
    pub comments: Option<String>,
    pub playcount: Option<i32>,
    pub lastplayed: Option<JsDate>,
    pub rating: Option<i32>,
    pub location: String,
    pub remixer: Option<String>,
    pub tonality: Option<String>,
    pub label: Option<String>,
    pub mix: Option<String>,
    pub color: Option<String>,
    pub tempos: Vec<JsTempo>,
    pub position_marks: Vec<JsPositionMark>,
}

impl IntoJs<JsTrack> for Track {
    fn into_js(self, env: &Env) -> JsTrack {
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
            tempos: self
                .tempos
                .iter()
                .map(|item| item.clone().into_js(env))
                .collect(),
            position_marks: self
                .position_marks
                .iter()
                .map(|item| item.clone().into_js(env))
                .collect(),
        }
    }
}

impl From<JsTrack> for Track {
    fn from(model: JsTrack) -> Self {
        let mut tempos = Vec::new();
        let mut position_marks = Vec::new();
        for item in model.tempos {
            tempos.push(Tempo::from(item));
        }
        for item in model.position_marks {
            position_marks.push(PositionMark::from(item));
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

#[napi(object, js_name = "PlaylistTrack")]
pub struct JsPlaylistTrack {
    pub key: String,
}

impl IntoJs<JsPlaylistTrack> for PlaylistTrack {
    fn into_js(self, _env: &Env) -> JsPlaylistTrack {
        JsPlaylistTrack { key: self.key }
    }
}

impl From<JsPlaylistTrack> for PlaylistTrack {
    fn from(model: JsPlaylistTrack) -> Self {
        Self { key: model.key }
    }
}

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

#[napi(js_name = "RekordboxXml")]
pub struct JsRekordboxXml {
    xml: RekordboxXml,
    //  tracks: Vec<JsTrack>
}

#[napi]
impl JsRekordboxXml {
    #[napi(constructor)]
    pub fn new(path: String) -> Self {
        let xml = RekordboxXml::new(path);
        Self { xml }
    }

    #[napi(factory)]
    pub fn load(path: String) -> Self {
        let xml = RekordboxXml::load(path);
        Self { xml }
    }

    #[napi]
    pub fn to_string(&mut self) -> String {
        self.xml.to_string().expect("Failed to dump XML to string")
    }

    #[napi]
    pub fn dump_copy(&mut self, path: String) {
        self.xml.dump_copy(path).expect("Failed to dump XML");
    }

    #[napi]
    pub fn dump(&mut self) {
        self.xml.dump().expect("Failed to dump XML");
    }

    #[napi]
    pub fn get_tracks(&mut self, env: Env) -> Vec<JsTrack> {
        let tracks = self.xml.get_tracks();
        let items: Vec<JsTrack> = tracks
            .into_iter()
            .map(|t| t.clone().into_js(&env))
            .collect();
        items
    }

    #[napi]
    pub fn get_track(&mut self, env: Env, index: u32) -> Option<JsTrack> {
        let idx: usize = index as usize;
        let item: Option<&mut Track> = self.xml.get_track(idx);
        println!("{:?}", item);
        match item {
            Some(track) => Some(track.clone().into_js(&env)),
            None => None,
        }
    }

    #[napi]
    pub fn get_track_by_id(&mut self, env: Env, track_id: String) -> Option<JsTrack> {
        let item: Option<&mut Track> = self.xml.get_track_by_id(&track_id);
        match item {
            Some(track) => Some(track.clone().into_js(&env)),
            None => None,
        }
    }

    #[napi]
    pub fn get_track_by_location(&mut self, env: Env, location: String) -> Option<JsTrack> {
        let item: Option<&mut Track> = self.xml.get_track_by_location(&location);
        match item {
            Some(track) => Some(track.clone().into_js(&env)),
            None => None,
        }
    }

    #[napi]
    pub fn get_track_by_key(
        &mut self,
        key: String,
        env: Env,
        key_type: JsPlaylistKeyType,
    ) -> Option<JsTrack> {
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
    pub fn new_track(&mut self, env: Env, trackid: String, location: String) -> JsTrack {
        let track = self.xml.new_track(&trackid, &location);
        track.clone().into_js(&env)
    }

    #[napi]
    pub fn update_track(&mut self, track: JsTrack) {
        let track = Track::from(track);
        self.xml
            .update_track(track)
            .expect("Failed to update track");
    }

    #[napi]
    pub fn remove_track(&mut self, track_id: String) {
        self.xml
            .remove_track(&track_id)
            .expect("Failed to remove track");
    }

    #[napi]
    pub fn clear_tracks(&mut self) {
        self.xml.clear_tracks();
    }
}
