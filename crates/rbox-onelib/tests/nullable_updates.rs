// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Model updates persist cleared nullable values in a OneLibrary database copy.

use rbox_db::{Model, ModelUpdate};
use rbox_onelib::{connection::establish_connection, Content, OneLibrary};
use rbox_testing::data::setup_one_library_path;

#[test]
fn content_updates_clear_nullable_fields() -> anyhow::Result<()> {
    let fixture = setup_one_library_path();
    let db = OneLibrary::from_path(fixture.path())?;
    let artist = db
        .get_artists()?
        .into_iter()
        .next()
        .expect("fixture artist");
    let mut row = db
        .get_contents()?
        .into_iter()
        .next()
        .expect("fixture track");
    let id = row.id;
    row.title = Some("Nullable update title".into());
    row.bpmx100 = Some(12345);
    row.artist_id = Some(artist.id);
    let mut row = db.update_content(row)?;
    assert!(row.title.is_some());
    assert_eq!(row.bpmx100, Some(12345));
    assert!(row.artist_id.is_some());
    let path = row.path.clone();

    row.title = None;
    row.bpmx100 = None;
    row.artist_id = None;
    let row = db.update_content(row)?;
    assert_eq!(row.title, None);
    assert_eq!(row.bpmx100, None);
    assert_eq!(row.artist_id, None);
    assert_eq!(row.path, path);
    assert_eq!(db.get_content_by_id(id)?, Some(row.clone()));

    let mut conn = establish_connection(fixture.path(), &Default::default())?;
    let mut row = row;
    row.title = Some("Restored title".into());
    let mut row = row.update(&mut conn)?;
    assert_eq!(row.title.as_deref(), Some("Restored title"));
    row.title = None;
    let row = row.update(&mut conn)?;
    assert_eq!(row.title, None);
    assert_eq!(Content::find(&mut conn, &id)?.unwrap().title, None);
    Ok(())
}
