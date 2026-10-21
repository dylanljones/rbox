// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Master database integration tests using shared fixture copies.

use diesel::connection::InstrumentationEvent;
use diesel::prelude::*;
use diesel::{Connection, RunQueryDsl};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use rbox_db::{validate_seq_numbers, Model, ModelInsert, ModelRelations, TreeSeq};
use rbox_masterdb::schema::*;
use rbox_masterdb::*;
use rbox_testing::data::{create_temp_dir, setup_master_db_dir_path, setup_master_db_path};

const SECONDS_THRESH: i64 = 10;

fn assert_no_content_relations(view: &DjmdContentView) {
    assert!(view.artist.is_none());
    assert!(view.remixer.is_none());
    assert!(view.original_artist.is_none());
    assert!(view.composer.is_none());
    assert!(view.album.is_none());
    assert!(view.album_artist.is_none());
    assert!(view.genre.is_none());
    assert!(view.label.is_none());
    assert!(view.key.is_none());
    assert!(view.color.is_none());
}

// -- Resolved album relationships --------------------------------------------------------------

#[rstest::rstest]
#[case(6)]
#[case(7)]
fn test_album_relationships(#[case] version: usize) -> anyhow::Result<()> {
    let fixture = setup_master_db_path(version);
    let mut db = MasterDb::new(fixture.path())?;
    let mut conn = db.pool.get()?;
    let artist = NewDjmdArtist::new("album artist").insert(&mut conn)?;

    let mut new = NewDjmdAlbum::new("resolved album");
    new.album_artist_id = Some(artist.id.clone());

    let album = new.insert(&mut conn)?;
    let expected = DjmdAlbumView {
        album: album.clone(),
        artist: Some(artist.clone()),
    };
    assert_eq!(album.clone().resolve(&mut conn)?, expected);
    let mut absent = album.clone();
    absent.album_artist_id = None;

    let absent_view = DjmdAlbumView {
        album: absent.clone(),
        artist: None,
    };
    assert_eq!(absent.clone().resolve(&mut conn)?, absent_view);
    let mut dangling = absent.clone();
    dangling.album_artist_id = Some("missing-artist".to_owned());

    let dangling_view = DjmdAlbumView {
        album: dangling.clone(),
        artist: None,
    };
    assert_eq!(dangling.clone().resolve(&mut conn)?, dangling_view);

    let inputs = vec![dangling, album.clone(), absent, album.clone()];
    assert_eq!(
        DjmdAlbum::resolve_many(&mut conn, inputs)?,
        vec![
            dangling_view,
            expected.clone(),
            absent_view,
            expected.clone()
        ]
    );
    assert_eq!(DjmdAlbum::find(&mut conn, &album.id)?, Some(album.clone()));
    assert_eq!(db.get_album_by_id(&album.id)?, Some(album.clone()));
    assert_eq!(db.get_album_with_relations(&album.id)?, Some(expected));
    assert!(db.get_album_with_relations("missing-album")?.is_none());
    let raw = db.get_albums()?;
    let views = db.get_albums_with_relations()?;
    assert_eq!(
        views.iter().map(|v| v.album.clone()).collect::<Vec<_>>(),
        raw
    );
    assert_eq!(views, DjmdAlbum::resolve_many(&mut conn, raw)?);
    Ok(())
}

#[rstest::rstest]
#[case(6)]
#[case(7)]
fn test_album_relationship_batches(#[case] version: usize) -> anyhow::Result<()> {
    let fixture = setup_master_db_path(version);
    let db = MasterDb::new(fixture.path())?;
    let mut conn = db.pool.get()?;
    let album = NewDjmdAlbum::new("batch album").insert(&mut conn)?;
    let mut rows = Vec::new();
    for index in 0..501 {
        let artist = NewDjmdArtist::new(format!("album artist {index}")).insert(&mut conn)?;

        let mut row = album.clone();
        row.album_artist_id = Some(artist.id);

        rows.push(row);
    }
    let queries = Arc::new(AtomicUsize::new(0));
    let observed = queries.clone();
    conn.set_instrumentation(move |event: InstrumentationEvent<'_>| {
        if let InstrumentationEvent::StartQuery { query, .. } = event {
            if query.to_string().trim_start().starts_with("SELECT") {
                observed.fetch_add(1, Ordering::Relaxed);
            }
        }
    });
    assert!(DjmdAlbum::resolve_many(&mut conn, vec![])?.is_empty());
    let empty = album.resolve(&mut conn)?;
    assert!(empty.artist.is_none());

    assert_eq!(queries.swap(0, Ordering::Relaxed), 0);
    let repeated = DjmdAlbum::resolve_many(&mut conn, vec![rows[0].clone(); 501])?;
    assert_eq!(repeated.len(), 501);
    assert!(repeated.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(queries.swap(0, Ordering::Relaxed), 1);
    for size in [1, 500, 501] {
        let views = DjmdAlbum::resolve_many(&mut conn, rows[..size].to_vec())?;
        assert_eq!(views.len(), size);
        for (view, row) in views.iter().zip(&rows) {
            assert_eq!(&view.album, row);
            assert_eq!(
                view.artist.as_ref().map(|a| &a.id),
                row.album_artist_id.as_ref()
            );
        }
        assert_eq!(
            queries.swap(0, Ordering::Relaxed),
            if size > 500 { 2 } else { 1 }
        );
    }

    diesel::sql_query("ALTER TABLE djmdArtist RENAME TO relationship_test_artist")
        .execute(&mut conn)?;
    assert!(rows.remove(0).resolve(&mut conn).is_err());
    Ok(())
}

// -- Resolved track relationships --------------------------------------------------------------

