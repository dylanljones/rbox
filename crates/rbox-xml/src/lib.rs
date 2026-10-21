// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Rekordbox XML file handler.
//!
//! The Rekordbox XML database is used for importing (and exporting) Rekordbox collections
//! including track metadata and playlists. They can also be used to share playlists between
//! two databases.
//!
//! This module provides functionality to read, modify, and write Rekordbox XML files.
//! It implements data structures for representing the XML elements of Rekordbox XML files
//! like tracks, playlists, cue points and beat grids. The main entry point is the
//! `RekordboxXml` struct which provides methods for loading, modifying, and saving XML files.
//!
//! # Example
//!
//! ```no_run
//! use rbox_xml::RekordboxXml;
//!
//! // Load an existing XML file
//! let mut xml = RekordboxXml::load("library.xml")?;
//!
//! // Access tracks
//! let tracks = xml.get_tracks();
//!
//! // Find a track
//! if let Some(track) = xml.get_track_by_id("1234") {
//!     println!("Found track: {}", track);
//! }
//!
//! // Save changes
//! xml.dump()?;
//! # Ok::<(), rbox_xml::XmlError>(())
//! ```

use std::fmt::{Debug, Display};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;

use chrono::NaiveDate;
use lazy_regex::regex_replace_all;
use quick_xml::events::{BytesDecl, Event};
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};

pub type Result<T> = std::result::Result<T, XmlError>;

/// A playlist operation rejected because it does not apply to the node type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlPlaylistOperation {
    AddChild,
    RemoveChild,
    ClearChildren,
    GetChild,
    GetTrack,
    AddTrack,
    RemoveTrack,
    ClearTracks,
}

impl Display for XmlPlaylistOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let operation = match self {
            Self::AddChild => "add a child node",
            Self::RemoveChild => "remove a child node",
            Self::ClearChildren => "clear child nodes",
            Self::GetChild => "get a child node",
            Self::GetTrack => "get a track",
            Self::AddTrack => "add a track",
            Self::RemoveTrack => "remove a track",
            Self::ClearTracks => "clear tracks",
        };
        f.write_str(operation)
    }
}

/// A Rekordbox XML entity that could not be found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlEntity {
    Track,
    ParentPlaylist,
}

impl Display for XmlEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Track => "track",
            Self::ParentPlaylist => "parent playlist",
        })
    }
}

