// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use rbox_testing::data::{get_temp_root_dir, setup_rekordbox_xml_path};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::NamedTempFile;

use rbox_xml::*;

fn temporary_xml_path(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must follow the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rbox-xml-{name}-{}-{nonce}.xml",
        std::process::id()
    ))
}

#[test]
fn test_read_xml() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let _xml = RekordboxXml::load(path.path())?;
    Ok(())
}

#[test]
fn test_write_xml() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let out = NamedTempFile::new_in(get_temp_root_dir())?;
    let xml = RekordboxXml::load(path.path())?;
    // Write to input file
    xml.dump()?;
    // Write to new file
    xml.dump_copy(out.path())?;
    Ok(())
}

// ---- Tracks -------------------------------------------------------------------------------------

#[test]
fn test_get_tracks() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 6);
    Ok(())
}

#[test]
fn test_get_track() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;

    let track1 = xml.get_track(0);
    assert!(track1.is_some());

    let track_none = xml.get_track(6);
    assert!(track_none.is_none());
    Ok(())
}

#[test]
fn test_get_track_by_id() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;

    let track = xml.get_track_by_id("253529738");
    assert!(track.is_some());

    let track_none = xml.get_track_by_id("12345");
    assert!(track_none.is_none());

    Ok(())
}

#[test]
fn test_get_track_location_decode() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;

    let track = xml.get_track_by_id("253529738").unwrap();
    let location = &track.location;
    let expected = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 1.mp3";
    assert_eq!(location, expected);

    Ok(())
}

#[test]
fn test_get_track_location_encode() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let out = NamedTempFile::new_in(get_temp_root_dir())?;
    let track_id = "253529738";
    let mut xml = RekordboxXml::load(path.path())?;

    let track = xml.get_track_by_id(&track_id).unwrap();
    let new_location = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 2.mp3";
    track.location = new_location.to_string();

    // Dump xml file and reload
    xml.dump_copy(&out)?;
    let mut xml2 = RekordboxXml::load(out.path())?;
    let track = xml2.get_track_by_id(&track_id).unwrap();
    assert_eq!(track.location, new_location);

    Ok(())
}

#[test]
fn test_get_track_by_location() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;

    let loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 1.mp3";
    let track = xml.get_track_by_location(loc);
    assert!(track.is_some());

    let loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track.mp3";
    let track = xml.get_track_by_location(loc);
    assert!(track.is_none());

    Ok(())
}

#[test]
fn test_get_track_by_key() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;
    let id = "253529738";
    let loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 1.mp3";

    let track = xml.get_track_by_key(loc, XmlPlaylistKeyType::Location);
    assert!(track.is_some());

    let track = xml.get_track_by_key(id, XmlPlaylistKeyType::TrackID);
    assert!(track.is_some());

    Ok(())
}

#[test]
fn test_add_track() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;
    let id = "1234567";
    let loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track.mp3";

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 6);

    let new_track = XmlTrack::new(id, loc);
    xml.add_track(new_track);

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 7);

    let track = xml.get_track_by_id(id);
    assert!(track.is_some());

    Ok(())
}

#[test]
fn test_new_track() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;
    let id = "1234567";
    let loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track.mp3";

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 6);

    xml.new_track(id, loc);

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 7);

    let track = xml.get_track_by_id(id);
    assert!(track.is_some());

    Ok(())
}

#[test]
fn test_remove_track() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;
    let id = "253529738";

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 6);

    xml.remove_track(id)?;

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 5);

    Ok(())
}

// ---- XmlPlaylists ----------------------------------------------------------------------------------

#[test]
fn test_new_playlist_folder_node() -> anyhow::Result<()> {
    let folder = XmlPlaylistNode::folder("Test Folder");
    assert_eq!(folder.name, "Test Folder");
    assert_eq!(folder.node_type, XmlPlaylistNodeType::Folder);
    assert!(folder.nodes.is_some());
    assert!(folder.tracks.is_none());
    Ok(())
}

#[test]
fn test_new_playlist_node() -> anyhow::Result<()> {
    let playlist = XmlPlaylistNode::playlist("Test Playlist", XmlPlaylistKeyType::TrackID);
    assert_eq!(playlist.name, "Test Playlist");
    assert_eq!(playlist.node_type, XmlPlaylistNodeType::Playlist);
    assert!(playlist.tracks.is_some());
    assert!(playlist.nodes.is_none());
    assert_eq!(playlist.key_type, Some(XmlPlaylistKeyType::TrackID));
    Ok(())
}

