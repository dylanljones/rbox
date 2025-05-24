// Author: Dylan Jones
// Date:   2025-05-06

use rbox::masterdb::{DjmdPlaylist, MasterPlaylistXml, PlaylistType};

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

fn assert_playlist_seq(items: Vec<DjmdPlaylist>) {
    let n = items.len() as i32;
    for i in 0..n {
        let item = &items[i as usize];
        assert_eq!(item.Seq, Some(i + 1));
    }
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

    // Check playlist XML
    let plxml = MasterPlaylistXml::load(db.plxml_path.unwrap().clone());
    let xml_item_opt = plxml.get_playlist(item.ID);
    assert!(xml_item_opt.is_some());
    let _xml_item = xml_item_opt.unwrap();

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
    let pl_xml_path = common::setup_master_playlist_xml()?;
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
    let plxml = MasterPlaylistXml::load(pl_xml_path.clone());
    let xml_item_opt = plxml.get_playlist(item.ID);
    assert!(xml_item_opt.is_some());
    let _xml_item = xml_item_opt.unwrap();

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
    Ok(())
}