/// Errors returned while reading, writing, or modifying a Rekordbox XML document.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum XmlError {
    /// An underlying filesystem operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The input could not be deserialized as a Rekordbox XML document.
    #[error("failed to deserialize XML: {0}")]
    Deserialize(#[from] quick_xml::de::DeError),

    /// The document could not be serialized as XML.
    #[error("failed to serialize XML: {0}")]
    Serialize(#[from] quick_xml::se::SeError),

    /// Serialized XML contained invalid UTF-8.
    #[error("serialized XML is not valid UTF-8: {0}")]
    Utf8(#[from] FromUtf8Error),

    /// An operation was attempted on an incompatible playlist node type.
    #[error("cannot {operation} on a {node_type} node")]
    InvalidOperation {
        operation: XmlPlaylistOperation,
        node_type: XmlPlaylistNodeType,
    },

    /// The in-memory document is missing data required by the XML format.
    #[error("invalid XML document: {0}")]
    InvalidDocument(&'static str),

    #[error("child node index {index} is out of bounds for {len} children")]
    ChildIndexOutOfBounds { index: usize, len: usize },

    /// A requested track or playlist does not exist.
    #[error("{entity} not found: {identifier}")]
    NotFound {
        entity: XmlEntity,
        identifier: String,
    },

    /// A numeric XML enum value is outside its supported range.
    #[error("invalid {field}: {value}")]
    InvalidValue { field: &'static str, value: i32 },
}

fn octet_to_hex(arg: u8) -> String {
    let r = format!("{:x}", arg);
    ((if r.len() == 1 { "0" } else { "" }).to_owned() + &r)
        .to_uppercase()
        .to_owned()
}

/// Convert a file name as std::path::Path into an URL in the file scheme.
///
/// The output is prefixed with `file://localhost/`.
/// This returns Err if the given path is not absolute or, on Windows,
/// if the prefix is not a disk prefix (e.g. C:) or a UNC prefix (\\).
fn encode_uri(s: impl AsRef<str>) -> String {
    let mut s = s.as_ref().to_string();
    s.insert_str(0, "file://localhost/");
    regex_replace_all!(r"[^A-Za-z0-9_\-\.:/\\]", s.as_ref(), |seq: &str| {
        let mut r = String::new();
        for ch in seq.to_owned().bytes() {
            r.push('%');
            r.push_str(octet_to_hex(ch).as_ref());
        }
        r.clone()
    })
    .into_owned()
}

/// Assuming the URL is in the file scheme or similar, convert its path to a String.
///
/// Note: This does not actually check the URL’s scheme, and may give nonsensical
/// results for other schemes.
fn decode_uri(s: impl AsRef<str>) -> String {
    let p = regex_replace_all!(r"(%[A-Fa-f0-9]{2})+", s.as_ref(), |seq: &str, _| {
        let mut r = Vec::<u8>::new();
        let inp: Vec<u8> = seq.to_owned().bytes().collect();
        let mut i: usize = 0;
        while i != inp.len() {
            r.push(
                u8::from_str_radix(
                    String::from_utf8_lossy(&[inp[i + 1], inp[i + 2]]).as_ref(),
                    16,
                )
                .unwrap_or(0),
            );
            i += 3;
        }
        String::from_utf8_lossy(r.as_ref()).into_owned().to_owned()
    })
    .into_owned();

    p.strip_prefix("file://localhost/")
        .unwrap_or(s.as_ref())
        .to_string()
}

fn serialize_location<S>(value: &String, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value_encoded = encode_uri(value);
    value_encoded.serialize(serializer)
}

fn deserialize_location<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <String>::deserialize(deserializer)?;
    let val = decode_uri(&s);
    Ok(val)
}

/// The product XML element of the Rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct XmlProduct {
    /// Name of product
    ///
    /// This name will be displayed in each application software.
    #[serde(rename = "@Name")]
    pub name: String,
    /// Version of application
    #[serde(rename = "@Version")]
    pub version: String,
    /// Name of company
    #[serde(rename = "@Company")]
    pub company: String,
}

// -- XmlCollection ----------------------------------------------------------------------------------

/// XmlTempo element representing the beat grid of a track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XmlTempo {
    /// Start position of BeatGrid
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@Inizio")]
    pub inizio: f64,
    /// Value of BPM
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@Bpm")]
    pub bpm: f64,
    /// Kind of musical meter (formatted)
    /// ex. 3/ 4, 4/ 4, 7/ 8…
    #[serde(rename = "@Metro")]
    pub metro: String,
    /// Beat number in the bar
    /// If the value of "Metro" is 4/ 4, the value should be 1, 2, 3 or 4.
    #[serde(rename = "@Battito")]
    pub battito: u16,
}

impl XmlTempo {
    pub fn new(inizio: f64, bpm: f64, metro: &str, battito: u16) -> Self {
        Self {
            inizio,
            bpm,
            metro: metro.to_string(),
            battito,
        }
    }
}

/// Position element for storing position markers like cue points of a track
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct XmlPositionMark {
    /// Name of position mark
    #[serde(rename = "@Name")]
    pub name: String,
    /// Type of position mark
    /// Cue = "@0", Fade- In = "1", Fade- Out = "2", Load = "3",  Loop = " 4"
    #[serde(rename = "@Type")]
    pub mark_type: u16,
    /// Start position of position mark
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@Start")]
    pub start: f64,
    /// End position of position mark
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@End")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,
    /// Number for identification of the position mark
    /// rekordbox : Hot Cue A,  B,  C : "0", "1", "2"; Memory Cue : "- 1"
    #[serde(rename = "@Num")]
    pub num: i32,
}

impl XmlPositionMark {
    pub fn new(name: &str, mark_type: u16, num: i32, start: f64, end: Option<f64>) -> Self {
        Self {
            name: name.to_string(),
            mark_type,
            start,
            end,
            num,
        }
    }
}

/// The track XML element of the Rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct XmlTrack {
    /// Identification of track
    #[serde(rename = "@TrackID")]
    pub trackid: String,
    /// Name of track
    #[serde(rename = "@Name")]
    pub name: Option<String>,
    /// Name of artist
    #[serde(rename = "@Artist")]
    pub artist: Option<String>,
    /// Name of composer (or producer)
    #[serde(rename = "@Composer")]
    pub composer: Option<String>,
    /// Name of Album
    #[serde(rename = "@Album")]
    pub album: Option<String>,
    /// Name of goupe
    #[serde(rename = "@Grouping")]
    pub grouping: Option<String>,
    /// Name of genre
    #[serde(rename = "@Genre")]
    pub genre: Option<String>,
    /// Type of audio file
    #[serde(rename = "@Kind")]
    pub kind: Option<String>,
    /// Size of audio file
    /// Unit : Octet
    #[serde(rename = "@Size")]
    pub size: Option<i32>,
    /// Duration of track
    /// Unit : Second (without decimal numbers)
    #[serde(rename = "@TotalTime")]
    pub totaltime: Option<i32>,
    /// Order number of the disc of the album
    #[serde(rename = "@DiscNumber")]
    pub discnumber: Option<i32>,
    /// Order number of the track in the album
    #[serde(rename = "@TrackNumber")]
    pub tracknumber: Option<i32>,
    /// Year of release
    #[serde(rename = "@Year")]
    pub year: Option<i32>,
    /// Value of average BPM
    /// Unit : Second (with decimal numbers)
    #[serde(rename = "@AverageBpm")]
    pub averagebpm: Option<f64>,
    /// Date of last modification
    /// Format : yyyy- mm- dd ; ex. : 2010- 08- 21
    #[serde(rename = "@DateModified")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datemodified: Option<NaiveDate>,
    /// Date of addition
    /// Format : yyyy- mm- dd ; ex. : 2010- 08- 21
    #[serde(rename = "@DateAdded")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dateadded: Option<NaiveDate>,
    /// Encoding bit rate
    /// Unit : Kbps
    #[serde(rename = "@BitRate")]
    pub bitrate: Option<i32>,
    /// Frequency of sampling
    /// Unit : Hertz
    #[serde(rename = "@SampleRate")]
    pub samplerate: Option<f64>,
    /// Comments
    #[serde(rename = "@Comments")]
    pub comments: Option<String>,
    /// Play count of the track
    #[serde(rename = "@PlayCount")]
    pub playcount: Option<i32>,
    /// Date of last playing
    /// Format : yyyy- mm- dd ; ex. : 2010- 08- 21
    #[serde(rename = "@LastPlayed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastplayed: Option<NaiveDate>,
    /// Rating of the track
    /// 0 star = "@0", 1 star = "51", 2 stars = "102", 3 stars = "153", 4 stars = "204", 5 stars = "255"
    #[serde(rename = "@Rating")]
    pub rating: Option<i32>,
    /// Location of the file
    /// includes the file name (URI formatted)
    #[serde(rename = "@Location")]
    #[serde(serialize_with = "serialize_location")]
    #[serde(deserialize_with = "deserialize_location")]
    pub location: String,
    /// Name of remixer
    #[serde(rename = "@Remixer")]
    pub remixer: Option<String>,
    /// Tonality (Kind of musical key)
    #[serde(rename = "@Tonality")]
    pub tonality: Option<String>,
    /// Name of record label
    #[serde(rename = "@Label")]
    pub label: Option<String>,
    /// Name of mix
    #[serde(rename = "@Mix")]
    pub mix: Option<String>,
    /// Colour for track grouping
    /// RGB format (3 bytes) ; rekordbox : Rose(0xFF007F), Red(0xFF0000), Orange(0xFFA500), Lemon(0xFFFF00), Green(0x00FF00), Turquoise(0x25FDE9),  Blue(0x0000FF), Violet(0x660099)
    #[serde(rename = "@Colour")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "TEMPO")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tempos: Vec<XmlTempo>,
    #[serde(rename = "POSITION_MARK")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub position_marks: Vec<XmlPositionMark>,
}

