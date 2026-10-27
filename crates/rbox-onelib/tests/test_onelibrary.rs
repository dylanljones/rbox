// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! One Library integration tests using shared fixture copies.

use diesel::connection::InstrumentationEvent;
use diesel::{Connection, RunQueryDsl};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tempfile::NamedTempFile;

use rbox_db::{Model, ModelDelete, ModelInsert, ModelRelations};
use rbox_onelib::connection::{establish_connection, ConnectionOptions};
use rbox_onelib::database::OneLibrary;
use rbox_onelib::models::*;
use rbox_testing::data::{get_temp_root_dir, setup_one_library_path};

#[test]
fn content_master_identity_lookup_is_scoped_to_source_library() -> anyhow::Result<()> {
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    let library = -795_645_379i64;
    let mut inserted = Vec::new();
    for (index, (source_content, source_library)) in [
        (Some(2_000_000_123), Some(library)),
        (Some(2_000_000_123), Some(42)),
        (Some(2_000_000_124), Some(library)),
        (Some(2_000_000_123), Some(library)),
        (None, None),
        (Some(0), Some(0)),
        (Some(i64::from(i32::MIN)), Some(library)),
    ]
    .into_iter()
    .enumerate()
    {
        let mut row = NewContent::new(format!("/master-identity/{index}.wav"));
        row.file_type = Some(1);
        row.master_content_id = source_content;
        row.master_db_id = source_library;
        inserted.push(row.insert(&mut conn)?);
    }
    assert_eq!(
        Content::find_by_master_content_id(&mut conn, 2_000_000_123, library)?
            .unwrap()
            .id,
        inserted[0].id
    );
    assert_eq!(
        Content::find_by_master_content_id(&mut conn, 2_000_000_123, 42)?
            .unwrap()
            .id,
        inserted[1].id
    );
    assert_eq!(
        Content::find_by_master_content_id(&mut conn, 2_000_000_124, library)?
            .unwrap()
            .id,
        inserted[2].id
    );
    assert!(inserted[0].master_db_id.unwrap() < 0);
    assert!(Content::find_by_master_content_id(&mut conn, 2_000_000_123, 999)?.is_none());
    assert!(Content::find_by_master_content_id(&mut conn, 2_000_000_125, library)?.is_none());
    assert_eq!(
        Content::find_by_master_content_id(&mut conn, i64::from(i32::MIN), library)?
            .unwrap()
            .id,
        inserted[6].id
    );
    assert!(Content::find_by_master_content_id(&mut conn, 0, 0)?.is_none());
    let db = OneLibrary::from_path(fixture.path())?;
    for (source_content, source_library) in [
        (2_000_000_123, library),
        (2_000_000_123, 42),
        (2_000_000_124, library),
        (i64::from(i32::MIN), library),
        (2_000_000_123, 999),
        (0, 0),
    ] {
        assert_eq!(
            db.get_content_by_master_content_id(source_content, source_library)?,
            Content::find_by_master_content_id(&mut conn, source_content, source_library)?
        );
    }
    // More than two batches, with matches at either end of the sorted source IDs.
    let mut source_ids = vec![
        2_000_000_124,
        2_000_000_123,
        i64::from(i32::MIN),
        0,
        2_000_000_123,
    ];
    source_ids.extend(1000..2200);
    let batch = Content::by_master_content_ids(&mut conn, &source_ids, library)?;
    assert_eq!(
        batch.iter().map(|row| row.id).collect::<Vec<_>>(),
        vec![
            inserted[0].id,
            inserted[2].id,
            inserted[3].id,
            inserted[6].id
        ]
    );
    assert_eq!(
        db.get_contents_by_master_content_ids(&source_ids, library)?,
        batch
    );
    assert_eq!(
        db.get_contents_by_master_content_ids(&source_ids, 42)?,
        vec![inserted[1].clone()]
    );
    for (ids, source_library) in [
        (&[][..], library),
        (&[0][..], library),
        (&source_ids[..], 0),
        (&source_ids[..], 999),
    ] {
        assert!(Content::by_master_content_ids(&mut conn, ids, source_library)?.is_empty());
        assert!(db
            .get_contents_by_master_content_ids(ids, source_library)?
            .is_empty());
    }
    Content::delete(&mut conn, &inserted[0].id)?;
    assert_eq!(
        Content::find_by_master_content_id(&mut conn, 2_000_000_123, library)?
            .unwrap()
            .id,
        inserted[3].id
    );
    Ok(())
}

