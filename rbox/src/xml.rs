// Author: Dylan Jones
// Date:   2025-05-05

use chrono::NaiveDate;
use lazy_regex::regex_replace_all;
use quick_xml;
use quick_xml::events::{BytesDecl, Event};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::io::Write;
use std::path::{Path, PathBuf};

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

fn serialize_location<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value_encoded = encode_uri(value);
    value_encoded.serialize(serializer)
}

fn deserialize_location<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <String>::deserialize(deserializer)?;
    let val = decode_uri(&s);
    Ok(val)
}

/// The product XML element of the Rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Product {
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

// -- Collection ----------------------------------------------------------------------------------

/// Tempo element representing the beat grid of a track.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Tempo {
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
    pub battito: i32,
}

impl Tempo {
    pub fn new(inizio: f64, bpm: f64, metro: &str, battito: i32) -> Self {
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
pub struct PositionMark {
    /// Name of position mark
    #[serde(rename = "@Name")]
    pub name: String,
    /// Type of position mark
    /// Cue = "@0", Fade- In = "1", Fade- Out = "2", Load = "3",  Loop = " 4"
    #[serde(rename = "@Type")]
    pub mark_type: i32,
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

impl PositionMark {
    pub fn new(name: &str, mark_type: i32, start: f64, end: Option<f64>, num: i32) -> Self {
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
pub struct Track {
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
    pub size: Option<i64>,
    /// Duration of track
    /// Unit : Second (without decimal numbers)
    #[serde(rename = "@TotalTime")]
    pub totaltime: Option<f64>,
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
    pub colour: Option<String>,
    #[serde(rename = "TEMPO")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tempos: Vec<Tempo>,
    #[serde(rename = "POSITION_MARK")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub position_marks: Vec<PositionMark>,
}

impl Track {
    pub fn new(trackid: &str, location: &str) -> Self {
        Self {
            trackid: trackid.to_string(),
            location: location.to_string(),
            ..Default::default()
        }
    }
}

impl Default for Track {
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
            colour: None,
            tempos: Vec::new(),
            position_marks: Vec::new(),
        }
    }
}

impl Display for Track {
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
pub struct Collection {
    /// Track entries in COLLECTION
    #[serde(rename = "TRACK")]
    pub tracks: Vec<Track>,
}

impl Serialize for Collection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Value<'a> {
            /// Number of TRACK in COLLECTION
            #[serde(rename = "@Entries")]
            entries: usize,
            /// Track entries in COLLECTION
            #[serde(rename = "TRACK")]
            tracks: &'a Vec<Track>,
        }

        let tracks: &Vec<Track> = self.tracks.as_ref();
        let value = Value {
            entries: tracks.len(),
            tracks: tracks,
        };
        value.serialize(serializer)
    }
}

// -- Playlists ------------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    /// Identification of track, "Track ID" or "Location" in "COLLECTION"
    #[serde(rename = "@Key")]
    pub key: String,
}

impl PlaylistTrack {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }

    pub fn from_track(track: &Track, key_type: &PlaylistKeyType) -> Self {
        let key = match key_type {
            PlaylistKeyType::TrackID => track.trackid.clone(),
            PlaylistKeyType::Location => track.location.clone(),
        };
        Self { key }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlaylistNodeType {
    Playlist, // Type="1"
    Folder,   // Type="0"
}

impl<'de> Deserialize<'de> for PlaylistNodeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "1" => Ok(PlaylistNodeType::Playlist),
            "0" => Ok(PlaylistNodeType::Folder),
            _ => Err(D::Error::custom(format!("Invalid node type: {}", s))),
        }
    }
}

impl Serialize for PlaylistNodeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            PlaylistNodeType::Playlist => "1",
            PlaylistNodeType::Folder => "0",
        };
        serializer.serialize_str(s)
    }
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone)]
pub enum PlaylistKeyType {
    TrackID = 0,  // KeyType="0"
    Location = 1, // KeyType="1"
}