impl XmlTrack {
    pub fn new(trackid: &str, location: &str) -> Self {
        Self {
            trackid: trackid.to_string(),
            location: location.to_string(),
            ..Default::default()
        }
    }
}

impl Default for XmlTrack {
    fn default() -> Self {
        Self {
            trackid: String::new(),
            location: String::new(),
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
        }
    }
}

impl Display for XmlTrack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TrackID: {}, Name: {}, Location: {}",
            self.trackid,
            self.name.as_deref().unwrap_or(""),
            self.location
        )
    }
}

/// The collection XML element containing the tracks of the Rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct XmlCollection {
    /// XmlTrack entries in COLLECTION
    #[serde(rename = "TRACK")]
    pub tracks: Vec<XmlTrack>,
}

impl Serialize for XmlCollection {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Value<'a> {
            /// Number of TRACK in COLLECTION
            #[serde(rename = "@Entries")]
            entries: usize,
            /// XmlTrack entries in COLLECTION
            #[serde(rename = "TRACK")]
            tracks: &'a Vec<XmlTrack>,
        }

        let tracks: &Vec<XmlTrack> = self.tracks.as_ref();
        let value = Value {
            entries: tracks.len(),
            tracks,
        };
        value.serialize(serializer)
    }
}

// -- XmlPlaylists ------------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct XmlPlaylistTrack {
    /// Identification of track, "Track ID" or "Location" in "COLLECTION"
    #[serde(rename = "@Key")]
    pub key: String,
}

impl XmlPlaylistTrack {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }

    pub fn from_track(track: &XmlTrack, key_type: &XmlPlaylistKeyType) -> Self {
        let key = match key_type {
            XmlPlaylistKeyType::TrackID => track.trackid.clone(),
            XmlPlaylistKeyType::Location => track.location.clone(),
        };
        Self { key }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum XmlPlaylistNodeType {
    Folder,   // Type="0"
    Playlist, // Type="1"
}

impl Display for XmlPlaylistNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Folder => "folder",
            Self::Playlist => "playlist",
        })
    }
}

impl Into<u16> for XmlPlaylistNodeType {
    fn into(self) -> u16 {
        match self {
            XmlPlaylistNodeType::Folder => 0,
            XmlPlaylistNodeType::Playlist => 1,
        }
    }
}

impl TryFrom<u16> for XmlPlaylistNodeType {
    type Error = XmlError;

    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(XmlPlaylistNodeType::Folder),
            1 => Ok(XmlPlaylistNodeType::Playlist),
            _ => Err(XmlError::InvalidValue {
                field: "playlist node type",
                value: value.into(),
            }),
        }
    }
}