#[test]
fn test_add_node_to_folder() -> anyhow::Result<()> {
    let mut folder = XmlPlaylistNode::folder("Parent Folder");
    let child_folder = XmlPlaylistNode::folder("Child Folder");
    folder.add_node(child_folder)?;
    assert_eq!(folder.nodes.as_ref().unwrap().len(), 1);
    assert_eq!(folder.nodes.as_ref().unwrap()[0].name, "Child Folder");
    Ok(())
}

#[test]
fn test_add_track_to_playlist() -> anyhow::Result<()> {
    let mut playlist = XmlPlaylistNode::playlist("Test Playlist", XmlPlaylistKeyType::TrackID);
    playlist.add_track_key("123")?;
    assert_eq!(playlist.tracks.as_ref().unwrap().len(), 1);
    assert_eq!(playlist.tracks.as_ref().unwrap()[0].key, "123");
    Ok(())
}

#[test]
fn test_remove_node_from_folder() -> anyhow::Result<()> {
    let mut folder = XmlPlaylistNode::folder("Parent Folder");
    folder.add_node(XmlPlaylistNode::folder("Child Folder"))?;
    folder.remove_node(0).unwrap();
    assert!(folder.nodes.as_ref().unwrap().is_empty());
    Ok(())
}

#[test]
fn test_remove_track_from_playlist() -> anyhow::Result<()> {
    let mut playlist = XmlPlaylistNode::playlist("Test Playlist", XmlPlaylistKeyType::TrackID);
    playlist.add_track_key("123")?;
    playlist.remove_track("123")?;
    assert!(playlist.tracks.as_ref().unwrap().is_empty());
    Ok(())
}

#[test]
fn test_get_child_by_path() -> anyhow::Result<()> {
    let mut root = XmlPlaylistNode::folder("Root");
    root.new_folder("Child Folder")?;
    let child = root.get_child_by_path(vec!["Child Folder".to_string()])?;
    assert!(child.is_some());
    assert_eq!(child.unwrap().name, "Child Folder");

    let child = root.get_child_by_path(vec!["Nonexistent".to_string()])?;
    assert!(child.is_none());

    Ok(())
}

#[test]
fn test_find_child() -> anyhow::Result<()> {
    let mut root = XmlPlaylistNode::folder("Root");
    root.new_folder("Child Folder 1").unwrap();
    root.new_folder("Child Folder 2").unwrap();

    let node = root.find_child("Child Folder 1")?;
    assert!(node.is_some());

    let node = root.find_child("Child Folder 2")?;
    assert!(node.is_some());

    let node = root.find_child("Child Folder 3")?;
    assert!(node.is_none());
    Ok(())
}

#[test]
fn test_clear_tracks_in_playlist() -> anyhow::Result<()> {
    let mut playlist = XmlPlaylistNode::playlist("Test Playlist", XmlPlaylistKeyType::TrackID);
    playlist.add_track_key("123").unwrap();
    playlist.add_track_key("456").unwrap();
    playlist.clear_tracks().unwrap();
    assert!(playlist.tracks.as_ref().unwrap().is_empty());
    Ok(())
}

#[test]
fn test_clear_nodes_in_folder() -> anyhow::Result<()> {
    let mut folder = XmlPlaylistNode::folder("Parent Folder");
    folder.new_folder("Child Folder 1").unwrap();
    folder.new_folder("Child Folder 2").unwrap();
    folder.clear_nodes().unwrap();
    assert!(folder.nodes.as_ref().unwrap().is_empty());
    Ok(())
}

#[test]
fn test_playlist_node_mutability() -> anyhow::Result<()> {
    let out = NamedTempFile::new_in(get_temp_root_dir())?;
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;

    let root = xml.get_playlist_root();
    let node = root.find_child("Test")?.unwrap();

    // Change name
    node.name = "Renamed".to_string();

    xml.dump_copy(&out)?;

    let mut xml2 = RekordboxXml::load(&out.path())?;
    let root2 = xml2.get_playlist_root();
    let node2 = root2.find_child("Renamed")?;
    assert!(node2.is_some());

    Ok(())
}

#[test]
fn test_folder_node_mutability() -> anyhow::Result<()> {
    let out = NamedTempFile::new_in(get_temp_root_dir())?;
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;

    let root = xml.get_playlist_root();
    let folder = root.new_folder("Folder")?;
    folder.new_playlist("Playlist", XmlPlaylistKeyType::TrackID)?;

    xml.dump_copy(&out)?;

    let mut xml2 = RekordboxXml::load(&out.path())?;
    let root2 = xml2.get_playlist_root();
    let folder2 = root2.find_child("Folder")?;
    assert!(folder2.is_some());
    let folder2 = folder2.unwrap();
    assert_eq!(folder2.node_type, XmlPlaylistNodeType::Folder);
    assert_eq!(folder2.nodes.as_ref().unwrap().len(), 1);

    Ok(())
}