fn assert_no_content_relations(view: &ContentView) {
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

#[test]
fn property_names_are_unique_model_keys() -> anyhow::Result<()> {
    use diesel::prelude::*;
    use diesel::result::{DatabaseErrorKind, Error as DieselError};
    use rbox_onelib::schema::{content, property};
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    let original = Property::new("unique property", 1).insert(&mut conn)?;
    assert!(Property::id_exists(&mut conn, "unique property")?);
    assert!(!Property::id_exists(&mut conn, "UNIQUE PROPERTY")?);
    assert!(matches!(
        original.clone().insert(&mut conn),
        Err(DieselError::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            _
        ))
    ));
    assert!(Property::insert_all(
        &mut conn,
        vec![Property::new("batch property", 1), original.clone()]
    )
    .is_err());
    assert!(!Property::id_exists(&mut conn, "batch property")?);
    // Native schemas need not contain a SQL UNIQUE constraint. Direct writes can
    // introduce ambiguity, but models must never silently select/update one copy.
    diesel::insert_into(property::table)
        .values(original.clone())
        .execute(&mut conn)?;
    assert!(matches!(
        Property::find(&mut conn, "unique property"),
        Err(DieselError::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            _
        ))
    ));
    assert!(Property::first(&mut conn).is_err());
    assert!(Property::set_number_of_contents_all(&mut conn, 123).is_err());
    let count: i64 = content::table.count().get_result(&mut conn)?;
    let mut new = NewContent::new("/duplicate-property.wav");
    new.file_type = Some(1);
    assert!(new.insert(&mut conn).is_err());
    assert_eq!(content::table.count().get_result::<i64>(&mut conn)?, count);
    Ok(())
}

#[test]
fn content_count_can_be_set_and_recalculated() -> anyhow::Result<()> {
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    Property::new("Additional count property", 0).insert(&mut conn)?;
    let count = u32::try_from(Content::all(&mut conn)?.len())?;
    Property::set_content_count(&mut conn, 123)?;
    assert!(Property::all(&mut conn)?
        .iter()
        .all(|p| p.number_of_contents == 123));
    assert!(Property::set_content_count(&mut conn, u32::MAX).is_err());
    assert!(Property::all(&mut conn)?
        .iter()
        .all(|p| p.number_of_contents == 123));
    assert_eq!(Property::update_content_count(&mut conn)?, count);
    assert!(Property::all(&mut conn)?
        .iter()
        .all(|p| p.number_of_contents == count as i32));
    Ok(())
}

#[test]
fn default_content_count_setter_updates_one_property_and_handles_empty_table() -> anyhow::Result<()>
{
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    Property::new("Additional property", 0).insert(&mut conn)?;
    Property::set_number_of_contents_all(&mut conn, 0)?;
    assert_eq!(Property::set_number_of_contents_default(&mut conn, 123)?, 1);
    let rows = Property::all(&mut conn)?;
    assert_eq!(
        rows.iter()
            .filter(|row| row.number_of_contents == 123)
            .count(),
        1
    );
    for row in rows {
        Property::delete(&mut conn, &row.device_name)?;
    }
    assert_eq!(Property::set_number_of_contents_default(&mut conn, 123)?, 0);
    Property::set_content_count(&mut conn, 123)?;
    Property::update_content_count(&mut conn)?;
    assert!(Property::all(&mut conn)?.is_empty());
    Ok(())
}