impl<'de> Deserialize<'de> for XmlPlaylistNodeType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "1" => Ok(XmlPlaylistNodeType::Playlist),
            "0" => Ok(XmlPlaylistNodeType::Folder),
            _ => Err(D::Error::custom(format!("Invalid node type: {}", s))),
        }
    }
}

impl Serialize for XmlPlaylistNodeType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            XmlPlaylistNodeType::Playlist => "1",
            XmlPlaylistNodeType::Folder => "0",
        };
        serializer.serialize_str(s)
    }
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone)]
pub enum XmlPlaylistKeyType {
    TrackID = 0,  // KeyType="0"
    Location = 1, // KeyType="1"
}

impl Into<u16> for XmlPlaylistKeyType {
    fn into(self) -> u16 {
        match self {
            XmlPlaylistKeyType::TrackID => 0,
            XmlPlaylistKeyType::Location => 1,
        }
    }
}

impl TryFrom<u16> for XmlPlaylistKeyType {
    type Error = XmlError;

    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(XmlPlaylistKeyType::TrackID),
            1 => Ok(XmlPlaylistKeyType::Location),
            _ => Err(XmlError::InvalidValue {
                field: "playlist key type",
                value: value.into(),
            }),
        }
    }
}

impl<'de> Deserialize<'de> for XmlPlaylistKeyType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "0" => Ok(XmlPlaylistKeyType::TrackID),
            "1" => Ok(XmlPlaylistKeyType::Location),
            _ => Err(D::Error::custom(format!("Invalid node type: {}", s))),
        }
    }
}

impl Serialize for XmlPlaylistKeyType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            XmlPlaylistKeyType::TrackID => "0",
            XmlPlaylistKeyType::Location => "1",
        };
        serializer.serialize_str(s)
    }
}

impl TryFrom<i32> for XmlPlaylistKeyType {
    type Error = XmlError;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(XmlPlaylistKeyType::TrackID),
            1 => Ok(XmlPlaylistKeyType::Location),
            _ => Err(XmlError::InvalidValue {
                field: "playlist key type",
                value,
            }),
        }
    }
}

#[derive(PartialEq, Clone, Deserialize)]
pub struct XmlPlaylistNode {
    /// Name of NODE
    #[serde(rename = "@Name")]
    pub name: String,
    /// Node type, "0" (Playlist) or "1"(Folder)
    #[serde(rename = "@Type")]
    pub node_type: XmlPlaylistNodeType,
    /// Kind of identification, "0" (Track ID) or "1" (Location)
    /// Only used for playlist nodes
    #[serde(rename = "@KeyType")]
    pub key_type: Option<XmlPlaylistKeyType>,
    /// Tracks contained in the playlist
    /// Only used for playlist nodes
    #[serde(rename = "TRACK")]
    pub tracks: Option<Vec<XmlPlaylistTrack>>,
    /// Child nodes
    /// Only used for folder nodes
    #[serde(rename = "NODE")]
    pub nodes: Option<Vec<XmlPlaylistNode>>,
}

impl Serialize for XmlPlaylistNode {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Folder<'a> {
            /// Name of NODE
            #[serde(rename = "@Name")]
            name: &'a String,
            /// Node type, "0" (Playlist) or "1"(Folder)
            #[serde(rename = "@Type")]
            node_type: &'a XmlPlaylistNodeType,
            /// Count
            #[serde(rename = "@Count")]
            count: usize,
            /// Nodes
            #[serde(rename = "NODE")]
            #[serde(skip_serializing_if = "Vec::is_empty")]
            nodes: &'a Vec<XmlPlaylistNode>,
        }

        #[derive(Serialize)]
        struct Playlist<'a> {
            /// Name of NODE
            #[serde(rename = "@Name")]
            name: &'a String,
            /// Node type, "0" (Playlist) or "1"(Folder)
            #[serde(rename = "@Type")]
            node_type: &'a XmlPlaylistNodeType,
            /// Number of TRACK in PLAYLIST
            #[serde(rename = "@Entries")]
            entries: usize,
            /// Kind of identification
            /// "0" (Track ID) or "1"(Location)
            #[serde(rename = "@KeyType")]
            key_type: &'a XmlPlaylistKeyType,
            #[serde(rename = "TRACK")]
            #[serde(skip_serializing_if = "Vec::is_empty")]
            tracks: &'a Vec<XmlPlaylistTrack>,
        }

        match self.node_type {
            XmlPlaylistNodeType::Playlist => {
                let key_type = self.key_type.as_ref().ok_or_else(|| {
                    <S::Error as serde::ser::Error>::custom("playlist node is missing its key type")
                })?;
                let empty = Vec::new();
                let children: &Vec<XmlPlaylistTrack> = self.tracks.as_ref().unwrap_or(&empty);
                let value = Playlist {
                    name: &self.name,
                    node_type: &XmlPlaylistNodeType::Playlist,
                    entries: children.len(),
                    key_type,
                    tracks: children,
                };
                value.serialize(serializer)
            }
            XmlPlaylistNodeType::Folder => {
                let empty = Vec::new();
                let children: &Vec<XmlPlaylistNode> = self.nodes.as_ref().unwrap_or(&empty);
                let value = Folder {
                    name: &self.name,
                    node_type: &XmlPlaylistNodeType::Folder,
                    count: children.len(),
                    nodes: children,
                };
                value.serialize(serializer)
            }
        }
    }
}

