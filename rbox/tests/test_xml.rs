// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use rbox::xml::*;
use tempfile::NamedTempFile;

mod common;

#[test]
fn test_read_xml() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path();
    let _xml = RekordboxXml::load(path.path());
    Ok(())
}

#[test]
fn test_write_xml() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path();
    let out = NamedTempFile::new_in(common::get_temp_root_dir())?;
    let xml = RekordboxXml::load(path.path());
    // Write to input file
    xml.dump()?;
    // Write to new file
    xml.dump_copy(out.path())?;
    Ok(())
}

// ---- Tracks -------------------------------------------------------------------------------------

#[test]
fn test_get_tracks() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 6);
    Ok(())
}

#[test]
fn test_get_track() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());

    let track1 = xml.get_track(0);
    assert!(track1.is_some());

    let track_none = xml.get_track(6);
    assert!(track_none.is_none());
    Ok(())
}

#[test]
fn test_get_track_by_id() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());

    let track = xml.get_track_by_id("253529738");
    assert!(track.is_some());

    let track_none = xml.get_track_by_id("12345");
    assert!(track_none.is_none());

    Ok(())
}

#[test]
fn test_get_track_location_decode() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());

    let track = xml.get_track_by_id("253529738").unwrap();
    let location = &track.location;
    let expected = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 1.mp3";
    assert_eq!(location, expected);

    Ok(())
}

#[test]
fn test_get_track_location_encode() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path();
    let out = NamedTempFile::new_in(common::get_temp_root_dir())?;
    let track_id = "253529738";
    let mut xml = RekordboxXml::load(path.path());

    let track = xml.get_track_by_id(&track_id).unwrap();
    let new_location = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 2.mp3";
    track.location = new_location.to_string();

    // Dump xml file and reload
    xml.dump_copy(&out)?;
    let mut xml2 = RekordboxXml::load(out.path());
    let track = xml2.get_track_by_id(&track_id).unwrap();
    assert_eq!(track.location, new_location);

    Ok(())
}

#[test]
fn test_get_track_by_location() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());

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
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());
    let id = "253529738";
    let loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track 1.mp3";

    let track = xml.get_track_by_key(loc, PlaylistKeyType::Location);
    assert!(track.is_some());

    let track = xml.get_track_by_key(id, PlaylistKeyType::TrackID);
    assert!(track.is_some());

    Ok(())
}

#[test]
fn test_add_track() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());
    let id = "1234567";
    let loc = "C:/Music/PioneerDJ/Demo Tracks/Demo Track.mp3";

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 6);

    let new_track = Track::new(id, loc);
    xml.add_track(new_track);

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 7);

    let track = xml.get_track_by_id(id);
    assert!(track.is_some());

    Ok(())
}

#[test]
fn test_new_track() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());
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
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());
    let id = "253529738";

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 6);

    xml.remove_track(id)?;

    let tracks = xml.get_tracks();
    assert_eq!(tracks.len(), 5);

    Ok(())
}

// ---- Playlists ----------------------------------------------------------------------------------

#[test]
fn test_new_playlist_folder_node() -> anyhow::Result<()> {
    let folder = PlaylistNode::folder("Test Folder");
    assert_eq!(folder.name, "Test Folder");
    assert_eq!(folder.node_type, PlaylistNodeType::Folder);
    assert!(folder.nodes.is_some());
    assert!(folder.tracks.is_none());
    Ok(())
}

#[test]
fn test_new_playlist_node() -> anyhow::Result<()> {
    let playlist = PlaylistNode::playlist("Test Playlist", PlaylistKeyType::TrackID);
    assert_eq!(playlist.name, "Test Playlist");
    assert_eq!(playlist.node_type, PlaylistNodeType::Playlist);
    assert!(playlist.tracks.is_some());
    assert!(playlist.nodes.is_none());
    assert_eq!(playlist.key_type, Some(PlaylistKeyType::TrackID));
    Ok(())
}