#[rstest::rstest]
#[case(6)]
#[case(7)]
fn test_content_relationships(#[case] version: usize) -> anyhow::Result<()> {
    let fixture = setup_master_db_path(version);
    let mut db = MasterDb::new(fixture.path())?;
    let mut conn = db.pool.get()?;
    let original = DjmdContent::all(&mut conn)?
        .into_iter()
        .next()
        .expect("fixture track");
    let mut track = original.clone();
    let artist = NewDjmdArtist::new("relation artist").insert(&mut conn)?;
    track.artist_id = Some(artist.id.clone());
    let remixer = NewDjmdArtist::new("relation remixer").insert(&mut conn)?;
    track.remixer_id = Some(remixer.id.clone());
    let original_artist = NewDjmdArtist::new("relation original_artist").insert(&mut conn)?;
    track.org_artist_id = Some(original_artist.id.clone());
    let composer = NewDjmdArtist::new("relation composer").insert(&mut conn)?;
    track.composer_id = Some(composer.id.clone());
    let album_artist = NewDjmdArtist::new("relation album artist").insert(&mut conn)?;
    let album = NewDjmdAlbum {
        album_artist_id: Some(album_artist.id.clone()),
        ..NewDjmdAlbum::new("relationship album")
    }
    .insert(&mut conn)?;
    track.album_id = Some(album.id.clone());
    let genre = NewDjmdGenre::new("relationship genre").insert(&mut conn)?;
    track.genre_id = Some(genre.id.clone());
    let label = NewDjmdLabel::new("relationship label").insert(&mut conn)?;
    track.label_id = Some(label.id.clone());
    let key = NewDjmdKey::new("relationship key").insert(&mut conn)?;
    track.key_id = Some(key.id.clone());
    let color = DjmdColor::all(&mut conn)?
        .into_iter()
        .next()
        .expect("fixture color");
    track.color_id = Some(color.id.clone());
    let view = track.clone().resolve(&mut conn)?;
    assert_eq!(view.content, track);
    assert_eq!(view.artist, Some(artist.clone()));
    assert_eq!(view.remixer, Some(remixer.clone()));
    assert_eq!(view.original_artist, Some(original_artist.clone()));
    assert_eq!(view.composer, Some(composer.clone()));
    assert_eq!(view.album, Some(album));
    assert_eq!(view.album_artist, Some(album_artist));
    assert_eq!(view.genre, Some(genre));
    assert_eq!(view.label, Some(label));
    assert_eq!(view.key, Some(key));
    assert_eq!(view.color, Some(color));
    let mut shared = track.clone();
    shared.artist_id = Some(artist.id.clone());
    shared.remixer_id = Some(artist.id.clone());
    shared.org_artist_id = Some(artist.id.clone());
    shared.composer_id = Some(artist.id.clone());
    let shared = shared.resolve(&mut conn)?;
    assert_eq!(shared.artist, Some(artist.clone()));
    assert_eq!(shared.remixer, Some(artist.clone()));
    assert_eq!(shared.original_artist, Some(artist.clone()));
    assert_eq!(shared.composer, Some(artist.clone()));
    // An existing album may have an absent or dangling artist reference.
    for artist_id in [None, Some("missing-album-artist".to_owned())] {
        let album = NewDjmdAlbum {
            album_artist_id: artist_id,
            ..NewDjmdAlbum::new("optional album artist")
        }
        .insert(&mut conn)?;
        let mut optional = track.clone();
        optional.album_id = Some(album.id);
        let view = optional.resolve(&mut conn)?;
        assert!(view.album.is_some());
        assert!(view.album_artist.is_none());
    }
    let mut absent = track.clone();
    absent.artist_id = None;
    absent.remixer_id = None;
    absent.org_artist_id = None;
    absent.composer_id = None;
    absent.album_id = None;
    absent.genre_id = None;
    absent.label_id = None;
    absent.key_id = None;
    absent.color_id = None;
    assert_no_content_relations(&absent.clone().resolve(&mut conn)?);
    let mut dangling = absent.clone();
    dangling.artist_id = Some("missing-relation".to_owned());
    dangling.remixer_id = Some("missing-relation".to_owned());
    dangling.org_artist_id = Some("missing-relation".to_owned());
    dangling.composer_id = Some("missing-relation".to_owned());
    dangling.album_id = Some("missing-relation".to_owned());
    dangling.genre_id = Some("missing-relation".to_owned());
    dangling.label_id = Some("missing-relation".to_owned());
    dangling.key_id = Some("missing-relation".to_owned());
    dangling.color_id = Some("missing-relation".to_owned());
    let dangling_view = dangling.clone().resolve(&mut conn)?;
    assert_no_content_relations(&dangling_view);
    assert_eq!(dangling_view.content, dangling);
    let inputs = vec![dangling, track.clone(), absent, track];
    let expected = inputs
        .iter()
        .cloned()
        .map(|row| row.resolve(&mut conn))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(DjmdContent::resolve_many(&mut conn, inputs)?, expected);

    // Loading supplied snapshots must not update or reload the stored track.
    assert_eq!(
        DjmdContent::find(&mut conn, &original.id)?.unwrap(),
        original
    );
    drop(conn);
    assert_eq!(db.get_content_by_id(&original.id)?.unwrap(), original);
    let resolved = db.get_content_with_relations(&original.id)?.unwrap();
    let mut conn = db.pool.get()?;
    assert_eq!(resolved, original.resolve(&mut conn)?);
    drop(conn);
    assert!(db.get_content_with_relations("missing-content")?.is_none());
    let raw = db.get_contents()?;
    let views = db.get_contents_with_relations()?;
    assert_eq!(
        views.iter().map(|v| v.content.clone()).collect::<Vec<_>>(),
        raw
    );
    let mut conn = db.pool.get()?;
    assert_eq!(views, DjmdContent::resolve_many(&mut conn, raw)?);
    Ok(())
}

#[rstest::rstest]
#[case(6)]
#[case(7)]
fn test_content_relationship_batches(#[case] version: usize) -> anyhow::Result<()> {
    let fixture = setup_master_db_path(version);
    let db = MasterDb::new(fixture.path())?;
    let mut conn = db.pool.get()?;
    let mut track = DjmdContent::all(&mut conn)?
        .into_iter()
        .next()
        .expect("fixture track");
    track.artist_id = None;
    track.remixer_id = None;
    track.org_artist_id = None;
    track.composer_id = None;
    track.album_id = None;
    track.genre_id = None;
    track.label_id = None;
    track.key_id = None;
    track.color_id = None;
    let mut rows = Vec::new();
    for index in 0..501 {
        let artist = NewDjmdArtist::new(format!("batch artist {index}")).insert(&mut conn)?;
        let mut row = track.clone();
        row.artist_id = Some(artist.id.clone());
        row.remixer_id = Some(artist.id.clone());
        row.org_artist_id = Some(artist.id.clone());
        row.composer_id = Some(artist.id.clone());
        rows.push(row);
    }
    let queries = Arc::new(AtomicUsize::new(0));
    let observed = queries.clone();
    conn.set_instrumentation(move |event: InstrumentationEvent<'_>| {
        if let InstrumentationEvent::StartQuery { query, .. } = event {
            if query.to_string().trim_start().starts_with("SELECT") {
                observed.fetch_add(1, Ordering::Relaxed);
            }
        }
    });
    assert!(DjmdContent::resolve_many(&mut conn, vec![])?.is_empty());
    assert_eq!(queries.load(Ordering::Relaxed), 0);
    assert_no_content_relations(&track.resolve(&mut conn)?);
    assert_eq!(queries.load(Ordering::Relaxed), 0);
    let repeated = DjmdContent::resolve_many(&mut conn, vec![rows[0].clone(); 501])?;
    assert_eq!(queries.swap(0, Ordering::Relaxed), 1);
    assert_eq!(repeated.len(), 501);
    for size in [1, 500, 501] {
        let views = DjmdContent::resolve_many(&mut conn, rows[..size].to_vec())?;
        assert_eq!(views.len(), size);
        for (view, row) in views.iter().zip(&rows) {
            assert_eq!(&view.content, row);
            assert_eq!(view.artist.as_ref().map(|a| &a.id), row.artist_id.as_ref());
            assert_eq!(view.remixer.as_ref().map(|a| &a.id), row.artist_id.as_ref());
            assert_eq!(
                view.original_artist.as_ref().map(|a| &a.id),
                row.artist_id.as_ref()
            );
            assert_eq!(
                view.composer.as_ref().map(|a| &a.id),
                row.artist_id.as_ref()
            );
        }
        assert_eq!(
            queries.swap(0, Ordering::Relaxed),
            if size > 500 { 2 } else { 1 }
        );
    }
    // A failed lookup is an error, not an absent relationship.
    diesel::sql_query("ALTER TABLE djmdArtist RENAME TO relationship_test_artist")
        .execute(&mut conn)?;
    assert!(rows.remove(0).resolve(&mut conn).is_err());
    Ok(())
}

#[test]
fn test_open_master_db() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let _db = MasterDb::new(db_path.path())?;
    Ok(())
}

// -- AgentRegistry --------------------------------------------------------------------------------

#[test]
fn test_get_local_usn() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let usn = db.get_local_usn()?;
    assert!(usn > 0);
    Ok(())
}

// -- Album ----------------------------------------------------------------------------------------

#[test]
fn test_get_albums() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let _album = db.get_albums()?;
    Ok(())
}

#[test]
fn test_get_album_by_id() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let album = db.get_album_by_id("1234")?;
    assert!(album.is_none());
    Ok(())
}

#[test]
fn test_get_album_by_name() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let name = "Name".to_string();

    let item = db.get_album_by_name("Name")?;
    assert!(item.is_none());

    db.create_album(name, None, None, None)?;

    let item = db.get_album_by_name("Name")?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_create_album() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    let old_usn = db.get_local_usn()?;
    // Insert a new album
    let name = "New Album".to_string();
    let artist = None;
    let image_path = None;
    let compilation = None;
    let new_album = db.create_album(name.clone(), artist, image_path, compilation)?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_album.name, name);
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(new_album.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let album = db.get_album_by_id(new_album.id.as_str())?;
    assert!(album.is_some());

    Ok(())
}

