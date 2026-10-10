// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

mod common;

use rbox::OneLibrary;
use tempfile::NamedTempFile;

fn assert_no_content_relations(view: &rbox::one_library::ContentView) {
    assert!(view.artist.is_none());
    assert!(view.remixer.is_none());
    assert!(view.original_artist.is_none());
    assert!(view.composer.is_none());
    assert!(view.lyricist.is_none());
    assert!(view.album.is_none());
    assert!(view.genre.is_none());
    assert!(view.label.is_none());
    assert!(view.key.is_none());
    assert!(view.color.is_none());
    assert!(view.image.is_none());
}

// -- Resolved album relationships --------------------------------------------------------------

#[test]
fn test_album_relationships() -> anyhow::Result<()> {
    use rbox::model_traits::{Model, ModelInsert, ModelRelations};
    use rbox::one_library::*;
    let fixture = common::setup_one_library_path();
    let db = OneLibrary::new(fixture.path())?;
    let mut conn = establish_connection(fixture.path().to_str().unwrap())?;
    let artist = NewArtist::new("album artist").insert(&mut conn)?;
    let image = NewImage::new("album image").insert(&mut conn)?;
    let mut new = NewAlbum::new("resolved album");
    new.artist_id = Some(artist.id);
    new.image_id = Some(image.id);
    let album = new.insert(&mut conn)?;
    let expected = AlbumView {
        album: album.clone(),
        artist: Some(artist.clone()),
        image: Some(image.clone()),
    };
    assert_eq!(album.clone().resolve(&mut conn)?, expected);
    let mut absent = album.clone();
    absent.artist_id = None;
    absent.image_id = None;
    let absent_view = AlbumView {
        album: absent.clone(),
        artist: None,
        image: None,
    };
    assert_eq!(absent.clone().resolve(&mut conn)?, absent_view);
    let mut dangling = absent.clone();
    dangling.artist_id = Some(i32::MAX);
    dangling.image_id = Some(i32::MAX);
    let dangling_view = AlbumView {
        album: dangling.clone(),
        artist: None,
        image: None,
    };
    assert_eq!(dangling.clone().resolve(&mut conn)?, dangling_view);
    // Missing one target must not hide another relationship.
    let mut partial = album.clone();
    partial.artist_id = Some(i32::MAX);
    let view = partial.resolve(&mut conn)?;
    assert!(view.artist.is_none());
    assert_eq!(view.image, Some(image));

    let inputs = vec![dangling, album.clone(), absent, album.clone()];
    assert_eq!(
        Album::resolve_many(&mut conn, inputs)?,
        vec![
            dangling_view,
            expected.clone(),
            absent_view,
            expected.clone()
        ]
    );
    assert_eq!(Album::find(&mut conn, &album.id)?, Some(album.clone()));
    assert_eq!(db.get_album_by_id(album.id)?, Some(album.clone()));
    assert_eq!(db.get_album_with_relations(album.id)?, Some(expected));
    assert!(db.get_album_with_relations(i32::MAX)?.is_none());
    let raw = db.get_albums()?;
    let views = db.get_albums_with_relations()?;
    assert_eq!(
        views.iter().map(|v| v.album.clone()).collect::<Vec<_>>(),
        raw
    );
    assert_eq!(views, Album::resolve_many(&mut conn, raw)?);
    Ok(())
}

