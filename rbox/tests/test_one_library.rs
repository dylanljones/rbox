// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

mod common;

use rbox::OneLibrary;
use tempfile::NamedTempFile;

fn temp_file() -> NamedTempFile {
    let tmp_root = common::get_temp_root_dir();
    let tmp = NamedTempFile::new_in(tmp_root).expect("Failed to create temp file");
    tmp
}

#[test]
fn test_open() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let _db = OneLibrary::new(db_path.path())?;
    Ok(())
}

#[test]
fn test_create() -> anyhow::Result<()> {
    let tmp = temp_file();
    let db = OneLibrary::create(tmp.path(), 0)?;
    let prop = db.get_first_property()?;
    assert!(prop.is_some());
    let prop = prop.unwrap();
    assert_eq!(prop.number_of_contents, 0);
    assert_eq!(prop.db_version, 1000);

    let categories = db.get_categories()?;
    assert!(categories.len() > 0);

    let colors = db.get_colors()?;
    assert!(colors.len() > 0);

    let menu_item = db.get_menu_items()?;
    assert!(menu_item.len() > 0);

    let sort = db.get_sorts()?;
    assert!(sort.len() > 0);

    Ok(())
}

// -- Album ----------------------------------------------------------------------------------------

#[test]
fn test_create_album() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let db = OneLibrary::new(db_path.path())?;
    let n = db.get_albums()?.len() as i32;
    let model = db.create_album("album1", None, None)?;
    assert_eq!(model.id, n + 1);
    Ok(())
}

#[test]
fn test_get_albums() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let db = OneLibrary::new(db_path.path())?;

    let res = db.get_albums()?;
    assert_eq!(res.len(), 0);

    db.create_album("album1", None, None)?;
    let res = db.get_albums()?;
    assert_eq!(res.len(), 1);
    Ok(())
}

#[test]
fn test_get_album_by_id() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let db = OneLibrary::new(db_path.path())?;
    let model = db.create_album("album1", None, None)?;
    let id = model.id;

    let res = db.get_album_by_id(id)?;
    assert!(res.is_some());

    let res = db.get_album_by_id(1000000)?;
    assert!(res.is_none());
    Ok(())
}

#[test]
fn test_get_album_by_name() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let db = OneLibrary::new(db_path.path())?;
    db.create_album("album1", None, None)?;

    let res = db.get_album_by_name("album1")?;
    assert!(res.is_some());

    let res = db.get_album_by_name("album2")?;
    assert!(res.is_none());
    Ok(())
}

#[test]
fn test_delete_album() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let db = OneLibrary::new(db_path.path())?;
    let model1 = db.create_album("album1", None, None)?;
    let _model2 = db.create_album("album2", None, None)?;

    db.delete_album(model1.id)?;

    let res = db.get_album_by_name("album1")?;
    assert!(res.is_none());
    let res = db.get_album_by_name("album2")?;
    assert!(res.is_some());
    Ok(())
}

// -- Artist ----------------------------------------------------------------------------------------

#[test]
fn test_create_artist() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let db = OneLibrary::new(db_path.path())?;
    let n = db.get_artists()?.len() as i32;
    let model = db.create_artist("artist1")?;
    assert_eq!(model.id, n + 1);
    Ok(())
}

#[test]
fn test_get_artists() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let db = OneLibrary::new(db_path.path())?;

    let res = db.get_artists()?;
    assert_eq!(res.len(), 1);

    db.create_artist("artist1")?;
    let res = db.get_artists()?;
    assert_eq!(res.len(), 2);
    Ok(())
}

#[test]
fn test_get_artist_by_id() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let db = OneLibrary::new(db_path.path())?;
    let model = db.create_artist("artist1")?;
    let id = model.id;

    let res = db.get_artist_by_id(id)?;
    assert!(res.is_some());

    let res = db.get_artist_by_id(1000000)?;
    assert!(res.is_none());
    Ok(())
}

#[test]
fn test_get_artist_by_name() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let db = OneLibrary::new(db_path.path())?;
    db.create_artist("artist1")?;

    let res = db.get_artist_by_name("artist1")?;
    assert!(res.is_some());

    let res = db.get_artist_by_name("artist2")?;
    assert!(res.is_none());
    Ok(())
}

#[test]
fn test_delete_artist() -> anyhow::Result<()> {
    let db_path = common::setup_one_library_path();
    let db = OneLibrary::new(db_path.path())?;
    let model1 = db.create_artist("artist1")?;
    let _model2 = db.create_artist("artist2")?;

    db.delete_artist(model1.id)?;

    let res = db.get_artist_by_name("artist1")?;
    assert!(res.is_none());
    let res = db.get_artist_by_name("artist2")?;
    assert!(res.is_some());

    // Delete existing artist and check references
    db.delete_artist(1)?;
    // todo!!!
    Ok(())
}