#[test]
fn content_mutations_maintain_counts_and_rollback_when_properties_fail() -> anyhow::Result<()> {
    fn new_content(path: &str) -> NewContent {
        let mut content = NewContent::new(path);
        content.file_type = Some(1);
        content
    }
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    Property::new("Count mutation test", 0).insert(&mut conn)?;
    let baseline = Content::all(&mut conn)?.len();
    let first = new_content("/count-test/one.wav").insert(&mut conn)?;
    assert!(Property::all(&mut conn)?
        .iter()
        .all(|p| p.number_of_contents as usize == baseline + 1));
    let batch = NewContent::insert_all(
        &mut conn,
        vec![
            new_content("/count-test/two.wav"),
            new_content("/count-test/three.wav"),
        ],
    )?;
    assert!(Property::all(&mut conn)?
        .iter()
        .all(|p| p.number_of_contents as usize == baseline + 3));
    diesel::sql_query("CREATE TEMP TRIGGER reject_count BEFORE UPDATE OF numberOfContents ON property BEGIN SELECT RAISE(ABORT, 'count failure'); END").execute(&mut conn)?;
    let before = Content::all(&mut conn)?;
    assert!(new_content("/count-test/fail.wav")
        .insert(&mut conn)
        .is_err());
    assert!(
        NewContent::insert_all(&mut conn, vec![new_content("/count-test/fail-batch.wav")]).is_err()
    );
    assert!(Content::delete(&mut conn, &first.id).is_err());
    assert!(Content::delete_all(&mut conn, batch.iter().map(|c| &c.id).collect()).is_err());
    assert_eq!(Content::all(&mut conn)?, before);
    assert!(Property::all(&mut conn)?
        .iter()
        .all(|p| p.number_of_contents as usize == baseline + 3));
    diesel::sql_query("DROP TRIGGER reject_count").execute(&mut conn)?;
    Content::delete(&mut conn, &first.id)?;
    assert!(Property::all(&mut conn)?
        .iter()
        .all(|p| p.number_of_contents as usize == baseline + 2));
    Content::delete_all(&mut conn, batch.iter().map(|c| &c.id).collect())?;
    assert!(Property::all(&mut conn)?
        .iter()
        .all(|p| p.number_of_contents as usize == baseline));
    Ok(())
}

// -- Resolved album relationships --------------------------------------------------------------