#[test]
fn test_album_relationship_batches() -> anyhow::Result<()> {
    use diesel::connection::InstrumentationEvent;
    use diesel::{Connection, RunQueryDsl};
    use rbox::model_traits::{ModelInsert, ModelRelations};
    use rbox::one_library::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    let fixture = common::setup_one_library_path();

    let mut conn = establish_connection(fixture.path().to_str().unwrap())?;
    let album = NewAlbum::new("batch album").insert(&mut conn)?;
    let mut rows = Vec::new();
    for index in 0..501 {
        let artist = NewArtist::new(format!("album artist {index}")).insert(&mut conn)?;
        let image = NewImage::new(format!("album image {index}")).insert(&mut conn)?;
        let mut row = album.clone();
        row.artist_id = Some(artist.id);
        row.image_id = Some(image.id);
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
    assert!(Album::resolve_many(&mut conn, vec![])?.is_empty());
    let empty = album.resolve(&mut conn)?;
    assert!(empty.artist.is_none());
    assert!(empty.image.is_none());
    assert_eq!(queries.swap(0, Ordering::Relaxed), 0);
    let repeated = Album::resolve_many(&mut conn, vec![rows[0].clone(); 501])?;
    assert_eq!(repeated.len(), 501);
    assert!(repeated.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(queries.swap(0, Ordering::Relaxed), 2);
    for size in [1, 500, 501] {
        let views = Album::resolve_many(&mut conn, rows[..size].to_vec())?;
        assert_eq!(views.len(), size);
        for (view, row) in views.iter().zip(&rows) {
            assert_eq!(&view.album, row);
            assert_eq!(view.artist.as_ref().map(|a| &a.id), row.artist_id.as_ref());
            assert_eq!(view.image.as_ref().map(|i| i.id), row.image_id);
        }
        assert_eq!(
            queries.swap(0, Ordering::Relaxed),
            if size > 500 { 4 } else { 2 }
        );
    }
    diesel::sql_query("ALTER TABLE image RENAME TO relationship_test_image").execute(&mut conn)?;
    assert!(rows[0].clone().resolve(&mut conn).is_err());
    diesel::sql_query("ALTER TABLE relationship_test_image RENAME TO image").execute(&mut conn)?;
    diesel::sql_query("ALTER TABLE artist RENAME TO relationship_test_artist")
        .execute(&mut conn)?;
    assert!(rows.remove(0).resolve(&mut conn).is_err());
    Ok(())
}

// -- Resolved category relationships -----------------------------------------------------------

#[test]
fn test_category_relationships() -> anyhow::Result<()> {
    use rbox::model_traits::{Model, ModelInsert, ModelRelations};
    use rbox::one_library::*;
    let fixture = common::setup_one_library_path();
    let db = OneLibrary::new(fixture.path())?;
    let mut conn = establish_connection(fixture.path().to_str().unwrap())?;
    let item = NewMenuItem::new("resolved menu", 0).insert(&mut conn)?;
    let category = NewCategory::new(item.id).insert(&mut conn)?;
    let expected = CategoryView {
        category: category.clone(),
        menu_item: Some(item),
    };
    assert_eq!(category.clone().resolve(&mut conn)?, expected);
    let mut dangling = category.clone();
    dangling.menu_item_id = i32::MAX;
    let missing = CategoryView {
        category: dangling.clone(),
        menu_item: None,
    };
    assert_eq!(dangling.clone().resolve(&mut conn)?, missing);
    // Supplied row order is independent of category sequence and retains duplicates.
    let mut snapshot = category.clone();
    snapshot.seq = 999;
    let snapshot_view = snapshot.clone().resolve(&mut conn)?;
    assert_eq!(snapshot_view.category, snapshot);
    let input = vec![dangling, category.clone(), snapshot, category.clone()];
    assert_eq!(
        Category::resolve_many(&mut conn, input)?,
        vec![missing, expected.clone(), snapshot_view, expected.clone()]
    );
    assert_eq!(
        Category::find(&mut conn, &category.id)?,
        Some(category.clone())
    );
    assert_eq!(db.get_category_by_id(category.id)?, Some(category.clone()));
    assert_eq!(db.get_category_with_relations(category.id)?, Some(expected));
    assert!(db.get_category_with_relations(i32::MAX)?.is_none());
    let raw = db.get_categories()?;
    let views = db.get_categories_with_relations()?;
    assert_eq!(
        views.iter().map(|v| v.category.clone()).collect::<Vec<_>>(),
        raw
    );
    assert_eq!(views, Category::resolve_many(&mut conn, raw)?);
    Ok(())
}

#[test]
fn test_category_relationship_batches() -> anyhow::Result<()> {
    use diesel::connection::InstrumentationEvent;
    use diesel::{Connection, RunQueryDsl};
    use rbox::model_traits::{ModelInsert, ModelRelations};
    use rbox::one_library::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    let fixture = common::setup_one_library_path();
    let mut conn = establish_connection(fixture.path().to_str().unwrap())?;
    let mut rows = Vec::new();
    for index in 0..501 {
        let item = NewMenuItem::new(format!("batch menu {index}"), 0).insert(&mut conn)?;
        rows.push(Category {
            id: index,
            menu_item_id: item.id,
            seq: 501 - index,
            is_visible: 1,
        });
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
    assert!(Category::resolve_many(&mut conn, vec![])?.is_empty());
    assert_eq!(queries.swap(0, Ordering::Relaxed), 0);
    let repeated = Category::resolve_many(&mut conn, vec![rows[0].clone(); 501])?;
    assert_eq!(repeated.len(), 501);
    assert!(repeated.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(queries.swap(0, Ordering::Relaxed), 1);
    for size in [1, 500, 501] {
        let views = Category::resolve_many(&mut conn, rows[..size].to_vec())?;
        assert_eq!(views.len(), size);
        for (view, row) in views.iter().zip(&rows) {
            assert_eq!(&view.category, row);
            assert_eq!(
                view.menu_item.as_ref().map(|m| m.id),
                Some(row.menu_item_id)
            );
        }
        assert_eq!(
            queries.swap(0, Ordering::Relaxed),
            if size > 500 { 2 } else { 1 }
        );
    }
    diesel::sql_query("ALTER TABLE menuItem RENAME TO relationship_test_menu")
        .execute(&mut conn)?;
    assert!(rows.remove(0).resolve(&mut conn).is_err());
    Ok(())
}

// -- Resolved track relationships --------------------------------------------------------------

#[test]
fn test_content_relationships() -> anyhow::Result<()> {
    use rbox::model_traits::{Model, ModelInsert, ModelRelations};
    use rbox::one_library::*;

    let fixture = common::setup_one_library_path();
    let db = OneLibrary::new(fixture.path())?;
    let mut conn = establish_connection(fixture.path().to_str().unwrap())?;
    let original = Content::all(&mut conn)?
        .into_iter()
        .next()
        .expect("fixture track");
    let mut track = original.clone();
    let artist = NewArtist::new("relation artist").insert(&mut conn)?;
    track.artist_id = Some(artist.id);
    let remixer = NewArtist::new("relation remixer").insert(&mut conn)?;
    track.remixer_id = Some(remixer.id);
    let original_artist = NewArtist::new("relation original_artist").insert(&mut conn)?;
    track.original_artist_id = Some(original_artist.id);
    let composer = NewArtist::new("relation composer").insert(&mut conn)?;
    track.composer_id = Some(composer.id);
    let lyricist = NewArtist::new("relation lyricist").insert(&mut conn)?;
    track.lyricist_id = Some(lyricist.id);
    let album = NewAlbum::new("relationship album").insert(&mut conn)?;
    track.album_id = Some(album.id);
    let genre = NewGenre::new("relationship genre").insert(&mut conn)?;
    track.genre_id = Some(genre.id);
    let label = NewLabel::new("relationship label").insert(&mut conn)?;
    track.label_id = Some(label.id);
    let key = NewKey::new("relationship key").insert(&mut conn)?;
    track.key_id = Some(key.id);
    let color = Color::all(&mut conn)?
        .into_iter()
        .next()
        .expect("fixture color");
    track.color_id = Some(color.id);
    let image = NewImage::new("relationship image").insert(&mut conn)?;
    track.image_id = Some(image.id);
    let view = track.clone().resolve(&mut conn)?;
    assert_eq!(view.content, track);
    assert_eq!(view.artist, Some(artist.clone()));
    assert_eq!(view.remixer, Some(remixer.clone()));
    assert_eq!(view.original_artist, Some(original_artist.clone()));
    assert_eq!(view.composer, Some(composer.clone()));
    assert_eq!(view.lyricist, Some(lyricist.clone()));
    assert_eq!(view.album, Some(album));
    assert_eq!(view.genre, Some(genre));
    assert_eq!(view.label, Some(label));
    assert_eq!(view.key, Some(key));
    assert_eq!(view.color, Some(color));
    assert_eq!(view.image, Some(image));
    let mut shared = track.clone();
    shared.artist_id = Some(artist.id);
    shared.remixer_id = Some(artist.id);
    shared.original_artist_id = Some(artist.id);
    shared.composer_id = Some(artist.id);
    shared.lyricist_id = Some(artist.id);
    let shared = shared.resolve(&mut conn)?;
    assert_eq!(shared.artist, Some(artist.clone()));
    assert_eq!(shared.remixer, Some(artist.clone()));
    assert_eq!(shared.original_artist, Some(artist.clone()));
    assert_eq!(shared.composer, Some(artist.clone()));
    assert_eq!(shared.lyricist, Some(artist.clone()));
    let mut absent = track.clone();
    absent.artist_id = None;
    absent.remixer_id = None;
    absent.original_artist_id = None;
    absent.composer_id = None;
    absent.lyricist_id = None;
    absent.album_id = None;
    absent.genre_id = None;
    absent.label_id = None;
    absent.key_id = None;
    absent.color_id = None;
    absent.image_id = None;
    assert_no_content_relations(&absent.clone().resolve(&mut conn)?);
    let mut dangling = absent.clone();
    dangling.artist_id = Some(i32::MAX);
    dangling.remixer_id = Some(i32::MAX);
    dangling.original_artist_id = Some(i32::MAX);
    dangling.composer_id = Some(i32::MAX);
    dangling.lyricist_id = Some(i32::MAX);
    dangling.album_id = Some(i32::MAX);
    dangling.genre_id = Some(i32::MAX);
    dangling.label_id = Some(i32::MAX);
    dangling.key_id = Some(i32::MAX);
    dangling.color_id = Some(i32::MAX);
    dangling.image_id = Some(i32::MAX);
    let dangling_view = dangling.clone().resolve(&mut conn)?;
    assert_no_content_relations(&dangling_view);
    assert_eq!(dangling_view.content, dangling);
    let inputs = vec![dangling, track.clone(), absent, track];
    let expected = inputs
        .iter()
        .cloned()
        .map(|row| row.resolve(&mut conn))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(Content::resolve_many(&mut conn, inputs)?, expected);

    // Loading supplied snapshots must not update or reload the stored track.
    assert_eq!(Content::find(&mut conn, &original.id)?.unwrap(), original);
    drop(conn);
    assert_eq!(db.get_content_by_id(original.id)?.unwrap(), original);
    let resolved = db.get_content_with_relations(original.id)?.unwrap();
    let mut conn = establish_connection(fixture.path().to_str().unwrap())?;
    assert_eq!(resolved, original.resolve(&mut conn)?);
    drop(conn);
    assert!(db.get_content_with_relations(i32::MAX)?.is_none());
    let raw = db.get_contents()?;
    let views = db.get_contents_with_relations()?;
    assert_eq!(
        views.iter().map(|v| v.content.clone()).collect::<Vec<_>>(),
        raw
    );
    let mut conn = establish_connection(fixture.path().to_str().unwrap())?;
    assert_eq!(views, Content::resolve_many(&mut conn, raw)?);
    Ok(())
}

#[test]
fn test_content_relationship_batches() -> anyhow::Result<()> {
    use diesel::connection::InstrumentationEvent;
    use diesel::{Connection, RunQueryDsl};
    use rbox::model_traits::{Model, ModelInsert, ModelRelations};
    use rbox::one_library::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    let fixture = common::setup_one_library_path();
    let mut conn = establish_connection(fixture.path().to_str().unwrap())?;
    let mut track = Content::all(&mut conn)?
        .into_iter()
        .next()
        .expect("fixture track");
    track.artist_id = None;
    track.remixer_id = None;
    track.original_artist_id = None;
    track.composer_id = None;
    track.lyricist_id = None;
    track.album_id = None;
    track.genre_id = None;
    track.label_id = None;
    track.key_id = None;
    track.color_id = None;
    track.image_id = None;
    let mut rows = Vec::new();
    for index in 0..501 {
        let artist = NewArtist::new(format!("batch artist {index}")).insert(&mut conn)?;
        let mut row = track.clone();
        row.artist_id = Some(artist.id);
        row.remixer_id = Some(artist.id);
        row.original_artist_id = Some(artist.id);
        row.composer_id = Some(artist.id);
        row.lyricist_id = Some(artist.id);
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
    assert!(Content::resolve_many(&mut conn, vec![])?.is_empty());
    assert_eq!(queries.load(Ordering::Relaxed), 0);
    assert_no_content_relations(&track.resolve(&mut conn)?);
    assert_eq!(queries.load(Ordering::Relaxed), 0);
    let repeated = Content::resolve_many(&mut conn, vec![rows[0].clone(); 501])?;
    assert_eq!(queries.swap(0, Ordering::Relaxed), 1);
    assert_eq!(repeated.len(), 501);
    for size in [1, 500, 501] {
        let views = Content::resolve_many(&mut conn, rows[..size].to_vec())?;
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
            assert_eq!(
                view.lyricist.as_ref().map(|a| &a.id),
                row.artist_id.as_ref()
            );
        }
        assert_eq!(
            queries.swap(0, Ordering::Relaxed),
            if size > 500 { 2 } else { 1 }
        );
    }
    // A failed lookup is an error, not an absent relationship.
    diesel::sql_query("ALTER TABLE artist RENAME TO relationship_test_artist")
        .execute(&mut conn)?;
    assert!(rows.remove(0).resolve(&mut conn).is_err());
    Ok(())
}

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