impl<'de> Deserialize<'de> for PlaylistKeyType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "0" => Ok(PlaylistKeyType::TrackID),
            "1" => Ok(PlaylistKeyType::Location),
            _ => Err(D::Error::custom(format!("Invalid node type: {}", s))),
        }
    }
}

impl Serialize for PlaylistKeyType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            PlaylistKeyType::TrackID => "0",
            PlaylistKeyType::Location => "1",
        };
        serializer.serialize_str(s)
    }
}

impl TryFrom<i32> for PlaylistKeyType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PlaylistKeyType::TrackID),
            1 => Ok(PlaylistKeyType::Location),
            _ => Err("Invalid value for PlaylistKeyType"),
        }
    }
}

#[derive(PartialEq, Clone, Deserialize)]
pub struct PlaylistNode {
    /// Name of NODE
    #[serde(rename = "@Name")]
    pub name: String,
    /// Node type, "0" (Playlist) or "1"(Folder)
    #[serde(rename = "@Type")]
    pub node_type: PlaylistNodeType,
    /// Kind of identification, "0" (Track ID) or "1" (Location)
    /// Only used for playlist nodes
    #[serde(rename = "@KeyType")]
    pub key_type: Option<PlaylistKeyType>,
    /// Tracks contained in the playlist
    /// Only used for playlist nodes
    #[serde(rename = "TRACK")]
    pub tracks: Option<Vec<PlaylistTrack>>,
    /// Child nodes
    /// Only used for folder nodes
    #[serde(rename = "NODE")]
    pub nodes: Option<Vec<PlaylistNode>>,
    /// Path of the playlist node from root
    /// Only used internally
    #[serde(skip)]
    pub node_path: Vec<String>,
}

impl Serialize for PlaylistNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
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
            node_type: &'a PlaylistNodeType,
            /// Count
            #[serde(rename = "@Count")]
            count: usize,
            /// Nodes
            #[serde(rename = "NODE")]
            #[serde(skip_serializing_if = "Vec::is_empty")]
            nodes: &'a Vec<PlaylistNode>,
        }

        #[derive(Serialize)]
        struct Playlist<'a> {
            /// Name of NODE
            #[serde(rename = "@Name")]
            name: &'a String,
            /// Node type, "0" (Playlist) or "1"(Folder)
            #[serde(rename = "@Type")]
            node_type: &'a PlaylistNodeType,
            /// Number of TRACK in PLAYLIST
            #[serde(rename = "@Entries")]
            entries: usize,
            /// Kind of identification
            /// "0" (Track ID) or "1"(Location)
            #[serde(rename = "@KeyType")]
            key_type: &'a PlaylistKeyType,
            #[serde(rename = "TRACK")]
            #[serde(skip_serializing_if = "Vec::is_empty")]
            tracks: &'a Vec<PlaylistTrack>,
        }

        match self.node_type {
            PlaylistNodeType::Playlist => {
                let key_type = self.key_type.as_ref().unwrap();
                let empty = Vec::new();
                let children: &Vec<PlaylistTrack> = self.tracks.as_ref().unwrap_or(&empty);
                let value = Playlist {
                    name: &self.name,
                    node_type: &PlaylistNodeType::Playlist,
                    entries: children.len(),
                    key_type: &key_type,
                    tracks: children,
                };
                value.serialize(serializer)
            }
            PlaylistNodeType::Folder => {
                let empty = Vec::new();
                let children: &Vec<PlaylistNode> = self.nodes.as_ref().unwrap_or(&empty);
                let value = Folder {
                    name: &self.name,
                    node_type: &PlaylistNodeType::Folder,
                    count: children.len(),
                    nodes: children,
                };
                value.serialize(serializer)
            }
        }
    }
}