#[test]
fn test_update_album() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    // Insert a new album
    let mut album = db.create_album("New Album".to_string(), None, None, None)?;
    let old_usn = db.get_local_usn()?;

    // Update the album
    let id = album.id.clone();
    let new_name = "Updated Album".to_string();
    album.name = new_name.clone();
    let updated = db.update_album(&mut album);
    let new_usn = db.get_local_usn()?;
    assert!(updated.is_ok());
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(updated?.rb_local_usn.unwrap(), new_usn);

    // Verify the update
    let updated_album = db.get_album_by_id(id.as_str())?;
    assert!(updated_album.is_some());
    assert_eq!(updated_album.unwrap().name, new_name);

    Ok(())
}

#[test]
fn test_delete_album() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    // Insert a new album
    let album = db.create_album("New Album".to_string(), None, None, None)?;

    // Refer to the album by its ID in content
    let contents = db.get_contents()?;
    let cid = contents[0].id.clone();
    let mut content = db
        .get_content_by_id(cid.as_str())?
        .expect("get content failed");
    content.album_id = Some(album.id.clone());
    db.update_content(&content)?;
    let linked_content = db.get_content_by_id(cid.as_str())?;
    assert_eq!(linked_content.unwrap().album_id, Some(album.id.clone()));

    // Delete the album
    let old_usn = db.get_local_usn()?;
    let id = album.id.clone();
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
    assert!(orphaned_content.unwrap().album_id.is_none());

    Ok(())
}

// -- Artist ---------------------------------------------------------------------------------------

#[test]
fn test_get_artist() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_artists()?;
    Ok(())
}

#[test]
fn test_get_artist_by_id() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let item = db.get_artist_by_id("1234")?;
    assert!(item.is_none());

    let items = db.get_artists()?;
    let artist = items[0].id.clone();
    let item = db.get_artist_by_id(artist.as_str())?;
    assert!(item.is_some());
    Ok(())
}

#[test]
fn test_get_artist_by_name() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let name = "Name".to_string();

    let item = db.get_artist_by_name("Name")?;
    assert!(item.is_none());

    db.create_artist(name)?;

    let item = db.get_artist_by_name("Name")?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_create_artist() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    let old_usn = db.get_local_usn()?;
    // Insert a new artist
    let name = "New Artist".to_string();
    let new_item = db.create_artist(name.clone())?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.name, name);
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item = db.get_artist_by_id(new_item.id.as_str())?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_update_artist() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    // Insert a new artist
    let mut item = db.create_artist("New Artist".to_string())?;
    let old_usn = db.get_local_usn()?;

    // Update the artist
    let id = item.id.clone();
    let new_name = "Updated Artist".to_string();
    item.name = new_name.clone();
    let updated = db.update_artist(&mut item);
    let new_usn = db.get_local_usn()?;
    assert!(updated.is_ok());
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(updated?.rb_local_usn.unwrap(), new_usn);

    // Verify the update
    let updated_item = db.get_artist_by_id(id.as_str())?;
    assert!(updated_item.is_some());
    assert_eq!(updated_item.unwrap().name, new_name);

    Ok(())
}

#[test]
fn test_delete_artist() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    // Insert a new artist
    let item = db.create_artist("New Artist".to_string())?;

    // Refer to the artist by its ID in content
    let contents = db.get_contents()?;
    let cid = contents[0].id.clone();
    let mut content = db
        .get_content_by_id(cid.as_str())?
        .expect("get content failed");
    content.artist_id = Some(item.id.clone());
    content.org_artist_id = Some(item.id.clone());
    db.update_content(&content)?;
    let linked_content = db.get_content_by_id(cid.as_str())?;
    assert_eq!(linked_content.unwrap().artist_id, Some(item.id.clone()));

    // Delete the artist
    let old_usn = db.get_local_usn()?;
    let id = item.id.clone();
    let deleted = db.delete_artist(id.as_str());
    let new_usn = db.get_local_usn()?;
    assert!(deleted.is_ok(), "delete failed: {:?}", deleted);
    assert_eq!(new_usn, old_usn + 1);

    // Verify the deletion
    let deleted_album = db.get_artist_by_id(id.as_str())?;
    assert!(deleted_album.is_none());

    // Verify orphaned content
    let orphaned_content = db.get_content_by_id(cid.as_str())?;
    assert!(orphaned_content.is_some());
    let orphaned = orphaned_content.clone().unwrap();
    assert!(orphaned.artist_id.is_none());
    assert!(orphaned.org_artist_id.is_none());

    Ok(())
}

// -- Content ----------------------------------------------------------------------------------

#[test]
fn test_get_content() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_contents()?;
    Ok(())
}

#[test]
fn test_get_content_by_id() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let item = db.get_content_by_id("1234")?;
    assert!(item.is_none());

    let items = db.get_contents()?;
    let reference = items[0].id.clone();

    let item = db.get_content_by_id(reference.as_str())?;
    assert!(item.is_some());
    Ok(())
}

#[test]
fn test_get_content_by_path() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let item = db.get_content_by_path("invalid/path")?;
    assert!(item.is_none());

    let items = db.get_contents()?;
    let reference = items[0].folder_path.clone().expect("No folder path");

    let item = db.get_content_by_path(reference.as_str())?;
    assert!(item.is_some());
    Ok(())
}

// -- Genre ----------------------------------------------------------------------------------------

#[test]
fn test_get_genre() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_genres()?;
    Ok(())
}

#[test]
fn test_get_genre_by_id() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let item = db.get_genre_by_id("1234")?;
    assert!(item.is_none());
    Ok(())
}

#[test]
fn test_get_genre_by_name() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let name = "Name".to_string();

    let item = db.get_genre_by_name("Name")?;
    assert!(item.is_none());

    db.create_genre(name)?;

    let item = db.get_genre_by_name("Name")?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_create_genre() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    let old_usn = db.get_local_usn()?;
    // Insert a new artist
    let name = "New Genre".to_string();
    let new_item = db.create_genre(name.clone())?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.name, name);
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item = db.get_genre_by_id(new_item.id.as_str())?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_update_genre() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    // Insert a new artist
    let mut item = db.create_genre("New Genre".to_string())?;
    let old_usn = db.get_local_usn()?;

    // Update the artist
    let id = item.id.clone();
    let new_name = "Updated Genre".to_string();
    item.name = new_name.clone();
    let updated = db.update_genre(&mut item);
    let new_usn = db.get_local_usn()?;
    assert!(updated.is_ok());
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(updated?.rb_local_usn.unwrap(), new_usn);

    // Verify the update
    let updated_item = db.get_genre_by_id(id.as_str())?;
    assert!(updated_item.is_some());
    assert_eq!(updated_item.unwrap().name, new_name);

    Ok(())
}

#[test]
fn test_delete_genre() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    // Insert a new artist
    let item = db.create_genre("New Genre".to_string())?;

    // Refer to the artist by its ID in content
    let contents = db.get_contents()?;
    let cid = contents[0].id.clone();
    let mut content = db
        .get_content_by_id(cid.as_str())?
        .expect("get content failed");
    content.genre_id = Some(item.id.clone());
    db.update_content(&content)?;
    let linked_content = db.get_content_by_id(cid.as_str())?;
    assert_eq!(linked_content.unwrap().genre_id, Some(item.id.clone()));

    // Delete the artist
    let old_usn = db.get_local_usn()?;
    let id = item.id.clone();
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
    assert!(orphaned.genre_id.is_none());

    Ok(())
}

// -- Key ------------------------------------------------------------------------------------------

#[test]
fn test_get_key() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_keys()?;
    Ok(())
}

#[test]
fn test_get_key_by_id() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let item = db.get_key_by_id("1234")?;
    assert!(item.is_none());
    Ok(())
}