impl XmlPlaylistNode {
    pub fn new(name: &str, node_type: XmlPlaylistNodeType) -> Self {
        Self {
            name: name.to_string(),
            node_type,
            key_type: None,
            tracks: None,
            nodes: None,
        }
    }

    pub fn playlist(name: &str, key_type: XmlPlaylistKeyType) -> Self {
        let mut node = Self::new(name, XmlPlaylistNodeType::Playlist);
        node.tracks = Some(Vec::new());
        node.key_type = Some(key_type);
        node
    }

    pub fn folder(name: &str) -> Self {
        let mut node = Self::new(name, XmlPlaylistNodeType::Folder);
        node.nodes = Some(Vec::new());
        node
    }

    pub fn add_node(&mut self, node: XmlPlaylistNode) -> Result<()> {
        if self.node_type == XmlPlaylistNodeType::Playlist {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::AddChild,
                node_type: XmlPlaylistNodeType::Playlist,
            });
        }
        self.nodes.get_or_insert_default().push(node);
        Ok(())
    }

    pub fn new_playlist(
        &mut self,
        name: &str,
        key_type: XmlPlaylistKeyType,
    ) -> Result<&mut XmlPlaylistNode> {
        if self.node_type == XmlPlaylistNodeType::Playlist {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::AddChild,
                node_type: XmlPlaylistNodeType::Playlist,
            });
        };
        let idx = self.nodes.as_ref().map_or(0, Vec::len);
        let node = Self::playlist(name, key_type);
        self.add_node(node)?;

        let node_out = &mut self.nodes.get_or_insert_default()[idx];
        Ok(node_out)
    }

    pub fn new_folder(&mut self, name: &str) -> Result<&mut XmlPlaylistNode> {
        if self.node_type == XmlPlaylistNodeType::Playlist {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::AddChild,
                node_type: XmlPlaylistNodeType::Playlist,
            });
        };
        let idx = self.nodes.as_ref().map_or(0, Vec::len);
        let node = Self::folder(name);
        self.add_node(node)?;

        let node_out = &mut self.nodes.get_or_insert_default()[idx];
        Ok(node_out)
    }

    pub fn remove_node(&mut self, index: usize) -> Result<()> {
        if self.node_type == XmlPlaylistNodeType::Playlist {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::RemoveChild,
                node_type: XmlPlaylistNodeType::Playlist,
            });
        };
        let nodes = self.nodes.get_or_insert_default();
        if index >= nodes.len() {
            return Err(XmlError::ChildIndexOutOfBounds {
                index,
                len: nodes.len(),
            });
        }
        nodes.remove(index);
        Ok(())
    }

    pub fn clear_nodes(&mut self) -> Result<()> {
        if self.node_type == XmlPlaylistNodeType::Playlist {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::ClearChildren,
                node_type: XmlPlaylistNodeType::Playlist,
            });
        };
        self.nodes.get_or_insert_default().clear();
        Ok(())
    }

    pub fn get_child(&mut self, idx: usize) -> Result<Option<&mut XmlPlaylistNode>> {
        if self.node_type == XmlPlaylistNodeType::Playlist {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::GetChild,
                node_type: XmlPlaylistNodeType::Playlist,
            });
        };
        let nodes = self.nodes.get_or_insert_default();
        Ok(nodes.get_mut(idx))
    }

    pub fn find_child(&mut self, name: &str) -> Result<Option<&mut XmlPlaylistNode>> {
        if self.node_type == XmlPlaylistNodeType::Playlist {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::GetChild,
                node_type: XmlPlaylistNodeType::Playlist,
            });
        };
        let nodes = self.nodes.get_or_insert_default();
        let node = nodes.iter_mut().find(|n| n.name == name);
        Ok(node)
    }

    pub fn get_child_by_path(&mut self, path: Vec<String>) -> Result<Option<&mut XmlPlaylistNode>> {
        let mut parent = self;

        for name in path {
            let Some(child) = parent.find_child(&name)? else {
                return Ok(None);
            };
            parent = child;
        }
        Ok(Some(parent))
    }

    pub fn get_track(&mut self, idx: usize) -> Result<Option<&XmlPlaylistTrack>> {
        if self.node_type == XmlPlaylistNodeType::Folder {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::GetTrack,
                node_type: XmlPlaylistNodeType::Folder,
            });
        };
        let tracks = self.tracks.get_or_insert_default();
        Ok(tracks.get(idx))
    }

    pub fn new_track(&mut self, key: &str) -> Result<()> {
        if self.node_type == XmlPlaylistNodeType::Folder {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::AddTrack,
                node_type: XmlPlaylistNodeType::Folder,
            });
        };
        let track_item = XmlPlaylistTrack::new(key);
        self.tracks.get_or_insert_default().push(track_item);
        Ok(())
    }
    pub fn add_track_key(&mut self, key: &str) -> Result<()> {
        if self.node_type == XmlPlaylistNodeType::Folder {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::AddTrack,
                node_type: XmlPlaylistNodeType::Folder,
            });
        };
        let track_item = XmlPlaylistTrack::new(key);
        self.tracks.get_or_insert_default().push(track_item);
        Ok(())
    }
    pub fn add_track(&mut self, track: &XmlTrack) -> Result<()> {
        if self.node_type == XmlPlaylistNodeType::Folder {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::AddTrack,
                node_type: XmlPlaylistNodeType::Folder,
            });
        };
        let key_type = self.key_type.as_ref().ok_or(XmlError::InvalidDocument(
            "playlist node is missing its key type",
        ))?;
        let track_item = XmlPlaylistTrack::from_track(track, key_type);
        self.tracks.get_or_insert_default().push(track_item);
        Ok(())
    }

    pub fn remove_track(&mut self, key: &str) -> Result<()> {
        if self.node_type == XmlPlaylistNodeType::Folder {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::RemoveTrack,
                node_type: XmlPlaylistNodeType::Folder,
            });
        };
        let tracks = self.tracks.get_or_insert_default();
        let index = tracks
            .iter()
            .position(|track| track.key == key)
            .ok_or_else(|| XmlError::NotFound {
                entity: XmlEntity::Track,
                identifier: key.to_owned(),
            })?;
        tracks.remove(index);
        Ok(())
    }

    pub fn clear_tracks(&mut self) -> Result<()> {
        if self.node_type == XmlPlaylistNodeType::Folder {
            return Err(XmlError::InvalidOperation {
                operation: XmlPlaylistOperation::ClearTracks,
                node_type: XmlPlaylistNodeType::Folder,
            });
        };
        self.tracks.get_or_insert_default().clear();
        Ok(())
    }
}