impl PlaylistNode {
    pub fn new(name: &str, node_type: PlaylistNodeType) -> Self {
        Self {
            name: name.to_string(),
            node_type: node_type,
            key_type: None,
            tracks: None,
            nodes: None,
            node_path: Vec::new(),
        }
    }

    pub fn playlist(name: &str, key_type: PlaylistKeyType) -> Self {
        let mut node = Self::new(name, PlaylistNodeType::Playlist);
        node.tracks = Some(Vec::new());
        node.key_type = Some(key_type);
        node
    }

    pub fn folder(name: &str) -> Self {
        let mut node = Self::new(name, PlaylistNodeType::Folder);
        node.nodes = Some(Vec::new());
        node
    }

    pub fn add_node(&mut self, mut node: PlaylistNode) {
        node.node_path.extend_from_slice(self.node_path.as_slice());
        node.node_path.push(self.name.clone());
        self.nodes.as_mut().unwrap().push(node);
    }

    pub fn add_playlist(&mut self, name: &str, key_type: PlaylistKeyType) -> anyhow::Result<()> {
        if self.node_type == PlaylistNodeType::Playlist {
            return Err(anyhow::anyhow!("Cannot add child node to playlist node"));
        };
        let node = Self::playlist(name, key_type);
        self.add_node(node);
        Ok(())
    }

    pub fn add_folder(&mut self, name: &str) -> anyhow::Result<()> {
        if self.node_type == PlaylistNodeType::Playlist {
            return Err(anyhow::anyhow!("Cannot add child node to playlist node"));
        };
        let node = Self::folder(name);
        self.add_node(node);
        Ok(())
    }

    pub fn add_track_key(&mut self, key: &str) -> anyhow::Result<()> {
        if self.node_type == PlaylistNodeType::Folder {
            return Err(anyhow::anyhow!("Cannot add track to folder node"));
        };
        let track_item = PlaylistTrack::new(key);
        self.tracks.as_mut().unwrap().push(track_item);
        Ok(())
    }

    pub fn add_track(&mut self, track: &Track) -> anyhow::Result<()> {
        if self.node_type == PlaylistNodeType::Folder {
            return Err(anyhow::anyhow!("Cannot get track of folder node"));
        };
        let key_type = self.key_type.as_ref().unwrap();
        let track_item = PlaylistTrack::from_track(track, key_type);
        self.tracks.as_mut().unwrap().push(track_item);
        Ok(())
    }

    pub fn get_child(&mut self, idx: usize) -> anyhow::Result<Option<&mut PlaylistNode>> {
        if self.node_type == PlaylistNodeType::Playlist {
            return Err(anyhow::anyhow!("Cannot get child of playlist node"));
        };
        let nodes = self.nodes.as_mut().unwrap();
        Ok(nodes.get_mut(idx))
    }

    pub fn find_child(&mut self, name: &str) -> anyhow::Result<Option<&mut PlaylistNode>> {
        if self.node_type == PlaylistNodeType::Playlist {
            return Err(anyhow::anyhow!("Cannot get child of playlist node"));
        };
        let nodes = self.nodes.as_mut().unwrap();
        let node = nodes.iter_mut().find(|n| n.name == name);
        Ok(node)
    }

    pub fn get_child_by_path(
        &mut self,
        path: Vec<String>,
    ) -> anyhow::Result<Option<&mut PlaylistNode>> {
        let mut parent = self;

        for name in path {
            let child = parent.find_child(&name)?;
            if child.is_none() {
                return Ok(None);
            }
            parent = child.unwrap();
        }
        Ok(Some(parent))
    }

    pub fn get_track(&mut self, idx: usize) -> anyhow::Result<Option<&PlaylistTrack>> {
        if self.node_type == PlaylistNodeType::Folder {
            return Err(anyhow::anyhow!("Cannot get track of folder node"));
        };
        let tracks = self.tracks.as_mut().unwrap();
        Ok(tracks.get(idx))
    }