#[test]
fn test_get_key_by_name() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let name = "Name".to_string();

    let item = db.get_key_by_name("Name")?;
    assert!(item.is_none());

    db.create_key(name)?;

    let item = db.get_key_by_name("Name")?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_create_key() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    let old_usn = db.get_local_usn()?;
    // Insert a new artist
    let name = "New Key".to_string();
    let new_item = db.create_key(name.clone())?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.name, name);
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item = db.get_key_by_id(new_item.id.as_str())?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_update_key() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    // Insert a new artist
    let mut item = db.create_key("New Key".to_string())?;
    let old_usn = db.get_local_usn()?;

    // Update the artist
    let id = item.id.clone();
    let new_name = "Updated Key".to_string();
    item.name = new_name.clone();
    let updated = db.update_key(&mut item);
    let new_usn = db.get_local_usn()?;
    assert!(updated.is_ok());
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(updated?.rb_local_usn.unwrap(), new_usn);

    // Verify the update
    let updated_item = db.get_key_by_id(id.as_str())?;
    assert!(updated_item.is_some());
    assert_eq!(updated_item.unwrap().name, new_name);

    Ok(())
}

#[test]
fn test_delete_key() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    // Insert a new artist
    let item = db.create_key("New Key".to_string())?;

    // Refer to the artist by its ID in content
    let contents = db.get_contents()?;
    let cid = contents[0].id.clone();
    let mut content = db
        .get_content_by_id(cid.as_str())?
        .expect("get content failed");
    content.key_id = Some(item.id.clone());
    db.update_content(&content)?;
    let linked_content = db.get_content_by_id(cid.as_str())?;
    assert_eq!(linked_content.unwrap().key_id, Some(item.id.clone()));

    // Delete the artist
    let old_usn = db.get_local_usn()?;
    let id = item.id.clone();
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
    assert!(orphaned.key_id.is_none());

    Ok(())
}

// -- Label ----------------------------------------------------------------------------------------

#[test]
fn test_get_label() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_labels()?;
    Ok(())
}

#[test]
fn test_get_label_by_id() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let item = db.get_label_by_id("1234")?;
    assert!(item.is_none());
    Ok(())
}

#[test]
fn test_get_label_by_name() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let name = "Name".to_string();

    let item = db.get_label_by_name("Name")?;
    assert!(item.is_none());

    db.create_label(name)?;

    let item = db.get_label_by_name("Name")?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_create_label() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    let old_usn = db.get_local_usn()?;
    // Insert a new label
    let name = "New Label".to_string();
    let new_item = db.create_label(name.clone())?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.name, name);
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item = db.get_label_by_id(new_item.id.as_str())?;
    assert!(item.is_some());

    Ok(())
}

#[test]
fn test_update_label() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    // Insert a new label
    let mut item = db.create_label("New Label".to_string())?;
    let old_usn = db.get_local_usn()?;

    // Update the artist
    let id = item.id.clone();
    let new_name = "Updated Label".to_string();
    item.name = new_name.clone();
    let updated = db.update_label(&mut item);
    let new_usn = db.get_local_usn()?;
    assert!(updated.is_ok());
    assert_eq!(new_usn, old_usn + 1);
    assert_eq!(updated?.rb_local_usn.unwrap(), new_usn);

    // Verify the update
    let updated_item = db.get_label_by_id(id.as_str())?;
    assert!(updated_item.is_some());
    assert_eq!(updated_item.unwrap().name, new_name);

    Ok(())
}

#[test]
fn test_delete_label() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;

    // Insert a new artist
    let item = db.create_label("New Label".to_string())?;

    // Refer to the artist by its ID in content
    let contents = db.get_contents()?;
    let cid = contents[0].id.clone();
    let mut content = db
        .get_content_by_id(cid.as_str())?
        .expect("get content failed");
    content.label_id = Some(item.id.clone());
    db.update_content(&content)?;
    let linked_content = db.get_content_by_id(cid.as_str())?;
    assert_eq!(linked_content.unwrap().label_id, Some(item.id.clone()));

    // Delete the artist
    let old_usn = db.get_local_usn()?;
    let id = item.id.clone();
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
    assert!(orphaned.label_id.is_none());

    Ok(())
}

// -- MyTag ------------------------------------------------------------------------------------

// -- Playlist ---------------------------------------------------------------------------------

fn assert_playlist_seq(items: Vec<DjmdPlaylist>) {
    let seq_nums = items.iter().map(|item| item.seq).collect::<Vec<i32>>();
    assert!(
        validate_seq_numbers(&seq_nums, 1),
        "Seq number mismatch: {seq_nums:?}"
    );
}

fn assert_all_playlist_seqs(db: &mut MasterDb) -> anyhow::Result<()> {
    let playlists = db.get_playlists()?;
    let mut conn = db.pool.get()?;
    for playlist in playlists {
        let kind: PlaylistType = playlist.attribute.try_into().unwrap();
        if kind != PlaylistType::Folder {
            continue;
        }
        let is_ok = DjmdPlaylist::validate_seq_numbers(&mut conn, &playlist.id)?;
        assert!(
            is_ok,
            "Seq number validation failed for playlist {}",
            playlist.id
        );
    }
    Ok(())
}

fn assert_playlist_song_seq(items: Vec<DjmdSongPlaylist>) {
    let seq_nums = items.iter().map(|item| item.track_no).collect::<Vec<i32>>();
    let n = items.len() as i32;
    for i in 0..n {
        let item = &items[i as usize];
        assert_eq!(item.track_no, i + 1, "Seq number mismatch: {seq_nums:?}");
    }
}

fn check_playlist_xml(db: &mut MasterDb) -> anyhow::Result<bool> {
    let xml = MasterPlaylistXml::load(db.playlist_xml_path().unwrap());
    // Check that playlist is in XML and update time is correct
    for playlist in db.get_playlists()? {
        let playlist_xml = xml.get_playlist(playlist.id.clone());
        if let Some(playlist_xml) = playlist_xml {
            if playlist.id != "100000" {
                let ts = playlist_xml.timestamp;
                let diff = playlist.updated_at.naive_utc() - ts;
                if diff.num_seconds().abs() > SECONDS_THRESH {
                    // If the difference is more than 1 second, we have a mismatch
                    eprintln!(
                        "Difference in timestamps for playlist {}: {} seconds",
                        playlist.id,
                        diff.num_seconds()
                    );
                    return Ok(false);
                }
                if playlist_xml.parent_id != playlist.parent_id {
                    // Check that parent ID matches
                    eprintln!("Parent ID mismatch for playlist {}", playlist.id);
                    return Ok(false);
                }
            }
        } else {
            eprintln!("Playlist {} not found in XML", playlist.id);
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
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_playlists()?;
    Ok(())
}

#[test]
fn test_get_playlist_children() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_playlist_children("root")?;
    Ok(())
}

#[test]
fn test_get_playlist_by_id() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let item = db.get_playlist_by_id("1234")?;
    assert!(item.is_none());
    Ok(())
}

#[test]
fn test_create_playlist() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    let old_usn = db.get_local_usn()?;
    // Insert
    let name = "Name".to_string();
    let attr = PlaylistType::List;
    let parent_id = "root".to_string();
    let seq = None;

    let new_item =
        db.create_playlist_node(name.clone(), attr, Some(parent_id.clone()), seq, None, None)?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.name, name, "Name mismatch");
    assert_eq!(new_item.parent_id, parent_id.clone(), "Parent ID mismatch");
    assert_eq!(new_usn, old_usn + 2, "USN mismatch");
    assert_eq!(
        new_item.rb_local_usn.unwrap(),
        new_usn,
        "Local USN mismatch"
    );

    // Verify the insertion
    let item_opt = db.get_playlist_by_id(new_item.id.as_str())?;
    assert!(item_opt.is_some());
    let item = item_opt.unwrap();

    // Verify seq number
    let items = db.get_playlist_children(parent_id.as_str())?;
    let n = items.len() as i32;
    assert_eq!(item.seq, n, "Seq number mismatch");

    assert_playlist_seq(items);

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
fn test_create_playlist_seq() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    // Insert root
    let name = "Name".to_string();
    let attr = PlaylistType::List;
    let parent_id = "root".to_string();
    let seq = 1;

    let new_item = db.create_playlist_node(
        name.clone(),
        attr,
        Some(parent_id.clone()),
        Some(seq),
        None,
        None,
    )?;

    // Verify the insertion
    let item = db.get_playlist_by_id(new_item.id.as_str())?;
    assert!(item.is_some());

    // Verify seq number
    let items = db.get_playlist_children(parent_id.as_str())?;
    item.unwrap().seq = seq;

    assert_playlist_seq(items);

    Ok(())
}

