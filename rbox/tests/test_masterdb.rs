// Author: Dylan Jones
// Date:   2025-05-06

use rbox::masterdb::{DjmdPlaylist, DjmdSongPlaylist, MasterDb, MasterPlaylistXml, PlaylistType};

mod common;

#[test]
fn test_open_master_db() -> anyhow::Result<()> {
    let _db = common::setup_master_db()?;
    Ok(())
}

// -- AgentRegistry --------------------------------------------------------------------------------

#[test]
fn test_get_local_usn() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let usn = db.get_local_usn()?;
    assert!(usn > 0);
    Ok(())
}

// -- Album ----------------------------------------------------------------------------------------

#[test]
fn test_get_albums() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let _album = db.get_album()?;
    Ok(())
}

#[test]
fn test_get_album_by_id() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let album = db.get_album_by_id("1234")?;
    assert!(album.is_none());
    Ok(())
}

#[test]
fn test_get_album_by_name() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let name = "Name".to_string();

    let item = db.get_album_by_name("Name")?;
    assert!(item.is_none());

    db.insert_album(name, None, None, None)?;

    let item = db.get_album_by_name("Name")?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_insert_album() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    let old_usn = db.get_local_usn()?;
    // Insert a new album
    let name = "New Album".to_string();
    let artist = None;
    let image_path = None;
    let compilation = None;
    let new_album = db.insert_album(name.clone(), artist, image_path, compilation)?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_album.Name, Some(name));
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(new_album.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let album = db.get_album_by_id(new_album.ID.as_str())?;
    assert!(album.is_some());

    Ok(())
}

#[test]
fn test_update_album() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    // Insert a new album
    let mut album = db.insert_album("New Album".to_string(), None, None, None)?;
    let old_usn = db.get_local_usn()?;

    // Update the album
    let id = album.ID.clone();
    let new_name = "Updated Album".to_string();
    album.Name = Some(new_name.clone());
    let updated = db.update_album(&mut album);
    let new_usn = db.get_local_usn()?;
    assert!(updated.is_ok());
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(updated?.rb_local_usn.unwrap(), new_usn);

    // Verify the update
    let updated_album = db.get_album_by_id(id.as_str())?;
    assert!(updated_album.is_some());
    assert_eq!(updated_album.unwrap().Name, Some(new_name));

    Ok(())
}

#[test]
fn test_delete_album() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    // Insert a new album
    let album = db.insert_album("New Album".to_string(), None, None, None)?;

    // Refer to the album by its ID in content
    let contents = db.get_content()?;
    let cid = contents[0].ID.clone();
    let mut content = db
        .get_content_by_id(cid.as_str())?
        .expect("get content failed");
    content.AlbumID = Some(album.ID.clone());
    db.update_content(&content)?;
    let linked_content = db.get_content_by_id(cid.as_str())?;
    assert_eq!(linked_content.unwrap().AlbumID, Some(album.ID.clone()));

    // Delete the album
    let old_usn = db.get_local_usn()?;
    let id = album.ID.clone();
    let deleted = db.delete_album(id.as_str());
    let new_usn = db.get_local_usn()?;
    assert!(deleted.is_ok());
    assert_eq!(new_usn, old_usn + 1);

    // Verify the deletion
    let deleted_album = db.get_album_by_id(id.as_str())?;
    assert!(deleted_album.is_none());

    // Verify orphaned content
    let orphaned_content = db.get_content_by_id(cid.as_str())?;
    assert!(orphaned_content.is_some());
    assert!(orphaned_content.unwrap().AlbumID.is_none());

    Ok(())
}

// -- Artist ---------------------------------------------------------------------------------------

#[test]
fn test_get_artist() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let _items = db.get_artist()?;
    Ok(())
}

#[test]
fn test_get_artist_by_id() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let item = db.get_artist_by_id("1234")?;
    assert!(item.is_none());

    let items = db.get_artist()?;
    let artist = items[0].ID.clone();
    let item = db.get_artist_by_id(artist.as_str())?;
    assert!(item.is_some());
    Ok(())
}

#[test]
fn test_get_artist_by_name() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let name = "Name".to_string();

    let item = db.get_artist_by_name("Name")?;
    assert!(item.is_none());

    db.insert_artist(name)?;

    let item = db.get_artist_by_name("Name")?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_insert_artist() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    let old_usn = db.get_local_usn()?;
    // Insert a new artist
    let name = "New Artist".to_string();
    let new_item = db.insert_artist(name.clone())?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.Name, Some(name));
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item = db.get_artist_by_id(new_item.ID.as_str())?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_update_artist() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    // Insert a new artist
    let mut item = db.insert_artist("New Artist".to_string())?;
    let old_usn = db.get_local_usn()?;

    // Update the artist
    let id = item.ID.clone();
    let new_name = "Updated Artist".to_string();
    item.Name = Some(new_name.clone());
    let updated = db.update_artist(&mut item);
    let new_usn = db.get_local_usn()?;
    assert!(updated.is_ok());
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(updated?.rb_local_usn.unwrap(), new_usn);

    // Verify the update
    let updated_item = db.get_artist_by_id(id.as_str())?;
    assert!(updated_item.is_some());
    assert_eq!(updated_item.unwrap().Name, Some(new_name));

    Ok(())
}