impl Debug for XmlPlaylistNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.node_type {
            XmlPlaylistNodeType::Playlist => f
                .debug_struct("PlaylistNode")
                .field("name", &self.name)
                .field("key_type", &self.key_type)
                // .field("tracks", &self.tracks)
                .finish(),
            XmlPlaylistNodeType::Folder => f
                .debug_struct("PlaylistNode")
                .field("name", &self.name)
                // .field("children", &self.nodes)
                .finish(),
        }
    }
}

impl Display for XmlPlaylistNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.node_type {
            XmlPlaylistNodeType::Playlist => {
                write!(f, "Playlist: {}", self.name)
            }
            XmlPlaylistNodeType::Folder => {
                write!(f, "Folder:   {}", self.name)
            }
        }
    }
}

/// The playlist node XML element of the Rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct XmlPlaylists {
    #[serde(rename = "NODE")]
    node: XmlPlaylistNode,
}

// -- Main XmlDocument -------------------------------------------------------------------------------

/// The root element of the Rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "DJ_PLAYLISTS")]
pub struct XmlDocument {
    /// Version of the XML format for share the playlists.
    #[serde(rename = "@Version")]
    pub version: String,
    /// XmlProduct information
    #[serde(rename = "PRODUCT")]
    pub product: XmlProduct,
    #[serde(rename = "COLLECTION")]
    pub collection: XmlCollection,
    #[serde(rename = "PLAYLISTS")]
    pub playlists: XmlPlaylists,
}

impl XmlDocument {
    pub fn new() -> Self {
        let version = "1.0.0";
        let product_name = "rbox";
        let product_version = "0.1.0";
        let product_company = "rbox";
        XmlDocument {
            version: version.to_string(),
            product: XmlProduct {
                name: product_name.to_string(),
                version: product_version.to_string(),
                company: product_company.to_string(),
            },
            collection: XmlCollection { tracks: Vec::new() },
            playlists: XmlPlaylists {
                node: XmlPlaylistNode::folder("ROOT"),
            },
        }
    }
}

/// Rekordbox XML file handler.
pub struct RekordboxXml {
    /// Path to the XML file
    path: PathBuf,
    /// XML document
    doc: XmlDocument,
}

impl Debug for RekordboxXml {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RekordboxXml")
            .field("path", &self.path)
            .field("doc", &self.doc)
            .finish()
    }
}