#[test]
fn test_create_playlist_folder() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    let old_usn = db.get_local_usn()?;
    // Insert
    let name = "Name".to_string();
    let attr = PlaylistType::Folder;
    let parent_id = "root".to_string();
    let seq = None;

    let new_item =
        db.create_playlist_node(name.clone(), attr, Some(parent_id.clone()), seq, None, None)?;
    let new_usn = db.get_local_usn()?;

    assert_eq!(new_item.name, name);
    assert_eq!(new_item.parent_id, parent_id.clone());
    assert_eq!(new_usn, old_usn + 2);
    assert_eq!(new_item.rb_local_usn.unwrap(), new_usn);

    // Verify the insertion
    let item_opt = db.get_playlist_by_id(new_item.id.as_str())?;
    assert!(item_opt.is_some());
    let item = item_opt.unwrap();

    // Verify seq number
    let items = db.get_playlist_children(parent_id.as_str())?;
    let n = items.len() as i32;
    assert_eq!(item.seq, n);

    assert_playlist_seq(items);

    // Check playlist XML
    assert_playlist_xml(&mut db);

    // Try adding a sub-playlist
    let sub_name = "Name".to_string();
    let sub_parent_id = Some(new_item.clone().id);

    let sub_item = db.create_playlist_node(
        sub_name.clone(),
        PlaylistType::List,
        sub_parent_id,
        None,
        None,
        None,
    )?;
    assert_eq!(sub_item.parent_id, new_item.id);
    assert_eq!(sub_item.seq, 1);

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
fn test_create_playlist_song() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    // Create playlist structure
    let pl = db.create_playlist("Playlist".to_string(), None, None, None, None)?;

    // Add a song to the playlist
    let contents = db.get_contents()?;
    let cid1 = contents[0].id.clone();
    let cid2 = contents[1].id.clone();
    let cid3 = contents[2].id.clone();
    let cid4 = contents[3].id.clone();

    // Insert song 1
    let song = db.create_playlist_song(&pl.id, &cid1, None)?;
    assert_eq!(song.playlist_id, pl.id.clone());
    assert_eq!(song.content_id, cid1.clone());
    assert_eq!(song.track_no, 1);

    // Insert song 2
    let song = db.create_playlist_song(&pl.id, &cid2, None)?;
    assert_eq!(song.playlist_id, pl.id.clone());
    assert_eq!(song.content_id, cid2.clone());
    assert_eq!(song.track_no, 2);

    // Insert song 3 at position 2
    let song = db.create_playlist_song(&pl.id, &cid3, Some(2))?;
    assert_eq!(song.playlist_id, pl.id.clone());
    assert_eq!(song.content_id, cid3.clone());
    let songs = db.get_playlist_songs(&pl.id)?;
    let content_ids: Vec<String> = songs.iter().map(|s| s.content_id.clone()).collect();
    assert_eq!(content_ids, vec![cid1, cid3, cid2]);

    assert_playlist_xml(&mut db);

    // Check error on invalid seq
    let res = db.create_playlist_song(&pl.id, &cid4, Some(0));
    assert!(res.is_err(), "Expected error on invalid seq");

    let res = db.create_playlist_song(&pl.id, &cid4, Some(5));
    assert!(res.is_err(), "Expected error on invalid seq");

    Ok(())
}

#[test]
fn test_delete_playlist() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    // Create playlist structure
    let folder = db.create_playlist_folder("Folder".to_string(), None, None)?;
    let pid = Some(folder.id.clone());
    let _pl1 = db.create_playlist("Sub 1".to_string(), pid.clone(), None, None, None)?;
    let pl2 = db.create_playlist("Sub 2".to_string(), pid.clone(), None, None, None)?;
    let pl3 = db.create_playlist("Sub 3".to_string(), pid.clone(), None, None, None)?;
    // Add a song to pl3
    let contents = db.get_contents()?;
    let cid1 = contents[0].id.clone();
    let cid2 = contents[1].id.clone();
    let cid3 = contents[2].id.clone();
    let song1 = db.create_playlist_song(&pl3.id, &cid1, None)?;
    let song2 = db.create_playlist_song(&pl3.id, &cid2, None)?;
    let song3 = db.create_playlist_song(&pl3.id, &cid3, None)?;

    let old_usn = db.get_local_usn()?;

    // Delete playlist
    let deleted = db.delete_playlist(&pl2.id)?;
    assert_eq!(deleted, 1);

    // Check that playlist was deleted
    let res = db.get_playlist_by_id(&pl2.id)?;
    assert!(res.is_none());

    // Check USN is correct (+1 for deleting)
    let new_usn = db.get_local_usn()?;
    assert_eq!(new_usn, old_usn + 1);

    let playlists = db.get_playlist_children(folder.id.as_str())?;
    assert_playlist_seq(playlists);

    assert_playlist_xml(&mut db);

    // Check that songs in pl3 are also deleted
    let deleted = db.delete_playlist(&pl3.id)?;
    assert_eq!(deleted, 1);
    let res = db.get_playlist_song_by_id(&song1.id)?;
    assert!(res.is_none());
    let res = db.get_playlist_song_by_id(&song2.id)?;
    assert!(res.is_none());
    let res = db.get_playlist_song_by_id(&song3.id)?;
    assert!(res.is_none());

    Ok(())
}

#[test]
fn test_delete_playlist_folder() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    // Create playlist structure
    let folder = db.create_playlist_folder("Folder".to_string(), None, None)?;
    let pid = Some(folder.id.clone());
    let pl1 = db.create_playlist("Sub 1".to_string(), pid.clone(), None, None, None)?;
    let pl2 = db.create_playlist("Sub 2".to_string(), pid.clone(), None, None, None)?;
    let pl3 = db.create_playlist("Sub 3".to_string(), pid.clone(), None, None, None)?;
    // Add a song to pl3
    let contents = db.get_contents()?;
    let cid1 = contents[0].id.clone();
    let cid2 = contents[1].id.clone();
    let cid3 = contents[2].id.clone();
    let song1 = db.create_playlist_song(&pl3.id, &cid1, None)?;
    let song2 = db.create_playlist_song(&pl3.id, &cid2, None)?;
    let song3 = db.create_playlist_song(&pl3.id, &cid3, None)?;

    let old_usn = db.get_local_usn()?;

    let deleted = db.delete_playlist(&folder.id)?;
    assert_eq!(deleted, 4); // Dont count the songs

    // Check USN (+1 for deleting with Seq update, +1 for children)
    let new_usn = db.get_local_usn()?;
    assert_eq!(new_usn, old_usn + 4); // Or 4??

    // Check that child playlists were deleted
    let res1 = db.get_playlist_by_id(&pl1.id)?;
    assert!(res1.is_none());
    let res2 = db.get_playlist_by_id(&pl2.id)?;
    assert!(res2.is_none());
    let res3 = db.get_playlist_by_id(&pl3.id)?;
    assert!(res3.is_none());

    assert_playlist_xml(&mut db);

    // Check that songs in pl3 are also deleted
    let res = db.get_playlist_song_by_id(&song1.id)?;
    assert!(res.is_none());
    let res = db.get_playlist_song_by_id(&song2.id)?;
    assert!(res.is_none());
    let res = db.get_playlist_song_by_id(&song3.id)?;
    assert!(res.is_none());

    Ok(())
}