#[test]
fn test_delete_artist() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    // Insert a new artist
    let item = db.insert_artist("New Artist".to_string())?;

    // Refer to the artist by its ID in content
    let contents = db.get_content()?;
    let cid = contents[0].ID.clone();
    let mut content = db
        .get_content_by_id(cid.as_str())?
        .expect("get content failed");
    content.ArtistID = Some(item.ID.clone());
    content.OrgArtistID = Some(item.ID.clone());
    db.update_content(&content)?;
    let linked_content = db.get_content_by_id(cid.as_str())?;
    assert_eq!(linked_content.unwrap().ArtistID, Some(item.ID.clone()));

    // Delete the artist
    let old_usn = db.get_local_usn()?;
    let id = item.ID.clone();
    let deleted = db.delete_artist(id.as_str());
    let new_usn = db.get_local_usn()?;
    assert!(deleted.is_ok());
    assert_eq!(new_usn, old_usn + 1);

    // Verify the deletion
    let deleted_album = db.get_artist_by_id(id.as_str())?;
    assert!(deleted_album.is_none());

    // Verify orphaned content
    let orphaned_content = db.get_content_by_id(cid.as_str())?;
    assert!(orphaned_content.is_some());
    let orphaned = orphaned_content.clone().unwrap();
    assert!(orphaned.ArtistID.is_none());
    assert!(orphaned.OrgArtistID.is_none());

    Ok(())
}

// -- Content ----------------------------------------------------------------------------------

#[test]
fn test_get_content() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let _items = db.get_content()?;
    Ok(())
}

#[test]
fn test_get_content_by_id() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let item = db.get_content_by_id("1234")?;
    assert!(item.is_none());

    let items = db.get_content()?;
    let reference = items[0].ID.clone();

    let item = db.get_content_by_id(reference.as_str())?;
    assert!(item.is_some());
    Ok(())
}

#[test]
fn test_get_content_by_path() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let item = db.get_content_by_path("invalid/path")?;
    assert!(item.is_none());

    let items = db.get_content()?;
    let reference = items[0].FolderPath.clone().expect("No folder path");

    let item = db.get_content_by_path(reference.as_str())?;
    assert!(item.is_some());
    Ok(())
}

// -- Genre ----------------------------------------------------------------------------------------

#[test]
fn test_get_genre() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let _items = db.get_genre()?;
    Ok(())
}

#[test]
fn test_get_genre_by_id() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let item = db.get_genre_by_id("1234")?;
    assert!(item.is_none());
    Ok(())
}

#[test]
fn test_get_genre_by_name() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let name = "Name".to_string();

    let item = db.get_genre_by_name("Name")?;
    assert!(item.is_none());

    db.insert_genre(name)?;

    let item = db.get_genre_by_name("Name")?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_insert_genre() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    let old_usn = db.get_local_usn()?;
    // Insert a new artist
    let name = "New Genre".to_string();
    let new_item = db.insert_genre(name.clone())?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.Name, Some(name));
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item = db.get_genre_by_id(new_item.ID.as_str())?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_update_genre() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    // Insert a new artist
    let mut item = db.insert_genre("New Genre".to_string())?;
    let old_usn = db.get_local_usn()?;

    // Update the artist
    let id = item.ID.clone();
    let new_name = "Updated Genre".to_string();
    item.Name = Some(new_name.clone());
    let updated = db.update_genre(&mut item);
    let new_usn = db.get_local_usn()?;
    assert!(updated.is_ok());
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(updated?.rb_local_usn.unwrap(), new_usn);

    // Verify the update
    let updated_item = db.get_genre_by_id(id.as_str())?;
    assert!(updated_item.is_some());
    assert_eq!(updated_item.unwrap().Name, Some(new_name));

    Ok(())
}

#[test]
fn test_delete_genre() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    // Insert a new artist
    let item = db.insert_artist("New Genre".to_string())?;

    // Refer to the artist by its ID in content
    let contents = db.get_content()?;
    let cid = contents[0].ID.clone();
    let mut content = db
        .get_content_by_id(cid.as_str())?
        .expect("get content failed");
    content.GenreID = Some(item.ID.clone());
    db.update_content(&content)?;
    let linked_content = db.get_content_by_id(cid.as_str())?;
    assert_eq!(linked_content.unwrap().GenreID, Some(item.ID.clone()));

    // Delete the artist
    let old_usn = db.get_local_usn()?;
    let id = item.ID.clone();
    let deleted = db.delete_genre(id.as_str());
    let new_usn = db.get_local_usn()?;
    assert!(deleted.is_ok());
    assert_eq!(new_usn, old_usn + 1);

    // Verify the deletion
    let deleted = db.get_genre_by_id(id.as_str())?;
    assert!(deleted.is_none());

    // Verify orphaned content
    let orphaned_content = db.get_content_by_id(cid.as_str())?;
    assert!(orphaned_content.is_some());
    let orphaned = orphaned_content.clone().unwrap();
    assert!(orphaned.GenreID.is_none());

    Ok(())
}

