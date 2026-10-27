// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Resolved track helpers, conversion, lookup semantics, and conversion errors on RB6/RB7.

use diesel::RunQueryDsl;
use rbox_core::{Track, TrackMetadata, TrackSource};
use rbox_db::{Model, ModelInsert};
use rbox_masterdb::{
    DjmdArtist, DjmdContent, DjmdContentView, DjmdProperty, MasterDb, NewDjmdAlbum, NewDjmdArtist,
    NewDjmdContent,
};
use rbox_testing::data::setup_master_db_path;

#[test]
fn converts_units_and_unsigned_fields() {
    let metadata = TrackMetadata {
        bpm: Some(128.126),
        length: Some(61.75),
        rating: Some(5),
        bit_depth: Some(24),
        sample_rate: Some(192000),
        track_no: Some(u32::MAX),
        disc_no: Some(2),
        release_year: Some(2026),
        bit_rate: Some(320),
        ..TrackMetadata::new("track.wav")
    };
    let row = NewDjmdContent::try_from(&metadata).unwrap();
    assert_eq!(row.bpm, Some(12813));
    assert_eq!(row.length, Some(62));
    assert_eq!(row.rating, Some(5));
    assert_eq!(row.bit_depth, Some(24));
    assert_eq!(row.sample_rate, Some(192000));
    assert_eq!(row.track_no, Some(i64::from(u32::MAX)));
    assert_eq!(row.disc_no, Some(2));
    assert_eq!(row.release_year, Some(2026));
    assert_eq!(row.bit_rate, Some(320));
}

#[test]
fn rejects_invalid_floats_and_integer_overflow() {
    for value in [f64::NAN, f64::INFINITY, -1.0, f64::MAX] {
        let mut metadata = TrackMetadata::new("track.wav");
        metadata.bpm = Some(value);
        assert!(NewDjmdContent::try_from(&metadata).is_err());
        metadata.bpm = None;
        metadata.length = Some(value);
        assert!(NewDjmdContent::try_from(&metadata).is_err());
    }
    let mut metadata = TrackMetadata::new("track.wav");
    metadata.rating = Some(6);
    assert!(NewDjmdContent::try_from(&metadata).is_err());
    assert!(NewDjmdContent::try_from(&TrackMetadata::new("track.wav")).is_ok());
}

#[test]
fn rejects_invalid_database_values() {
    for mut row in [
        DjmdContent {
            dj_play_count: Some(-1),
            ..Default::default()
        },
        DjmdContent {
            color_id: Some("256".into()),
            ..Default::default()
        },
        DjmdContent {
            color_id: Some("pink".into()),
            ..Default::default()
        },
        DjmdContent {
            track_no: Some(-1),
            ..Default::default()
        },
        DjmdContent {
            bit_depth: Some(65536),
            ..Default::default()
        },
        DjmdContent {
            release_year: Some(-1),
            ..Default::default()
        },
        DjmdContent {
            rating: Some(6),
            ..Default::default()
        },
        DjmdContent {
            file_size: Some(-1),
            ..Default::default()
        },
        DjmdContent {
            bpm: Some(-1),
            ..Default::default()
        },
        DjmdContent {
            length: Some(-1),
            ..Default::default()
        },
    ] {
        row.id = "42".into();
        let view = DjmdContentView {
            content: row,
            artist: None,
            remixer: None,
            original_artist: None,
            composer: None,
            album: None,
            album_artist: None,
            genre: None,
            label: None,
            key: None,
            color: None,
        };
        assert!(Track::try_from(view).is_err());
    }
}