impl RekordboxXml {
    /// Create a new Rekordbox XML file.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let p = path.as_ref().to_path_buf();
        let doc = XmlDocument::new();
        RekordboxXml { path: p, doc }
    }

    /// Read a Rekordbox XML file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p = path.as_ref().to_path_buf();
        let file = std::fs::File::open(&p)?;
        let reader = std::io::BufReader::new(file);
        let doc: XmlDocument = quick_xml::de::from_reader(reader)?;
        Ok(RekordboxXml { path: p, doc })
    }

    /// Dump the XML document to a string.
    pub fn to_string(&self) -> Result<String> {
        // Create XML declaration
        let buffer: Vec<u8> = Vec::new();
        let mut writer = quick_xml::Writer::new(buffer);
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
        let mut decl_string = String::from_utf8(writer.into_inner())?;

        // Serialize the XML document to a string
        let mut string = String::new();
        let mut ser = quick_xml::se::Serializer::new(&mut string);
        ser.indent(' ', 2);
        self.doc.serialize(ser)?;

        // Prepend the XML declaration to the serialized string
        decl_string.push('\n');
        string.insert_str(0, &decl_string);
        Ok(string)
    }

    /// Write the XML document to a file.
    pub fn dump_copy<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // Serialize the XML document to a string
        let xml_string = self.to_string()?;
        // Write the XML string to a file
        let mut file = std::fs::File::create(path)?;
        file.write_all(xml_string.as_bytes())?;
        Ok(())
    }

    /// Write the XML document to the original file.
    pub fn dump(&self) -> Result<()> {
        self.dump_copy(&self.path)?;
        Ok(())
    }

    // -- Xml file info ----------------------------------------------------------------------------

    /// Return the XML format version (DJ_PLAYLISTS Version).
    pub fn get_version(&self) -> &str {
        &self.doc.version
    }

    /// Set the XML format version (DJ_PLAYLISTS Version). Persist with `dump` or `dump_copy`.
    pub fn set_version(&mut self, value: impl Into<String>) {
        self.doc.version = value.into();
    }

    /// Return the name of the application that produced the XML.
    pub fn get_product_name(&self) -> &str {
        &self.doc.product.name
    }

    /// Set the name of the application that produced the XML. Persist with `dump` or `dump_copy`.
    pub fn set_product_name(&mut self, value: impl Into<String>) {
        self.doc.product.name = value.into();
    }

    /// Return the version of the application that produced the XML.
    pub fn get_product_version(&self) -> &str {
        &self.doc.product.version
    }

    /// Set the version of the application that produced the XML. Persist with `dump` or `dump_copy`.
    pub fn set_product_version(&mut self, value: impl Into<String>) {
        self.doc.product.version = value.into();
    }

    /// Return the company that produced the application.
    pub fn get_product_company(&self) -> &str {
        &self.doc.product.company
    }

    /// Set the company that produced the application. Persist with `dump` or `dump_copy`.
    pub fn set_product_company(&mut self, value: impl Into<String>) {
        self.doc.product.company = value.into();
    }

    // -- Track handling --------------------------------------------------------------------------

    pub fn get_tracks(&mut self) -> &mut Vec<XmlTrack> {
        &mut self.doc.collection.tracks
    }

    pub fn set_tracks(&mut self, tracks: Vec<XmlTrack>) {
        self.doc.collection.tracks = tracks;
    }

    pub fn get_track(&mut self, index: usize) -> Option<&mut XmlTrack> {
        self.doc.collection.tracks.get_mut(index)
    }

    pub fn get_track_by_id(&mut self, track_id: &str) -> Option<&mut XmlTrack> {
        let item = self
            .doc
            .collection
            .tracks
            .iter_mut()
            .find(|t| &t.trackid == track_id);
        if let Some(item) = item {
            Some(item)
        } else {
            None
        }
    }

    pub fn get_track_by_location(&mut self, location: &str) -> Option<&mut XmlTrack> {
        let item = self
            .doc
            .collection
            .tracks
            .iter_mut()
            .find(|t| t.location == location);
        if let Some(item) = item {
            Some(item)
        } else {
            None
        }
    }

    pub fn get_track_by_key(
        &mut self,
        key: &str,
        key_type: XmlPlaylistKeyType,
    ) -> Option<&mut XmlTrack> {
        match key_type {
            XmlPlaylistKeyType::TrackID => self.get_track_by_id(key),
            XmlPlaylistKeyType::Location => self.get_track_by_location(key),
        }
    }

    pub fn add_track(&mut self, track: XmlTrack) {
        self.doc.collection.tracks.push(track);
    }

    pub fn new_track(&mut self, trackid: &str, location: &str) -> &mut XmlTrack {
        let track = XmlTrack::new(trackid, location);
        let index = self.doc.collection.tracks.len();
        self.doc.collection.tracks.push(track);
        self.doc.collection.tracks.get_mut(index).unwrap()
    }

    pub fn update_track(&mut self, track: XmlTrack) -> Result<()> {
        if let Some(existing_track) = self
            .doc
            .collection
            .tracks
            .iter_mut()
            .find(|t| t.trackid == track.trackid)
        {
            *existing_track = track;
            Ok(())
        } else {
            Err(XmlError::NotFound {
                entity: XmlEntity::Track,
                identifier: track.trackid,
            })
        }
    }

    pub fn remove_track(&mut self, track_id: &str) -> Result<()> {
        // Remove the track from the collection
        let index = self
            .doc
            .collection
            .tracks
            .iter()
            .position(|t| t.trackid == track_id)
            .ok_or_else(|| XmlError::NotFound {
                entity: XmlEntity::Track,
                identifier: track_id.to_owned(),
            })?;
        self.doc.collection.tracks.remove(index);

        // Remove any references to the track in playlists
        Ok(())
    }

    pub fn clear_tracks(&mut self) {
        self.doc.collection.tracks.clear()
    }

    // -- Playlist handling -----------------------------------------------------------------------

    pub fn get_playlist_root(&mut self) -> &mut XmlPlaylistNode {
        &mut self.doc.playlists.node
    }

    pub fn set_playlist_root(&mut self, node: XmlPlaylistNode) {
        self.doc.playlists.node = node;
    }

    pub fn get_playlist_by_path(
        &mut self,
        path: Vec<String>,
    ) -> Result<Option<&mut XmlPlaylistNode>> {
        let root = &mut self.doc.playlists.node;
        root.get_child_by_path(path)
    }

    pub fn new_playlist(
        &mut self,
        name: &str,
        key_type: XmlPlaylistKeyType,
        parent_path: Vec<String>,
    ) -> Result<()> {
        let Some(parent) = self.get_playlist_by_path(parent_path.clone())? else {
            return Err(XmlError::NotFound {
                entity: XmlEntity::ParentPlaylist,
                identifier: parent_path.join("/"),
            });
        };
        parent.new_playlist(name, key_type)?;
        Ok(())
    }

    pub fn new_folder(&mut self, name: &str, parent_path: Vec<String>) -> Result<()> {
        let Some(parent) = self.get_playlist_by_path(parent_path.clone())? else {
            return Err(XmlError::NotFound {
                entity: XmlEntity::ParentPlaylist,
                identifier: parent_path.join("/"),
            });
        };
        parent.new_folder(name)?;
        Ok(())
    }

    pub fn add_track_key_to_playlist(&mut self, key: &str, parent_path: Vec<String>) -> Result<()> {
        let Some(parent) = self.get_playlist_by_path(parent_path.clone())? else {
            return Err(XmlError::NotFound {
                entity: XmlEntity::ParentPlaylist,
                identifier: parent_path.join("/"),
            });
        };
        parent.new_track(key)?;
        Ok(())
    }

    pub fn add_track_to_playlist(
        &mut self,
        track: &XmlTrack,
        parent_path: Vec<String>,
    ) -> Result<()> {
        let Some(parent) = self.get_playlist_by_path(parent_path.clone())? else {
            return Err(XmlError::NotFound {
                entity: XmlEntity::ParentPlaylist,
                identifier: parent_path.join("/"),
            });
        };
        parent.add_track(track)?;
        Ok(())
    }
}