// -- Key ------------------------------------------------------------------------------------------

#[test]
fn test_get_key() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let _items = db.get_key()?;
    Ok(())
}

#[test]
fn test_get_key_by_id() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let item = db.get_key_by_id("1234")?;
    assert!(item.is_none());
    Ok(())
}

#[test]
fn test_get_key_by_name() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let name = "Name".to_string();

    let item = db.get_key_by_name("Name")?;
    assert!(item.is_none());

    db.insert_key(name)?;

    let item = db.get_key_by_name("Name")?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_insert_key() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    let old_usn = db.get_local_usn()?;
    // Insert a new artist
    let name = "New Key".to_string();
    let new_item = db.insert_key(name.clone())?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.ScaleName, Some(name));
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item = db.get_key_by_id(new_item.ID.as_str())?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_update_key() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    // Insert a new artist
    let mut item = db.insert_key("New Key".to_string())?;
    let old_usn = db.get_local_usn()?;

    // Update the artist
    let id = item.ID.clone();
    let new_name = "Updated Key".to_string();
    item.ScaleName = Some(new_name.clone());
    let updated = db.update_key(&mut item);
    let new_usn = db.get_local_usn()?;
    assert!(updated.is_ok());
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(updated?.rb_local_usn.unwrap(), new_usn);

    // Verify the update
    let updated_item = db.get_key_by_id(id.as_str())?;
    assert!(updated_item.is_some());
    assert_eq!(updated_item.unwrap().ScaleName, Some(new_name));

    Ok(())
}

#[test]
fn test_delete_key() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    // Insert a new artist
    let item = db.insert_artist("New Key".to_string())?;

    // Refer to the artist by its ID in content
    let contents = db.get_content()?;
    let cid = contents[0].ID.clone();
    let mut content = db
        .get_content_by_id(cid.as_str())?
        .expect("get content failed");
    content.KeyID = Some(item.ID.clone());
    db.update_content(&content)?;
    let linked_content = db.get_content_by_id(cid.as_str())?;
    assert_eq!(linked_content.unwrap().KeyID, Some(item.ID.clone()));

    // Delete the artist
    let old_usn = db.get_local_usn()?;
    let id = item.ID.clone();
    let deleted = db.delete_key(id.as_str());
    let new_usn = db.get_local_usn()?;
    assert!(deleted.is_ok());
    assert_eq!(new_usn, old_usn + 1);

    // Verify the deletion
    let deleted = db.get_key_by_id(id.as_str())?;
    assert!(deleted.is_none());

    // Verify orphaned content
    let orphaned_content = db.get_content_by_id(cid.as_str())?;
    assert!(orphaned_content.is_some());
    let orphaned = orphaned_content.clone().unwrap();
    assert!(orphaned.KeyID.is_none());

    Ok(())
}

// -- Label ----------------------------------------------------------------------------------------

#[test]
fn test_get_label() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let _items = db.get_label()?;
    Ok(())
}

#[test]
fn test_get_label_by_id() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let item = db.get_label_by_id("1234")?;
    assert!(item.is_none());
    Ok(())
}

#[test]
fn test_get_label_by_name() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let name = "Name".to_string();

    let item = db.get_label_by_name("Name")?;
    assert!(item.is_none());

    db.insert_label(name)?;

    let item = db.get_label_by_name("Name")?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_insert_label() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    let old_usn = db.get_local_usn()?;
    // Insert a new label
    let name = "New Label".to_string();
    let new_item = db.insert_label(name.clone())?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.Name, Some(name));
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item = db.get_label_by_id(new_item.ID.as_str())?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_update_label() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    // Insert a new label
    let mut item = db.insert_label("New Label".to_string())?;
    let old_usn = db.get_local_usn()?;

    // Update the artist
    let id = item.ID.clone();
    let new_name = "Updated Label".to_string();
    item.Name = Some(new_name.clone());
    let updated = db.update_label(&mut item);
    let new_usn = db.get_local_usn()?;
    assert!(updated.is_ok());
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(updated?.rb_local_usn.unwrap(), new_usn);

    // Verify the update
    let updated_item = db.get_label_by_id(id.as_str())?;
    assert!(updated_item.is_some());
    assert_eq!(updated_item.unwrap().Name, Some(new_name));

    Ok(())
}

