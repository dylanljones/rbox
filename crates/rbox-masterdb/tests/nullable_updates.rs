// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Model updates persist cleared nullable values in RB6 and RB7 database copies.

use rbox_db::{Model, ModelUpdate};
use rbox_masterdb::{AgentRegistry, DjmdContent, MasterDb};
use rbox_testing::data::setup_master_db_path;

#[test]
fn content_updates_clear_nullable_fields() -> anyhow::Result<()> {
    for version in [6, 7] {
        let fixture = setup_master_db_path(version);
        let mut db = MasterDb::new(fixture.path())?;
        db.set_unsafe_writes(true);
        let artist = db.create_artist("Nullable update artist".into())?;
        let mut row = db
            .get_contents()?
            .into_iter()
            .next()
            .expect("fixture track");
        let id = row.id.clone();
        let path = row.folder_path.clone();
        row.title = Some("Nullable update title".into());
        row.bpm = Some(12345);
        row.artist_id = Some(artist.id);
        db.update_content(&row)?;
        let mut row = db.get_content_by_id(&id)?.expect("updated track");
        assert!(row.title.is_some());
        assert_eq!(row.bpm, Some(12345));
        assert!(row.artist_id.is_some());

        let mut conn = db.pool.get()?;
        let usn = AgentRegistry::local_usn(&mut conn)?;
        row.title = None;
        row.bpm = None;
        row.artist_id = None;
        db.update_content(&row)?;
        let row = DjmdContent::find(&mut conn, &id)?.expect("cleared track");
        assert_eq!(row.title, None);
        assert_eq!(row.bpm, None);
        assert_eq!(row.artist_id, None);
        assert_eq!(row.folder_path, path);
        assert_eq!(row.rb_local_usn, Some(usn + 1));
        assert_eq!(AgentRegistry::local_usn(&mut conn)?, usn + 1);

        // The trait-based API also persists None and can restore cleared values.
        let mut row = row;
        row.title = Some("Restored title".into());
        let mut row = ModelUpdate::update(row, &mut conn)?;
        assert_eq!(row.title.as_deref(), Some("Restored title"));
        row.title = None;
        let row = ModelUpdate::update(row, &mut conn)?;
        assert_eq!(row.title, None);
        assert_eq!(DjmdContent::find(&mut conn, &id)?.unwrap().title, None);
    }
    Ok(())
}
