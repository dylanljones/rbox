// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Local track defaults, explicit overrides, and transaction rollback.

use diesel::prelude::*;
use rbox_db::{Model, ModelInsert};
use rbox_masterdb::schema::djmdProperty;
use rbox_masterdb::{
    AgentRegistry, DjmdArtist, DjmdContent, MasterDb, NewDjmdArtist, NewDjmdContent,
};
use rbox_testing::data::{create_temp_dir, setup_master_db_path};

#[test]
fn local_import_defaults_and_overrides() -> anyhow::Result<()> {
    for version in [6, 7] {
        let fixture = setup_master_db_path(version);
        let db = MasterDb::new(fixture.path())?;
        let mut conn = db.pool.get()?;
        let files = create_temp_dir();
        let path = files.path().join("defaults.mp3");
        std::fs::write(&path, b"fixture")?;
        let identity: (String, String) = djmdProperty::table
            .select((djmdProperty::db_id, djmdProperty::device_id))
            .first(&mut conn)?;
        let row = NewDjmdContent::new(path.to_string_lossy()).insert(&mut conn)?;
        assert_eq!(row.master_db_id.as_deref(), Some(identity.0.as_str()));
        assert_eq!(row.device_id.as_deref(), Some(identity.1.as_str()));
        assert_eq!(row.master_song_id.as_ref(), Some(&row.id));
        for text in [
            &row.delivery_control,
            &row.modified_by_rbm,
            &row.org_folder_path,
            &row.reserved1,
            &row.delivery_comment,
            &row.subtitle,
            &row.lyricist,
        ] {
            assert_eq!(text.as_deref(), Some(""));
        }
        assert_eq!(row.ext_info.as_deref(), Some("null"));
        assert_eq!(row.color_id.as_deref(), Some("0"));
        assert_eq!(row.video_associate.as_deref(), Some("0"));
        for value in [
            row.rating,
            row.disc_no,
            row.dj_play_count,
            row.sampler_track_info,
            row.sampler_play_offset,
            row.lyric_status,
            row.service_id,
        ] {
            assert_eq!(value, Some(0));
        }
        assert_eq!(row.sampler_gain, Some(0.));
        assert_eq!(row.content_link, None);
        assert_eq!(row.usn, None);
        let path = files.path().join("overrides.mp3");
        std::fs::write(&path, b"fixture")?;
        let row = NewDjmdContent {
            folder_path: path.to_string_lossy().into_owned(),
            rating: Some(4),
            disc_no: Some(2),
            dj_play_count: Some(7),
            color_id: Some("3".into()),
            video_associate: Some("custom".into()),
            ext_info: Some("{}".into()),
            subtitle: Some("mix".into()),
            lyricist: Some("writer".into()),
            delivery_comment: Some("comment".into()),
            org_folder_path: Some("/original".into()),
            reserved1: Some("reserved".into()),
            sampler_track_info: Some(2),
            sampler_play_offset: Some(3),
            sampler_gain: Some(0.5),
            lyric_status: Some(1),
            content_link: Some(42),
            ..Default::default()
        }
        .insert(&mut conn)?;
        assert_eq!(
            (row.rating, row.disc_no, row.dj_play_count),
            (Some(4), Some(2), Some(7))
        );
        assert_eq!(row.content_link, Some(42));
        assert_eq!(row.ext_info.as_deref(), Some("{}"));
        assert_eq!(row.color_id.as_deref(), Some("3"));
        assert_eq!(row.video_associate.as_deref(), Some("custom"));
        assert_eq!(row.subtitle.as_deref(), Some("mix"));
        assert_eq!(row.lyricist.as_deref(), Some("writer"));
        assert_eq!(row.delivery_comment.as_deref(), Some("comment"));
        assert_eq!(row.org_folder_path.as_deref(), Some("/original"));
        assert_eq!(row.reserved1.as_deref(), Some("reserved"));
        assert_eq!(
            (
                row.sampler_track_info,
                row.sampler_play_offset,
                row.lyric_status
            ),
            (Some(2), Some(3), Some(1))
        );
        assert_eq!(row.sampler_gain, Some(0.5));
    }
    Ok(())
}

#[test]
fn outer_transaction_rolls_back_dependencies_content_and_counter() -> anyhow::Result<()> {
    for version in [6, 7] {
        let fixture = setup_master_db_path(version);
        let db = MasterDb::new(fixture.path())?;
        let mut conn = db.pool.get()?;
        let files = create_temp_dir();
        let path = files.path().join("rollback.mp3");
        std::fs::write(&path, b"fixture")?;
        let before = AgentRegistry::local_usn(&mut conn)?;
        let count = DjmdContent::all(&mut conn)?.len();
        let result: diesel::QueryResult<()> = conn.immediate_transaction(|conn| {
            let artist = NewDjmdArtist::new("__import_rollback__").insert(conn)?;
            NewDjmdContent::new(path.to_string_lossy())
                .artist_id(artist.id)
                .insert(conn)?;
            Err(diesel::result::Error::RollbackTransaction)
        });
        assert!(result.is_err());
        assert_eq!(AgentRegistry::local_usn(&mut conn)?, before);
        assert_eq!(DjmdContent::all(&mut conn)?.len(), count);
        assert!(DjmdArtist::find_by_name(&mut conn, "__import_rollback__")?.is_none());
    }
    Ok(())
}