#[test]
fn test_delete_label() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;

    // Insert a new artist
    let item = db.insert_label("New Label".to_string())?;

    // Refer to the artist by its ID in content
    let contents = db.get_content()?;
    let cid = contents[0].ID.clone();
    let mut content = db
        .get_content_by_id(cid.as_str())?
        .expect("get content failed");
    content.LabelID = Some(item.ID.clone());
    db.update_content(&content)?;
    let linked_content = db.get_content_by_id(cid.as_str())?;
    assert_eq!(linked_content.unwrap().LabelID, Some(item.ID.clone()));

    // Delete the artist
    let old_usn = db.get_local_usn()?;
    let id = item.ID.clone();
    let deleted = db.delete_label(id.as_str());
    let new_usn = db.get_local_usn()?;
    assert!(deleted.is_ok());
    assert_eq!(new_usn, old_usn + 1);

    // Verify the deletion
    let deleted = db.get_label_by_id(id.as_str())?;
    assert!(deleted.is_none());

    // Verify orphaned content
    let orphaned_content = db.get_content_by_id(cid.as_str())?;
    assert!(orphaned_content.is_some());
    let orphaned = orphaned_content.clone().unwrap();
    assert!(orphaned.LabelID.is_none());

    Ok(())
}

// -- MyTag ------------------------------------------------------------------------------------

// -- Playlist ---------------------------------------------------------------------------------

fn assert_playlist_seq(items: Vec<DjmdPlaylist>) {
    let n = items.len() as i32;
    for i in 0..n {
        let item = &items[i as usize];
        assert_eq!(item.Seq, Some(i + 1));
    }
}

fn assert_playlist_song_seq(items: Vec<DjmdSongPlaylist>) {
    let n = items.len() as i32;
    for i in 0..n {
        let item = &items[i as usize];
        assert_eq!(item.TrackNo, Some(i + 1));
    }
}

fn check_playlist_xml(db: &mut MasterDb) -> anyhow::Result<bool> {
    let xml = MasterPlaylistXml::load(db.plxml_path.clone().unwrap());
    // Check that playlist is in XML and update time is correct
    for playlist in db.get_playlist()? {
        let playlist_xml = xml.get_playlist(playlist.ID.clone());
        if let Some(playlist_xml) = playlist_xml {
            if playlist.ID != "100000" {
                let ts = playlist_xml.timestamp;
                let diff = playlist.updated_at.naive_utc() - ts;
                if diff.num_seconds().abs() > 1 {
                    // If the difference is more than 1 second, we have a mismatch
                    eprintln!(
                        "Difference in timestamps for playlist {}: {} seconds",
                        playlist.ID,
                        diff.num_seconds()
                    );
                    return Ok(false);
                }
                if playlist_xml.parent_id != playlist.ParentID.unwrap_or("0".to_string()) {
                    // Check that parent ID matches
                    eprintln!("Parent ID mismatch for playlist {}", playlist.ID);
                    return Ok(false);
                }
            }
        } else {
            eprintln!("Playlist {} not found in XML", playlist.ID);
            return Ok(false);
        }
    }
    // Check that there are no items in the XML that are not in the db
    for playlist_xml in xml.get_playlists() {
        let id = &playlist_xml.id;
        let item = db.get_playlist_by_id(id)?;
        if item.is_none() {
            // If the item is not in the db, we have a mismatch
            eprintln!("Playlist {} found in XML but not in DB", id);
            return Ok(false);
        }
    }
    Ok(true)
}

fn assert_playlist_xml(db: &mut MasterDb) {
    let res = check_playlist_xml(db).expect("Failed to check playlist XML");
    assert!(res);
}

#[test]
fn test_get_playlist() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let _items = db.get_playlist()?;
    Ok(())
}

#[test]
fn test_get_playlist_children() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let _items = db.get_playlist_children("root")?;
    Ok(())
}

#[test]
fn test_get_playlist_by_id() -> anyhow::Result<()> {
    let mut db = common::setup_master_db()?;
    let item = db.get_playlist_by_id("1234")?;
    assert!(item.is_none());
    Ok(())
}

#[test]
fn test_insert_playlist() -> anyhow::Result<()> {
    let _pl_xml_path = common::setup_master_playlist_xml()?;
    let mut db = common::setup_master_db()?;

    let old_usn = db.get_local_usn()?;
    // Insert
    let name = "Name".to_string();
    let attr = PlaylistType::Playlist;
    let parent_id = "root".to_string();
    let seq = None;

    let new_item =
        db.insert_playlist(name.clone(), attr, Some(parent_id.clone()), seq, None, None)?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.Name, Some(name));
    assert_eq!(new_item.ParentID, Some(parent_id.clone()));
    assert_eq!(new_usn, old_usn + 2);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item_opt = db.get_playlist_by_id(new_item.ID.as_str())?;
    assert!(item_opt.is_some());
    let item = item_opt.unwrap();

    // Verify seq number
    let items = db.get_playlist_children(parent_id.as_str())?;
    let n = items.len() as i32;
    assert_eq!(item.Seq, Some(n));

    assert_playlist_seq(items);

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
fn test_insert_playlist_seq() -> anyhow::Result<()> {
    let _pl_xml_path = common::setup_master_playlist_xml()?;
    let mut db = common::setup_master_db()?;

    // Insert root
    let name = "Name".to_string();
    let attr = PlaylistType::Playlist;
    let parent_id = "root".to_string();
    let seq = Some(1);

    let new_item =
        db.insert_playlist(name.clone(), attr, Some(parent_id.clone()), seq, None, None)?;

    // Verify the insertion
    let item = db.get_playlist_by_id(new_item.ID.as_str())?;
    assert!(item.is_some());

    // Verify seq number
    let items = db.get_playlist_children(parent_id.as_str())?;
    item.unwrap().Seq = seq;

    assert_playlist_seq(items);

    Ok(())
}