    fn update_path(&mut self, parent_path: Vec<String>) {
        // Set this node's path
        let mut path = parent_path;
        path.push(self.name.clone());
        self.node_path = path;

        // Recursively update paths for all children
        if let Some(ref mut children) = self.nodes {
            for child in children.iter_mut() {
                child.update_path(self.node_path.clone());
            }
        }
    }

    fn set_path(&mut self, path: Vec<String>) {
        self.node_path = path;
        // Recursively update paths for all children
        if let Some(ref mut children) = self.nodes {
            for child in children.iter_mut() {
                child.update_path(self.node_path.clone());
            }
        }
    }
}

impl Debug for PlaylistNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.node_type {
            PlaylistNodeType::Playlist => f
                .debug_struct("PlaylistNode")
                .field("name", &self.name)
                .field("key_type", &self.key_type.clone().unwrap())
                .field("node_path", &self.node_path)
                // .field("tracks", &self.tracks)
                .finish(),
            PlaylistNodeType::Folder => f
                .debug_struct("PlaylistNode")
                .field("name", &self.name)
                .field("node_path", &self.node_path)
                // .field("children", &self.nodes)
                .finish(),
        }
    }
}

impl Display for PlaylistNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.node_type {
            PlaylistNodeType::Playlist => {
                write!(f, "Playlist: {}", self.name)
            }
            PlaylistNodeType::Folder => {
                write!(f, "Folder:   {}", self.name)
            }
        }
    }
}

/// The playlist node XML element of the Rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Playlists {
    #[serde(rename = "NODE")]
    node: PlaylistNode,
}

// -- Main Document -------------------------------------------------------------------------------

/// The root element of the Rekordbox XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "MASTER_PLAYLIST")]
pub struct Document {
    /// Version of the XML format for share the playlists.
    #[serde(rename = "@Version")]
    pub version: String,
    /// Product information
    #[serde(rename = "PRODUCT")]
    pub product: Product,
    #[serde(rename = "COLLECTION")]
    pub collection: Collection,
    #[serde(rename = "PLAYLISTS")]
    pub playlists: Playlists,
}

impl Document {
    pub fn new() -> Self {
        let version = "1.0.0";
        let product_name = "rekordbox-io";
        let product_version = "0.1.0";
        let product_company = "rekordbox-io";
        Document {
            version: version.to_string(),
            product: Product {
                name: product_name.to_string(),
                version: product_version.to_string(),
                company: product_company.to_string(),
            },
            collection: Collection { tracks: Vec::new() },
            playlists: Playlists {
                node: PlaylistNode::folder("ROOT"),
            },
        }
    }
}

/// Rekordbox XML file handler.
pub struct RekordboxXml {
    /// Path to the XML file
    path: PathBuf,
    /// XML document
    doc: Document,
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
    pub fn new<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> Self {
        let p = Path::new(&path).to_path_buf();
        let doc = Document::new();
        let mut xml = RekordboxXml { path: p, doc };
        xml.doc.playlists.node.set_path(Vec::new());
        xml
    }

    /// Read a Rekordbox XML file.
    pub fn load<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> Self {
        let p = Path::new(&path).to_path_buf();
        let file = std::fs::File::open(&p).expect("File not found");
        let reader = std::io::BufReader::new(file);
        let doc: Document = quick_xml::de::from_reader(reader).expect("failed to deserialize XML");
        let mut xml = RekordboxXml { path: p, doc };
        xml.doc.playlists.node.set_path(Vec::new());
        xml
    }

    /// Dump the XML document to a string.
    pub fn to_string(&self) -> anyhow::Result<String> {
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
    pub fn dump_copy<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        // Serialize the XML document to a string
        let xml_string = self.to_string()?;
        // Write the XML string to a file
        let mut file = std::fs::File::create(path).expect("Failed to create file");
        file.write_all(xml_string.as_bytes())?;
        Ok(())
    }

