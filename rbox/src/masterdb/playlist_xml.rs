// Author: Dylan Jones
// Date:   2025-05-05

use chrono::{DateTime, NaiveDateTime};
use quick_xml;
use quick_xml::events::{BytesDecl, Event};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Custom serializer for the Node ID
///
/// Converts from normal ID to hexadecimal format for XML
fn serialize_node_id<S>(id: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if id == "root" {
        return serializer.serialize_str("0");
    }
    match id.parse::<u64>() {
        Ok(number) => serializer.serialize_str(&format!("{:X}", number)),
        Err(e) => {
            use serde::ser::Error;
            Err(S::Error::custom(format!("Failed to parse ID: {}", e)))
        }
    }
}

/// Custom deserializer for the Node ID
///
/// Converts from hexadecimal format in XML to normal ID
fn deserialize_node_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let hexid = String::deserialize(deserializer)?;
    if hexid == "0" {
        return Ok("root".to_string());
    }
    match u64::from_str_radix(&hexid, 16) {
        Ok(id) => Ok(id.to_string()),
        Err(e) => {
            use serde::de::Error;
            Err(D::Error::custom(format!(
                "Failed to convert hex to ID: {}",
                e
            )))
        }
    }
}

/// Custom serializer for timestamps
/// Converts from chrono DateTime to milliseconds timestamp for XML
fn serialize_timestamp<S>(datetime: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert DateTime to milliseconds timestamp
    let timestamp_ms = datetime.and_utc().timestamp_millis();
    serializer.serialize_i64(timestamp_ms)
}

/// Custom deserializer for timestamps
/// Converts from milliseconds timestamp in XML to chrono DateTime
fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    // Get the timestamp in milliseconds from XML
    let timestamp_ms = i64::deserialize(deserializer)?;
    // Convert milliseconds to seconds and nanoseconds
    let seconds = timestamp_ms / 1000;
    let nanos = ((timestamp_ms % 1000) * 1_000_000) as u32;
    // Create DateTime from timestamp using the recommended method
    match DateTime::from_timestamp(seconds, nanos) {
        Some(dt) => Ok(dt.naive_utc()),
        None => {
            use serde::de::Error;
            Err(D::Error::custom(format!(
                "Invalid timestamp: {}",
                timestamp_ms
            )))
        }
    }
}

/// The product XML element of the Rekordbox masterPlaylist6 XML file.
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

/// Playlist node XML element of the Rekordbox masterPlaylist6 XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Node {
    /// ID of the playlist
    #[serde(
        rename = "@Id",
        serialize_with = "serialize_node_id",
        deserialize_with = "deserialize_node_id"
    )]
    pub id: String,
    /// Parent ID of the playlist
    #[serde(
        rename = "@ParentId",
        serialize_with = "serialize_node_id",
        deserialize_with = "deserialize_node_id"
    )]
    pub parent_id: String,
    /// Playlist attribute
    #[serde(rename = "@Attribute")]
    pub attribute: i32,
    /// Update timestamp
    #[serde(
        rename = "@Timestamp",
        serialize_with = "serialize_timestamp",
        deserialize_with = "deserialize_timestamp"
    )]
    pub timestamp: NaiveDateTime,
    /// Library Type (?)
    #[serde(rename = "@Lib_Type")]
    pub lib_type: i32,
    /// Check type (?)
    #[serde(rename = "@CheckType")]
    pub check_type: i32,
}

/// Playlist parent element of the Rekordbox masterPlaylist6 XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Playlists {
    /// Playlist Nodes
    #[serde(rename = "NODE")]
    pub nodes: Vec<Node>,
}

/// The root element of the Rekordbox masterPlaylist6 XML file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "MASTER_PLAYLIST")]
pub struct Document {
    /// Version of the XML format for share the playlists.
    ///
    /// The latest version is 1,0,0.
    #[serde(rename = "@Version")]
    pub version: String,
    /// Automatic sync flag.
    #[serde(rename = "@AutomaticSync")]
    pub automatic_sync: i32,
    /// Product information
    #[serde(rename = "PRODUCT")]
    pub product: Product,
    /// Playlists information
    #[serde(rename = "PLAYLISTS")]
    pub playlists: Playlists,
}

/// Rekordbox masterPlaylist6 XML file handler.
pub struct MasterPlaylistXml {
    /// Path to the XML file
    path: PathBuf,
    /// XML document
    pub doc: Document,
}

impl MasterPlaylistXml {
    /// Create a new Rekordbox masterPlaylist6 XML file.
    pub fn new<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> Self {
        let p = Path::new(&path).to_path_buf();
        let version = "3.0.0";
        let automatic_sync = 0;
        let product_name = "rekordbox";
        let product_version = "6.0.0";
        let product_company = "Pioneer DJ";
        let doc = Document {
            version: version.to_string(),
            automatic_sync,
            product: Product {
                name: product_name.to_string(),
                version: product_version.to_string(),
                company: product_company.to_string(),
            },
            playlists: Playlists { nodes: Vec::new() },
        };

        MasterPlaylistXml { path: p, doc }
    }

    /// Read a Rekordbox masterPlaylist6 XML file.
    pub fn load<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> Self {
        let p = Path::new(&path).to_path_buf();
        let file = std::fs::File::open(path).expect("File not found");
        let reader = std::io::BufReader::new(file);
        let doc: Document = quick_xml::de::from_reader(reader).expect("failed to deserialize XML");
        MasterPlaylistXml { path: p, doc }
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
        ser.indent(' ', 4);
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

    // --------------------------------------------------------------------------------------------

    /// Get all playlist nodes from the XML document.
    pub fn get_playlists(&self) -> Vec<Node> {
        self.doc.playlists.nodes.clone()
    }

    /// Get a playlist node by the *database ID* from the XML document.
    ///
    /// The ID is the same as the one used in the database.
    /// It is not the same as the one used in the XML file.
    pub fn get_playlist(&self, id: String) -> Option<Node> {
        for node in &self.doc.playlists.nodes {
            if node.id == id {
                return Some(node.clone());
            }
        }
        None
    }

    /// Create a new playlist node and add it to the XML document.
    ///
    /// The ID is the same as the one used in the database.
    /// It is not the same as the one used in the XML file.
    pub fn add(&mut self, id: String, parent_id: String, attribute: i32, timestamp: NaiveDateTime) {
        let node = Node {
            id,
            parent_id,
            attribute,
            timestamp,
            lib_type: 0,
            check_type: 0,
        };
        self.doc.playlists.nodes.push(node);
    }

    /// Remove a playlist node from the XML document.
    ///
    /// The ID is the same as the one used in the database.
    /// It is not the same as the one used in the XML file.
    pub fn remove(&mut self, id: String) {
        self.doc.playlists.nodes.retain(|node| node.id != id);
    }

    pub fn update(&mut self, id: String, timestamp: NaiveDateTime) {
        for node in &mut self.doc.playlists.nodes {
            if node.id == id {
                node.timestamp = timestamp;
                break;
            }
        }
    }

    pub fn update_parent(&mut self, id: String, parent_id: String, timestamp: NaiveDateTime) {
        for node in &mut self.doc.playlists.nodes {
            if node.id == id {
                node.parent_id = parent_id;
                node.timestamp = timestamp;
                break;
            }
        }
    }
}