/// Deprecated compatibility name; use [`XmlTrack`].
#[deprecated(note = "use XmlTrack instead")]
pub type Track = XmlTrack;

/// Deprecated compatibility name; use [`XmlTempo`].
#[deprecated(note = "use XmlTempo instead")]
pub type Tempo = XmlTempo;

/// Deprecated compatibility name; use [`XmlPositionMark`].
#[deprecated(note = "use XmlPositionMark instead")]
pub type PositionMark = XmlPositionMark;

/// Deprecated compatibility name; use [`XmlPlaylistTrack`].
#[deprecated(note = "use XmlPlaylistTrack instead")]
pub type PlaylistTrack = XmlPlaylistTrack;

/// Deprecated compatibility name; use [`XmlPlaylistNode`].
#[deprecated(note = "use XmlPlaylistNode instead")]
pub type PlaylistNode = XmlPlaylistNode;

/// Deprecated compatibility name; use [`XmlPlaylistNodeType`].
#[deprecated(note = "use XmlPlaylistNodeType instead")]
pub type PlaylistNodeType = XmlPlaylistNodeType;

/// Deprecated compatibility name; use [`XmlPlaylistKeyType`].
#[deprecated(note = "use XmlPlaylistKeyType instead")]
pub type PlaylistKeyType = XmlPlaylistKeyType;

/// Deprecated compatibility name; use [`XmlCollection`].
#[deprecated(note = "use XmlCollection instead")]
pub type Collection = XmlCollection;

/// Deprecated compatibility name; use [`XmlProduct`].
#[deprecated(note = "use XmlProduct instead")]
pub type Product = XmlProduct;

/// Deprecated compatibility name; use [`XmlPlaylists`].
#[deprecated(note = "use XmlPlaylists instead")]
pub type Playlists = XmlPlaylists;

/// Deprecated compatibility name; use [`XmlDocument`].
#[deprecated(note = "use XmlDocument instead")]
pub type Document = XmlDocument;

/// Deprecated compatibility name; use [`XmlPlaylistOperation`].
#[deprecated(note = "use XmlPlaylistOperation instead")]
pub type PlaylistOperation = XmlPlaylistOperation;