#[test]
fn test_insert_playlist_folder() -> anyhow::Result<()> {
    let _pl_xml_path = common::setup_master_playlist_xml()?;
    let mut db = common::setup_master_db()?;

    let old_usn = db.get_local_usn()?;
    // Insert
    let name = "Name".to_string();
    let attr = PlaylistType::Folder;
    let parent_id = "root".to_string();
    let seq = None;

    let new_item =
        db.insert_playlist(name.clone(), attr, Some(parent_id.clone()), seq, None, None)?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.Name, Some(name));
    assert_eq!(new_item.ParentID, Some(parent_id.clone()));
    assert_eq!(new_usn, old_usn + 2);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item_opt = db.get_playlist_by_id(new_item.ID.as_str())?;
    assert!(item_opt.is_some());
    let item = item_opt.unwrap();

    // Verify seq number
    let items = db.get_playlist_children(parent_id.as_str())?;
    let n = items.len() as i32;
    assert_eq!(item.Seq, Some(n));

    assert_playlist_seq(items);

    // Check playlist XML
    assert_playlist_xml(&mut db);

    // Try adding a sub-playlist
    let sub_name = "Name".to_string();
    let sub_parent_id = Some(new_item.clone().ID);

    let sub_item = db.insert_playlist(
        sub_name.clone(),
        PlaylistType::Playlist,
        sub_parent_id,
        None,
        None,
        None,
    )?;
    assert_eq!(sub_item.ParentID, Some(new_item.ID));
    assert_eq!(sub_item.Seq, Some(1));

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
fn test_insert_playlist_song() -> anyhow::Result<()> {
    common::setup_master_playlist_xml()?;
    let mut db = common::setup_master_db()?;

    // Create playlist structure
    let pl = db.create_playlist("Playlist".to_string(), None, None, None, None)?;

    // Add a song to the playlist
    let contents = db.get_content()?;
    let cid1 = contents[0].ID.clone();
    let cid2 = contents[1].ID.clone();
    let cid3 = contents[2].ID.clone();
    let cid4 = contents[3].ID.clone();

    // Insert song 1
    let song = db.insert_playlist_song(&pl.ID, &cid1, None)?;
    assert_eq!(song.PlaylistID, Some(pl.ID.clone()));
    assert_eq!(song.ContentID, Some(cid1.clone()));
    assert_eq!(song.TrackNo, Some(1));

    // Insert song 2
    let song = db.insert_playlist_song(&pl.ID, &cid2, None)?;
    assert_eq!(song.PlaylistID, Some(pl.ID.clone()));
    assert_eq!(song.ContentID, Some(cid2.clone()));
    assert_eq!(song.TrackNo, Some(2));

    // Insert song 3 at position 2
    let song = db.insert_playlist_song(&pl.ID, &cid3, Some(2))?;
    assert_eq!(song.PlaylistID, Some(pl.ID.clone()));
    assert_eq!(song.ContentID, Some(cid3.clone()));
    let songs = db.get_playlist_songs(&pl.ID)?;
    let content_ids: Vec<String> = songs.iter().map(|s| s.ContentID.clone().unwrap()).collect();
    assert_eq!(content_ids, vec![cid1, cid3, cid2]);

    assert_playlist_xml(&mut db);

    // Check error on invalid seq
    let res = db.insert_playlist_song(&pl.ID, &cid4, Some(0));
    assert!(res.is_err(), "Expected error on invalid seq");

    let res = db.insert_playlist_song(&pl.ID, &cid4, Some(4));
    assert!(res.is_err(), "Expected error on invalid seq");

    Ok(())
}