    /// Write the XML document to the original file.
    pub fn dump(&self) -> anyhow::Result<()> {
        self.dump_copy(&self.path)?;
        Ok(())
    }

    // -- Track handling --------------------------------------------------------------------------

    pub fn get_tracks(&mut self) -> &mut Vec<Track> {
        &mut self.doc.collection.tracks
    }

    pub fn get_track_by_id(&self, track_id: &str) -> Option<Track> {
        let item = self
            .doc
            .collection
            .tracks
            .iter()
            .find(|t| &t.trackid == track_id);
        if let Some(item) = item {
            Some(item.clone())
        } else {
            None
        }
    }

    pub fn get_track_by_location(&self, location: &str) -> Option<Track> {
        let item = self
            .doc
            .collection
            .tracks
            .iter()
            .find(|t| t.location == location);
        if let Some(item) = item {
            Some(item.clone())
        } else {
            None
        }
    }

    pub fn get_track_by_key(&self, key: &str, key_type: PlaylistKeyType) -> Option<Track> {
        match key_type {
            PlaylistKeyType::TrackID => self.get_track_by_id(key),
            PlaylistKeyType::Location => self.get_track_by_location(key),
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.doc.collection.tracks.push(track);
    }

    pub fn update_track(&mut self, track: Track) -> anyhow::Result<()> {
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
            Err(anyhow::anyhow!("Track not found"))
        }
    }

    pub fn remove_track(&mut self, track_id: &str) -> anyhow::Result<()> {
        // Remove the track from the collection
        let index = self
            .doc
            .collection
            .tracks
            .iter()
            .position(|t| t.trackid == track_id)
            .ok_or_else(|| anyhow::anyhow!("Track not found"))?;
        self.doc.collection.tracks.remove(index);

        // Remove any references to the track in playlists
        Ok(())
    }

    // -- Playlist handling -----------------------------------------------------------------------

    pub fn get_playlist_root(&mut self) -> &mut PlaylistNode {
        &mut self.doc.playlists.node
    }

    pub fn get_playlist_by_path(
        &mut self,
        path: Vec<String>,
    ) -> anyhow::Result<Option<&mut PlaylistNode>> {
        let root = &mut self.doc.playlists.node;
        root.get_child_by_path(path)
    }

    pub fn add_playlist(
        &mut self,
        name: &str,
        key_type: PlaylistKeyType,
        parent_path: Vec<String>,
    ) -> anyhow::Result<()> {
        let parent = self.get_playlist_by_path(parent_path)?;
        if parent.is_none() {
            return Err(anyhow::anyhow!("Parent playlist not found"));
        }
        parent.unwrap().add_playlist(name, key_type)?;
        Ok(())
    }

    pub fn add_folder(&mut self, name: &str, parent_path: Vec<String>) -> anyhow::Result<()> {
        let parent = self.get_playlist_by_path(parent_path)?;
        if parent.is_none() {
            return Err(anyhow::anyhow!("Parent playlist not found"));
        }
        parent.unwrap().add_folder(name)?;
        Ok(())
    }

    pub fn add_track_key_to_playlist(
        &mut self,
        key: &str,
        parent_path: Vec<String>,
    ) -> anyhow::Result<()> {
        let parent = self.get_playlist_by_path(parent_path)?;
        if parent.is_none() {
            return Err(anyhow::anyhow!("Parent playlist not found"));
        }
        parent.unwrap().add_track_key(key)?;
        Ok(())
    }

    pub fn add_track_to_playlist(
        &mut self,
        track: &Track,
        parent_path: Vec<String>,
    ) -> anyhow::Result<()> {
        let parent = self.get_playlist_by_path(parent_path)?;
        if parent.is_none() {
            return Err(anyhow::anyhow!("Parent playlist not found"));
        }
        parent.unwrap().add_track(track)?;
        Ok(())
    }
}