#[test]
pub fn test_delete_playlist_song() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    // Create playlist structure
    let pl = db.create_playlist("Playlist".to_string(), None, None, None, None)?;

    // Add a songs to the playlist
    let contents = db.get_contents()?;
    let cid1 = contents[0].id.clone();
    let cid2 = contents[1].id.clone();
    let cid3 = contents[2].id.clone();
    let song1 = db.create_playlist_song(&pl.id, &cid1, None)?;
    let song2 = db.create_playlist_song(&pl.id, &cid2, None)?;
    let song3 = db.create_playlist_song(&pl.id, &cid3, None)?;

    let old_usn = db.get_local_usn()?;
    // Delete one song
    let deleted = db.delete_playlist_song(&song2.id)?;
    assert_eq!(deleted, 1);

    // Check that the song was deleted
    let res = db.get_playlist_song_by_id(&song2.id)?;
    assert!(res.is_none());

    // Check that the other songs are still there
    let res = db.get_playlist_song_by_id(&song1.id)?;
    assert!(res.is_some());
    let res = db.get_playlist_song_by_id(&song3.id)?;
    assert!(res.is_some());

    // Check USN is correct (+1 for deleting)
    let new_usn = db.get_local_usn()?;
    assert_eq!(new_usn, old_usn + 1);

    // Check sequence of remaining songs
    let songs = db.get_playlist_songs(&pl.id)?;
    assert_playlist_song_seq(songs);

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
pub fn test_move_playlist_seq() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    // Create playlist structure
    let root = db.create_playlist_folder("folder".into(), None, None)?;
    let f1 = db.create_playlist_folder("f1".into(), Some(root.id.clone()), None)?;
    let _f2 = db.create_playlist_folder("f2".into(), Some(root.id.clone()), None)?;
    let p1 = db.create_playlist("pl1".into(), Some(root.id.clone()), None, None, None)?;
    let p2 = db.create_playlist("pl2".into(), Some(root.id.clone()), None, None, None)?;
    let p3 = db.create_playlist("pl3".into(), Some(root.id.clone()), None, None, None)?;
    let _p4 = db.create_playlist("pl4".into(), Some(root.id.clone()), None, None, None)?;
    let _p5 = db.create_playlist("subpl1".into(), Some(f1.id.clone()), None, None, None)?;
    let _p6 = db.create_playlist("subpl2".into(), Some(f1.id.clone()), None, None, None)?;
    let _p7 = db.create_playlist("subpl3".into(), Some(f1.id.clone()), None, None, None)?;
    let _p8 = db.create_playlist("subpl4".into(), Some(f1.id.clone()), None, None, None)?;

    let children = db.get_playlist_children(&root.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl1", "pl2", "pl3", "pl4"]);

    // Move playlist 1 down to position 5
    db.move_playlist(&p1.id, Some(5), None)?;
    let children = db.get_playlist_children(&root.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl2", "pl3", "pl1", "pl4"]);
    assert_playlist_seq(children);

    // Move playlist 2 down to position 6 (end)
    db.move_playlist(&p2.id, Some(6), None)?;
    let children = db.get_playlist_children(&root.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl3", "pl1", "pl4", "pl2"]);
    assert_playlist_seq(children);

    // Move playlist 3 up to position 2
    db.move_playlist(&p3.id, Some(2), None)?;
    let children = db.get_playlist_children(&root.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["f1", "pl3", "f2", "pl1", "pl4", "pl2"]);
    assert_playlist_seq(children);

    // Move playlist 1 up to position 1 (start)
    db.move_playlist(&p1.id, Some(1), None)?;
    let children = db.get_playlist_children(&root.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["pl1", "f1", "pl3", "f2", "pl4", "pl2"]);
    assert_playlist_seq(children);

    // Check error on invalid seq
    let res = db.move_playlist(&p2.id, Some(0), None);
    assert!(res.is_err());

    let res = db.move_playlist(&p2.id, Some(7), None);
    assert!(res.is_err());

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
pub fn test_move_playlist_parent() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    // Create playlist structure
    let root = db.create_playlist_folder("folder".into(), None, None)?;
    let f1 = db.create_playlist_folder("f1".into(), Some(root.id.clone()), None)?;
    let _f2 = db.create_playlist_folder("f2".into(), Some(root.id.clone()), None)?;
    let p1 = db.create_playlist("pl1".into(), Some(root.id.clone()), None, None, None)?;
    let p2 = db.create_playlist("pl2".into(), Some(root.id.clone()), None, None, None)?;
    let p3 = db.create_playlist("pl3".into(), Some(root.id.clone()), None, None, None)?;
    let p4 = db.create_playlist("pl4".into(), Some(root.id.clone()), None, None, None)?;
    let _p5 = db.create_playlist("subpl1".into(), Some(f1.id.clone()), None, None, None)?;
    let _p6 = db.create_playlist("subpl2".into(), Some(f1.id.clone()), None, None, None)?;
    let _p7 = db.create_playlist("subpl3".into(), Some(f1.id.clone()), None, None, None)?;
    let _p8 = db.create_playlist("subpl4".into(), Some(f1.id.clone()), None, None, None)?;

    let children = db.get_playlist_children(&root.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl1", "pl2", "pl3", "pl4"]);

    let children = db.get_playlist_children(&f1.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["subpl1", "subpl2", "subpl3", "subpl4"]);

    // Move playlist 1 to end of f1
    db.move_playlist(&p1.id, None, Some(f1.id.clone()))?;
    let children = db.get_playlist_children(&root.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl2", "pl3", "pl4"]);
    assert_playlist_seq(children);
    let children = db.get_playlist_children(&f1.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["subpl1", "subpl2", "subpl3", "subpl4", "pl1"]);
    assert_playlist_seq(children);

    // Move playlist 2 to explicit seq (3) of f1
    db.move_playlist(&p2.id, Some(3), Some(f1.id.clone()))?;
    let children = db.get_playlist_children(&root.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl3", "pl4"]);
    assert_playlist_seq(children);
    let children = db.get_playlist_children(&f1.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(
        names,
        vec!["subpl1", "subpl2", "pl2", "subpl3", "subpl4", "pl1"]
    );
    assert_playlist_seq(children);

    // Move playlist 3 to start of f1
    db.move_playlist(&p3.id, Some(1), Some(f1.id.clone()))?;
    let children = db.get_playlist_children(&root.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(names, vec!["f1", "f2", "pl4"]);
    assert_playlist_seq(children);

    let children = db.get_playlist_children(&f1.id)?;
    let names: Vec<String> = children.iter().map(|p| p.name.clone()).collect();
    assert_eq!(
        names,
        vec!["pl3", "subpl1", "subpl2", "pl2", "subpl3", "subpl4", "pl1"]
    );
    assert_playlist_seq(children);

    // Check error on invalid parent and/or seq
    let res = db.move_playlist(&p4.id, None, Some("1234".into()));
    assert!(res.is_err(), "invalid parent expected");

    let res = db.move_playlist(&p4.id, Some(0), Some(f1.id.clone()));
    assert!(res.is_err(), "invalid seq expected: seq=0");

    let count = db.get_playlist_children(&f1.id)?.len();
    let res = db.move_playlist(&p4.id, Some(9), Some(f1.id.clone()));
    assert!(res.is_err(), "invalid seq expected: seq=9, count={count}");

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
pub fn test_move_all_playlists() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    // Create playlist structure
    let root = db.create_playlist_folder("folder".into(), None, None)?;
    let f1 = db.create_playlist_folder("f1".into(), Some(root.id.clone()), None)?;
    let f2 = db.create_playlist_folder("f2".into(), Some(root.id.clone()), None)?;
    let p1 = db.create_playlist("pl1".into(), Some(root.id.clone()), None, None, None)?;
    let _p2 = db.create_playlist("pl2".into(), Some(root.id.clone()), None, None, None)?;
    let p3 = db.create_playlist("pl3".into(), Some(root.id.clone()), None, None, None)?;
    let _p4 = db.create_playlist("pl4".into(), Some(root.id.clone()), None, None, None)?;
    let _p5 = db.create_playlist("subpl1".into(), Some(f1.id.clone()), None, None, None)?;
    let p6 = db.create_playlist("subpl2".into(), Some(f1.id.clone()), None, None, None)?;
    let p7 = db.create_playlist("subpl3".into(), Some(f1.id.clone()), None, None, None)?;
    let _p8 = db.create_playlist("subpl4".into(), Some(f1.id.clone()), None, None, None)?;

    assert_all_playlist_seqs(&mut db)?;

    let ids = vec![
        p1.id.as_str(),
        p3.id.as_str(),
        p6.id.as_str(),
        p7.id.as_str(),
    ];
    db.move_playlists(ids, f2.id.as_str(), None)?;

    assert_all_playlist_seqs(&mut db)?;
    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
pub fn test_move_playlist_song() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    // Create playlist structure
    let pl = db.create_playlist("Playlist".to_string(), None, None, None, None)?;
    // Add a songs to the playlist
    let contents = db.get_contents()?;
    let cid1 = contents[0].id.clone();
    let cid2 = contents[1].id.clone();
    let cid3 = contents[2].id.clone();
    let cid4 = contents[3].id.clone();
    let cid5 = contents[4].id.clone();
    let song1 = db.create_playlist_song(&pl.id, &cid1, None)?;
    let song2 = db.create_playlist_song(&pl.id, &cid2, None)?;
    let song3 = db.create_playlist_song(&pl.id, &cid3, None)?;
    let song4 = db.create_playlist_song(&pl.id, &cid4, None)?;
    let song5 = db.create_playlist_song(&pl.id, &cid5, None)?;
    let sid1 = song1.id.clone();
    let sid2 = song2.id.clone();
    let sid3 = song3.id.clone();
    let sid4 = song4.id.clone();
    let sid5 = song5.id.clone();
    let songs = db.get_playlist_songs(&pl.id)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.id.clone()).collect();
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
    db.move_playlist_song(&song4.id, 2)?;
    let songs = db.get_playlist_songs(&pl.id)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.id.clone()).collect();
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
    db.move_playlist_song(&song4.id, 4)?;
    let songs = db.get_playlist_songs(&pl.id)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.id.clone()).collect();
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
    db.move_playlist_song(&song2.id, 3)?;
    let songs = db.get_playlist_songs(&pl.id)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.id.clone()).collect();
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
    db.move_playlist_song(&song4.id, 1)?;
    let songs = db.get_playlist_songs(&pl.id)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.id.clone()).collect();
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
    db.move_playlist_song(&song1.id, 5)?;
    let songs = db.get_playlist_songs(&pl.id)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.id.clone()).collect();
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
    let res = db.move_playlist_song(&song2.id, 0);
    assert!(res.is_err());
    let res = db.move_playlist_song(&song2.id, 6);
    assert!(res.is_err());

    assert_playlist_xml(&mut db);

    Ok(())
}

#[test]
pub fn test_move_all_playlist_songs() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;

    // Create playlist structure
    let pl = db.create_playlist("Playlist".to_string(), None, None, None, None)?;
    // Add a songs to the playlist
    let contents = db.get_contents()?;
    let cid1 = contents[0].id.clone();
    let cid2 = contents[1].id.clone();
    let cid3 = contents[2].id.clone();
    let cid4 = contents[3].id.clone();
    let cid5 = contents[4].id.clone();
    let song1 = db.create_playlist_song(&pl.id, &cid1, None)?;
    let song2 = db.create_playlist_song(&pl.id, &cid2, None)?;
    let song3 = db.create_playlist_song(&pl.id, &cid3, None)?;
    let song4 = db.create_playlist_song(&pl.id, &cid4, None)?;
    let song5 = db.create_playlist_song(&pl.id, &cid5, None)?;
    let sid1 = song1.id.clone();
    let sid2 = song2.id.clone();
    let sid3 = song3.id.clone();
    let sid4 = song4.id.clone();
    let sid5 = song5.id.clone();
    let songs = db.get_playlist_songs(&pl.id)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.id.clone()).collect();
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

    // Move song 3 and 5 to position 2
    db.move_playlist_songs(vec![&song3.id, &song5.id], 2)?;

    let songs = db.get_playlist_songs(&pl.id)?;
    let song_ids: Vec<String> = songs.iter().map(|s| s.id.clone()).collect();
    assert_eq!(
        song_ids,
        vec![
            sid1.clone(),
            sid3.clone(),
            sid5.clone(),
            sid2.clone(),
            sid4.clone(),
        ]
    );
    assert_playlist_song_seq(songs);

    Ok(())
}