#[test]
fn test_delete_playlist() -> anyhow::Result<()> {
    common::setup_master_playlist_xml()?;
    let mut db = common::setup_master_db()?;

    // Create playlist structure
    let folder = db.create_playlist_folder("Folder".to_string(), None, None)?;
    let pid = Some(folder.ID.clone());
    let _pl1 = db.create_playlist("Sub 1".to_string(), pid.clone(), None, None, None)?;
    let pl2 = db.create_playlist("Sub 2".to_string(), pid.clone(), None, None, None)?;
    let pl3 = db.create_playlist("Sub 3".to_string(), pid.clone(), None, None, None)?;
    // Add a song to pl3
    let contents = db.get_content()?;
    let cid1 = contents[0].ID.clone();
    let cid2 = contents[1].ID.clone();
    let cid3 = contents[2].ID.clone();
    let song1 = db.insert_playlist_song(&pl3.ID, &cid1, None)?;
    let song2 = db.insert_playlist_song(&pl3.ID, &cid2, None)?;
    let song3 = db.insert_playlist_song(&pl3.ID, &cid3, None)?;

    let old_usn = db.get_local_usn()?;

    // Delete playlist
    let deleted = db.delete_playlist(&pl2.ID)?;
    assert_eq!(deleted, 1);

    // Check that playlist was deleted
    let res = db.get_playlist_by_id(&pl2.ID)?;
    assert!(res.is_none());

    // Check USN is correct (+1 for deleting)
    let new_usn = db.get_local_usn()?;
    assert_eq!(new_usn, old_usn + 1);

    let playlists = db.get_playlist_children(folder.ID.as_str())?;
    assert_playlist_seq(playlists);

    assert_playlist_xml(&mut db);

    // Check that songs in pl3 are also deleted
    let deleted = db.delete_playlist(&pl3.ID)?;
    assert_eq!(deleted, 4);
    let res = db.get_playlist_song_by_id(&song1.ID)?;
    assert!(res.is_none());
    let res = db.get_playlist_song_by_id(&song2.ID)?;
    assert!(res.is_none());
    let res = db.get_playlist_song_by_id(&song3.ID)?;
    assert!(res.is_none());

    Ok(())
}

#[test]
fn test_delete_playlist_folder() -> anyhow::Result<()> {
    common::setup_master_playlist_xml()?;
    let mut db = common::setup_master_db()?;

    // Create playlist structure
    let folder = db.create_playlist_folder("Folder".to_string(), None, None)?;
    let pid = Some(folder.ID.clone());
    let pl1 = db.create_playlist("Sub 1".to_string(), pid.clone(), None, None, None)?;
    let pl2 = db.create_playlist("Sub 2".to_string(), pid.clone(), None, None, None)?;
    let pl3 = db.create_playlist("Sub 3".to_string(), pid.clone(), None, None, None)?;
    // Add a song to pl3
    let contents = db.get_content()?;
    let cid1 = contents[0].ID.clone();
    let cid2 = contents[1].ID.clone();
    let cid3 = contents[2].ID.clone();
    let song1 = db.insert_playlist_song(&pl3.ID, &cid1, None)?;
    let song2 = db.insert_playlist_song(&pl3.ID, &cid2, None)?;
    let song3 = db.insert_playlist_song(&pl3.ID, &cid3, None)?;

    let old_usn = db.get_local_usn()?;

    let deleted = db.delete_playlist(&folder.ID)?;
    assert_eq!(deleted, 7);

    // Check USN
    let new_usn = db.get_local_usn()?;
    assert_eq!(new_usn, old_usn + 4);

    // Check that child playlists were deleted
    let res1 = db.get_playlist_by_id(&pl1.ID)?;
    assert!(res1.is_none());
    let res2 = db.get_playlist_by_id(&pl2.ID)?;
    assert!(res2.is_none());
    let res3 = db.get_playlist_by_id(&pl3.ID)?;
    assert!(res3.is_none());

    assert_playlist_xml(&mut db);

    // Check that songs in pl3 are also deleted
    let res = db.get_playlist_song_by_id(&song1.ID)?;
    assert!(res.is_none());
    let res = db.get_playlist_song_by_id(&song2.ID)?;
    assert!(res.is_none());
    let res = db.get_playlist_song_by_id(&song3.ID)?;
    assert!(res.is_none());

    Ok(())
}