#[test]
fn view_copies_resolved_names_and_preserves_optional_fields() {
    let view = DjmdContentView {
        content: DjmdContent {
            id: "42".into(),
            uuid: "uuid".into(),
            master_db_id: Some("not-the-owning-library".into()),
            image_path: Some("/artwork/cover.jpg".into()),
            analysis_data_path: Some("/ANLZ/ANLZ0000.DAT".into()),
            color_id: Some("3".into()),
            dj_play_count: Some(120),
            hot_cue_auto_load: Some("unknown-setting".into()),
            delivery_control: Some("on".into()),
            delivery_comment: Some("Publish comment".into()),
            title: Some("Title".into()),
            length: Some(62),
            file_size: Some(i64::MAX),
            ..Default::default()
        },
        artist: Some(DjmdArtist {
            name: "Artist".into(),
            ..Default::default()
        }),
        remixer: Some(DjmdArtist {
            name: "Remixer".into(),
            ..Default::default()
        }),
        original_artist: None,
        composer: None,
        album: None,
        album_artist: Some(DjmdArtist {
            name: "Album artist".into(),
            ..Default::default()
        }),
        genre: None,
        label: None,
        key: None,
        color: None,
    };
    let borrowed = Track::try_from(&view).unwrap();
    let track: Track = view.try_into().unwrap();
    assert_eq!(borrowed.id, track.id);
    assert_eq!(track.id, Some(42));
    assert_eq!(track.master_content_id, Some(42));
    assert_eq!(track.source, TrackSource::MasterDb);
    assert!(track.master_db_id.is_none()); // A detached content view has no library context.
    assert_eq!(
        track.artwork_path.as_deref(),
        Some(std::path::Path::new("/artwork/cover.jpg"))
    );
    assert_eq!(
        track.analysis_data_path.as_deref(),
        Some(std::path::Path::new("/ANLZ/ANLZ0000.DAT"))
    );
    assert_eq!(track.color, Some(3));
    assert_eq!(track.play_count, Some(120));
    assert_eq!(track.hot_cue_auto_load.as_deref(), Some("unknown-setting"));
    assert_eq!(track.delivery_control.as_deref(), Some("on"));
    assert_eq!(track.delivery_comment.as_deref(), Some("Publish comment"));
    assert_eq!(track.uuid, "uuid");
    assert_eq!(track.title.as_deref(), Some("Title"));
    assert_eq!(track.length, Some(62.0));
    assert_eq!(track.file_size, Some(i64::MAX as u64));
    assert_eq!(track.artist.as_deref(), Some("Artist"));
    assert_eq!(track.remixer.as_deref(), Some("Remixer"));
    assert_eq!(track.album_artist.as_deref(), Some("Album artist"));
    assert_eq!(track.rating, None);
    assert_eq!(track.bpm, None);
    assert!(track.path.as_os_str().is_empty());
    let metadata = TrackMetadata::from(track.clone());
    assert_eq!(metadata.artist, track.artist);
    let insert: NewDjmdContent = metadata.try_into().unwrap();
    assert_eq!(insert.artist_id, None);
}

#[test]
fn shared_track_ids_parse_checked_unsigned_values() {
    let mut view = DjmdContentView {
        content: DjmdContent::default(),
        artist: None,
        remixer: None,
        original_artist: None,
        composer: None,
        album: None,
        album_artist: None,
        genre: None,
        label: None,
        key: None,
        color: None,
    };
    for invalid in ["", "-1", "4294967296", "not-numeric"] {
        view.content.id = invalid.into();
        assert!(Track::try_from(&view).is_err());
    }
    view.content.id = u32::MAX.to_string();
    let track = Track::try_from(view).unwrap();
    assert_eq!(track.id, Some(u32::MAX));
    assert_eq!(track.master_content_id, Some(u32::MAX));
}