#[test]
fn load_returns_typed_io_error() {
    let path = temporary_xml_path("missing");

    let error = RekordboxXml::load(path).unwrap_err();

    assert!(matches!(error, XmlError::Io(_)));
}

#[test]
fn load_returns_typed_deserialization_error() {
    let path = temporary_xml_path("malformed");
    fs::write(&path, "<DJ_PLAYLISTS>").unwrap();

    let error = RekordboxXml::load(&path).unwrap_err();
    fs::remove_file(path).unwrap();

    assert!(matches!(error, XmlError::Deserialize(_)));
}

#[test]
fn node_operations_return_structural_errors() {
    let mut playlist = XmlPlaylistNode::playlist("Playlist", XmlPlaylistKeyType::TrackID);
    assert!(matches!(
        playlist.new_folder("Folder"),
        Err(XmlError::InvalidOperation {
            operation: XmlPlaylistOperation::AddChild,
            node_type: XmlPlaylistNodeType::Playlist,
        })
    ));
    assert!(matches!(
        playlist.get_child(0),
        Err(XmlError::InvalidOperation {
            operation: XmlPlaylistOperation::GetChild,
            node_type: XmlPlaylistNodeType::Playlist,
        })
    ));

    let mut folder = XmlPlaylistNode::folder("Folder");
    assert!(matches!(
        folder.get_track(0),
        Err(XmlError::InvalidOperation {
            operation: XmlPlaylistOperation::GetTrack,
            node_type: XmlPlaylistNodeType::Folder,
        })
    ));
    assert!(matches!(
        folder.remove_node(0),
        Err(XmlError::ChildIndexOutOfBounds { index: 0, len: 0 })
    ));
}

#[test]
fn lookup_errors_include_the_missing_key_or_path() {
    let mut xml = RekordboxXml::new("library.xml");

    assert!(matches!(
        xml.update_track(XmlTrack::new("42", "/music/track.mp3")),
        Err(XmlError::NotFound {
            entity: XmlEntity::Track,
            identifier,
        }) if identifier == "42"
    ));

    let path = vec!["Missing".to_owned()];
    assert!(matches!(
        xml.new_folder("Child", path),
        Err(XmlError::NotFound {
            entity: XmlEntity::ParentPlaylist,
            identifier,
        }) if identifier == "Missing"
    ));
}

#[test]
fn enum_conversions_return_value_specific_errors() {
    assert!(matches!(
        XmlPlaylistNodeType::try_from(2),
        Err(XmlError::InvalidValue {
            field: "playlist node type",
            value: 2,
        })
    ));
    assert!(matches!(
        XmlPlaylistKeyType::try_from(-1),
        Err(XmlError::InvalidValue {
            field: "playlist key type",
            value: -1,
        })
    ));
}

#[test]
fn serialization_returns_typed_error_for_invalid_playlist() {
    let mut xml = RekordboxXml::new("library.xml");
    xml.set_playlist_root(XmlPlaylistNode::new(
        "Invalid",
        XmlPlaylistNodeType::Playlist,
    ));

    let error = xml.to_string().unwrap_err();

    assert!(matches!(error, XmlError::Serialize(_)));
}

#[test]
fn test_file_information_round_trip() -> anyhow::Result<()> {
    let path = setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path())?;
    let original: XmlDocument = quick_xml::de::from_str(&xml.to_string()?)?;
    assert_eq!(xml.get_version(), original.version);
    assert_eq!(xml.get_product_name(), original.product.name);
    assert_eq!(xml.get_product_version(), original.product.version);
    assert_eq!(xml.get_product_company(), original.product.company);

    xml.set_version("1.0.1");
    xml.set_product_name("Library & <Tools>");
    xml.set_product_version("2.3.4");
    xml.set_product_company("?? \"Company\"");
    xml.dump()?;
    let loaded = RekordboxXml::load(path.path())?;
    assert_eq!(loaded.get_version(), "1.0.1");
    assert_eq!(loaded.get_product_name(), "Library & <Tools>");
    assert_eq!(loaded.get_product_version(), "2.3.4");
    assert_eq!(loaded.get_product_company(), "?? \"Company\"");
    let updated: XmlDocument = quick_xml::de::from_str(&loaded.to_string()?)?;
    assert_eq!(updated.collection, original.collection);
    assert_eq!(updated.playlists, original.playlists);
    Ok(())
}