#[test]
pub fn test_delete_playlist_song() -> anyhow::Result<()> {
    common::setup_master_playlist_xml()?;
    let mut db = common::setup_master_db()?;

    // Create playlist structure
    let pl = db.create_playlist("Playlist".to_string(), None, None, None, None)?;

    // Add a songs to the playlist
    let contents = db.get_content()?;
    let cid1 = contents[0].ID.clone();
    let cid2 = contents[1].ID.clone();
    let cid3 = contents[2].ID.clone();
    let song1 = db.insert_playlist_song(&pl.ID, &cid1, None)?;
    let song2 = db.insert_playlist_song(&pl.ID, &cid2, None)?;
    let song3 = db.insert_playlist_song(&pl.ID, &cid3, None)?;

    let old_usn = db.get_local_usn()?;
    // Delete one song
    let deleted = db.delete_playlist_song(&song2.ID)?;
    assert_eq!(deleted, 1);

    // Check that the song was deleted
    let res = db.get_playlist_song_by_id(&song2.ID)?;
    assert!(res.is_none());

    // Check that the other songs are still there
    let res = db.get_playlist_song_by_id(&song1.ID)?;
    assert!(res.is_some());
    let res = db.get_playlist_song_by_id(&song3.ID)?;
    assert!(res.is_some());

    // Check USN is correct (+1 for deleting)
    let new_usn = db.get_local_usn()?;
    assert_eq!(new_usn, old_usn + 1);

    // Check sequence of remaining songs
    let songs = db.get_playlist_songs(&pl.ID)?;
    assert_playlist_song_seq(songs);

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
pub fn test_move_playlist_seq() -> anyhow::Result<()> {
    common::setup_master_playlist_xml()?;
    let mut db = common::setup_master_db()?;

    // Create playlist structure
    let root = db.create_playlist_folder("folder".into(), None, None)?;
    let f1 = db.create_playlist_folder("f1".into(), Some(root.ID.clone()), None)?;
    let _f2 = db.create_playlist_folder("f2".into(), Some(root.ID.clone()), None)?;
    let p1 = db.create_playlist("pl1".into(), Some(root.ID.clone()), None, None, None)?;
    let p2 = db.create_playlist("pl2".into(), Some(root.ID.clone()), None, None, None)?;
    let p3 = db.create_playlist("pl3".into(), Some(root.ID.clone()), None, None, None)?;
    let _p4 = db.create_playlist("pl4".into(), Some(root.ID.clone()), None, None, None)?;
    let _p5 = db.create_playlist("subpl1".into(), Some(f1.ID.clone()), None, None, None)?;
    let _p6 = db.create_playlist("subpl2".into(), Some(f1.ID.clone()), None, None, None)?;
    let _p7 = db.create_playlist("subpl3".into(), Some(f1.ID.clone()), None, None, None)?;
    let _p8 = db.create_playlist("subpl4".into(), Some(f1.ID.clone()), None, None, None)?;

    let children = db.get_playlist_children(&root.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl1", "pl2", "pl3", "pl4"]);

    // Move playlist 1 down to position 5
    db.move_playlist(&p1.ID, Some(5), None)?;
    let children = db.get_playlist_children(&root.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl2", "pl3", "pl1", "pl4"]);
    assert_playlist_seq(children);

    // Move playlist 2 down to position 6 (end)
    db.move_playlist(&p2.ID, Some(6), None)?;
    let children = db.get_playlist_children(&root.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl3", "pl1", "pl4", "pl2"]);
    assert_playlist_seq(children);

    // Move playlist 3 up to position 2
    db.move_playlist(&p3.ID, Some(2), None)?;
    let children = db.get_playlist_children(&root.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["f1", "pl3", "f2", "pl1", "pl4", "pl2"]);
    assert_playlist_seq(children);

    // Move playlist 1 up to position 1 (start)
    db.move_playlist(&p1.ID, Some(1), None)?;
    let children = db.get_playlist_children(&root.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["pl1", "f1", "pl3", "f2", "pl4", "pl2"]);
    assert_playlist_seq(children);

    // Check error on invalid seq
    let res = db.move_playlist(&p2.ID, Some(0), None);
    assert!(res.is_err());

    let res = db.move_playlist(&p2.ID, Some(7), None);
    assert!(res.is_err());

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
pub fn test_move_playlist_parent() -> anyhow::Result<()> {
    common::setup_master_playlist_xml()?;
    let mut db = common::setup_master_db()?;

    // Create playlist structure
    let root = db.create_playlist_folder("folder".into(), None, None)?;
    let f1 = db.create_playlist_folder("f1".into(), Some(root.ID.clone()), None)?;
    let _f2 = db.create_playlist_folder("f2".into(), Some(root.ID.clone()), None)?;
    let p1 = db.create_playlist("pl1".into(), Some(root.ID.clone()), None, None, None)?;
    let p2 = db.create_playlist("pl2".into(), Some(root.ID.clone()), None, None, None)?;
    let p3 = db.create_playlist("pl3".into(), Some(root.ID.clone()), None, None, None)?;
    let p4 = db.create_playlist("pl4".into(), Some(root.ID.clone()), None, None, None)?;
    let _p5 = db.create_playlist("subpl1".into(), Some(f1.ID.clone()), None, None, None)?;
    let _p6 = db.create_playlist("subpl2".into(), Some(f1.ID.clone()), None, None, None)?;
    let _p7 = db.create_playlist("subpl3".into(), Some(f1.ID.clone()), None, None, None)?;
    let _p8 = db.create_playlist("subpl4".into(), Some(f1.ID.clone()), None, None, None)?;

    let children = db.get_playlist_children(&root.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl1", "pl2", "pl3", "pl4"]);

    let children = db.get_playlist_children(&f1.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["subpl1", "subpl2", "subpl3", "subpl4"]);

    // Move playlist 1 to end of f1
    db.move_playlist(&p1.ID, None, Some(f1.ID.clone()))?;
    let children = db.get_playlist_children(&root.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl2", "pl3", "pl4"]);
    assert_playlist_seq(children);
    let children = db.get_playlist_children(&f1.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["subpl1", "subpl2", "subpl3", "subpl4", "pl1"]);
    assert_playlist_seq(children);

    // Move playlist 2 to explicit seq (3) of f1
    db.move_playlist(&p2.ID, Some(3), Some(f1.ID.clone()))?;
    let children = db.get_playlist_children(&root.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl3", "pl4"]);
    assert_playlist_seq(children);
    let children = db.get_playlist_children(&f1.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(
        names,
        vec!["subpl1", "subpl2", "pl2", "subpl3", "subpl4", "pl1"]
    );
    assert_playlist_seq(children);

    // Move playlist 3 to start of f1
    db.move_playlist(&p3.ID, Some(1), Some(f1.ID.clone()))?;
    let children = db.get_playlist_children(&root.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl4"]);
    assert_playlist_seq(children);
    let children = db.get_playlist_children(&f1.ID)?;
    let names: Vec<String> = children.iter().map(|p| p.Name.clone().unwrap()).collect();
    assert_eq!(
        names,
        vec!["pl3", "subpl1", "subpl2", "pl2", "subpl3", "subpl4", "pl1"]
    );
    assert_playlist_seq(children);

    // Check error on invalid parent and/or seq
    let res = db.move_playlist(&p4.ID, None, Some("1234".into()));
    assert!(res.is_err());

    let res = db.move_playlist(&p4.ID, Some(0), Some(f1.ID.clone()));
    assert!(res.is_err());

    let res = db.move_playlist(&p4.ID, Some(8), Some(f1.ID.clone()));
    assert!(res.is_err());

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
pub fn test_move_playlist_song() -> anyhow::Result<()> {
    common::setup_master_playlist_xml()?;
    let mut db = common::setup_master_db()?;

    // Create playlist structure
    let pl = db.create_playlist("Playlist".to_string(), None, None, None, None)?;
    // Add a songs to the playlist
    let contents = db.get_content()?;
    let cid1 = contents[0].ID.clone();
    let cid2 = contents[1].ID.clone();
    let cid3 = contents[2].ID.clone();
    let cid4 = contents[3].ID.clone();
    let cid5 = contents[4].ID.clone();
    let song1 = db.insert_playlist_song(&pl.ID, &cid1, None)?;
    let song2 = db.insert_playlist_song(&pl.ID, &cid2, None)?;
    let song3 = db.insert_playlist_song(&pl.ID, &cid3, None)?;
    let song4 = db.insert_playlist_song(&pl.ID, &cid4, None)?;
    let song5 = db.insert_playlist_song(&pl.ID, &cid5, None)?;
    let sid1 = song1.ID.clone();
    let sid2 = song2.ID.clone();
    let sid3 = song3.ID.clone();
    let sid4 = song4.ID.clone();
    let sid5 = song5.ID.clone();
    let songs = db.get_playlist_songs(&pl.ID)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.ID.clone()).collect();
    assert_eq!(
        song_ids,
        vec![
            sid1.clone(),
            sid2.clone(),
            sid3.clone(),
            sid4.clone(),
            sid5.clone()
        ]
    );
    assert_playlist_song_seq(songs);

    // Move song 4 to position 2
    db.move_playlist_song(&song4.ID, 2)?;
    let songs = db.get_playlist_songs(&pl.ID)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.ID.clone()).collect();
    assert_eq!(
        song_ids,
        vec![
            sid1.clone(),
            sid4.clone(),
            sid2.clone(),
            sid3.clone(),
            sid5.clone()
        ]
    );
    assert_playlist_song_seq(songs);

    // Move song 4 to position 4
    db.move_playlist_song(&song4.ID, 4)?;
    let songs = db.get_playlist_songs(&pl.ID)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.ID.clone()).collect();
    assert_eq!(
        song_ids,
        vec![
            sid1.clone(),
            sid2.clone(),
            sid3.clone(),
            sid4.clone(),
            sid5.clone()
        ]
    );
    assert_playlist_song_seq(songs);

    // Move song 2 to position 3
    db.move_playlist_song(&song2.ID, 3)?;
    let songs = db.get_playlist_songs(&pl.ID)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.ID.clone()).collect();
    assert_eq!(
        song_ids,
        vec![
            sid1.clone(),
            sid3.clone(),
            sid2.clone(),
            sid4.clone(),
            sid5.clone()
        ]
    );
    assert_playlist_song_seq(songs);

    // Move song 4 to position 1
    db.move_playlist_song(&song4.ID, 1)?;
    let songs = db.get_playlist_songs(&pl.ID)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.ID.clone()).collect();
    assert_eq!(
        song_ids,
        vec![
            sid4.clone(),
            sid1.clone(),
            sid3.clone(),
            sid2.clone(),
            sid5.clone()
        ]
    );
    assert_playlist_song_seq(songs);

    // Move song 1 to position 5
    db.move_playlist_song(&song1.ID, 5)?;
    let songs = db.get_playlist_songs(&pl.ID)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.ID.clone()).collect();
    assert_eq!(
        song_ids,
        vec![
            sid4.clone(),
            sid3.clone(),
            sid2.clone(),
            sid5.clone(),
            sid1.clone()
        ]
    );
    assert_playlist_song_seq(songs);

    // Check error on invalid seq
    let res = db.move_playlist_song(&song2.ID, 0);
    assert!(res.is_err());
    let res = db.move_playlist_song(&song2.ID, 6);
    assert!(res.is_err());

    assert_playlist_xml(&mut db);

    Ok(())
}