#[rstest::rstest]
#[case(6)]
#[case(7)]
fn reads_resolved_tracks(#[case] version: usize) -> anyhow::Result<()> {
    let fixture = setup_master_db_path(version);
    let db = MasterDb::new(fixture.path())?;
    let mut conn = db.pool.get()?;
    let library_id = DjmdProperty::all(&mut conn)?.remove(0).db_id;
    let mut row = DjmdContent::all(&mut conn)?.remove(0);
    let artist = NewDjmdArtist::new("Track artist").insert(&mut conn)?;
    let album_artist = NewDjmdArtist::new("Album artist").insert(&mut conn)?;
    let album = NewDjmdAlbum {
        album_artist_id: Some(album_artist.id),
        ..NewDjmdAlbum::new("Track album")
    }
    .insert(&mut conn)?;
    row.artist_id = Some(artist.id);
    row.album_id = Some(album.id);
    row.bpm = Some(12825);
    row.length = Some(123);
    row.file_size = Some(456789);
    row.folder_path = Some("C:/music/Ã¦â€”Â¥Ã¦Å“Â¬Ã¨ÂªÅ¾.mp3".into());
    row.composer_id = None;
    row.master_db_id = Some("different-row-provenance".into());
    row.org_artist_id = Some("missing-artist".into());
    row = row.update(&mut conn)?;

    let track: Track = db.get_track_by_id(&row.id)?.unwrap();
    assert_eq!(track.id, row.id.parse().ok());
    assert_eq!(track.master_content_id, Some(row.id.parse()?));
    assert_eq!(track.master_db_id, Some(library_id.parse()?));
    assert_eq!(track.source, TrackSource::MasterDb);
    assert_eq!(track.uuid, row.uuid);
    assert_eq!(track.artist.as_deref(), Some("Track artist"));
    assert_eq!(track.album.as_deref(), Some("Track album"));
    assert_eq!(track.album_artist.as_deref(), Some("Album artist"));
    assert_eq!(track.bpm, Some(128.25));
    assert_eq!(track.length, Some(123.0));
    assert_eq!(track.file_size, Some(456789));
    assert!(track.composer.is_none());
    assert!(track.org_artist.is_none());
    for path in [
        "C:/music/Ã¦â€”Â¥Ã¦Å“Â¬Ã¨ÂªÅ¾.mp3",
        r"C:\music\Ã¦â€”Â¥Ã¦Å“Â¬Ã¨ÂªÅ¾.mp3",
    ] {
        let found = db.get_track_by_path(path)?.unwrap();
        assert_eq!(found.id, Some(row.id.parse()?));
        assert_eq!(found.master_db_id, Some(library_id.parse()?));
    }

    let all = db.get_tracks()?;
    let raw = DjmdContent::all(&mut conn)?;
    assert_eq!(all.len(), raw.len());
    let found = all
        .iter()
        .find(|track| track.id == row.id.parse().ok())
        .unwrap();
    assert_eq!(found.master_db_id, track.master_db_id);
    assert_eq!(found.album_artist, track.album_artist);
    let other_id = &raw.iter().find(|other| other.id != row.id).unwrap().id;
    let selected = db.get_tracks_by_ids(&[other_id, &row.id, "missing", &row.id])?;
    assert_eq!(selected.len(), 2);
    assert!(selected
        .iter()
        .all(|t| t.master_db_id == library_id.parse().ok()));
    assert!(selected
        .iter()
        .any(|track| track.id == other_id.parse().ok()));
    assert_eq!(
        selected
            .iter()
            .find(|track| track.id == row.id.parse().ok())
            .unwrap()
            .artist,
        track.artist
    );
    // Reads and changes to returned snapshots do not write the underlying row.
    let mut detached = track;
    detached.title = Some("Detached edit".into());
    assert_eq!(DjmdContent::find(&mut conn, &row.id)?, Some(row));
    Ok(())
}

#[rstest::rstest]
#[case(6)]
#[case(7)]
fn missing_tracks_and_conversion_errors(#[case] version: usize) -> anyhow::Result<()> {
    let fixture = setup_master_db_path(version);
    let db = MasterDb::new(fixture.path())?;
    assert!(db.get_track_by_id("missing")?.is_none());
    assert!(db.get_track_by_path("/missing/track.mp3")?.is_none());
    assert!(db.get_tracks_by_ids(&[])?.is_empty());
    assert!(db.get_tracks_by_ids(&["missing"])?.is_empty());

    let mut conn = db.pool.get()?;
    let mut row = DjmdContent::all(&mut conn)?.remove(0);
    row.rating = Some(6);
    row.folder_path = Some("/invalid/metadata.mp3".into());
    let row = row.update(&mut conn)?;
    for result in [
        db.get_track_by_id(&row.id).map(|_| ()),
        db.get_track_by_path("/invalid/metadata.mp3").map(|_| ()),
        db.get_tracks_by_ids(&[&row.id]).map(|_| ()),
        db.get_tracks().map(|_| ()),
    ] {
        assert!(matches!(
            result,
            Err(rbox_masterdb::Error::InvalidOperation(_))
        ));
    }
    Ok(())
}

#[rstest::rstest]
#[case(6)]
#[case(7)]
fn library_identity_requires_unambiguous_property(#[case] version: usize) -> anyhow::Result<()> {
    use rbox_masterdb::schema::djmdProperty;

    let fixture = setup_master_db_path(version);
    let db = MasterDb::new(fixture.path())?;
    let mut conn = db.pool.get()?;
    let id = DjmdContent::all(&mut conn)?.remove(0).id;
    let mut property = DjmdProperty::all(&mut conn)?.remove(0);
    property.db_id = "another-library".into();
    diesel::insert_into(djmdProperty::table)
        .values(property)
        .execute(&mut conn)?;
    assert!(matches!(
        db.get_track_by_id(&id),
        Err(rbox_masterdb::Error::InvalidOperation(_))
    ));

    diesel::delete(djmdProperty::table).execute(&mut conn)?;
    let track = db.get_track_by_id(&id)?.unwrap();
    assert_eq!(track.master_content_id, Some(id.parse()?));
    assert!(track.master_db_id.is_none());
    Ok(())
}