#[test]
fn test_add_node_to_folder() -> anyhow::Result<()> {
    let mut folder = PlaylistNode::folder("Parent Folder");
    let child_folder = PlaylistNode::folder("Child Folder");
    folder.add_node(child_folder);
    assert_eq!(folder.nodes.as_ref().unwrap().len(), 1);
    assert_eq!(folder.nodes.as_ref().unwrap()[0].name, "Child Folder");
    Ok(())
}

#[test]
fn test_add_track_to_playlist() -> anyhow::Result<()> {
    let mut playlist = PlaylistNode::playlist("Test Playlist", PlaylistKeyType::TrackID);
    playlist.add_track_key("123")?;
    assert_eq!(playlist.tracks.as_ref().unwrap().len(), 1);
    assert_eq!(playlist.tracks.as_ref().unwrap()[0].key, "123");
    Ok(())
}

#[test]
fn test_remove_node_from_folder() -> anyhow::Result<()> {
    let mut folder = PlaylistNode::folder("Parent Folder");
    folder.add_node(PlaylistNode::folder("Child Folder"));
    folder.remove_node(0).unwrap();
    assert!(folder.nodes.as_ref().unwrap().is_empty());
    Ok(())
}

#[test]
fn test_remove_track_from_playlist() -> anyhow::Result<()> {
    let mut playlist = PlaylistNode::playlist("Test Playlist", PlaylistKeyType::TrackID);
    playlist.add_track_key("123")?;
    playlist.remove_track("123")?;
    assert!(playlist.tracks.as_ref().unwrap().is_empty());
    Ok(())
}

#[test]
fn test_get_child_by_path() -> anyhow::Result<()> {
    let mut root = PlaylistNode::folder("Root");
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
    let mut root = PlaylistNode::folder("Root");
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
    let mut playlist = PlaylistNode::playlist("Test Playlist", PlaylistKeyType::TrackID);
    playlist.add_track_key("123").unwrap();
    playlist.add_track_key("456").unwrap();
    playlist.clear_tracks().unwrap();
    assert!(playlist.tracks.as_ref().unwrap().is_empty());
    Ok(())
}

#[test]
fn test_clear_nodes_in_folder() -> anyhow::Result<()> {
    let mut folder = PlaylistNode::folder("Parent Folder");
    folder.new_folder("Child Folder 1").unwrap();
    folder.new_folder("Child Folder 2").unwrap();
    folder.clear_nodes().unwrap();
    assert!(folder.nodes.as_ref().unwrap().is_empty());
    Ok(())
}

#[test]
fn test_playlist_node_mutability() -> anyhow::Result<()> {
    let out = NamedTempFile::new_in(common::testdata_dir())?;
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());

    let root = xml.get_playlist_root();
    let node = root.find_child("Test")?.unwrap();

    // Change name
    node.name = "Renamed".to_string();

    xml.dump_copy(&out)?;

    let mut xml2 = RekordboxXml::load(&out.path());
    let root2 = xml2.get_playlist_root();
    let node2 = root2.find_child("Renamed")?;
    assert!(node2.is_some());

    Ok(())
}

#[test]
fn test_folder_node_mutability() -> anyhow::Result<()> {
    let out = NamedTempFile::new_in(common::testdata_dir())?;
    let path = common::setup_rekordbox_xml_path();
    let mut xml = RekordboxXml::load(path.path());

    let root = xml.get_playlist_root();
    let folder = root.new_folder("Folder")?;
    folder.new_playlist("Playlist", PlaylistKeyType::TrackID)?;

    xml.dump_copy(&out)?;

    let mut xml2 = RekordboxXml::load(&out.path());
    let root2 = xml2.get_playlist_root();
    let folder2 = root2.find_child("Folder")?;
    assert!(folder2.is_some());
    let folder2 = folder2.unwrap();
    assert_eq!(folder2.node_type, PlaylistNodeType::Folder);
    assert_eq!(folder2.nodes.as_ref().unwrap().len(), 1);

    Ok(())
}