// -- Related tracks ------------------------------------------------------------------------------

#[test]
fn test_get_related_tracks() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_related_tracks()?;

    let db_path = setup_master_db_path(7);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_related_tracks()?;
    Ok(())
}

// -- Sampler -------------------------------------------------------------------------------------

#[test]
fn test_get_sampler() -> anyhow::Result<()> {
    let db_path = setup_master_db_path(6);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_samplers()?;

    let db_path = setup_master_db_path(7);
    let mut db = MasterDb::new(db_path.path())?;
    let _items = db.get_samplers()?;
    Ok(())
}

// -- Smart playlists ------------------------------------------------------------------------------

#[test]
fn test_get_smart_playlist_contents() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;
    // Operates on a disposable copy, so allow writes regardless of whether Rekordbox is running.
    db.set_unsafe_writes(true);

    let all_contents = db.get_contents()?;
    let with_title = all_contents.iter().filter(|c| c.title.is_some()).count();
    assert!(with_title > 0, "fixture should contain titled tracks");

    // Smart playlist matching every track that has a title (`Name contains ""`).
    let mut smart = SmartList::new("0", LogicalOperator::All);
    smart.new_condition(
        Property::Name,
        Operator::Contains,
        "".to_string(),
        None,
        None,
    );
    let pl = db.create_playlist_node(
        "Smart All".to_string(),
        PlaylistType::SmartList,
        None,
        None,
        None,
        Some(smart.to_string()?),
    )?;
    assert_eq!(pl.attribute, PlaylistType::SmartList);

    // `get_playlist_contents` evaluates the smart list instead of returning an empty vec.
    let contents = db.get_playlist_contents(&pl.id)?;
    assert_eq!(contents.len(), with_title);

    Ok(())
}

#[test]
fn test_get_smart_playlist_contents_filtered() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;
    db.set_unsafe_writes(true);

    // Pick an existing track and build a smart list matching it by exact title.
    let sample = db
        .get_contents()?
        .into_iter()
        .find(|c| c.title.is_some())
        .expect("fixture should contain a titled track");
    let title = sample.title.clone().unwrap();

    let mut smart = SmartList::new("0", LogicalOperator::All);
    smart.new_condition(Property::Name, Operator::Equal, title.clone(), None, None);
    let pl = db.create_playlist_node(
        "Smart One".to_string(),
        PlaylistType::SmartList,
        None,
        None,
        None,
        Some(smart.to_string()?),
    )?;

    let contents = db.get_playlist_contents(&pl.id)?;
    assert!(
        contents.iter().any(|c| c.id == sample.id),
        "smart playlist should return the matching track"
    );
    assert!(
        contents
            .iter()
            .all(|c| c.title.as_deref() == Some(title.as_str())),
        "all returned tracks should match the smart-list condition"
    );

    Ok(())
}

#[test]
fn test_get_regular_playlist_contents_unaffected() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;
    db.set_unsafe_writes(true);

    // A regular (non-smart) playlist still goes through the normal song-playlist lookup
    // and is empty until songs are added.
    let pl = db.create_playlist("Regular".to_string(), None, None, None, None)?;
    let contents = db.get_playlist_contents(&pl.id)?;
    assert_eq!(contents.len(), 0);

    Ok(())
}