#[test]
fn test_album_relationships() -> anyhow::Result<()> {
    let fixture = setup_one_library_path();
    let db = OneLibrary::from_path(fixture.path())?;
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
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
    dangling.artist_id = Some(i64::MAX);
    dangling.image_id = Some(i64::MAX);
    let dangling_view = AlbumView {
        album: dangling.clone(),
        artist: None,
        image: None,
    };
    assert_eq!(dangling.clone().resolve(&mut conn)?, dangling_view);
    // Missing one target must not hide another relationship.
    let mut partial = album.clone();
    partial.artist_id = Some(i64::MAX);
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
    assert!(db.get_album_with_relations(i64::MAX)?.is_none());
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
    let fixture = setup_one_library_path();

    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
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
    let fixture = setup_one_library_path();
    let db = OneLibrary::from_path(fixture.path())?;
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    let item = NewMenuItem::new("resolved menu", 0).insert(&mut conn)?;
    let category = NewCategory::new(item.id).insert(&mut conn)?;
    let expected = CategoryView {
        category: category.clone(),
        menu_item: Some(item),
    };
    assert_eq!(category.clone().resolve(&mut conn)?, expected);
    let mut dangling = category.clone();
    dangling.menu_item_id = i64::MAX;
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
    assert!(db.get_category_with_relations(i64::MAX)?.is_none());
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
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    let mut rows = Vec::new();
    for index in 0..501 {
        let item = NewMenuItem::new(format!("batch menu {index}"), 0).insert(&mut conn)?;
        rows.push(Category {
            id: index,
            menu_item_id: item.id,
            seq: (501 - index) as i32,
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
    let fixture = setup_one_library_path();
    let db = OneLibrary::from_path(fixture.path())?;
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
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
    dangling.artist_id = Some(i64::MAX);
    dangling.remixer_id = Some(i64::MAX);
    dangling.original_artist_id = Some(i64::MAX);
    dangling.composer_id = Some(i64::MAX);
    dangling.lyricist_id = Some(i64::MAX);
    dangling.album_id = Some(i64::MAX);
    dangling.genre_id = Some(i64::MAX);
    dangling.label_id = Some(i64::MAX);
    dangling.key_id = Some(i64::MAX);
    dangling.color_id = Some(i64::MAX);
    dangling.image_id = Some(i64::MAX);
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
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    assert_eq!(resolved, original.resolve(&mut conn)?);
    drop(conn);
    assert!(db.get_content_with_relations(i64::MAX)?.is_none());
    let raw = db.get_contents()?;
    let views = db.get_contents_with_relations()?;
    assert_eq!(
        views.iter().map(|v| v.content.clone()).collect::<Vec<_>>(),
        raw
    );
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    assert_eq!(views, Content::resolve_many(&mut conn, raw)?);
    Ok(())
}

#[test]
fn test_content_relationship_batches() -> anyhow::Result<()> {
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
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
    let tmp_root = get_temp_root_dir();
    let tmp = NamedTempFile::new_in(tmp_root).expect("Failed to create temp file");
    tmp
}

#[test]
fn test_open() -> anyhow::Result<()> {
    let db_path = setup_one_library_path();
    let _db = OneLibrary::from_path(db_path.path())?;
    Ok(())
}

#[test]
fn test_open_directory_and_explicit_file() -> anyhow::Result<()> {
    let fixture = setup_one_library_path();
    let root = tempfile::tempdir()?;
    let standard = root.path().join("exportLibrary.db");
    let renamed = root.path().join("backup.db");
    std::fs::copy(fixture.path(), &standard)?;
    std::fs::copy(fixture.path(), &renamed)?;
    let directory_db = OneLibrary::new(root.path())?;
    let explicit_db = OneLibrary::from_path(&renamed)?;
    assert_eq!(
        directory_db.get_contents()?.len(),
        explicit_db.get_contents()?.len()
    );
    assert!(OneLibrary::new(&standard).is_err());
    assert!(matches!(
        OneLibrary::from_path(root.path()),
        Err(rbox_onelib::Error::FileNotFound(_))
    ));
    Ok(())
}

#[test]
fn test_open_missing_database_does_not_create_it() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let expected = root.path().join("exportLibrary.db");
    for result in [
        OneLibrary::new(root.path()),
        OneLibrary::from_path(&expected),
    ] {
        assert!(matches!(result, Err(rbox_onelib::Error::FileNotFound(path)) if path == expected));
    }
    assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    Ok(())
}

#[test]
fn test_creation_reopens_explicit_file_path() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("custom.db");
    // Creation currently initializes an existing empty file, as in test_create.
    std::fs::File::create(&path)?;
    let created = OneLibrary::create(&path, 42)?;
    assert_eq!(
        created.get_first_property()?.unwrap().my_tag_master_dbid,
        42
    );
    drop(created);
    let reopened = OneLibrary::create_if_not_exists(&path, 99)?;
    assert_eq!(
        reopened.get_first_property()?.unwrap().my_tag_master_dbid,
        42
    );
    assert!(!root.path().join("exportLibrary.db").exists());
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
    let db_path = setup_one_library_path();
    let db = OneLibrary::from_path(db_path.path())?;
    let n = db.get_albums()?.len() as i64;
    let model = db.create_album("album1", None, None)?;
    assert_eq!(model.id, n + 1);
    Ok(())
}

#[test]
fn test_get_albums() -> anyhow::Result<()> {
    let db_path = setup_one_library_path();
    let db = OneLibrary::from_path(db_path.path())?;

    let res = db.get_albums()?;
    assert_eq!(res.len(), 0);

    db.create_album("album1", None, None)?;
    let res = db.get_albums()?;
    assert_eq!(res.len(), 1);
    Ok(())
}

#[test]
fn test_get_album_by_id() -> anyhow::Result<()> {
    let db_path = setup_one_library_path();
    let db = OneLibrary::from_path(db_path.path())?;
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
    let db_path = setup_one_library_path();
    let db = OneLibrary::from_path(db_path.path())?;
    db.create_album("album1", None, None)?;

    let res = db.get_album_by_name("album1")?;
    assert!(res.is_some());

    let res = db.get_album_by_name("album2")?;
    assert!(res.is_none());
    Ok(())
}

#[test]
fn test_delete_album() -> anyhow::Result<()> {
    let db_path = setup_one_library_path();
    let db = OneLibrary::from_path(db_path.path())?;
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
    let db_path = setup_one_library_path();
    let db = OneLibrary::from_path(db_path.path())?;
    let n = db.get_artists()?.len() as i64;
    let model = db.create_artist("artist1")?;
    assert_eq!(model.id, n + 1);
    Ok(())
}

#[test]
fn test_get_artists() -> anyhow::Result<()> {
    let db_path = setup_one_library_path();
    let db = OneLibrary::from_path(db_path.path())?;

    let res = db.get_artists()?;
    assert_eq!(res.len(), 1);

    db.create_artist("artist1")?;
    let res = db.get_artists()?;
    assert_eq!(res.len(), 2);
    Ok(())
}

#[test]
fn test_get_artist_by_id() -> anyhow::Result<()> {
    let db_path = setup_one_library_path();
    let db = OneLibrary::from_path(db_path.path())?;
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
    let db_path = setup_one_library_path();
    let db = OneLibrary::from_path(db_path.path())?;
    db.create_artist("artist1")?;

    let res = db.get_artist_by_name("artist1")?;
    assert!(res.is_some());

    let res = db.get_artist_by_name("artist2")?;
    assert!(res.is_none());
    Ok(())
}

#[test]
fn test_delete_artist() -> anyhow::Result<()> {
    let db_path = setup_one_library_path();
    let db = OneLibrary::from_path(db_path.path())?;
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

#[test]
fn shared_track_conversions_preserve_metadata_and_resolved_names() -> anyhow::Result<()> {
    use rbox_core::{Track, TrackSource};
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    let artist = NewArtist::new("Mapping artist").insert(&mut conn)?;
    let image = NewImage::new("/art.jpg").insert(&mut conn)?;
    let mut track = Track::new(r"/Contents/Artist\track.wav");
    track.id = Some(999);
    track.title = Some("Mapping track".into());
    track.search_str = Some("search title".into());
    track.file_type = Some(11);
    track.file_size = Some(123456);
    track.bpm = Some(123.456);
    track.length = Some(123.6);
    track.rating = Some(5);
    track.master_content_id = Some(123);
    track.master_db_id = Some(456);
    track.date_created = Some("2026-01-02".into());
    track.stock_date = Some("2026-02-03".into());
    track.analysis_data_path = Some(r"/PIONEER\USBANLZ/ANLZ0000.DAT".into());
    track.hot_cue_auto_load = Some("On".into());
    track.delivery_control = Some("7".into());
    track.artist = Some("not automatically resolved".into());
    let mut new = NewContent::try_from(&track)?;
    assert_eq!(new.path, "/Contents/Artist/track.wav");
    assert_eq!(new.file_name.as_deref(), Some("track.wav"));
    assert_eq!(new.bpmx100, Some(12346));
    assert_eq!(new.length, Some(124));
    assert_eq!(new.artist_id, None);
    assert_eq!(new.image_id, None);
    assert_eq!(new.is_hot_cue_auto_load_on, Some(1));
    new.artist_id = Some(artist.id);
    new.lyricist_id = Some(artist.id);
    new.image_id = Some(image.id);
    let view = new.insert(&mut conn)?.resolve(&mut conn)?;
    let mapped = Track::try_from(&view)?;
    assert_eq!(mapped.source, TrackSource::OneLib);
    assert_eq!(mapped.artist.as_deref(), Some("Mapping artist"));
    assert_eq!(mapped.lyricist.as_deref(), Some("Mapping artist"));
    assert_eq!(mapped.artwork_path, Some("/art.jpg".into()));
    assert_eq!(mapped.date_created, track.date_created);
    assert_eq!(mapped.stock_date, track.stock_date);
    assert_eq!(mapped.master_db_id, track.master_db_id);
    assert_eq!(mapped.title, track.title);
    assert_eq!(mapped.search_str, track.search_str);
    assert_eq!(mapped.bpm, Some(123.46));
    assert_eq!(mapped.length, Some(124.0));
    assert_eq!(mapped.hot_cue_auto_load.as_deref(), Some("1"));
    assert_eq!(mapped.delivery_control.as_deref(), Some("7"));
    assert_eq!(
        mapped.analysis_data_path,
        Some("/PIONEER/USBANLZ/ANLZ0000.DAT".into())
    );
    assert!(mapped.album.is_none());
    assert!(mapped.album_artist.is_none());
    let back = NewContent::try_from(mapped)?;
    assert_eq!(back.file_size, Some(123456));
    assert_eq!(back.is_kuvo_deliver_status_on, Some(7));
    assert_eq!(Track::try_from(view)?.source, TrackSource::OneLib);
    Ok(())
}

#[test]
fn shared_track_conversions_reject_invalid_values() -> anyhow::Result<()> {
    use rbox_core::Track;
    let track = Track::new("/Contents/track.wav");
    for value in [f64::NAN, f64::INFINITY, -1.0, f64::from(i32::MAX)] {
        let mut input = track.clone();
        input.bpm = Some(value);
        assert!(NewContent::try_from(input).is_err());
    }
    let mut input = track.clone();
    input.length = Some(f64::from(i32::MAX) + 1.0);
    assert!(NewContent::try_from(input).is_err());
    let mut input = track.clone();
    input.file_size = Some(i64::MAX as u64 + 1);
    assert!(NewContent::try_from(input).is_err());
    let mut input = track.clone();
    input.rating = Some(6);
    assert!(NewContent::try_from(input).is_err());
    let mut input = track.clone();
    input.master_content_id = Some(u32::MAX);
    assert_eq!(
        NewContent::try_from(input)?.master_content_id,
        Some(i64::from(u32::MAX))
    );
    let mut input = track.clone();
    input.stock_date = Some("not a date".into());
    assert!(NewContent::try_from(input).is_err());
    let mut input = track.clone();
    input.hot_cue_auto_load = Some("unknown".into());
    assert!(NewContent::try_from(input).is_err());
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    let view = NewContent::try_from(track)?
        .insert(&mut conn)?
        .resolve(&mut conn)?;
    assert!(Track::try_from(&view)?.rating.is_none());
    for value in [i64::from(i32::MIN) - 1, i64::from(u32::MAX) + 1] {
        let mut invalid = view.clone();
        invalid.content.master_db_id = Some(value);
        assert!(Track::try_from(invalid).is_err());
    }
    for value in [i64::from(u32::MAX), i64::from(i32::MAX) + 1] {
        let mut valid = view.clone();
        valid.content.master_db_id = Some(value);
        valid.content.master_content_id = Some(value);
        let mapped = Track::try_from(valid)?;
        assert_eq!(mapped.master_db_id, Some(value as u32));
        assert_eq!(mapped.master_content_id, Some(value as u32));
    }
    let mut invalid = view.clone();
    invalid.content.file_size = Some(-1);
    assert!(Track::try_from(invalid).is_err());
    let mut invalid = view.clone();
    invalid.content.bit_depth = Some(65536);
    assert!(Track::try_from(invalid).is_err());
    let mut invalid = view.clone();
    invalid.content.rating = Some(6);
    assert!(Track::try_from(invalid).is_err());
    let mut invalid = view;
    invalid.content.color_id = Some(256);
    assert!(Track::try_from(invalid).is_err());
    Ok(())
}

#[test]
fn content_album_artist_resolves_independently_and_handles_missing_targets() -> anyhow::Result<()> {
    use rbox_core::Track;
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    let artist = NewArtist::new("Content artist").insert(&mut conn)?;
    let album_artist = NewArtist::new("Album artist").insert(&mut conn)?;
    let album = NewAlbum::new("Album artist test")
        .artist_id(album_artist.id)
        .insert(&mut conn)?;
    let no_artist = NewAlbum::new("No artist").insert(&mut conn)?;
    let mut new = NewContent::new("/Contents/album-artist.wav");
    new.file_type = Some(11);
    new.artist_id = Some(artist.id);
    new.album_id = Some(album.id);
    let row = new.insert(&mut conn)?;
    let view = row.clone().resolve(&mut conn)?;
    assert_eq!(view.artist, Some(artist));
    assert_eq!(view.album_artist, Some(album_artist.clone()));
    assert_eq!(
        Track::try_from(view)?.album_artist.as_deref(),
        Some("Album artist")
    );
    let mut missing_album = row.clone();
    missing_album.album_id = Some(i64::MAX);
    let mut absent_album = row.clone();
    absent_album.album_id = None;
    let mut absent_artist = row.clone();
    absent_artist.album_id = Some(no_artist.id);
    let views = Content::resolve_many(
        &mut conn,
        vec![
            row.clone(),
            missing_album,
            absent_album,
            absent_artist,
            row.clone(),
        ],
    )?;
    assert_eq!(views[0].album_artist, Some(album_artist.clone()));
    assert_eq!(views[4], views[0]);
    for view in &views[1..4] {
        assert!(view.album_artist.is_none());
        assert!(view.artist.is_some());
    }
    // Simulate a dangling artist reference without modifying a source fixture.
    diesel::sql_query("DELETE FROM artist WHERE artist_id = ?")
        .bind::<diesel::sql_types::BigInt, _>(album_artist.id)
        .execute(&mut conn)?;
    let view = row.resolve(&mut conn)?;
    assert!(view.album.is_some());
    assert!(view.album_artist.is_none());
    assert!(view.artist.is_some());
    Ok(())
}

#[test]
fn shared_track_handler_queries() -> anyhow::Result<()> {
    use rbox_core::TrackSource;
    let fixture = setup_one_library_path();
    let db = OneLibrary::from_path(fixture.path())?;
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    let artist = NewArtist::new("Handler album artist").insert(&mut conn)?;
    let album = NewAlbum::new("Handler album")
        .artist_id(artist.id)
        .insert(&mut conn)?;
    let mut new = NewContent::new("/Contents/handler/track.wav");
    new.file_type = Some(11);
    new.album_id = Some(album.id);
    new.bpmx100 = Some(12345);
    let content = new.insert(&mut conn)?;
    let track = db.get_track_by_id(content.id)?.unwrap();
    assert_eq!(track.id, Some(content.id as u32));
    assert_eq!(track.source, TrackSource::OneLib);
    assert_eq!(track.album_artist.as_deref(), Some("Handler album artist"));
    assert_eq!(track.bpm, Some(123.45));
    assert_eq!(
        db.get_track_by_path(r"\Contents\handler\track.wav")?
            .unwrap()
            .id,
        track.id
    );
    assert!(db.get_track_by_id(i64::MAX)?.is_none());
    assert!(db.get_track_by_path("/not-present.wav")?.is_none());
    assert!(db.get_tracks_by_ids(&[])?.is_empty());
    let mut ids = vec![content.id; 1001];
    ids.extend(1_000_000..1_001_001);
    let found = db.get_tracks_by_ids(&ids)?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, track.id);
    assert!(db.get_tracks()?.iter().any(|row| row.id == track.id));
    diesel::sql_query("UPDATE content SET rating = 6 WHERE content_id = ?")
        .bind::<diesel::sql_types::BigInt, _>(content.id)
        .execute(&mut conn)?;
    assert!(db.get_track_by_id(content.id).is_err());
    assert!(db.get_track_by_path(&content.path).is_err());
    assert!(db.get_tracks_by_ids(&[content.id]).is_err());
    assert!(db.get_tracks().is_err());
    Ok(())
}

#[test]
fn shared_track_master_library_id_preserves_unsigned_identity() -> anyhow::Result<()> {
    use rbox_core::Track;
    let fixture = setup_one_library_path();
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default())?;
    for (unsigned, signed) in [
        (0u32, 0),
        (2147483647u32, i64::from(i32::MAX)),
        (2147483648u32, i64::from(i32::MIN)),
        (3499321917u32, -795645379),
        (4294967295u32, -1),
    ] {
        let mut track = Track::new("/Contents/identity.wav");
        track.master_db_id = Some(unsigned);
        let new = NewContent::try_from(&track)?;
        assert_eq!(new.master_db_id, Some(signed));
        let view = new.insert(&mut conn)?.resolve(&mut conn)?;
        assert_eq!(Track::try_from(view)?.master_db_id, Some(unsigned));
    }

    Ok(())
}