#[test]
fn test_get_smart_playlist_contents_all_property_types_execute() -> anyhow::Result<()> {
    let tmp_dir = setup_master_db_dir_path(6);
    let mut db = MasterDb::new(tmp_dir.path().join("rekordbox").join("master.db"))?;
    db.set_unsafe_writes(true);

    let total = db.get_contents()?.len();

    // One representative condition per property/operator combination Rekordbox can produce.
    // The point is to prove every generated query is valid SQL and returns without error,
    // so any smart list configuration yields its tracks instead of an empty list.
    let cases: Vec<(Property, Operator, &str, Option<&str>)> = vec![
        (Property::Artist, Operator::Contains, "a", None),
        (Property::Album, Operator::Contains, "a", None),
        (Property::AlbumArtist, Operator::Contains, "a", None),
        (Property::OriginalArtist, Operator::Contains, "a", None),
        (Property::BPM, Operator::Greater, "120", None),
        (Property::Grouping, Operator::Equal, "1", None),
        (Property::Comments, Operator::Contains, "a", None),
        (Property::Producer, Operator::Contains, "a", None),
        (Property::StockDate, Operator::Less, "2030-01-01", None),
        (Property::DateCreated, Operator::InLast, "365", Some("day")),
        (Property::Counter, Operator::Greater, "0", None),
        (Property::Filename, Operator::Contains, "a", None),
        (Property::Genre, Operator::Contains, "a", None),
        (Property::Key, Operator::Contains, "a", None),
        (Property::Label, Operator::Contains, "a", None),
        (Property::MixName, Operator::Contains, "a", None),
        (Property::MyTag, Operator::Contains, "1", None),
        (Property::Rating, Operator::Greater, "0", None),
        (Property::DateReleased, Operator::Less, "2030-01-01", None),
        (Property::Remixer, Operator::Contains, "a", None),
        (Property::Duration, Operator::Less, "100000", None),
        (Property::Name, Operator::Contains, "a", None),
        (Property::Year, Operator::Greater, "0", None),
    ];

    for (i, (property, operator, left, unit)) in cases.into_iter().enumerate() {
        let mut smart = SmartList::new("0", LogicalOperator::All);
        smart.new_condition(
            property.clone(),
            operator,
            left.to_string(),
            None,
            unit.map(|u| u.to_string()),
        );
        let pl = db.create_playlist_node(
            format!("Smart {i}"),
            PlaylistType::SmartList,
            None,
            None,
            None,
            Some(smart.to_string()?),
        )?;

        let contents = db
            .get_playlist_contents(&pl.id)
            .unwrap_or_else(|e| panic!("contents failed for {property:?}: {e}"));
        assert!(
            contents.len() <= total,
            "{property:?} returned too many rows"
        );
    }

    // A combined multi-condition smart list (several artist roles + a date) must also work.
    let mut smart = SmartList::new("0", LogicalOperator::Any);
    smart.new_condition(Property::Artist, Operator::Contains, "a".into(), None, None);
    smart.new_condition(
        Property::Remixer,
        Operator::Contains,
        "b".into(),
        None,
        None,
    );
    smart.new_condition(
        Property::Producer,
        Operator::Contains,
        "c".into(),
        None,
        None,
    );
    smart.new_condition(
        Property::AlbumArtist,
        Operator::Contains,
        "d".into(),
        None,
        None,
    );
    smart.new_condition(
        Property::DateCreated,
        Operator::InLast,
        "30".into(),
        None,
        Some("day".into()),
    );
    let pl = db.create_playlist_node(
        "Smart Combined".to_string(),
        PlaylistType::SmartList,
        None,
        None,
        None,
        Some(smart.to_string()?),
    )?;
    let contents = db.get_playlist_contents(&pl.id)?;
    assert!(contents.len() <= total);

    Ok(())
}
#[test]
fn file_type_codes_and_extensions() {
    for (code, kind) in [
        (0, FileType::INVALID),
        (1, FileType::MP3),
        (3, FileType::MP4),
        (4, FileType::M4A),
        (5, FileType::FLAC),
        (6, FileType::ALAC),
        (11, FileType::WAV),
        (12, FileType::AIFF),
        (16, FileType::VIDEO),
        (26, FileType::StreamApple),
    ] {
        assert_eq!(kind as i32, code);
        assert_eq!(FileType::try_from(code), Ok(kind));
    }
    assert_eq!(FileType::AAC, FileType::M4A);
    assert_eq!(FileType::AIF, FileType::AIFF);
    assert_eq!(FileType::default(), FileType::INVALID);
    for value in [-1, 2, 20, 999] {
        assert!(FileType::try_from(value).is_err());
    }
    for (ext, code) in [
        ("MP3", 1),
        ("mp4", 3),
        ("m4a", 4),
        ("aac", 4),
        ("alac", 6),
        ("flac", 5),
        ("wav", 11),
        ("aif", 12),
        ("aiff", 12),
        ("avi", 16),
        ("m4v", 16),
        ("mov", 16),
        ("mpg", 16),
    ] {
        assert_eq!(FileType::try_from_extension(ext).unwrap() as i32, code);
    }
    assert!(FileType::try_from_extension("unsupported").is_err());
}

#[test]
fn rejected_content_does_not_change_registry_or_rows() -> anyhow::Result<()> {
    for version in [6, 7] {
        let fixture = setup_master_db_path(version);
        let mut db = MasterDb::new(fixture.path())?;
        db.set_unsafe_writes(true); // Only this temporary fixture copy is writable.
        let files = create_temp_dir();
        let before_usn = db.get_local_usn()?;
        let before_count = db.get_contents()?.len();
        for name in ["track.unsupported", "no_extension"] {
            let path = files.path().join(name);
            std::fs::write(&path, b"")?;
            let error = db.create_content(&path).unwrap_err();
            assert!(error.to_string().contains(if name.contains('.') {
                "Unknown file type"
            } else {
                "Missing or invalid file extension"
            }));
            assert_eq!(db.get_local_usn()?, before_usn);
            assert_eq!(db.get_contents()?.len(), before_count);
        }
    }
    Ok(())
}

#[test]
fn content_insertion_stores_correct_file_type_codes() -> anyhow::Result<()> {
    let fixture = setup_master_db_path(6);
    let mut db = MasterDb::new(fixture.path())?;
    db.set_unsafe_writes(true);
    let files = create_temp_dir();
    for (ext, code) in [("mp3", 1), ("m4a", 4), ("alac", 6), ("mov", 16)] {
        let path = files.path().join(format!("track.{ext}"));
        // Insertion records metadata; it does not decode or analyze audio.
        std::fs::write(&path, b"")?;
        let row = db.create_content(&path)?;
        assert_eq!(row.file_type, code);
        assert_eq!(db.get_content_by_id(&row.id)?.unwrap().file_type, code);
    }
    Ok(())
}

#[test]
fn content_insertion_accepts_an_explicit_file_type() -> anyhow::Result<()> {
    let fixture = setup_master_db_path(6);
    let mut db = MasterDb::new(fixture.path())?;
    db.set_unsafe_writes(true);
    let files = create_temp_dir();

    let extensionless = files.path().join("recording");
    std::fs::write(&extensionless, b"")?;
    let mut conn = db.pool.get()?;
    let row = NewDjmdContent::new(extensionless.to_string_lossy())
        .file_type(FileType::WAV.into())
        .insert(&mut conn)?;
    assert_eq!(row.file_type, FileType::WAV as i32);

    Ok(())
}

#[test]
fn add_content_inserts_a_configured_model() -> anyhow::Result<()> {
    let fixture = setup_master_db_path(7);
    let files = create_temp_dir();
    let path = files.path().join("configured-track");
    std::fs::write(&path, b"")?;
    let mut db = MasterDb::new(fixture.path())?;
    db.set_unsafe_writes(true);

    let content = db.add_content(
        NewDjmdContent::new(path.to_string_lossy())
            .file_type(FileType::WAV.into())
            .title("Configured title"),
    )?;

    assert_eq!(content.file_type, FileType::WAV as i32);
    assert_eq!(content.title.as_deref(), Some("Configured title"));
    assert_eq!(db.get_content_by_id(&content.id)?, Some(content));
    Ok(())
}

#[test]
fn raw_file_type_values_remain_readable() -> anyhow::Result<()> {
    let fixture = setup_master_db_path(6);
    let mut db = MasterDb::new(fixture.path())?;
    let row = db.get_contents()?.remove(0);
    let mut conn = db.pool.get()?;
    diesel::update(djmdContent::table.find(&row.id))
        .set(djmdContent::file_type.eq(999))
        .execute(&mut conn)?;
    assert_eq!(db.get_content_by_id(&row.id)?.unwrap().file_type, 999);
    Ok(())
}
