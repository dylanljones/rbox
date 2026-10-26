// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Application models, lazy reads, provenance, ordering, and atomic mutations.

use std::fs;
use std::io::Cursor;

use rbox_devlib::binary::{
    DatabaseKind, DeviceSqlString, Row, RowRecord, StandardRow, StringEncoding, TrackRow,
    PAGE_HEADER_SIZE,
};
use rbox_devlib::models::{Content, NewContent};
use rbox_devlib::{CreateOptions, DeviceSqlConnection, Error, Result};
use rbox_testing::data::{create_temp_dir, export_dir};

// Binary-only fixture for unusual color IDs used by read/relationship tests.
fn seed_color(
    conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>,
    id: u16,
    name: &str,
) -> Result<rbox_devlib::models::Color> {
    use rbox_devlib::{binary::ColorRow, models::Color};
    let mut tx = conn.transaction();
    let handle = tx.insert(
        Color::TABLE,
        &Row::Standard(StandardRow::Color(ColorRow {
            id,
            name: DeviceSqlString::encode(name, StringEncoding::ShortAscii)?,
            unknown_00: 0,
            unknown_04: 0,
            unknown_07: 0,
        })),
    )?;
    let model = Color::try_from(tx.read(&handle)?)?;
    tx.commit()?;
    Ok(model)
}

#[test]
fn color_updates_preserve_identity_enforce_unique_names_and_roundtrip() -> Result<()> {
    use rbox_devlib::models::Color;
    let mut conn = DeviceSqlConnection::create_with_options(
        DatabaseKind::Standard,
        &CreateOptions::standard("2026-10-25", "USB")?,
    )?;
    let colors = Color::all(&mut conn)?;
    assert_eq!(colors.len(), 8);
    let original = colors[0].clone();
    assert_eq!(original.clone().update(&mut conn)?, original);
    let mut edited = original.clone();
    edited.name = colors[1].name.clone();
    assert!(matches!(
        edited.update(&mut conn),
        Err(Error::UniqueConstraintViolation {
            fields: &["name"],
            ..
        })
    ));
    let mut edited = original.clone();
    edited.id = 99;
    assert!(matches!(
        edited.update(&mut conn),
        Err(Error::InvalidModelField { field: "id", .. })
    ));
    let mut edited = original.clone();
    edited.name = "Custom \u{97f3}".repeat(20);
    let updated = edited.update(&mut conn)?;
    assert_eq!(original.clone().update(&mut conn)?, updated);
    let Row::Standard(StandardRow::Color(old)) = original.raw_record().decode()? else {
        panic!()
    };
    let Row::Standard(StandardRow::Color(new)) = updated.raw_record().decode()? else {
        panic!()
    };
    assert_eq!(
        (old.unknown_00, old.unknown_04, old.unknown_07),
        (new.unknown_00, new.unknown_04, new.unknown_07)
    );
    let mut bytes = Vec::new();
    conn.write_to(&mut bytes)?;
    let mut reopened =
        DeviceSqlConnection::from_reader(Cursor::new(bytes.clone()), DatabaseKind::Standard)?;
    assert_eq!(
        Color::find(&mut reopened, &updated.id)?.unwrap().name,
        updated.name
    );
    reopened.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
        dirty_pages: Some(0),
        ..Default::default()
    })?;
    let mut edited = updated.clone();
    edited.name = "Changed".into();
    assert!(matches!(
        edited.update(&mut reopened),
        Err(Error::DirtyPageLimit { .. })
    ));
    let mut after = Vec::new();
    reopened.write_to(&mut after)?;
    assert_eq!(after, bytes);
    let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
    assert!(matches!(
        updated.update(&mut extended),
        Err(Error::WrongDatabaseKind { .. })
    ));
    Ok(())
}

mod content_mutations {
    //! Content field mapping and deletion across standard/extended overlays.
    use super::*;
    use rbox_devlib::models::{
        ExportProperty, History, HistoryContent, MyTag, MyTagContent, NewHistory,
        NewHistoryContent, NewMyTag, NewMyTagContent, NewPlaylist, NewPlaylistContent,
        NewRecommendedLike, Playlist, PlaylistContent, RecommendedLike,
    };
    type Memory = DeviceSqlConnection<Cursor<Vec<u8>>>;

    fn bytes<R: std::io::Read + std::io::Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    fn pair() -> Result<(Memory, Memory)> {
        let mut standard = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("2026-10-25", "Test")?,
        )?;
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        for id in [1, 2] {
            NewContent::new(id, format!("/Contents/{id}.wav")).insert(&mut standard)?;
        }
        NewPlaylist::new(10, 0, 1, "Playlist").insert(&mut standard)?;
        NewHistory::new(20, "History").insert(&mut standard)?;
        NewMyTag::new(30, 0, 0, "Tag").insert(&mut extended)?;
        for (id, seq) in [(1, 1), (1, 3), (2, 8)] {
            NewPlaylistContent::new(10, id, seq).insert(&mut standard)?;
        }
        for (id, seq) in [(1, 1), (2, 9)] {
            NewHistoryContent::new(20, id, seq).insert(&mut standard)?;
        }
        for id in [1, 1, 2] {
            NewMyTagContent::new(id, 30).insert(&mut extended)?;
        }
        for (a, b) in [(1, 2), (2, 1), (1, 1), (2, 2)] {
            NewRecommendedLike::new(a, b, 0x1234, 7, 8).insert(&mut extended)?;
        }
        Ok((standard, extended))
    }

    #[test]
    fn updates_map_all_public_fields_preserve_unknowns_and_follow_relocations() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut input = NewContent::new(1, "/Contents/original.wav");
        input.isrc = "USABC1234567".into();
        input.creation.content_link = 0x12345678;
        input.creation.raw_analysis = 0x9876;
        input.creation.unknown_strings =
            ["one", "two", "three", "four", "five", "six", "seven"].map(str::to_owned);
        let original = input.clone().insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        assert_eq!(original.clone().update(&mut conn)?, original);
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = original.clone();
        let mut desired = input;
        desired.artist_id = 2;
        edited.artist_id = desired.artist_id;
        desired.album_id = 3;
        edited.album_id = desired.album_id;
        desired.genre_id = 4;
        edited.genre_id = desired.genre_id;
        desired.label_id = 5;
        edited.label_id = desired.label_id;
        desired.key_id = 6;
        edited.key_id = desired.key_id;
        desired.artwork_id = 7;
        edited.artwork_id = desired.artwork_id;
        desired.composer_id = 8;
        edited.composer_id = desired.composer_id;
        desired.original_artist_id = 9;
        edited.original_artist_id = desired.original_artist_id;
        desired.remixer_id = 10;
        edited.remixer_id = desired.remixer_id;
        desired.master_content_id = 11;
        edited.master_content_id = desired.master_content_id;
        desired.master_db_id = 12;
        edited.master_db_id = desired.master_db_id;
        desired.sample_rate = 13;
        edited.sample_rate = desired.sample_rate;
        desired.file_size = 14;
        edited.file_size = desired.file_size;
        desired.bitrate = 15;
        edited.bitrate = desired.bitrate;
        desired.bpmx100 = 16;
        edited.bpmx100 = desired.bpmx100;
        desired.length = 17;
        edited.length = desired.length;
        desired.track_no = 18;
        edited.track_no = desired.track_no;
        desired.disc_no = 19;
        edited.disc_no = desired.disc_no;
        desired.play_count = 20;
        edited.play_count = desired.play_count;
        desired.release_year = 21;
        edited.release_year = desired.release_year;
        desired.bit_depth = 22;
        edited.bit_depth = desired.bit_depth;
        desired.color_id = 23;
        edited.color_id = desired.color_id;
        desired.rating = 24;
        edited.rating = desired.rating;
        desired.file_type = 25;
        edited.file_type = desired.file_type;
        desired.title = "title changed".into();
        edited.title = desired.title.clone();
        desired.comment = "comment changed".into();
        edited.comment = desired.comment.clone();
        desired.mix_name = "mix_name changed".into();
        edited.mix_name = desired.mix_name.clone();
        desired.lyricist = "lyricist changed".into();
        edited.lyricist = desired.lyricist.clone();
        desired.message = "message changed".into();
        edited.message = desired.message.clone();
        desired.publish_track_info = "publish_track_info changed".into();
        edited.publish_track_info = desired.publish_track_info.clone();
        desired.hot_cue_auto_load = "hot_cue_auto_load changed".into();
        edited.hot_cue_auto_load = desired.hot_cue_auto_load.clone();
        desired.date_added = "date_added changed".into();
        edited.date_added = desired.date_added.clone();
        desired.release_date = "release_date changed".into();
        edited.release_date = desired.release_date.clone();
        desired.analysis_date = "analysis_date changed".into();
        edited.analysis_date = desired.analysis_date.clone();
        desired.analysis_path = "analysis_path changed".into();
        edited.analysis_path = desired.analysis_path.clone();
        desired.file_name = "file_name changed".into();
        edited.file_name = desired.file_name.clone();
        desired.path = "path changed".into();
        edited.path = desired.path.clone();
        desired.title = "Long title ".repeat(80);
        edited.title = desired.title.clone();
        let updated = edited.update(&mut conn)?;
        let Row::Standard(StandardRow::Track(actual)) = updated.raw_record().decode()? else {
            panic!()
        };
        let mut expected = desired.to_binary()?;
        expected.header.index_shift = actual.header.index_shift; // Placement is engine-owned.
        assert_eq!(actual.header, expected.header);
        assert_eq!(actual.text, expected.text);
        assert_ne!(
            (
                original.raw_record().page(),
                original.raw_record().slot().slot
            ),
            (
                updated.raw_record().page(),
                updated.raw_record().slot().slot
            )
        );
        assert_eq!(original.clone().update(&mut conn)?, updated);
        let mut stale = original;
        stale.comment = "Another edit".into();
        let updated = stale.update(&mut conn)?;
        assert_eq!(updated.title, desired.title);
        assert_eq!(updated.comment, "Another edit");
        assert_eq!(updated.isrc()?, "USABC1234567");
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert_eq!(Content::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn deletion_cleans_both_files_and_preserves_parents_survivors_and_sequences() -> Result<()> {
        let (mut standard, mut extended) = pair()?;
        let survivor = Content::find(&mut standard, &2)?
            .unwrap()
            .raw_record()
            .decode()?;
        assert_eq!(
            Content::delete_with_extended(&mut standard, &mut extended, &1)?,
            9
        );
        assert_eq!(
            Content::delete_with_extended(&mut standard, &mut extended, &1)?,
            0
        );
        assert!(Content::find(&mut standard, &1)?.is_none());
        assert_eq!(
            Content::find(&mut standard, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            survivor
        );
        assert_eq!(ExportProperty::all(&mut standard)?[0].track_count, 1);
        let playlists = PlaylistContent::all(&mut standard)?;
        assert_eq!(playlists.len(), 1);
        assert_eq!((playlists[0].content_id, playlists[0].seq), (2, 8));
        let histories = HistoryContent::all(&mut standard)?;
        assert_eq!(histories.len(), 1);
        assert_eq!((histories[0].content_id, histories[0].seq), (2, 9));
        assert!(Playlist::find(&mut standard, &10)?.is_some());
        assert!(History::find(&mut standard, &20)?.is_some());
        assert!(MyTag::find(&mut extended, &30)?.is_some());
        let tags = MyTagContent::all(&mut extended)?;
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].content_id, 2);
        let recommendations = RecommendedLike::all(&mut extended)?;
        assert_eq!(recommendations.len(), 1);
        assert_eq!(
            (
                recommendations[0].content_id_1,
                recommendations[0].content_id_2
            ),
            (2, 2)
        );
        let mut standard = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut standard)?),
            DatabaseKind::Standard,
        )?;
        let mut extended = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut extended)?),
            DatabaseKind::Extended,
        )?;
        assert!(Content::find(&mut standard, &1)?.is_none());
        assert!(MyTagContent::by_content(&mut extended, &1)?.is_empty());
        assert!(RecommendedLike::by_content(&mut extended, &1)?.is_empty());
        assert_eq!(
            Content::delete_with_extended(&mut standard, &mut extended, &2)?,
            5
        );
        assert_eq!(ExportProperty::all(&mut standard)?[0].track_count, 0);
        Ok(())
    }

    #[test]
    fn failures_in_either_file_roll_back_all_staged_cleanup() -> Result<()> {
        let (mut standard, mut extended) = pair()?;
        let original_standard = bytes(&mut standard)?;
        let original_extended = bytes(&mut extended)?;
        for fail_extended in [false, true] {
            let mut standard = DeviceSqlConnection::from_reader(
                Cursor::new(original_standard.clone()),
                DatabaseKind::Standard,
            )?;
            let mut extended = DeviceSqlConnection::from_reader(
                Cursor::new(original_extended.clone()),
                DatabaseKind::Extended,
            )?;
            let limited = if fail_extended {
                &mut extended
            } else {
                &mut standard
            };
            limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
                dirty_pages: Some(0),
                ..Default::default()
            })?;
            assert!(matches!(
                Content::delete_with_extended(&mut standard, &mut extended, &1),
                Err(Error::DirtyPageLimit { .. })
            ));
            assert_eq!(bytes(&mut standard)?, original_standard);
            assert_eq!(bytes(&mut extended)?, original_extended);
        }
        // Failure after track and occurrence deletions have been staged.
        let property = ExportProperty::all(&mut standard)?.remove(0);
        let raw = property.raw_record();
        let offset = raw.page().0 as usize * standard.header().prefix.page_size as usize
            + PAGE_HEADER_SIZE
            + raw.slot().heap_offset as usize;
        let mut corrupt = original_standard;
        corrupt[offset..offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        let mut standard =
            DeviceSqlConnection::from_reader(Cursor::new(corrupt.clone()), DatabaseKind::Standard)?;
        assert!(Content::delete_with_extended(&mut standard, &mut extended, &1).is_err());
        assert_eq!(bytes(&mut standard)?, corrupt);
        assert_eq!(bytes(&mut extended)?, original_extended);
        Ok(())
    }

    #[test]
    fn update_validation_missing_rows_and_standard_only_delete() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let content = NewContent::new(1, "/Contents/track.wav").insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        let mut edited = content.clone();
        edited.id = 2;
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut edited = content.clone();
        edited.path.clear();
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::InvalidModelField { field: "path", .. })
        ));
        let mut edited = content.clone();
        edited.title = "bad\0title".into();
        assert!(matches!(edited.update(&mut conn), Err(Error::EmbeddedNul)));
        let mut edited = content.clone();
        edited.title = "x".repeat(5000);
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::NoRowSpace { .. })
        ));
        assert_eq!(bytes(&mut conn)?, before);
        let mut ext = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            content.clone().update(&mut ext),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Content::delete(&mut ext, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Content::delete(&mut conn, &0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(Content::delete(&mut conn, &99)?, 0);
        assert_eq!(Content::delete(&mut conn, &1)?, 1);
        assert!(matches!(
            content.update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));
        Ok(())
    }
}

#[test]
fn album_uniqueness_uses_name_and_current_album_artist() -> Result<()> {
    use rbox_devlib::models::{Album, NewAlbum};
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let first = NewAlbum::new(1, 10, "Album").insert(&mut conn)?;
    let second = NewAlbum::new(2, 20, "Album").insert(&mut conn)?;
    let other = NewAlbum::new(3, 10, "Other").insert(&mut conn)?;
    NewAlbum::new(4, 0, "Album").insert(&mut conn)?;
    for artist in [0, 10, 20] {
        assert!(matches!(
            NewAlbum::new(5, artist, "Album").insert(&mut conn),
            Err(Error::UniqueConstraintViolation {
                fields: &["name", "artist_id"],
                ..
            })
        ));
    }
    first.clone().update(&mut conn)?;
    let mut before = Vec::new();
    conn.write_to(&mut before)?;
    let mut edited = second;
    edited.artist_id = 10;
    assert!(matches!(
        edited.update(&mut conn),
        Err(Error::UniqueConstraintViolation { .. })
    ));
    let mut edited = other;
    edited.name = "Album".into();
    assert!(matches!(
        edited.update(&mut conn),
        Err(Error::UniqueConstraintViolation { .. })
    ));
    let mut after = Vec::new();
    conn.write_to(&mut after)?;
    assert_eq!(after, before);
    // A stale rename must check the current artist, not the snapshot's artist.
    let mut edited = first.clone();
    edited.artist_id = 30;
    edited.update(&mut conn)?;
    NewAlbum::new(5, 30, "Taken").insert(&mut conn)?;
    let mut stale = first;
    stale.name = "Taken".into();
    assert!(matches!(
        stale.update(&mut conn),
        Err(Error::UniqueConstraintViolation { .. })
    ));
    assert_eq!(Album::find(&mut conn, &1)?.unwrap().name, "Album");
    let mut bytes = Vec::new();
    conn.write_to(&mut bytes)?;
    let mut reopened =
        DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
    assert!(matches!(
        NewAlbum::new(6, 30, "Album").insert(&mut reopened),
        Err(Error::UniqueConstraintViolation { .. })
    ));
    Album::delete(&mut reopened, &1)?;
    NewAlbum::new(6, 30, "Album").insert(&mut reopened)?;
    Ok(())
}

#[test]
fn named_models_find_exact_names_without_resolving_relations() -> Result<()> {
    use rbox_devlib::models::{
        Album, Artist, Color, Genre, Key, Label, NewAlbum, NewArtist, NewGenre, NewKey, NewLabel,
    };
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    macro_rules! check {
        ($model:ident, $input:expr) => {{
            assert!($model::find_by_name(&mut conn, "Name")?.is_none());
            assert!(!$model::name_exists(&mut conn, "Name")?);
            let model = $input.insert(&mut conn)?;
            assert!($model::name_exists(&mut conn, "Name")?);
            assert!(!$model::name_exists(&mut conn, "name")?);
            assert!(!$model::name_exists(&mut conn, "Name ")?);
            assert_eq!(
                $model::find_by_name(&mut conn, "Name")?.unwrap().id,
                model.id
            );
            assert!($model::find_by_name(&mut conn, "name")?.is_none());
            assert!($model::find_by_name(&mut conn, "Name ")?.is_none());
            let mut edited = model;
            edited.name = "\u{97f3}".into();
            edited.update(&mut conn)?;
            assert!($model::find_by_name(&mut conn, "Name")?.is_none());
            assert!($model::find_by_name(&mut conn, "\u{97f3}")?.is_some());
            assert!(!$model::name_exists(&mut conn, "Name")?);
            assert!($model::name_exists(&mut conn, "\u{97f3}")?);
            let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
            assert!(matches!(
                $model::name_exists(&mut extended, "Name"),
                Err(Error::WrongDatabaseKind { .. })
            ));
            assert!(matches!(
                $model::find_by_name(&mut extended, "Name"),
                Err(Error::WrongDatabaseKind { .. })
            ));
        }};
    }
    check!(Artist, NewArtist::new(1, "Name"));
    check!(Genre, NewGenre::new(1, "Name"));
    check!(Key, NewKey::new(1, 23, "Name"));
    check!(Label, NewLabel::new(1, "Name"));
    check!(Album, NewAlbum::new(1, 999, "Name"));
    NewAlbum::new(2, 1000, "\u{97f3}").insert(&mut conn)?;
    let first = Album::all(&mut conn)?
        .into_iter()
        .find(|a| a.name == "\u{97f3}")
        .unwrap();
    assert_eq!(
        Album::find_by_name(&mut conn, "\u{97f3}")?.unwrap().id,
        first.id
    );
    let mut colors = DeviceSqlConnection::create_with_options(
        DatabaseKind::Standard,
        &CreateOptions::standard("2026-10-25", "USB")?,
    )?;
    let mut color = Color::all(&mut colors)?.remove(0);
    color.name = "Custom".into();
    let color = color.update(&mut colors)?;
    assert_eq!(
        Color::find_by_name(&mut colors, "Custom")?.unwrap().id,
        color.id
    );
    assert!(Color::find_by_name(&mut colors, "custom")?.is_none());
    assert!(Color::name_exists(&mut colors, "Custom")?);
    assert!(!Color::name_exists(&mut colors, "custom")?);
    assert!(Album::name_exists(&mut conn, "\u{97f3}")?);
    Ok(())
}

mod name_constraints {
    //! Model name constraints compare decoded text and roll back conflicts.
    use super::*;
    use rbox_devlib::models::{Artist, Genre, Key, Label, NewArtist, NewGenre, NewKey, NewLabel};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn all_named_models_enforce_unique_names_on_insert_and_rename() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        macro_rules! check {
            ($model:ident, $new:expr) => {{
                let new = $new;
                let first = new(1, "Name").insert(&mut conn)?;
                let second = new(2, "Other").insert(&mut conn)?;
                // Self updates and identical names in different tables are valid.
                first.clone().update(&mut conn)?;
                let before = bytes(&mut conn)?;
                assert!(matches!(new(3, "Name").insert(&mut conn),
                    Err(Error::UniqueConstraintViolation { table, fields: &["name"] }) if table == $model::TABLE));
                let mut edited = second.clone();
                edited.name = "Name".into();
                assert!(matches!(edited.update(&mut conn), Err(Error::UniqueConstraintViolation { .. })));
                assert_eq!(bytes(&mut conn)?, before);
                new(3, "name").insert(&mut conn)?;
                new(4, "Name ").insert(&mut conn)?;
                new(5, "").insert(&mut conn)?;
                assert!(matches!(new(6, "").insert(&mut conn), Err(Error::UniqueConstraintViolation { .. })));
                let mut changed = second.clone();
                changed.name = "Taken".into();
                changed.update(&mut conn)?;
                let mut stale = first.clone();
                stale.name = "Taken".into();
                assert!(matches!(stale.update(&mut conn), Err(Error::UniqueConstraintViolation { .. })));
                $model::delete(&mut conn, &1)?;
                new(6, "Name").insert(&mut conn)?;
                let mut reopened = DeviceSqlConnection::from_reader(Cursor::new(bytes(&mut conn)?), DatabaseKind::Standard)?;
                assert!(matches!(new(7, "Name").insert(&mut reopened), Err(Error::UniqueConstraintViolation { .. })));
            }};
        }
        check!(Artist, |id, name: &str| NewArtist::new(id, name));
        check!(Genre, |id, name: &str| NewGenre::new(id, name));
        check!(Key, |id, name: &str| NewKey::new(id, 77, name));
        check!(Label, |id, name: &str| NewLabel::new(id, name));
        Ok(())
    }

    #[test]
    fn equal_names_conflict_across_binary_string_encodings() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = NewGenre::new(1, "House").to_binary()?;
        row.name = DeviceSqlString::encode("House", StringEncoding::Utf16Le)?;
        let mut tx = conn.transaction();
        tx.insert(Genre::TABLE, &Row::Standard(StandardRow::Genre(row)))?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            NewGenre::new(2, "House").insert(&mut conn),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }
}

mod recommendation_deletions {
    //! Recommendation pair uniqueness and occurrence deletion.
    use super::*;
    use rbox_devlib::binary::ExtendedRow;
    use rbox_devlib::models::{NewRecommendedLike, RecommendedLike};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn rejects_duplicate_pairs_and_allows_reinsertion_after_deletion() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let first = NewRecommendedLike::new(1, 2, 7, 8, 9).insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        for value in [7, 99] {
            assert!(matches!(
                NewRecommendedLike::new(1, 2, value, 8, 9).insert(&mut conn),
                Err(Error::UniqueConstraintViolation {
                    fields: &["content_id_1", "content_id_2"],
                    ..
                })
            ));
        }
        assert_eq!(bytes(&mut conn)?, before);
        let reverse = NewRecommendedLike::new(2, 1, 17, 18, 19)
            .insert(&mut conn)?
            .raw_record()
            .decode()?;
        let same = NewRecommendedLike::new(1, 1, 0, 0, 0).insert(&mut conn)?;
        assert!(matches!(
            NewRecommendedLike::new(1, 1, 1, 2, 3).insert(&mut conn),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        let mut edited = first.clone();
        edited.content_id_1 = 99;
        edited.unknown_00 = 42;
        assert_eq!(edited.delete(&mut conn)?, 1);
        assert!(first.delete(&mut conn).is_err());
        NewRecommendedLike::new(1, 2, 7, 8, 9)
            .insert(&mut conn)?
            .delete(&mut conn)?;
        assert_eq!(same.delete(&mut conn)?, 1);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Extended,
        )?;
        let rows = RecommendedLike::all(&mut reopened)?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].raw_record().decode()?, reverse);
        assert!(matches!(
            NewRecommendedLike::new(2, 1, 0, 0, 0).insert(&mut reopened),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        Ok(())
    }

    #[test]
    fn rejects_stale_rows_wrong_namespaces_and_failed_writes() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let original = NewRecommendedLike::new(1, 2, 7, 8, 9).insert(&mut conn)?;
        let mut tx = conn.transaction();
        let handle = tx.locate(
            RecommendedLike::TABLE,
            original.raw_record().page(),
            original.raw_record().slot().slot,
        )?;
        tx.update(
            &handle,
            &Row::Extended(ExtendedRow::RecommendLike(
                NewRecommendedLike::new(1, 2, 77, 8, 9).to_binary()?,
            )),
        )?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            original.clone().delete(&mut conn),
            Err(Error::StaleModelRow { .. })
        ));
        assert_eq!(bytes(&mut conn)?, before);
        let mut limited =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Extended)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(0),
            ..Default::default()
        })?;
        let current = RecommendedLike::all(&mut limited)?.remove(0);
        assert!(matches!(
            current.delete(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut limited)?, before);
        let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        assert!(matches!(
            original.delete(&mut standard),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }
}

#[test]
fn browse_name_queries_and_first_properties_handle_empty_and_initialized_tables() -> Result<()> {
    use rbox_devlib::binary::ExtendedRow;
    use rbox_devlib::models::{BrowseColumn, DatabaseProperty, ExportProperty};
    let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
    assert!(BrowseColumn::find_by_name(&mut standard, "COLOR")?.is_none());
    assert!(!BrowseColumn::name_exists(&mut standard, "COLOR")?);
    assert!(ExportProperty::first(&mut standard)?.is_none());
    assert!(DatabaseProperty::first(&mut extended)?.is_none());
    assert!(matches!(
        BrowseColumn::name_exists(&mut extended, "COLOR"),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        ExportProperty::first(&mut extended),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        DatabaseProperty::first(&mut standard),
        Err(Error::WrongDatabaseKind { .. })
    ));
    let mut standard = DeviceSqlConnection::create_with_options(
        DatabaseKind::Standard,
        &CreateOptions::standard("2026-10-26", "USB")?,
    )?;
    let color = BrowseColumn::find_by_name(&mut standard, "COLOR")?.unwrap();
    assert_eq!(color.name, "COLOR");
    assert!(BrowseColumn::name_exists(&mut standard, "COLOR")?);
    assert!(!BrowseColumn::name_exists(&mut standard, "color")?);
    assert!(!BrowseColumn::name_exists(&mut standard, " COLOR")?);
    assert!(!BrowseColumn::name_exists(
        &mut standard,
        "\u{fffa}COLOR\u{fffb}"
    )?);
    assert_eq!(
        ExportProperty::first(&mut standard)?,
        ExportProperty::all(&mut standard)?.into_iter().next()
    );
    // Multiple properties retain physical order rather than requiring a singleton.
    let property = ExportProperty::first(&mut standard)?.unwrap();
    let mut tx = standard.transaction();
    tx.insert(ExportProperty::TABLE, &property.raw_record().decode()?)?;
    tx.commit()?;
    assert_eq!(ExportProperty::all(&mut standard)?.len(), 2);
    assert_eq!(
        ExportProperty::first(&mut standard)?,
        ExportProperty::all(&mut standard)?.into_iter().next()
    );
    let mut extended = DeviceSqlConnection::create_with_options(
        DatabaseKind::Extended,
        &CreateOptions::extended(42)?,
    )?;
    let property = DatabaseProperty::first(&mut extended)?.unwrap();
    let Row::Extended(ExtendedRow::DatabaseProperty(mut row)) = property.raw_record().decode()?
    else {
        panic!()
    };
    row.header.my_tag_master_db_id = 99;
    let mut tx = extended.transaction();
    tx.insert(
        DatabaseProperty::TABLE,
        &Row::Extended(ExtendedRow::DatabaseProperty(row)),
    )?;
    tx.commit()?;
    assert_eq!(DatabaseProperty::all(&mut extended)?.len(), 2);
    assert_eq!(
        DatabaseProperty::first(&mut extended)?
            .unwrap()
            .my_tag_master_db_id,
        42
    );
    Ok(())
}

mod property_mutations {
    //! Update-only properties retain occurrence identity and reserved fields.
    use super::*;
    use rbox_devlib::binary::ExtendedRow;
    use rbox_devlib::models::{DatabaseProperty, ExportProperty};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn export_updates_all_fields_and_rejects_stale_count_snapshots() -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("2026-10-25", "USB")?,
        )?;
        let original = ExportProperty::all(&mut conn)?.remove(0);
        let before = bytes(&mut conn)?;
        original.clone().update(&mut conn)?;
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = original.clone();
        edited.track_count = 42;
        edited.background_color = 7;
        edited.date = "2026-10-26".into();
        edited.version = "1001".into();
        edited.name = "My USB".into();
        let updated = edited.update(&mut conn)?;
        assert_eq!((updated.track_count, updated.background_color), (42, 7));
        assert_eq!(
            (&*updated.date, &*updated.version, &*updated.name),
            ("2026-10-26", "1001", "My USB")
        );
        // Growing the name may relocate the row, leaving the old slot deleted.
        assert!(original.update(&mut conn).is_err());
        let before = bytes(&mut conn)?;
        let mut invalid = updated.clone();
        invalid.name = "bad\0name".into();
        assert!(matches!(invalid.update(&mut conn), Err(Error::EmbeddedNul)));
        assert_eq!(bytes(&mut conn)?, before);
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(before), DatabaseKind::Standard)?;
        assert_eq!(ExportProperty::all(&mut reopened)?[0], updated);
        NewContent::new(1, "/Contents/test.wav").insert(&mut reopened)?;
        assert!(matches!(
            updated.update(&mut reopened),
            Err(Error::StaleModelRow { .. })
        ));
        assert_eq!(ExportProperty::all(&mut reopened)?[0].track_count, 1);
        Ok(())
    }

    #[test]
    fn database_identity_updates_one_occurrence_and_preserves_reserved_data() -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Extended,
            &CreateOptions::extended(42)?,
        )?;
        let original = DatabaseProperty::all(&mut conn)?.remove(0);
        let Row::Extended(ExtendedRow::DatabaseProperty(mut row)) =
            original.raw_record().decode()?
        else {
            panic!()
        };
        row.header.unknown_words = [1, 2, 3, 4, 5];
        let mut tx = conn.transaction();
        let handle = tx.locate(
            DatabaseProperty::TABLE,
            original.raw_record().page(),
            original.raw_record().slot().slot,
        )?;
        tx.update(
            &handle,
            &Row::Extended(ExtendedRow::DatabaseProperty(row.clone())),
        )?;
        tx.insert(
            DatabaseProperty::TABLE,
            &Row::Extended(ExtendedRow::DatabaseProperty(row.clone())),
        )?;
        tx.commit()?;
        let original = DatabaseProperty::all(&mut conn)?.remove(0);
        let before = bytes(&mut conn)?;
        original.clone().update(&mut conn)?;
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = original.clone();
        edited.my_tag_master_db_id = 0;
        let updated = edited.update(&mut conn)?;
        let Row::Extended(ExtendedRow::DatabaseProperty(actual)) = updated.raw_record().decode()?
        else {
            panic!()
        };
        row.header.my_tag_master_db_id = 0;
        assert_eq!(actual, row);
        assert!(matches!(
            original.update(&mut conn),
            Err(Error::StaleModelRow { .. })
        ));
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Extended,
        )?;
        assert_eq!(
            DatabaseProperty::all(&mut reopened)?
                .iter()
                .map(|r| r.my_tag_master_db_id)
                .collect::<Vec<_>>(),
            [0, 42]
        );
        let before = bytes(&mut reopened)?;
        reopened.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(0),
            ..Default::default()
        })?;
        let mut edited = DatabaseProperty::all(&mut reopened)?.remove(0);
        edited.my_tag_master_db_id = 99;
        assert!(matches!(
            edited.update(&mut reopened),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut reopened)?, before);
        let mut standard = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("date", "name")?,
        )?;
        assert!(matches!(
            updated.update(&mut standard),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            ExportProperty::all(&mut standard)?
                .remove(0)
                .update(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }
}

mod playlist_mutations {
    //! Playlist hierarchy edits and occurrence deletion preserve unrelated rows.
    use super::*;
    use rbox_devlib::models::{NewPlaylist, NewPlaylistContent, Playlist, PlaylistContent};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn edits_validate_parents_preserve_reserved_fields_and_merge_stale_snapshots() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let root = NewPlaylist::folder(1, 0, 0, "Root").insert(&mut conn)?;
        NewPlaylist::folder(2, 1, 0, "Nested").insert(&mut conn)?;
        let mut input = NewPlaylist::new(3, 1, 7, "List");
        input.unknown_04 = 42;
        let playlist = input.insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        playlist.clone().update(&mut conn)?;
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = playlist.clone();
        edited.name = "Long name".repeat(100);
        edited.parent_id = 2;
        edited.seq = 19;
        let updated = edited.update(&mut conn)?;
        assert_eq!(playlist.clone().update(&mut conn)?, updated);
        let Row::Standard(StandardRow::PlaylistTree(row)) = updated.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(row.unknown_04, 42);
        let before = bytes(&mut conn)?;
        for parent in [1, 2, 3, 999] {
            let mut edited = root.clone();
            edited.parent_id = parent;
            assert!(edited.update(&mut conn).is_err());
        }
        let mut edited = playlist.clone();
        edited.attribute = 1;
        assert!(edited.update(&mut conn).is_err());
        let mut edited = playlist.clone();
        edited.id = 99;
        assert!(edited.update(&mut conn).is_err());
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = playlist;
        edited.name.clear();
        let updated = edited.update(&mut conn)?;
        assert_eq!((updated.parent_id, updated.seq), (2, 19));
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(Playlist::find(&mut reopened, &3)?.unwrap().name.is_empty());
        Ok(())
    }

    #[test]
    fn cascade_is_atomic_and_keeps_contents_and_other_playlists() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let root = NewPlaylist::folder(1, 0, 0, "Root").insert(&mut conn)?;
        NewPlaylist::folder(2, 1, 0, "Nested").insert(&mut conn)?;
        NewPlaylist::new(3, 2, 9, "List").insert(&mut conn)?;
        NewPlaylist::new(4, 0, 10, "Keep").insert(&mut conn)?;
        let content = NewContent::new(99, "/Contents/test.wav")
            .insert(&mut conn)?
            .raw_record()
            .decode()?;
        for playlist in [3, 3, 4] {
            NewPlaylistContent::new(playlist, 99, 7).insert(&mut conn)?;
        }
        let before = bytes(&mut conn)?;
        let mut limited =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(1),
            ..Default::default()
        })?;
        assert!(matches!(
            Playlist::delete(&mut limited, &1),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut limited)?, before);
        assert_eq!(Playlist::delete(&mut conn, &1)?, 5);
        assert_eq!(Playlist::delete(&mut conn, &1)?, 0);
        assert!(matches!(
            root.update(&mut conn),
            Err(Error::ModelNotFound { .. })
        ));
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        let playlists = Playlist::all(&mut reopened)?;
        assert_eq!(playlists.len(), 1);
        assert_eq!((playlists[0].id, playlists[0].seq), (4, 10));
        let entries = PlaylistContent::all(&mut reopened)?;
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].playlist_id, entries[0].seq), (4, 7));
        assert_eq!(
            Content::find(&mut reopened, &99)?
                .unwrap()
                .raw_record()
                .decode()?,
            content
        );
        Ok(())
    }

    #[test]
    fn occurrence_deletion_and_pair_removal_preserve_other_plays() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let first = NewPlaylistContent::new(1, 2, 3).insert(&mut conn)?;
        for seq in [3, 9] {
            NewPlaylistContent::new(1, 2, seq).insert(&mut conn)?;
        }
        let keep = NewPlaylistContent::new(1, 3, 12).insert(&mut conn)?;
        let mut edited = first.clone();
        edited.content_id = 99;
        assert_eq!(edited.delete(&mut conn)?, 1);
        assert!(first.delete(&mut conn).is_err());
        assert_eq!(PlaylistContent::remove(&mut conn, &2, &1)?, 2);
        assert_eq!(PlaylistContent::remove(&mut conn, &2, &1)?, 0);
        assert_eq!(PlaylistContent::all(&mut conn)?[0].seq, 12);
        let mut tx = conn.transaction();
        let handle = tx.locate(
            PlaylistContent::TABLE,
            keep.raw_record().page(),
            keep.raw_record().slot().slot,
        )?;
        tx.update(
            &handle,
            &Row::Standard(StandardRow::PlaylistEntry(
                NewPlaylistContent::new(1, 3, 15).to_binary()?,
            )),
        )?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            keep.clone().delete(&mut conn),
            Err(Error::StaleModelRow { .. })
        ));
        assert!(PlaylistContent::remove(&mut conn, &0, &1).is_err());
        assert!(Playlist::delete(&mut conn, &0).is_err());
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            keep.delete(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Playlist::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            PlaylistContent::remove(&mut extended, &2, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }
}

mod my_tag_mutations {
    //! Hierarchy validation, cascading cleanup and duplicate associations.
    use super::*;
    use rbox_devlib::binary::ExtendedRow;
    use rbox_devlib::models::{MyTag, MyTagContent, NewMyTag, NewMyTagContent};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_reserved_data_and_validate_current_ancestry() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let category = NewMyTag::category(1, 0, "Category").insert(&mut conn)?;
        NewMyTag::category(2, 4, "Other").insert(&mut conn)?;
        let mut input = NewMyTag::new(3, 1, 7, "Tag");
        input.unknown_04 = 42;
        let row = input.to_binary()?;
        let mut tx = conn.transaction();
        let handle = tx.insert(
            MyTag::TABLE,
            &Row::Extended(ExtendedRow::MyTag(row.clone())),
        )?;
        let tag = MyTag::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        tag.clone().update(&mut conn)?;
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = tag.clone();
        edited.name = "Long tag".repeat(12);
        edited.parent_id = 2;
        edited.seq = 19;
        let updated = edited.update(&mut conn)?;
        assert_eq!(tag.clone().update(&mut conn)?, updated);
        let Row::Extended(ExtendedRow::MyTag(updated_row)) = updated.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(updated_row.header.unknown_04, 42);
        assert_eq!(updated_row.unknown_string, row.unknown_string);
        assert_eq!((updated.parent_id, updated.seq), (2, 19));
        for parent in [3, 999] {
            let mut edited = tag.clone();
            edited.parent_id = parent;
            assert!(matches!(
                edited.update(&mut conn),
                Err(Error::InvalidModelField {
                    field: "parent_id",
                    ..
                })
            ));
        }
        let mut edited = category.clone();
        edited.parent_id = 3;
        assert!(edited.update(&mut conn).is_err());
        let mut child_category = NewMyTag::category(4, 0, "Child");
        child_category.parent_id = 1;
        child_category.insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        let mut edited = category.clone();
        edited.parent_id = 4;
        assert!(edited.update(&mut conn).is_err());
        let mut edited = tag.clone();
        edited.attribute = 1;
        assert!(edited.update(&mut conn).is_err());
        let mut edited = tag.clone();
        edited.id = 5;
        assert!(edited.update(&mut conn).is_err());
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = tag;
        edited.name.clear();
        let updated = edited.update(&mut conn)?;
        assert_eq!((updated.parent_id, updated.seq), (2, 19));
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Extended,
        )?;
        assert!(MyTag::find(&mut reopened, &3)?.unwrap().name.is_empty());
        Ok(())
    }

    #[test]
    fn category_deletion_cascades_and_rolls_back_on_resource_failure() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let root = NewMyTag::category(1, 0, "Root").insert(&mut conn)?;
        let mut nested = NewMyTag::category(2, 0, "Nested");
        nested.parent_id = 1;
        nested.insert(&mut conn)?;
        NewMyTag::new(3, 2, 9, "Tag").insert(&mut conn)?;
        NewMyTag::category(4, 10, "Keep").insert(&mut conn)?;
        for tag in [1, 2, 3, 3, 4] {
            NewMyTagContent::new(99, tag).insert(&mut conn)?;
        }
        let before = bytes(&mut conn)?;
        let mut limited =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Extended)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(1),
            ..Default::default()
        })?;
        assert!(matches!(
            MyTag::delete(&mut limited, &1),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut limited)?, before);
        assert_eq!(MyTag::delete(&mut conn, &1)?, 7);
        assert_eq!(MyTag::delete(&mut conn, &1)?, 0);
        assert!(matches!(
            root.update(&mut conn),
            Err(Error::ModelNotFound { .. })
        ));
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Extended,
        )?;
        let tags = MyTag::all(&mut reopened)?;
        assert_eq!(tags.len(), 1);
        assert_eq!((tags[0].id, tags[0].seq), (4, 10));
        let links = MyTagContent::all(&mut reopened)?;
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].my_tag_id, 4);
        Ok(())
    }

    #[test]
    fn associations_support_single_occurrences_pair_removal_and_stale_checks() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let first = NewMyTagContent::new(1, 2).insert(&mut conn)?;
        for _ in 0..2 {
            NewMyTagContent::new(1, 2).insert(&mut conn)?;
        }
        let keep = NewMyTagContent::new(2, 2).insert(&mut conn)?;
        let mut edited = first.clone();
        edited.content_id = 99;
        assert_eq!(edited.delete(&mut conn)?, 1);
        assert!(first.delete(&mut conn).is_err());
        assert_eq!(MyTagContent::remove(&mut conn, &1, &2)?, 2);
        assert_eq!(MyTagContent::remove(&mut conn, &1, &2)?, 0);
        let mut tx = conn.transaction();
        let handle = tx.locate(
            MyTagContent::TABLE,
            keep.raw_record().page(),
            keep.raw_record().slot().slot,
        )?;
        tx.update(
            &handle,
            &Row::Extended(ExtendedRow::TrackMyTag(
                NewMyTagContent::new(3, 2).to_binary()?,
            )),
        )?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            keep.clone().delete(&mut conn),
            Err(Error::StaleModelRow { .. })
        ));
        assert!(MyTagContent::remove(&mut conn, &0, &2).is_err());
        assert!(MyTag::delete(&mut conn, &0).is_err());
        assert_eq!(bytes(&mut conn)?, before);
        let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        assert!(matches!(
            keep.delete(&mut standard),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            MyTag::delete(&mut standard, &2),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            MyTagContent::remove(&mut standard, &1, &2),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }
}

mod history_mutations {
    //! History name edits and deletion of histories or individual occurrences.
    use super::*;
    use rbox_devlib::models::{History, HistoryContent, NewHistory, NewHistoryContent};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn history_updates_follow_original_id_and_preserve_unedited_names() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let history = NewHistory::new(1, "Set").insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        history.clone().update(&mut conn)?;
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = history.clone();
        edited.name = "Long set \u{97f3}".repeat(100);
        let updated = edited.update(&mut conn)?;
        assert_eq!(history.clone().update(&mut conn)?, updated);
        let mut edited = history.clone();
        edited.id = 2;
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let before = bytes(&mut conn)?;
        for name in ["bad\0name".to_owned(), "x".repeat(5000)] {
            let mut edited = history.clone();
            edited.name = name;
            assert!(edited.update(&mut conn).is_err());
        }
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = history.clone();
        edited.name.clear();
        assert!(edited.update(&mut conn)?.name.is_empty());
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(History::find(&mut reopened, &1)?.unwrap().name.is_empty());
        assert_eq!(History::delete(&mut reopened, &1)?, 1);
        assert!(matches!(
            history.update(&mut reopened),
            Err(Error::ModelNotFound { .. })
        ));
        Ok(())
    }

    #[test]
    fn deletion_preserves_duplicates_other_histories_and_contents() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewHistory::new(1, "First").insert(&mut conn)?;
        NewHistory::new(2, "Second").insert(&mut conn)?;
        NewContent::new(1, "/Contents/test.wav").insert(&mut conn)?;
        let first = NewHistoryContent::new(1, 1, 3).insert(&mut conn)?;
        NewHistoryContent::new(1, 1, 3).insert(&mut conn)?;
        NewHistoryContent::new(1, 1, 9).insert(&mut conn)?;
        NewHistoryContent::new(2, 1, 12).insert(&mut conn)?;
        let content = Content::find(&mut conn, &1)?
            .unwrap()
            .raw_record()
            .decode()?;
        let mut edited_snapshot = first.clone();
        edited_snapshot.history_id = 2;
        edited_snapshot.seq = 99;
        assert_eq!(edited_snapshot.delete(&mut conn)?, 1);
        assert!(first.delete(&mut conn).is_err());
        assert_eq!(
            HistoryContent::by_history(&mut conn, &1)?
                .iter()
                .map(|r| r.seq)
                .collect::<Vec<_>>(),
            [3, 9]
        );
        assert_eq!(History::delete(&mut conn, &1)?, 3);
        assert_eq!(History::delete(&mut conn, &1)?, 0);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(History::find(&mut reopened, &1)?.is_none());
        assert!(History::find(&mut reopened, &2)?.is_some());
        let entries = HistoryContent::all(&mut reopened)?;
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].history_id, entries[0].seq), (2, 12));
        assert_eq!(
            Content::find(&mut reopened, &1)?
                .unwrap()
                .raw_record()
                .decode()?,
            content
        );
        Ok(())
    }

    #[test]
    fn stale_occurrences_wrong_namespaces_and_resource_failures_are_rejected() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let history = NewHistory::new(1, "Set").insert(&mut conn)?;
        let entry = NewHistoryContent::new(1, 1, 1).insert(&mut conn)?;
        let mut tx = conn.transaction();
        let handle = tx.locate(
            HistoryContent::TABLE,
            entry.raw_record().page(),
            entry.raw_record().slot().slot,
        )?;
        tx.update(
            &handle,
            &Row::Standard(StandardRow::HistoryEntry(
                NewHistoryContent::new(1, 2, 1).to_binary()?,
            )),
        )?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            entry.clone().delete(&mut conn),
            Err(Error::StaleModelRow { .. })
        ));
        assert!(History::delete(&mut conn, &0).is_err());
        assert_eq!(History::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            history.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            History::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            entry.delete(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let mut limited =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        // The history page can be staged, but the separate entry page cannot.
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(1),
            ..Default::default()
        })?;
        assert!(matches!(
            History::delete(&mut limited, &1),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut limited)?, before);
        Ok(())
    }
}

mod key_mutations {
    //! Key edits, persistence, and atomic content-reference cleanup.
    use super::*;
    use rbox_devlib::binary::TrackHeader;
    use rbox_devlib::models::{ExportProperty, Key, NewKey};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_and_support_growth_stale_snapshots_and_empty_names() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = NewKey::new(1, 17, "House").to_binary()?;
        row.name = DeviceSqlString::encode("House", StringEncoding::Utf16Le)?;
        let mut tx = conn.transaction();
        let handle = tx.insert(Key::TABLE, &Row::Standard(StandardRow::Key(row)))?;
        let key = Key::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert_eq!(key.clone().update(&mut conn)?, key);
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = key.clone();
        edited.name = "House \u{97f3}".repeat(100);
        let updated = edited.update(&mut conn)?;
        assert_ne!(
            updated.raw_record().slot().slot,
            key.raw_record().slot().slot
        );
        assert_eq!(key.clone().update(&mut conn)?, updated);
        assert_eq!(updated.secondary_id, 17);
        let mut secondary_edit = key.clone();
        secondary_edit.secondary_id = 99;
        let secondary_updated = secondary_edit.update(&mut conn)?;
        assert_eq!(secondary_updated.secondary_id, 99);
        assert_eq!(secondary_updated.name, updated.name);
        let Row::Standard(StandardRow::Key(previous_row)) = updated.raw_record().decode()? else {
            panic!()
        };
        let Row::Standard(StandardRow::Key(current_row)) =
            secondary_updated.raw_record().decode()?
        else {
            panic!()
        };
        assert_eq!(previous_row.name, current_row.name);
        let mut edited = key;
        edited.name.clear();
        let updated = edited.update(&mut conn)?;
        assert!(updated.name.is_empty());
        assert_eq!(updated.secondary_id, 99);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert_eq!(Key::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn deletion_clears_only_matching_references_and_preserves_counts_and_binary_fields(
    ) -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("2026-10-25", "Test")?,
        )?;
        NewKey::new(1, 17, "House").insert(&mut conn)?;
        let keep = NewKey::new(2, 29, "Techno").insert(&mut conn)?;
        let mut expected = Vec::new();
        for (id, key_id) in [(1, 1), (2, 2), (3, 1), (4, 0)] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.key_id = key_id;
            content.creation.content_link = 12345;
            let content = content.insert(&mut conn)?;
            let Row::Standard(StandardRow::Track(mut row)) = content.raw_record().decode()? else {
                panic!()
            };
            if key_id == 1 {
                row.header.key_id = 0;
            }
            expected.push(Row::Standard(StandardRow::Track(row)));
        }
        let properties = ExportProperty::all(&mut conn)?;
        assert_eq!(Key::delete(&mut conn, &1)?, 3);
        assert_eq!(Key::delete(&mut conn, &1)?, 0);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(Key::find(&mut reopened, &1)?.is_none());
        assert_eq!(
            Key::find(&mut reopened, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            keep.raw_record().decode()?
        );
        assert_eq!(ExportProperty::all(&mut reopened)?, properties);
        let contents = Content::all(&mut reopened)?;
        assert_eq!(contents.len(), expected.len());
        for (content, expected) in contents.iter().zip(expected) {
            assert_eq!(content.raw_record().decode()?, expected);
        }
        Ok(())
    }

    #[test]
    fn invalid_edits_and_failed_reference_cleanup_leave_database_unchanged() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let key = NewKey::new(1, 17, "House").insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        let mut edited = key.clone();
        edited.id = 2;
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        for name in ["bad\0name".to_owned(), "x".repeat(5000)] {
            let mut edited = key.clone();
            edited.name = name;
            assert!(edited.update(&mut conn).is_err());
        }
        assert!(Key::delete(&mut conn, &0).is_err());
        assert_eq!(Key::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            key.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Key::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.key_id = 1;
            records.push(content.insert(&mut conn)?);
        }
        let record = records[1].raw_record();
        let offset = record.page().0 as usize * conn.header().prefix.page_size as usize
            + PAGE_HEADER_SIZE
            + record.slot().heap_offset as usize
            + TrackHeader::BINARY_SIZE
            + size_of::<u16>();
        let mut before = bytes(&mut conn)?;
        before[offset..offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        let mut broken =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        assert!(Key::delete(&mut broken, &1).is_err());
        assert_eq!(bytes(&mut broken)?, before);
        assert_eq!(broken.modified_page_count(), 0);
        assert_eq!(Key::delete(&mut conn, &1)?, 3);
        assert!(matches!(
            key.update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));
        Ok(())
    }
}

mod label_mutations {
    //! Label edits, persistence, and atomic content-reference cleanup.
    use super::*;
    use rbox_devlib::binary::TrackHeader;
    use rbox_devlib::models::{ExportProperty, Label, NewLabel};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_and_support_growth_stale_snapshots_and_empty_names() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = NewLabel::new(1, "Label A").to_binary()?;
        row.name = DeviceSqlString::encode("Label A", StringEncoding::Utf16Le)?;
        let mut tx = conn.transaction();
        let handle = tx.insert(Label::TABLE, &Row::Standard(StandardRow::Label(row)))?;
        let label = Label::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert_eq!(label.clone().update(&mut conn)?, label);
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = label.clone();
        edited.name = "Label A \u{97f3}".repeat(100);
        let updated = edited.update(&mut conn)?;
        assert_ne!(
            updated.raw_record().slot().slot,
            label.raw_record().slot().slot
        );
        assert_eq!(label.clone().update(&mut conn)?, updated);
        let mut edited = label;
        edited.name.clear();
        let updated = edited.update(&mut conn)?;
        assert!(updated.name.is_empty());
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert_eq!(Label::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn deletion_clears_only_matching_references_and_preserves_counts_and_binary_fields(
    ) -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("2026-10-25", "Test")?,
        )?;
        NewLabel::new(1, "Label A").insert(&mut conn)?;
        let keep = NewLabel::new(2, "Label B").insert(&mut conn)?;
        let mut expected = Vec::new();
        for (id, label_id) in [(1, 1), (2, 2), (3, 1), (4, 0)] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.label_id = label_id;
            content.creation.content_link = 12345;
            let content = content.insert(&mut conn)?;
            let Row::Standard(StandardRow::Track(mut row)) = content.raw_record().decode()? else {
                panic!()
            };
            if label_id == 1 {
                row.header.label_id = 0;
            }
            expected.push(Row::Standard(StandardRow::Track(row)));
        }
        let properties = ExportProperty::all(&mut conn)?;
        assert_eq!(Label::delete(&mut conn, &1)?, 3);
        assert_eq!(Label::delete(&mut conn, &1)?, 0);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(Label::find(&mut reopened, &1)?.is_none());
        assert_eq!(
            Label::find(&mut reopened, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            keep.raw_record().decode()?
        );
        assert_eq!(ExportProperty::all(&mut reopened)?, properties);
        let contents = Content::all(&mut reopened)?;
        assert_eq!(contents.len(), expected.len());
        for (content, expected) in contents.iter().zip(expected) {
            assert_eq!(content.raw_record().decode()?, expected);
        }
        Ok(())
    }

    #[test]
    fn invalid_edits_and_failed_reference_cleanup_leave_database_unchanged() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let label = NewLabel::new(1, "Label A").insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        let mut edited = label.clone();
        edited.id = 2;
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        for name in ["bad\0name".to_owned(), "x".repeat(5000)] {
            let mut edited = label.clone();
            edited.name = name;
            assert!(edited.update(&mut conn).is_err());
        }
        assert!(Label::delete(&mut conn, &0).is_err());
        assert_eq!(Label::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            label.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Label::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.label_id = 1;
            records.push(content.insert(&mut conn)?);
        }
        let record = records[1].raw_record();
        let offset = record.page().0 as usize * conn.header().prefix.page_size as usize
            + PAGE_HEADER_SIZE
            + record.slot().heap_offset as usize
            + TrackHeader::BINARY_SIZE
            + size_of::<u16>();
        let mut before = bytes(&mut conn)?;
        before[offset..offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        let mut broken =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        assert!(Label::delete(&mut broken, &1).is_err());
        assert_eq!(bytes(&mut broken)?, before);
        assert_eq!(broken.modified_page_count(), 0);
        assert_eq!(Label::delete(&mut conn, &1)?, 3);
        assert!(matches!(
            label.update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));
        Ok(())
    }
}

mod genre_mutations {
    //! Genre edits, persistence, and atomic content-reference cleanup.
    use super::*;
    use rbox_devlib::binary::TrackHeader;
    use rbox_devlib::models::{ExportProperty, Genre, NewGenre};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_and_support_growth_stale_snapshots_and_empty_names() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = NewGenre::new(1, "House").to_binary()?;
        row.name = DeviceSqlString::encode("House", StringEncoding::Utf16Le)?;
        let mut tx = conn.transaction();
        let handle = tx.insert(Genre::TABLE, &Row::Standard(StandardRow::Genre(row)))?;
        let genre = Genre::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert_eq!(genre.clone().update(&mut conn)?, genre);
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = genre.clone();
        edited.name = "House \u{97f3}".repeat(100);
        let updated = edited.update(&mut conn)?;
        assert_ne!(
            updated.raw_record().slot().slot,
            genre.raw_record().slot().slot
        );
        assert_eq!(genre.clone().update(&mut conn)?, updated);
        let mut edited = genre;
        edited.name.clear();
        let updated = edited.update(&mut conn)?;
        assert!(updated.name.is_empty());
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert_eq!(Genre::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn deletion_clears_only_matching_references_and_preserves_counts_and_binary_fields(
    ) -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("2026-10-25", "Test")?,
        )?;
        NewGenre::new(1, "House").insert(&mut conn)?;
        let keep = NewGenre::new(2, "Techno").insert(&mut conn)?;
        let mut expected = Vec::new();
        for (id, genre_id) in [(1, 1), (2, 2), (3, 1), (4, 0)] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.genre_id = genre_id;
            content.creation.content_link = 12345;
            let content = content.insert(&mut conn)?;
            let Row::Standard(StandardRow::Track(mut row)) = content.raw_record().decode()? else {
                panic!()
            };
            if genre_id == 1 {
                row.header.genre_id = 0;
            }
            expected.push(Row::Standard(StandardRow::Track(row)));
        }
        let properties = ExportProperty::all(&mut conn)?;
        assert_eq!(Genre::delete(&mut conn, &1)?, 3);
        assert_eq!(Genre::delete(&mut conn, &1)?, 0);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(Genre::find(&mut reopened, &1)?.is_none());
        assert_eq!(
            Genre::find(&mut reopened, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            keep.raw_record().decode()?
        );
        assert_eq!(ExportProperty::all(&mut reopened)?, properties);
        let contents = Content::all(&mut reopened)?;
        assert_eq!(contents.len(), expected.len());
        for (content, expected) in contents.iter().zip(expected) {
            assert_eq!(content.raw_record().decode()?, expected);
        }
        Ok(())
    }

    #[test]
    fn invalid_edits_and_failed_reference_cleanup_leave_database_unchanged() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let genre = NewGenre::new(1, "House").insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        let mut edited = genre.clone();
        edited.id = 2;
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        for name in ["bad\0name".to_owned(), "x".repeat(5000)] {
            let mut edited = genre.clone();
            edited.name = name;
            assert!(edited.update(&mut conn).is_err());
        }
        assert!(Genre::delete(&mut conn, &0).is_err());
        assert_eq!(Genre::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            genre.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Genre::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.genre_id = 1;
            records.push(content.insert(&mut conn)?);
        }
        let record = records[1].raw_record();
        let offset = record.page().0 as usize * conn.header().prefix.page_size as usize
            + PAGE_HEADER_SIZE
            + record.slot().heap_offset as usize
            + TrackHeader::BINARY_SIZE
            + size_of::<u16>();
        let mut before = bytes(&mut conn)?;
        before[offset..offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        let mut broken =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        assert!(Genre::delete(&mut broken, &1).is_err());
        assert_eq!(bytes(&mut broken)?, before);
        assert_eq!(broken.modified_page_count(), 0);
        assert_eq!(Genre::delete(&mut conn, &1)?, 3);
        assert!(matches!(
            genre.update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));
        Ok(())
    }
}

mod artwork_mutations {
    //! Artwork path edits and reference cleanup never manipulate image files.

    use super::*;
    use rbox_devlib::binary::TrackHeader;
    use rbox_devlib::models::{Artwork, ExportProperty, NewArtwork};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_handle_growth_and_refresh_stale_snapshots() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = NewArtwork::new(1, "/Art/original.jpg").to_binary()?;
        row.name = DeviceSqlString::encode("/Art/original.jpg", StringEncoding::Utf16Le)?;
        let mut tx = conn.transaction();
        let handle = tx.insert(Artwork::TABLE, &Row::Standard(StandardRow::Artwork(row)))?;
        let artwork = Artwork::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert_eq!(artwork.clone().update(&mut conn)?, artwork);
        assert_eq!(bytes(&mut conn)?, before);

        let mut edited = artwork.clone();
        edited.path = format!("/Art/{}.jpg", "long-directory/".repeat(80));
        let updated = edited.update(&mut conn)?;
        assert_ne!(
            (
                updated.raw_record().page(),
                updated.raw_record().slot().slot
            ),
            (
                artwork.raw_record().page(),
                artwork.raw_record().slot().slot
            )
        );
        assert_eq!(artwork.clone().update(&mut conn)?, updated);
        let mut stale = artwork;
        stale.path = "Art\\cover-\u{97f3}.jpg".into();
        let updated = stale.update(&mut conn)?;
        assert_eq!(updated.path, "Art\\cover-\u{97f3}.jpg");
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert_eq!(Artwork::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn invalid_paths_ids_namespaces_and_resource_failures_are_atomic() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let artwork = NewArtwork::new(1, "/Art/cover.jpg").insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        let mut edited = artwork.clone();
        edited.id = 2;
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut edited = artwork.clone();
        edited.path.clear();
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::InvalidModelField { field: "path", .. })
        ));
        let mut edited = artwork.clone();
        edited.path = "bad\0path".into();
        assert!(matches!(edited.update(&mut conn), Err(Error::EmbeddedNul)));
        let mut edited = artwork.clone();
        edited.path = "x".repeat(5000);
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::NoRowSpace { .. })
        ));
        assert!(matches!(
            Artwork::delete(&mut conn, &0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(Artwork::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            artwork.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Artwork::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let mut limited =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(0),
            ..Default::default()
        })?;
        let mut edited = artwork.clone();
        edited.path = "/Art/new.jpg".into();
        assert!(matches!(
            edited.update(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(matches!(
            Artwork::delete(&mut limited, &1),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut limited)?, before);
        assert_eq!(Artwork::delete(&mut conn, &1)?, 1);
        assert!(matches!(
            artwork.clone().update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));
        let mut tx = conn.transaction();
        for _ in 0..2 {
            tx.insert(
                Artwork::TABLE,
                &Row::Standard(StandardRow::Artwork(
                    NewArtwork::new(1, "/Art/duplicate.jpg").to_binary()?,
                )),
            )?;
        }
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            artwork.update(&mut conn),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert!(matches!(
            Artwork::delete(&mut conn, &1),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }

    #[test]
    fn deletion_preserves_image_files_and_clears_only_matching_content_references() -> Result<()> {
        let dir = create_temp_dir();
        let image = dir.path().join("cover.jpg");
        fs::write(&image, b"keep image bytes")?;
        let options = CreateOptions::standard("2026-10-25", "Test")?;
        let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
        let artwork = NewArtwork::new(1, image.to_string_lossy()).insert(&mut conn)?;
        NewArtwork::new(2, "/Art/keep.jpg").insert(&mut conn)?;
        let mut expected = Vec::new();
        for (id, artwork_id) in [(1, 1), (2, 2), (3, 1), (4, 0)] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.artwork_id = artwork_id;
            let model = content.insert(&mut conn)?;
            let Row::Standard(StandardRow::Track(mut row)) = model.raw_record().decode()? else {
                panic!()
            };
            if artwork_id == 1 {
                row.header.artwork_id = 0;
            }
            expected.push(Row::Standard(StandardRow::Track(row)));
        }
        let properties = ExportProperty::all(&mut conn)?;
        let keep = Artwork::find(&mut conn, &2)?
            .unwrap()
            .raw_record()
            .decode()?;
        let mut edited = artwork;
        let new_path = dir.path().join("new.jpg");
        edited.path = new_path.to_string_lossy().into_owned();
        edited.update(&mut conn)?;
        assert!(!new_path.exists());
        fs::write(&new_path, b"keep replacement image")?;
        assert_eq!(Artwork::delete(&mut conn, &1)?, 3);
        assert_eq!(Artwork::delete(&mut conn, &1)?, 0);
        assert!(Artwork::find(&mut conn, &1)?.is_none());
        assert_eq!(
            Artwork::find(&mut conn, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            keep
        );
        assert_eq!(ExportProperty::all(&mut conn)?, properties);
        assert_eq!(fs::read(&image)?, b"keep image bytes");
        assert_eq!(fs::read(&new_path)?, b"keep replacement image");
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        let contents = Content::all(&mut reopened)?;
        assert_eq!(contents.len(), expected.len());
        for (content, expected) in contents.into_iter().zip(expected) {
            assert_eq!(content.raw_record().decode()?, expected);
        }
        assert!(Artwork::find(&mut reopened, &1)?.is_none());
        Ok(())
    }

    #[test]
    fn malformed_content_rolls_back_artwork_and_prior_reference_updates() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewArtwork::new(1, "/Art/keep.jpg").insert(&mut conn)?;
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.artwork_id = 1;
            records.push(content.insert(&mut conn)?);
        }
        let record = records[1].raw_record();
        let offset = record.page().0 as usize * conn.header().prefix.page_size as usize
            + PAGE_HEADER_SIZE
            + record.slot().heap_offset as usize
            + TrackHeader::BINARY_SIZE
            + size_of::<u16>();
        let mut before = bytes(&mut conn)?;
        before[offset..offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        let mut conn =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        assert!(Artwork::delete(&mut conn, &1).is_err());
        assert_eq!(conn.modified_page_count(), 0);
        assert!(Artwork::find(&mut conn, &1)?.is_some());
        assert_eq!(Content::find(&mut conn, &1)?.unwrap().artwork_id, 1);
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }
}

mod artist_mutations {
    //! Artist mutations preserve binary data and clear all artist roles atomically.

    use super::*;
    use rbox_devlib::binary::TrackHeader;
    use rbox_devlib::models::{Album, Artist, ExportProperty, NewAlbum, NewArtist};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_and_slot_bits_and_resolve_relocated_snapshots() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = NewArtist::new(1, "Original").to_binary()?;
        row.name = DeviceSqlString::encode("Original", StringEncoding::Utf16Le)?;
        row.header.index_shift = 0x0b;
        let mut tx = conn.transaction();
        let handle = tx.insert(Artist::TABLE, &Row::Standard(StandardRow::Artist(row)))?;
        let artist = Artist::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert_eq!(artist.clone().update(&mut conn)?, artist);
        assert_eq!(bytes(&mut conn)?, before);

        // Intervening low-level changes must survive a model rename.
        let mut tx = conn.transaction();
        let raw = artist.raw_record();
        let handle = tx.locate(Artist::TABLE, raw.page(), raw.slot().slot)?;
        let Row::Standard(StandardRow::Artist(mut row)) = tx.read(&handle)?.decode()? else {
            panic!()
        };
        row.header.index_shift = (row.header.index_shift & !0x1f) | 0x15;
        tx.update(&handle, &Row::Standard(StandardRow::Artist(row)))?;
        tx.commit()?;
        let mut renamed = artist.clone();
        renamed.name = "Long artist name ".repeat(70);
        let renamed = renamed.update(&mut conn)?;
        assert_ne!(
            (
                renamed.raw_record().page(),
                renamed.raw_record().slot().slot
            ),
            (artist.raw_record().page(), artist.raw_record().slot().slot)
        );
        let Row::Standard(StandardRow::Artist(row)) = renamed.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(row.header.index_shift & 0x1f, 0x15);
        // An unchanged stale snapshot preserves the current name.
        assert_eq!(artist.clone().update(&mut conn)?, renamed);
        let mut stale = artist;
        stale.name = "Latest edit".into();
        let updated = stale.update(&mut conn)?;
        assert_eq!(updated.name, "Latest edit");
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert_eq!(Artist::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn invalid_missing_duplicate_and_wrong_namespace_mutations_are_rejected() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let artist = NewArtist::new(1, "Artist").insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        let mut edited = artist.clone();
        edited.id = 2;
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut edited = artist.clone();
        edited.name = "bad\0name".into();
        assert!(matches!(edited.update(&mut conn), Err(Error::EmbeddedNul)));
        let mut edited = artist.clone();
        edited.name = "x".repeat(5000);
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::NoRowSpace { .. })
        ));
        assert!(matches!(
            Artist::delete(&mut conn, &0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(Artist::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            artist.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Artist::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));

        // Reopening removes the preexisting dirty overlay so the budget applies to this edit.
        let mut limited =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(0),
            ..Default::default()
        })?;
        let mut edited = artist.clone();
        edited.name = "New".into();
        assert!(matches!(
            edited.update(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(matches!(
            Artist::delete(&mut limited, &1),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut limited)?, before);

        assert_eq!(Artist::delete(&mut conn, &1)?, 1);
        assert!(matches!(
            artist.clone().update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));
        let mut tx = conn.transaction();
        for _ in 0..2 {
            tx.insert(
                Artist::TABLE,
                &Row::Standard(StandardRow::Artist(
                    NewArtist::new(1, "Duplicate").to_binary()?,
                )),
            )?;
        }
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            artist.update(&mut conn),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert!(matches!(
            Artist::delete(&mut conn, &1),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }

    #[test]
    fn deletion_clears_all_roles_once_per_row_and_preserves_other_binary_fields() -> Result<()> {
        let options = CreateOptions::standard("2026-10-25", "Test")?;
        let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
        NewArtist::new(1, "Delete").insert(&mut conn)?;
        NewArtist::new(2, "Keep").insert(&mut conn)?;
        let mut expected_albums = Vec::new();
        for (id, artist_id) in [(10, 1), (11, 1), (12, 2), (13, 0)] {
            let mut album = NewAlbum::new(id, artist_id, format!("Album {id}"));
            album.unknown_04 = 0x12345678;
            let album = album.insert(&mut conn)?;
            let Row::Standard(StandardRow::Album(mut row)) = album.raw_record().decode()? else {
                panic!()
            };
            if row.header.artist_id == 1 {
                row.header.artist_id = 0;
            }
            expected_albums.push(Row::Standard(StandardRow::Album(row)));
        }
        let mut expected_tracks = Vec::new();
        for (i, roles) in [
            [1, 2, 0, 2],
            [2, 1, 2, 0],
            [0, 2, 1, 2],
            [2, 0, 2, 1],
            [1, 1, 1, 1],
            [2, 2, 2, 2],
            [0, 0, 0, 0],
        ]
        .into_iter()
        .enumerate()
        {
            let mut content = NewContent::new(i as u32 + 1, format!("/Contents/{i}.wav"));
            [
                content.artist_id,
                content.original_artist_id,
                content.composer_id,
                content.remixer_id,
            ] = roles;
            content.album_id = 10;
            let content = content.insert(&mut conn)?;
            let Row::Standard(StandardRow::Track(mut row)) = content.raw_record().decode()? else {
                panic!()
            };
            for reference in [
                &mut row.header.artist_id,
                &mut row.header.original_artist_id,
                &mut row.header.composer_id,
                &mut row.header.remixer_id,
            ] {
                if *reference == 1 {
                    *reference = 0;
                }
            }
            expected_tracks.push(Row::Standard(StandardRow::Track(row)));
        }
        let properties = ExportProperty::all(&mut conn)?;
        let keep = Artist::find(&mut conn, &2)?
            .unwrap()
            .raw_record()
            .decode()?;
        assert_eq!(Artist::delete(&mut conn, &1)?, 8); // Artist + two albums + five tracks.
        assert_eq!(Artist::delete(&mut conn, &1)?, 0);
        assert!(Artist::find(&mut conn, &1)?.is_none());
        assert_eq!(
            Artist::find(&mut conn, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            keep
        );
        assert_eq!(ExportProperty::all(&mut conn)?, properties);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        let albums = Album::all(&mut reopened)?;
        let tracks = Content::all(&mut reopened)?;
        assert_eq!(albums.len(), expected_albums.len());
        assert_eq!(tracks.len(), expected_tracks.len());
        for (album, expected) in albums.into_iter().zip(expected_albums) {
            assert_eq!(album.raw_record().decode()?, expected);
        }
        for (track, expected) in tracks.into_iter().zip(expected_tracks) {
            assert_eq!(track.raw_record().decode()?, expected);
        }
        assert!(Artist::find(&mut reopened, &1)?.is_none());
        Ok(())
    }

    #[test]
    fn malformed_content_rolls_back_artist_album_and_prior_content_changes() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewArtist::new(1, "Keep on failure").insert(&mut conn)?;
        NewAlbum::new(1, 1, "Album").insert(&mut conn)?;
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.composer_id = 1;
            records.push(content.insert(&mut conn)?);
        }
        let record = records[1].raw_record();
        let offset = record.page().0 as usize * conn.header().prefix.page_size as usize
            + PAGE_HEADER_SIZE
            + record.slot().heap_offset as usize
            + TrackHeader::BINARY_SIZE
            + size_of::<u16>();
        let mut before = bytes(&mut conn)?;
        before[offset..offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        let mut conn =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        assert!(Artist::delete(&mut conn, &1).is_err());
        assert_eq!(conn.modified_page_count(), 0);
        assert!(Artist::find(&mut conn, &1)?.is_some());
        assert_eq!(Album::find(&mut conn, &1)?.unwrap().artist_id, 1);
        assert_eq!(Content::find(&mut conn, &1)?.unwrap().composer_id, 1);
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }
}

mod album_mutations {
    //! Album updates preserve current binary data; deletions clear references atomically.

    use super::*;
    use rbox_devlib::binary::TrackHeader;
    use rbox_devlib::models::{Album, ExportProperty, NewAlbum, NewArtist};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_and_current_reserved_fields_and_resolve_relocations() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut binary = NewAlbum::new(1, 0, "Original").to_binary()?;
        binary.name = DeviceSqlString::encode("Original", StringEncoding::Utf16Le)?;
        binary.header.unknown_04 = 0x12345678;
        let mut tx = conn.transaction();
        let handle = tx.insert(Album::TABLE, &Row::Standard(StandardRow::Album(binary)))?;
        let album = Album::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert_eq!(album.clone().update(&mut conn)?, album);
        assert_eq!(bytes(&mut conn)?, before);

        // An artist-only edit preserves the original UTF-16 name frame.
        let mut artist_edit = album.clone();
        artist_edit.artist_id = 42;
        let updated = artist_edit.update(&mut conn)?;
        let Row::Standard(StandardRow::Album(row)) = updated.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(
            row.name,
            DeviceSqlString::encode("Original", StringEncoding::Utf16Le)?
        );

        // A low-level edit made after the snapshot must survive the model update.
        let mut row = row;
        row.header.unknown_10 = 0xabcdef;
        let mut tx = conn.transaction();
        let raw = updated.raw_record();
        let handle = tx.locate(Album::TABLE, raw.page(), raw.slot().slot)?;
        tx.update(&handle, &Row::Standard(StandardRow::Album(row)))?;
        tx.commit()?;
        let mut renamed = album.clone();
        renamed.name = "Long album name ".repeat(70);
        let renamed = renamed.update(&mut conn)?;
        assert_eq!(renamed.artist_id, 42);
        assert_ne!(
            (
                renamed.raw_record().page(),
                renamed.raw_record().slot().slot
            ),
            (album.raw_record().page(), album.raw_record().slot().slot)
        );
        let Row::Standard(StandardRow::Album(row)) = renamed.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(
            (row.header.unknown_04, row.header.unknown_10),
            (0x12345678, 0xabcdef)
        );

        // This snapshot still points to the retired slot, but ID lookup finds the new row.
        let mut stale = album;
        stale.artist_id = 99;
        let updated = stale.update(&mut conn)?;
        assert_eq!(updated.name, renamed.name);
        assert_eq!(updated.artist_id, 99);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert_eq!(Album::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn invalid_updates_and_missing_or_ambiguous_ids_leave_the_database_unchanged() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let album = NewAlbum::new(1, 0, "Album").insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        let mut edited = album.clone();
        edited.id = 2;
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut edited = album.clone();
        edited.name = "bad\0name".into();
        assert!(matches!(edited.update(&mut conn), Err(Error::EmbeddedNul)));
        let mut edited = album.clone();
        edited.name = "x".repeat(5000);
        assert!(matches!(
            edited.update(&mut conn),
            Err(Error::NoRowSpace { .. })
        ));
        assert_eq!(bytes(&mut conn)?, before);

        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            album.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Album::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Album::delete(&mut conn, &0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(Album::delete(&mut conn, &99)?, 0);
        assert_eq!(Album::delete(&mut conn, &1)?, 1);
        assert!(matches!(
            album.clone().update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));

        let mut tx = conn.transaction();
        for _ in 0..2 {
            tx.insert(
                Album::TABLE,
                &Row::Standard(StandardRow::Album(
                    NewAlbum::new(1, 0, "Duplicate").to_binary()?,
                )),
            )?;
        }
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            album.update(&mut conn),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert!(matches!(
            Album::delete(&mut conn, &1),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }

    #[test]
    fn deletion_clears_only_matching_references_and_keeps_counts_and_artists() -> Result<()> {
        let options = CreateOptions::standard("2026-10-25", "Test")?;
        let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
        let artist = NewArtist::new(1, "Artist").insert(&mut conn)?;
        NewAlbum::new(10, artist.id, "Delete").insert(&mut conn)?;
        let other = NewAlbum::new(20, artist.id, "Keep").insert(&mut conn)?;
        let mut expected = Vec::new();
        for (id, album_id) in [(1, 10), (2, 20), (3, 10), (4, 0)] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.album_id = album_id;
            let model = content.insert(&mut conn)?;
            let Row::Standard(StandardRow::Track(mut row)) = model.raw_record().decode()? else {
                panic!()
            };
            if album_id == 10 {
                row.header.album_id = 0;
            }
            expected.push(row);
        }
        let properties = ExportProperty::all(&mut conn)?;
        assert_eq!(Album::delete(&mut conn, &10)?, 3);
        assert_eq!(Album::delete(&mut conn, &10)?, 0);
        assert!(Album::find(&mut conn, &10)?.is_none());
        let kept = Album::find(&mut conn, &20)?.unwrap();
        assert_eq!(kept.raw_record().as_bytes(), other.raw_record().as_bytes());
        assert_eq!(
            rbox_devlib::models::Artist::find(&mut conn, &1)?,
            Some(artist)
        );
        assert_eq!(ExportProperty::all(&mut conn)?, properties);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        let contents = Content::all(&mut reopened)?;
        assert_eq!(contents.len(), expected.len());
        for (model, expected) in contents.into_iter().zip(expected) {
            assert_eq!(
                model.raw_record().decode()?,
                Row::Standard(StandardRow::Track(expected))
            );
        }
        assert!(Album::find(&mut reopened, &10)?.is_none());
        Ok(())
    }

    #[test]
    fn deletion_rolls_back_album_and_earlier_reference_updates_on_decode_failure() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewAlbum::new(1, 0, "Keep on failure").insert(&mut conn)?;
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewContent::new(id, format!("/Contents/{id}.wav"));
            content.album_id = 1;
            records.push(content.insert(&mut conn)?);
        }
        let record = records[1].raw_record();
        let offset = record.page().0 as usize * conn.header().prefix.page_size as usize
            + PAGE_HEADER_SIZE
            + record.slot().heap_offset as usize
            + TrackHeader::BINARY_SIZE
            + size_of::<u16>(); // Skip the far-offset array's marker.
        let mut before = bytes(&mut conn)?;
        // Corrupt the second related track's first string offset, after its valid header.
        before[offset..offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        let mut conn =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        assert!(Album::delete(&mut conn, &1).is_err());
        assert_eq!(conn.modified_page_count(), 0);
        assert!(Album::find(&mut conn, &1)?.is_some());
        assert_eq!(Content::find(&mut conn, &1)?.unwrap().album_id, 1);
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }
}

mod album_relations {
    //! Explicit album artist resolution, detached views and batch reuse.

    use super::*;
    use rbox_devlib::models::{Album, Artist, NewAlbum, NewArtist};

    #[test]
    fn album_views_keep_original_rows_local_edits_and_input_order() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewArtist::new(1, "First artist").insert(&mut conn)?;
        NewArtist::new(2, "Second artist").insert(&mut conn)?;
        NewAlbum::new(10, 1, "First album").insert(&mut conn)?;
        NewAlbum::new(20, 2, "Second album").insert(&mut conn)?;
        let albums = Album::all(&mut conn)?;
        let views = Album::all_with_relations(&mut conn)?;
        assert_eq!(views.len(), 2);
        for (view, album) in views.iter().zip(&albums) {
            assert_eq!(&view.album, album);
            assert_eq!(view.raw_record(), album.raw_record());
        }
        assert_eq!(views[0].name, "First album");
        assert_eq!(views[0].artist.as_ref().unwrap().name, "First artist");
        assert_eq!(views[1].artist.as_ref().unwrap().name, "Second artist");
        assert_eq!(
            Album::find_with_relations(&mut conn, &10)?,
            Some(views[0].clone())
        );
        assert!(!format!("{:?}", views[0]).contains("raw:"));
        assert!(!format!("{:#?}", views[0]).contains("raw:"));
        let mut edited = albums[0].clone();
        edited.name = "Local name".into();
        edited.artist_id = 2;
        let batch =
            Album::resolve_many(&mut conn, vec![albums[1].clone(), edited.clone(), edited])?;
        assert_eq!(
            batch.iter().map(|view| view.id).collect::<Vec<_>>(),
            vec![20, 10, 10]
        );
        assert_eq!(batch[1], batch[2]);
        assert_eq!(batch[1].name, "Local name");
        assert_eq!(batch[1].artist.as_ref().unwrap().name, "Second artist");
        assert_eq!(Album::find(&mut conn, &10)?, Some(albums[0].clone()));
        Ok(())
    }

    #[test]
    fn missing_and_zero_artists_are_optional_and_wrong_namespaces_are_rejected() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let reads = conn.read_stats();
        assert!(Album::resolve_many(&mut conn, Vec::new())?.is_empty());
        assert_eq!(conn.read_stats(), reads);
        assert!(Album::find_with_relations(&mut conn, &99)?.is_none());
        let album = NewAlbum::new(1, 0, "Unassigned").insert(&mut conn)?;
        let reads = conn.read_stats();
        assert!(album.clone().resolve(&mut conn)?.artist.is_none());
        assert_eq!(conn.read_stats(), reads);
        let mut missing = album.clone();
        missing.artist_id = 99;
        let view = missing.resolve(&mut conn)?;
        assert_eq!(view.artist_id, 99);
        assert!(view.artist.is_none());
        NewArtist::new(99, "Now present").insert(&mut conn)?;
        assert_eq!(
            view.album.resolve(&mut conn)?.artist.unwrap().name,
            "Now present"
        );
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            album.resolve(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Album::resolve_many(&mut extended, Vec::new()),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Album::find_with_relations(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Album::all_with_relations(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }

    #[test]
    fn batch_resolution_reuses_artist_reads_without_a_page_cache() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewArtist::new(1, "Shared artist").insert(&mut conn)?;
        NewAlbum::new(1, 1, "Album").insert(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut single =
            DeviceSqlConnection::from_reader(Cursor::new(bytes.clone()), DatabaseKind::Standard)?;
        let mut batch =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        for conn in [&mut single, &mut batch] {
            conn.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
                cached_pages: 0,
                ..Default::default()
            })?;
        }
        let album = Album::find(&mut single, &1)?.unwrap();
        let repeated = Album::find(&mut batch, &1)?.unwrap();
        let before_single = single.read_stats().page_reads;
        let before_batch = batch.read_stats().page_reads;
        album.resolve(&mut single)?;
        let views = Album::resolve_many(&mut batch, vec![repeated; 50])?;
        let single_reads = single.read_stats().page_reads - before_single;
        assert!(single_reads > 0);
        assert_eq!(batch.read_stats().page_reads - before_batch, single_reads);
        assert_eq!(views.len(), 50);
        assert_eq!(batch.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn related_errors_propagate_and_committed_artist_changes_refresh_new_views() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewArtist::new(1, "Original").insert(&mut conn)?;
        let album = NewAlbum::new(1, 1, "Album").insert(&mut conn)?;
        let mut bad = NewArtist::new(2, "Bad").to_binary()?;
        bad.name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let mut tx = conn.transaction();
        tx.insert(Artist::TABLE, &Row::Standard(StandardRow::Artist(bad)))?;
        tx.commit()?;
        // Malformed text in an unrelated artist is not decoded.
        let old = album.clone().resolve(&mut conn)?;
        let mut invalid = album.clone();
        invalid.artist_id = 2;
        assert!(
            matches!(invalid.resolve(&mut conn), Err(Error::ModelText { field: "name", source, .. }) if matches!(*source, Error::InvalidUtf16))
        );
        let raw = old.artist.as_ref().unwrap().raw_record();
        let mut tx = conn.transaction();
        let handle = tx.locate(Artist::TABLE, raw.page(), raw.slot().slot)?;
        tx.update(
            &handle,
            &Row::Standard(StandardRow::Artist(
                NewArtist::new(1, "Updated").to_binary()?,
            )),
        )?;
        tx.commit()?;
        assert_eq!(
            album.clone().resolve(&mut conn)?.artist.unwrap().name,
            "Updated"
        );
        assert_eq!(old.artist.unwrap().name, "Original");
        let mut tx = conn.transaction();
        tx.insert(
            Artist::TABLE,
            &Row::Standard(StandardRow::Artist(
                NewArtist::new(1, "Duplicate").to_binary()?,
            )),
        )?;
        tx.commit()?;
        assert!(
            matches!(album.resolve(&mut conn), Err(Error::DuplicateId { table, id: 1 }) if table == Artist::TABLE)
        );
        Ok(())
    }
}

mod content_relations {
    //! Explicit metadata views, shared targets, missing references and decoding errors.

    use super::*;
    use rbox_devlib::models::{
        Artist, NewAlbum, NewArtist, NewArtwork, NewGenre, NewKey, NewLabel,
    };

    #[test]
    fn resolves_all_roles_album_artist_and_metadata_without_losing_content() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        for (id, name) in [
            (1, "Main"),
            (2, "Remixer"),
            (3, "Original"),
            (4, "Composer"),
            (5, "Album artist"),
        ] {
            NewArtist::new(id, name).insert(&mut conn)?;
        }
        NewAlbum::new(10, 5, "Album").insert(&mut conn)?;
        NewGenre::new(11, "Genre").insert(&mut conn)?;
        NewLabel::new(12, "Label").insert(&mut conn)?;
        NewKey::new(13, 23, "Am").insert(&mut conn)?;
        seed_color(&mut conn, 255, "Custom color")?;
        NewArtwork::new(14, "/PIONEER/Artwork/image.jpg").insert(&mut conn)?;
        let mut input = NewContent::new(100, "/Contents/Track.wav");
        input.artist_id = 1;
        input.remixer_id = 2;
        input.original_artist_id = 3;
        input.composer_id = 4;
        input.album_id = 10;
        input.genre_id = 11;
        input.label_id = 12;
        input.key_id = 13;
        input.color_id = 255;
        input.artwork_id = 14;
        let content = input.insert(&mut conn)?;
        let raw = content.raw_record().clone();
        let view = content.resolve(&mut conn)?;
        assert_eq!(view.id, 100);
        assert_eq!(view.artist.as_ref().unwrap().name, "Main");
        assert_eq!(view.remixer.as_ref().unwrap().name, "Remixer");
        assert_eq!(view.original_artist.as_ref().unwrap().name, "Original");
        assert_eq!(view.composer.as_ref().unwrap().name, "Composer");
        assert_eq!(view.album.as_ref().unwrap().name, "Album");
        assert_eq!(view.album_artist.as_ref().unwrap().name, "Album artist");
        assert_eq!(view.genre.as_ref().unwrap().name, "Genre");
        assert_eq!(view.label.as_ref().unwrap().name, "Label");
        assert_eq!(view.key.as_ref().unwrap().name, "Am");
        assert_eq!(view.color.as_ref().unwrap().id, 255);
        assert_eq!(
            view.artwork.as_ref().unwrap().path,
            "/PIONEER/Artwork/image.jpg"
        );
        assert_eq!(view.raw_record(), &raw);
        assert_eq!(
            Content::find_with_relations(&mut conn, &100)?,
            Some(view.clone())
        );
        assert_eq!(Content::all_with_relations(&mut conn)?, vec![view.clone()]);
        assert!(!format!("{view:?}").contains("raw:"));
        let mut edited = view.content.clone();
        edited.title = "Local title".into();
        edited.artist_id = 2;
        let batch = Content::resolve_many(
            &mut conn,
            vec![edited.clone(), view.content.clone(), edited],
        )?;
        assert_eq!(batch[0].title, "Local title");
        assert_eq!(batch[0].artist.as_ref().unwrap().name, "Remixer");
        assert_eq!(batch[1], view);
        assert_eq!(batch[0], batch[2]);
        assert_eq!(Content::find(&mut conn, &100)?.unwrap().artist_id, 1);
        Ok(())
    }

    #[test]
    fn missing_targets_remain_none_and_empty_resolution_reads_nothing() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let reads = conn.read_stats();
        assert!(Content::resolve_many(&mut conn, Vec::new())?.is_empty());
        assert_eq!(conn.read_stats(), reads);
        assert!(Content::find_with_relations(&mut conn, &123)?.is_none());
        let content = NewContent::new(1, "/Contents/Track.wav").insert(&mut conn)?;
        let reads = conn.read_stats();
        let empty = content.clone().resolve(&mut conn)?;
        assert_eq!(conn.read_stats(), reads);
        assert!(empty.artist.is_none() && empty.album.is_none() && empty.album_artist.is_none());
        let mut missing = content.clone();
        missing.artist_id = 999;
        missing.remixer_id = 999;
        missing.original_artist_id = 999;
        missing.composer_id = 999;
        missing.album_id = 999;
        missing.genre_id = 999;
        missing.label_id = 999;
        missing.key_id = 999;
        missing.artwork_id = 999;
        missing.color_id = 255;
        let view = missing.resolve(&mut conn)?;
        assert!(
            view.artist.is_none()
                && view.remixer.is_none()
                && view.original_artist.is_none()
                && view.composer.is_none()
        );
        assert!(
            view.album.is_none()
                && view.album_artist.is_none()
                && view.genre.is_none()
                && view.label.is_none()
        );
        assert!(view.key.is_none() && view.artwork.is_none() && view.color.is_none());
        assert_eq!(view.artist_id, 999);
        NewAlbum::new(999, 777, "Missing artist").insert(&mut conn)?;
        let view = view.content.resolve(&mut conn)?;
        assert!(view.album.is_some() && view.album_artist.is_none());
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            content.resolve(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Content::resolve_many(&mut extended, Vec::new()),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Content::find_with_relations(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Content::all_with_relations(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }

    #[test]
    fn shared_targets_are_loaded_once_per_batch_even_without_page_cache() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewArtist::new(1, "Shared").insert(&mut conn)?;
        NewAlbum::new(1, 1, "Album").insert(&mut conn)?;
        let mut input = NewContent::new(1, "/Contents/Track.wav");
        input.artist_id = 1;
        input.remixer_id = 1;
        input.original_artist_id = 1;
        input.composer_id = 1;
        input.album_id = 1;
        input.insert(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut single =
            DeviceSqlConnection::from_reader(Cursor::new(bytes.clone()), DatabaseKind::Standard)?;
        let mut batch =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        for conn in [&mut single, &mut batch] {
            conn.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
                cached_pages: 0,
                ..Default::default()
            })?;
        }
        let content = Content::find(&mut single, &1)?.unwrap();
        let repeated = Content::find(&mut batch, &1)?.unwrap();
        let before_single = single.read_stats().page_reads;
        let before_batch = batch.read_stats().page_reads;
        content.resolve(&mut single)?;
        let views = Content::resolve_many(&mut batch, vec![repeated; 50])?;
        let single_reads = single.read_stats().page_reads - before_single;
        let batch_reads = batch.read_stats().page_reads - before_batch;
        assert!(single_reads > 0);
        assert_eq!(batch_reads, single_reads);
        assert_eq!(views.len(), 50);
        assert_eq!(batch.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn referenced_errors_propagate_unreferenced_text_stays_lazy_and_edits_refresh() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewArtist::new(1, "Good").insert(&mut conn)?;
        let mut bad = NewArtist::new(2, "Bad").to_binary()?;
        bad.name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let mut tx = conn.transaction();
        tx.insert(Artist::TABLE, &Row::Standard(StandardRow::Artist(bad)))?;
        tx.commit()?;
        let mut input = NewContent::new(1, "/Contents/Track.wav");
        input.artist_id = 1;
        let content = input.insert(&mut conn)?;
        let old = content.clone().resolve(&mut conn)?;
        assert_eq!(old.artist.as_ref().unwrap().name, "Good");
        let mut invalid = content.clone();
        invalid.artist_id = 2;
        assert!(
            matches!(invalid.resolve(&mut conn), Err(Error::ModelText { table, field: "name", source, .. }) if table == Artist::TABLE && matches!(*source, Error::InvalidUtf16))
        );
        let raw = old.artist.as_ref().unwrap().raw_record();
        let mut tx = conn.transaction();
        let h = tx.locate(Artist::TABLE, raw.page(), raw.slot().slot)?;
        tx.update(
            &h,
            &Row::Standard(StandardRow::Artist(
                NewArtist::new(1, "Updated").to_binary()?,
            )),
        )?;
        tx.commit()?;
        assert_eq!(
            content.clone().resolve(&mut conn)?.artist.unwrap().name,
            "Updated"
        );
        assert_eq!(old.artist.unwrap().name, "Good");
        let mut tx = conn.transaction();
        tx.insert(
            Artist::TABLE,
            &Row::Standard(StandardRow::Artist(
                NewArtist::new(1, "Duplicate").to_binary()?,
            )),
        )?;
        tx.commit()?;
        assert!(
            matches!(content.resolve(&mut conn), Err(Error::DuplicateId { table, id: 1 }) if table == Artist::TABLE)
        );
        Ok(())
    }

    #[test]
    fn native_demo_content_resolves_catalog_rows() -> Result<()> {
        let mut conn = demo()?;
        let rows = Content::all(&mut conn)?;
        let views = Content::all_with_relations(&mut conn)?;
        assert_eq!(views.len(), rows.len());
        assert!(views.iter().any(|view| view.artist.is_some()));
        for (view, content) in views.into_iter().zip(rows) {
            assert_eq!(view.content, content);
            assert_eq!(view.artist, Artist::find(&mut conn, &content.artist_id)?);
        }
        assert_eq!(conn.modified_page_count(), 0);
        Ok(())
    }
}

mod configuration {
    //! Browse configuration, properties and raw recommendation pairs.

    use super::*;
    use rbox_devlib::binary::{
        BrowseCategoryRow, BrowseColumnRow, BrowseSortRow, ExtendedRow, RowRecord,
    };
    use rbox_devlib::models::{
        BrowseCategory, BrowseColumn, BrowseSort, DatabaseProperty, ExportProperty,
        NewRecommendedLike, RecommendedLike,
    };

    // Seed binary rows independently of the model API under test.
    fn seed_browse(
        conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>,
        row: StandardRow,
    ) -> Result<RowRecord> {
        let table = match &row {
            StandardRow::BrowseColumn(_) => BrowseColumn::TABLE,
            StandardRow::BrowseCategory(_) => BrowseCategory::TABLE,
            StandardRow::BrowseSort(_) => BrowseSort::TABLE,
            _ => unreachable!("browse fixture only"),
        };
        let mut tx = conn.transaction();
        let handle = tx.insert(table, &Row::Standard(row))?;
        let record = tx.read(&handle)?;
        tx.commit()?;
        Ok(record)
    }

    #[test]
    fn browse_column_updates_keep_wrapping_and_resolve_growing_labels_by_id() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        for (id, stored) in [(0, "\u{fffa}COLOR\u{fffb}"), (1, "COLOR")] {
            let column = BrowseColumn::try_from(seed_browse(
                &mut conn,
                StandardRow::BrowseColumn(BrowseColumnRow {
                    id,
                    code: 0xabcd,
                    name: DeviceSqlString::encode(stored, StringEncoding::Utf16Le)?,
                }),
            )?)?;
            let mut before = Vec::new();
            conn.write_to(&mut before)?;
            assert_eq!(column.clone().update(&mut conn)?, column);
            let mut after = Vec::new();
            conn.write_to(&mut after)?;
            assert_eq!(before, after);
            let mut edited = column.clone();
            edited.name = "Long label ".repeat(80);
            let updated = edited.update(&mut conn)?;
            assert_eq!(updated.code, 0xabcd);
            let Row::Standard(StandardRow::BrowseColumn(row)) = updated.raw_record().decode()?
            else {
                panic!()
            };
            let expected = if id == 0 {
                format!("\u{fffa}{}\u{fffb}", updated.name)
            } else {
                updated.name.clone()
            };
            assert_eq!(row.name.text()?, expected);
            assert_ne!(
                (column.raw_record().page(), column.raw_record().slot().slot),
                (
                    updated.raw_record().page(),
                    updated.raw_record().slot().slot
                )
            );
            assert_eq!(column.clone().update(&mut conn)?, updated);
            let mut stale = column;
            stale.name = "New label".into();
            assert_eq!(stale.update(&mut conn)?.name, "New label");
        }
        let expected = BrowseColumn::all(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(BrowseColumn::all(&mut reopened)?, expected);
        Ok(())
    }

    #[test]
    fn browse_settings_update_exact_occurrences_and_reject_stale_snapshots() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        for _ in 0..2 {
            seed_browse(
                &mut conn,
                StandardRow::BrowseCategory(BrowseCategoryRow {
                    column_id: 1,
                    selector: 2,
                    unknown_04: 0xee,
                    state: 3,
                    position: 9,
                }),
            )?;
            seed_browse(
                &mut conn,
                StandardRow::BrowseSort(BrowseSortRow {
                    column_id: 1,
                    selector: 2,
                    raw_tail: 0xabcd1234,
                }),
            )?;
        }
        let categories = BrowseCategory::all(&mut conn)?;
        let sorts = BrowseSort::all(&mut conn)?;
        let mut before = Vec::new();
        conn.write_to(&mut before)?;
        categories[0].clone().update(&mut conn)?;
        sorts[0].clone().update(&mut conn)?;
        let mut after = Vec::new();
        conn.write_to(&mut after)?;
        assert_eq!(before, after);
        let mut category = categories[1].clone();
        category.state = 1;
        category.position = 0;
        let category = category.update(&mut conn)?;
        assert_eq!(category.unknown_04, 0xee);
        let mut sort = sorts[1].clone();
        sort.raw_tail = 0xabcd0001;
        let sort = sort.update(&mut conn)?;
        assert_eq!(sort.raw_tail, 0xabcd0001);
        assert!(matches!(
            categories[1].clone().update(&mut conn),
            Err(Error::StaleModelRow { .. })
        ));
        assert!(matches!(
            sorts[1].clone().update(&mut conn),
            Err(Error::StaleModelRow { .. })
        ));
        // Editing one occurrence must not invalidate another identical occurrence.
        categories[0].clone().update(&mut conn)?;
        sorts[0].clone().update(&mut conn)?;
        let expected_categories = BrowseCategory::all(&mut conn)?;
        let expected_sorts = BrowseSort::all(&mut conn)?;
        assert_eq!(
            expected_categories
                .iter()
                .map(|r| (r.state, r.position))
                .collect::<Vec<_>>(),
            [(3, 9), (1, 0)]
        );
        assert_eq!(
            expected_sorts
                .iter()
                .map(|r| r.raw_tail)
                .collect::<Vec<_>>(),
            [0xabcd1234, 0xabcd0001]
        );
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(BrowseCategory::all(&mut reopened)?, expected_categories);
        assert_eq!(BrowseSort::all(&mut reopened)?, expected_sorts);
        let mut tx = conn.transaction();
        let raw = category.raw_record();
        let handle = tx.locate(raw.table(), raw.page(), raw.slot().slot)?;
        tx.delete(&handle)?;
        let raw = sort.raw_record();
        let handle = tx.locate(raw.table(), raw.page(), raw.slot().slot)?;
        tx.delete(&handle)?;
        tx.commit()?;
        assert!(matches!(
            category.update(&mut conn),
            Err(Error::RowNotFound { .. })
        ));
        assert!(matches!(
            sort.update(&mut conn),
            Err(Error::RowNotFound { .. })
        ));
        Ok(())
    }

    #[test]
    fn browse_updates_reject_identity_changes_wrong_kinds_and_failed_writes() -> Result<()> {
        let options = CreateOptions::standard("2026-10-25", "Test")?;
        let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
        let column = BrowseColumn::all(&mut conn)?.remove(0);
        let category = BrowseCategory::all(&mut conn)?.remove(0);
        let sort = BrowseSort::all(&mut conn)?.remove(0);
        let mut changed = column.clone();
        changed.id ^= 1;
        assert!(matches!(
            changed.update(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut changed = column.clone();
        changed.code ^= 1;
        assert!(matches!(
            changed.update(&mut conn),
            Err(Error::InvalidModelField { field: "code", .. })
        ));
        let mut changed = category.clone();
        changed.column_id ^= 1;
        assert!(matches!(
            changed.update(&mut conn),
            Err(Error::InvalidModelField { .. })
        ));
        let mut changed = category.clone();
        changed.selector ^= 1;
        assert!(matches!(
            changed.update(&mut conn),
            Err(Error::InvalidModelField { .. })
        ));
        let mut changed = sort.clone();
        changed.column_id ^= 1;
        assert!(matches!(
            changed.update(&mut conn),
            Err(Error::InvalidModelField { .. })
        ));
        let mut changed = sort.clone();
        changed.selector ^= 1;
        assert!(matches!(
            changed.update(&mut conn),
            Err(Error::InvalidModelField { .. })
        ));
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            column.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            category.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            sort.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let mut before = Vec::new();
        conn.write_to(&mut before)?;
        let mut changed = column.clone();
        changed.name = "bad\0label".into();
        assert!(matches!(changed.update(&mut conn), Err(Error::EmbeddedNul)));
        let mut changed = column.clone();
        changed.name = "x".repeat(5000);
        assert!(matches!(
            changed.update(&mut conn),
            Err(Error::NoRowSpace { .. })
        ));
        let mut after = Vec::new();
        conn.write_to(&mut after)?;
        assert_eq!(before, after);
        let mut limited =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(0),
            ..Default::default()
        })?;
        let mut changed = column.clone();
        changed.name = "New".into();
        assert!(matches!(
            changed.update(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        let mut changed = category;
        changed.state ^= 1;
        assert!(matches!(
            changed.update(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        let mut changed = sort;
        changed.raw_tail ^= 1;
        assert!(matches!(
            changed.update(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        let mut after = Vec::new();
        limited.write_to(&mut after)?;
        assert_eq!(before, after);
        let raw = column.raw_record();
        let mut tx = conn.transaction();
        let handle = tx.locate(raw.table(), raw.page(), raw.slot().slot)?;
        tx.delete(&handle)?;
        tx.commit()?;
        assert!(matches!(
            column.update(&mut conn),
            Err(Error::ModelNotFound { .. })
        ));
        Ok(())
    }

    #[test]
    fn browse_labels_use_plain_text_and_preserve_original_binary_framing() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        for (index, (stored, plain)) in [
            ("\u{fffa}COLOR\u{fffb}", "COLOR"),
            (
                "\u{fffa}Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ\u{fffb}",
                "Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ",
            ),
            ("\u{fffa}\u{fffb}", ""),
            ("\u{fffa} COLOR \u{fffb}", " COLOR "),
            ("A\u{fffa}B\u{fffb}C", "A\u{fffa}B\u{fffb}C"),
            ("COLOR", "COLOR"),
            ("\u{fffa}COLOR", "\u{fffa}COLOR"),
            ("COLOR\u{fffb}", "COLOR\u{fffb}"),
            (
                "\u{fffa}\u{fffa}COLOR\u{fffb}\u{fffb}",
                "\u{fffa}COLOR\u{fffb}",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let binary = rbox_devlib::binary::BrowseColumnRow {
                id: 100 + index as u16,
                code: 0x80,
                name: DeviceSqlString::encode(stored, StringEncoding::Utf16Le)?,
            };
            let mut tx = conn.transaction();
            let handle = tx.insert(
                BrowseColumn::TABLE,
                &Row::Standard(StandardRow::BrowseColumn(binary)),
            )?;
            let record = tx.read(&handle)?;
            let model = BrowseColumn::try_from(record.clone())?;
            assert_eq!(model.name, plain);
            assert_eq!(model.raw_record(), &record);
            tx.commit()?;
        }
        let models = BrowseColumn::all(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(BrowseColumn::all(&mut reopened)?, models);
        Ok(())
    }

    #[test]
    fn browse_models_preserve_raw_settings_framing_and_repeated_rows() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let reads = conn.read_stats();
        drop(BrowseColumn::iter(&mut conn)?);
        drop(BrowseCategory::iter(&mut conn)?);
        drop(BrowseSort::iter(&mut conn)?);
        drop(ExportProperty::iter(&mut conn)?);
        assert_eq!(conn.read_stats(), reads);
        assert!(BrowseColumn::find(&mut conn, &1)?.is_none());
        let mut column = BrowseColumn::try_from(seed_browse(
            &mut conn,
            StandardRow::BrowseColumn(BrowseColumnRow {
                id: u16::MAX,
                code: 0xabcd,
                name: DeviceSqlString::encode("Ã©Å¸Â³Ã¦Â¥Â½", StringEncoding::Utf16Le)?,
            }),
        )?)?;
        assert_eq!((column.id, column.code), (u16::MAX, 0xabcd));
        column.name = "Local edit".into();
        assert_eq!(
            BrowseColumn::find(&mut conn, &u16::MAX)?.unwrap().name,
            "Ã©Å¸Â³Ã¦Â¥Â½"
        );
        let reads = conn.read_stats();
        assert!(BrowseColumn::id_exists(&mut conn, &u16::MAX)?);
        assert!(!BrowseColumn::id_exists(&mut conn, &1)?);
        assert_eq!(conn.read_stats(), reads);
        BrowseColumn::try_from(seed_browse(
            &mut conn,
            StandardRow::BrowseColumn(BrowseColumnRow {
                id: 1,
                code: 0,
                name: DeviceSqlString::encode("", StringEncoding::Utf16Le)?,
            }),
        )?)?;
        assert!(BrowseColumn::id_exists(&mut conn, &1)?);
        for position in [9, 1, 1] {
            BrowseCategory::try_from(seed_browse(
                &mut conn,
                StandardRow::BrowseCategory(BrowseCategoryRow {
                    column_id: u16::MAX,
                    selector: 0,
                    unknown_04: 0xee,
                    state: 0x80,
                    position: position,
                }),
            )?)?;
            BrowseSort::try_from(seed_browse(
                &mut conn,
                StandardRow::BrowseSort(BrowseSortRow {
                    column_id: u16::MAX,
                    selector: 0,
                    raw_tail: 0x1234abcd,
                }),
            )?)?;
        }
        let categories = BrowseCategory::by_column(&mut conn, &u16::MAX)?;
        assert_eq!(
            categories
                .iter()
                .map(|c| (c.unknown_04, c.state, c.position))
                .collect::<Vec<_>>(),
            vec![(0xee, 0x80, 9), (0xee, 0x80, 1), (0xee, 0x80, 1)]
        );
        let sorts = BrowseSort::by_column(&mut conn, &u16::MAX)?;
        assert_eq!(
            sorts
                .iter()
                .map(|s| (s.selector, s.raw_tail))
                .collect::<Vec<_>>(),
            vec![(0, 0x1234abcd); 3]
        );
        assert!(BrowseCategory::by_column(&mut conn, &1)?.is_empty());
        assert!(BrowseSort::by_column(&mut conn, &1)?.is_empty());
        let columns = BrowseColumn::all(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(BrowseColumn::all(&mut reopened)?, columns);
        assert_eq!(BrowseCategory::all(&mut reopened)?, categories);
        assert_eq!(BrowseSort::all(&mut reopened)?, sorts);
        Ok(())
    }

    #[test]
    fn initialized_properties_support_binary_updates_and_track_count_maintenance() -> Result<()> {
        let options = CreateOptions::standard("2026-10-25", "Library Ã©Å¸Â³Ã¦Â¥Â½")?;
        let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
        let properties = ExportProperty::all(&mut conn)?;
        assert_eq!(properties.len(), 1);
        assert_eq!(properties[0].track_count, 0);
        assert_eq!(properties[0].date, "2026-10-25");
        assert_eq!(properties[0].version, "1000");
        let record = properties[0].raw_record();
        let Row::Standard(StandardRow::ExportProperty(mut row)) = record.decode()? else {
            panic!()
        };
        row.track_count = 123;
        row.unknown_08 = 7;
        row.unknown_0a = 0xabcd;
        row.background_color = 0x80;
        row.separator = 0x9876;
        row.version = DeviceSqlString::encode("custom", StringEncoding::ShortAscii)?;
        let binary = Row::Standard(StandardRow::ExportProperty(row));
        let mut tx = conn.transaction();
        let handle = tx.locate(record.table(), record.page(), record.slot().slot)?;
        let handle = tx.update(&handle, &binary)?;
        let first = ExportProperty::try_from(tx.read(&handle)?)?;
        // Readers and count maintenance also tolerate multiple live property rows.
        tx.insert(ExportProperty::TABLE, &binary)?;
        tx.commit()?;
        assert_eq!((first.track_count, first.background_color), (123, 0x80));
        assert_eq!(first.name, "Library Ã©Å¸Â³Ã¦Â¥Â½");
        let Row::Standard(StandardRow::ExportProperty(row)) = first.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(
            (row.unknown_08, row.unknown_0a, row.separator),
            (7, 0xabcd, 0x9876)
        );
        NewContent::new(1, "/Contents/Track.wav").insert(&mut conn)?;
        let properties = ExportProperty::all(&mut conn)?;
        assert_eq!(properties.len(), 2);
        assert!(properties
            .iter()
            .all(|p| p.track_count == 1 && p.version == "custom"));
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(ExportProperty::all(&mut reopened)?, properties);
        Ok(())
    }

    #[test]
    fn extended_properties_and_pairs_preserve_explicit_unknowns_and_duplicates() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let reads = conn.read_stats();
        drop(DatabaseProperty::iter(&mut conn)?);
        drop(RecommendedLike::iter(&mut conn)?);
        assert_eq!(conn.read_stats(), reads);
        assert!(DatabaseProperty::all(&mut conn)?.is_empty());
        assert!(RecommendedLike::all(&mut conn)?.is_empty());
        let mut options = CreateOptions::extended(0)?;
        let table = options
            .tables
            .iter_mut()
            .find(|t| t.table == DatabaseProperty::TABLE)
            .unwrap();
        let Row::Extended(ExtendedRow::DatabaseProperty(row)) = &mut table.rows[0] else {
            panic!()
        };
        row.header.unknown_words = [1, 2, 3, 4, 5];
        table.rows.push(table.rows[0].clone());
        let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Extended, &options)?;
        let model = DatabaseProperty::all(&mut conn)?.remove(0);
        assert_eq!(model.my_tag_master_db_id, 0);
        let Row::Extended(ExtendedRow::DatabaseProperty(row)) = model.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(row.header.unknown_words, [1, 2, 3, 4, 5]);
        assert!(row.unknown_strings.iter().all(|s| s.as_bytes() == [3]));
        let record = model.raw_record();
        let mut binary = record.decode()?;
        let Row::Extended(ExtendedRow::DatabaseProperty(row)) = &mut binary else {
            panic!()
        };
        row.header.my_tag_master_db_id = 42;
        let mut tx = conn.transaction();
        let handle = tx.locate(record.table(), record.page(), record.slot().slot)?;
        let handle = tx.update(&handle, &binary)?;
        let updated = DatabaseProperty::try_from(tx.read(&handle)?)?;
        assert_eq!(updated.my_tag_master_db_id, 42);
        assert_eq!(updated.raw_record().decode()?, binary);
        tx.commit()?;
        assert_eq!(model.my_tag_master_db_id, 0);
        for (first, second) in [(10, 20), (20, 10), (10, 20), (10, 10), (30, 40)] {
            // Existing binary files can contain duplicates even though model
            // insertion now rejects them. Reads must continue to preserve them.
            let row = NewRecommendedLike::new(first, second, u64::MAX, 0x12345678, 0xabcdef01)
                .to_binary()?;
            let mut tx = conn.transaction();
            tx.insert(
                RecommendedLike::TABLE,
                &Row::Extended(ExtendedRow::RecommendLike(row)),
            )?;
            tx.commit()?;
        }
        assert_eq!(RecommendedLike::by_content(&mut conn, &10)?.len(), 4);
        assert!(RecommendedLike::by_content(&mut conn, &99)?.is_empty());
        let pairs = RecommendedLike::all(&mut conn)?;
        assert_eq!(
            pairs
                .iter()
                .map(|p| (p.content_id_1, p.content_id_2))
                .collect::<Vec<_>>(),
            vec![(10, 20), (20, 10), (10, 20), (10, 10), (30, 40)]
        );
        assert!(pairs.iter().all(|p| p.unknown_00 == u64::MAX
            && p.unknown_10 == 0x12345678
            && p.unknown_14 == 0xabcdef01));
        let properties = DatabaseProperty::all(&mut conn)?;
        assert_eq!(properties.len(), 2);
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Extended)?;
        assert_eq!(RecommendedLike::all(&mut reopened)?, pairs);
        assert_eq!(DatabaseProperty::all(&mut reopened)?, properties);
        Ok(())
    }

    #[test]
    fn native_models_retain_raw_windows_and_decoded_fields() -> Result<()> {
        let mut standard = demo()?;
        let bytes = fs::read(export_dir(7).join("rekordbox/exportExt.pdb"))?;
        let mut extended =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Extended)?;
        macro_rules! check {
            ($conn:ident, $model:ident, $namespace:ident, $variant:ident, |$m:ident, $r:ident| $verify:block) => {{
                let raw = $conn.iter_raw($model::TABLE)?.collect::<Result<Vec<_>>>()?;
                let models = $model::all(&mut $conn)?;
                assert_eq!(raw.len(), models.len());
                for ($m, record) in models.into_iter().zip(raw) {
                    assert_eq!($m.raw_record(), &record);
                    let Row::$namespace($namespace::$variant($r)) = record.decode()? else {
                        panic!()
                    };
                    $verify
                }
            }};
        }
        // Match namespace enum names separately from Row's Standard/Extended variants.
        use rbox_devlib::binary::{ExtendedRow as Extended, StandardRow as Standard};
        check!(standard, BrowseColumn, Standard, BrowseColumn, |m, r| {
            assert_eq!((m.id, m.code), (r.id, r.code));
            assert_eq!(r.name.text()?, format!("\u{fffa}{}\u{fffb}", m.name));
            assert!(!m.name.contains(['\u{fffa}', '\u{fffb}']));
            assert_eq!(BrowseColumn::find(&mut standard, &m.id)?, Some(m));
        });
        check!(
            standard,
            BrowseCategory,
            Standard,
            BrowseCategory,
            |m, r| {
                assert_eq!(
                    (m.column_id, m.selector, m.unknown_04, m.state, m.position),
                    (r.column_id, r.selector, r.unknown_04, r.state, r.position)
                );
            }
        );
        check!(standard, BrowseSort, Standard, BrowseSort, |m, r| {
            assert_eq!(
                (m.column_id, m.selector, m.raw_tail),
                (r.column_id, r.selector, r.raw_tail)
            );
        });
        check!(
            standard,
            ExportProperty,
            Standard,
            ExportProperty,
            |m, r| {
                assert_eq!(
                    (m.track_count, m.background_color),
                    (r.track_count, r.background_color)
                );
                assert_eq!(
                    (m.date, m.version, m.name),
                    (r.date.text()?, r.version.text()?, r.name.text()?)
                );
            }
        );
        check!(
            extended,
            DatabaseProperty,
            Extended,
            DatabaseProperty,
            |m, r| {
                assert_eq!(m.my_tag_master_db_id, r.header.my_tag_master_db_id);
            }
        );
        check!(
            extended,
            RecommendedLike,
            Extended,
            RecommendLike,
            |m, r| {
                assert_eq!(
                    (
                        m.unknown_00,
                        m.content_id_1,
                        m.content_id_2,
                        m.unknown_10,
                        m.unknown_14
                    ),
                    (
                        r.unknown_00,
                        r.track_id_1,
                        r.track_id_2,
                        r.unknown_10,
                        r.unknown_14
                    )
                );
            }
        );
        assert_eq!(standard.modified_page_count(), 0);
        assert_eq!(extended.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn configuration_errors_preserve_pages_and_reject_wrong_namespaces() -> Result<()> {
        let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        macro_rules! wrong {
            ($conn:ident, $model:ident) => {{
                assert!(matches!(
                    $model::iter(&mut $conn),
                    Err(Error::WrongDatabaseKind { .. })
                ));
                assert!(matches!(
                    $model::all(&mut $conn),
                    Err(Error::WrongDatabaseKind { .. })
                ));
            }};
            ($conn:ident, $model:ident, $input:expr) => {{
                assert!(matches!(
                    $model::iter(&mut $conn),
                    Err(Error::WrongDatabaseKind { .. })
                ));
                assert!(matches!(
                    $model::all(&mut $conn),
                    Err(Error::WrongDatabaseKind { .. })
                ));
                assert!(matches!(
                    $input.insert(&mut $conn),
                    Err(Error::WrongDatabaseKind { .. })
                ));
            }};
        }
        wrong!(extended, BrowseColumn);
        wrong!(extended, BrowseCategory);
        wrong!(extended, BrowseSort);
        wrong!(extended, ExportProperty);
        wrong!(standard, DatabaseProperty);
        wrong!(
            standard,
            RecommendedLike,
            NewRecommendedLike::new(1, 2, 3, 4, 5)
        );
        assert!(matches!(
            BrowseColumn::find(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            BrowseColumn::id_exists(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            BrowseCategory::by_column(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            BrowseSort::by_column(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            RecommendedLike::by_content(&mut standard, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let column = BrowseColumn::try_from(seed_browse(
            &mut standard,
            StandardRow::BrowseColumn(BrowseColumnRow {
                id: 1,
                code: 0x80,
                name: DeviceSqlString::encode("Genre", StringEncoding::Utf16Le)?,
            }),
        )?)?;
        let record = column.raw_record().clone();
        assert!(matches!(
            BrowseCategory::try_from(record.clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            BrowseSort::try_from(record.clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            ExportProperty::try_from(record.clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            DatabaseProperty::try_from(record.clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            RecommendedLike::try_from(record),
            Err(Error::UnsupportedModelRow { .. })
        ));
        let category = BrowseCategory::try_from(seed_browse(
            &mut standard,
            StandardRow::BrowseCategory(BrowseCategoryRow {
                column_id: 1,
                selector: 1,
                unknown_04: 99,
                state: 1,
                position: 0,
            }),
        )?)?;
        assert!(matches!(
            BrowseColumn::try_from(category.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        let mut before = Vec::new();
        standard.write_to(&mut before)?;
        let state = (standard.page_count(), standard.modified_page_count());
        assert!(matches!(
            NewRecommendedLike::new(0, 2, 3, 4, 5).insert(&mut extended),
            Err(Error::InvalidModelField {
                field: "content_id_1",
                ..
            })
        ));
        assert!(matches!(
            NewRecommendedLike::new(1, 0, 3, 4, 5).insert(&mut extended),
            Err(Error::InvalidModelField {
                field: "content_id_2",
                ..
            })
        ));
        assert_eq!(
            (standard.page_count(), standard.modified_page_count()),
            state
        );
        let mut after = Vec::new();
        standard.write_to(&mut after)?;
        assert_eq!(before, after);
        assert_eq!(extended.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn dirty_page_limits_rollback_recommendation_insertions() -> Result<()> {
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        extended.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(0),
            ..Default::default()
        })?;
        assert!(matches!(
            NewRecommendedLike::new(1, 2, 3, 4, 5).insert(&mut extended),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(RecommendedLike::all(&mut extended)?.is_empty());
        assert_eq!(extended.modified_page_count(), 0);
        Ok(())
    }
    #[test]
    fn malformed_text_keeps_field_context_and_binary_transactions_remain_explicit() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let bad = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let column = BrowseColumnRow {
            id: 1,
            code: 0,
            name: bad.clone(),
        };
        let mut tx = conn.transaction();
        tx.insert(
            BrowseColumn::TABLE,
            &Row::Standard(StandardRow::BrowseColumn(column)),
        )?;
        tx.commit()?;
        assert!(BrowseColumn::id_exists(&mut conn, &1)?);
        assert!(
            matches!(BrowseColumn::find(&mut conn, &1), Err(Error::ModelText { field: "name", source, .. }) if matches!(*source, Error::InvalidUtf16))
        );
        for field in ["date", "version", "name"] {
            let options = CreateOptions::standard("date", "name")?;
            let mut row = options
                .tables
                .iter()
                .flat_map(|table| &table.rows)
                .find_map(|row| match row {
                    Row::Standard(StandardRow::ExportProperty(row)) => Some(row.clone()),
                    _ => None,
                })
                .unwrap();
            match field {
                "date" => row.date = bad.clone(),
                "version" => row.version = bad.clone(),
                _ => row.name = bad.clone(),
            }
            let mut tx = conn.transaction();
            let handle = tx.insert(
                ExportProperty::TABLE,
                &Row::Standard(StandardRow::ExportProperty(row)),
            )?;
            assert!(
                matches!(ExportProperty::try_from(tx.read(&handle)?), Err(Error::ModelText { field: actual, source, .. }) if actual == field && matches!(*source, Error::InvalidUtf16))
            );
            tx.rollback();
        }
        assert!(ExportProperty::all(&mut conn)?.is_empty());
        let mut tx = conn.transaction();
        let h = tx.insert(
            BrowseSort::TABLE,
            &Row::Standard(StandardRow::BrowseSort(BrowseSortRow {
                column_id: 1,
                selector: 0,
                raw_tail: u32::MAX,
            })),
        )?;
        assert_eq!(BrowseSort::try_from(tx.read(&h)?)?.raw_tail, u32::MAX);
        tx.rollback();
        assert!(BrowseSort::all(&mut conn)?.is_empty());
        Ok(())
    }
}

mod history {
    //! History tables remain distinct from regular playlists and retain every play.

    use super::*;
    use rbox_devlib::models::{
        History, HistoryContent, NewHistory, NewHistoryContent, NewPlaylist, PlaylistContent,
    };

    #[test]
    fn history_round_trip_preserves_occurrences_order_and_separate_playlist_ids() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let reads = conn.read_stats();
        drop(History::iter(&mut conn)?);
        drop(HistoryContent::iter(&mut conn)?);
        assert_eq!(conn.read_stats(), reads);
        assert!(History::all(&mut conn)?.is_empty());
        assert!(History::find(&mut conn, &1)?.is_none());
        assert!(HistoryContent::by_history(&mut conn, &1)?.is_empty());
        NewPlaylist::new(1, 0, 1, "Regular playlist").insert(&mut conn)?;
        let mut detached = NewHistory::new(1, "Session Ã©Å¸Â³Ã¦Â¥Â½").insert(&mut conn)?;
        detached.name = "Local edit".into();
        assert_eq!(
            History::find(&mut conn, &1)?.unwrap().name,
            "Session Ã©Å¸Â³Ã¦Â¥Â½"
        );
        let reads = conn.read_stats();
        assert!(History::id_exists(&mut conn, &1)?);
        assert!(!History::id_exists(&mut conn, &2)?);
        assert_eq!(conn.read_stats(), reads);
        NewHistory::new(2, "").insert(&mut conn)?;
        assert!(History::id_exists(&mut conn, &2)?);
        NewContent::new(10, "/Contents/One.wav").insert(&mut conn)?;
        NewContent::new(20, "/Contents/Two.wav").insert(&mut conn)?;
        for (playlist, track, position) in [
            (1, 10, 30),
            (1, 20, 4),
            (1, 10, 4),
            (1, 10, 4),
            (1, 10, 0),
            (2, 20, 1),
        ] {
            NewHistoryContent::new(playlist, track, position).insert(&mut conn)?;
        }
        let ordered = HistoryContent::by_history(&mut conn, &1)?;
        assert_eq!(
            ordered
                .iter()
                .map(|e| (e.content_id, e.seq))
                .collect::<Vec<_>>(),
            vec![(10, 0), (20, 4), (10, 4), (10, 4), (10, 30)]
        );
        assert_ne!(
            ordered[2].raw_record().slot().slot,
            ordered[3].raw_record().slot().slot
        );
        assert_eq!(HistoryContent::by_history(&mut conn, &2)?.len(), 1);
        assert!(HistoryContent::by_history(&mut conn, &99)?.is_empty());
        assert!(PlaylistContent::all(&mut conn)?.is_empty());
        let all = HistoryContent::all(&mut conn)?;
        assert_eq!(
            all.iter()
                .map(|e| (e.history_id, e.content_id, e.seq))
                .collect::<Vec<_>>(),
            vec![
                (1, 10, 30),
                (1, 20, 4),
                (1, 10, 4),
                (1, 10, 4),
                (1, 10, 0),
                (2, 20, 1)
            ]
        );
        // History's on-disk field order differs from regular playlist entries.
        assert_eq!(
            &all[0].raw_record().as_bytes()[..12],
            &[10, 0, 0, 0, 1, 0, 0, 0, 30, 0, 0, 0]
        );
        let playlists = History::all(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(History::all(&mut reopened)?, playlists);
        assert_eq!(HistoryContent::all(&mut reopened)?, all);
        assert_eq!(HistoryContent::by_history(&mut reopened, &1)?, ordered);
        Ok(())
    }

    #[test]
    fn history_insert_errors_preserve_the_connection() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewHistory::new(1, "Original").insert(&mut conn)?;
        let state = (conn.page_count(), conn.modified_page_count());
        let mut before = Vec::new();
        conn.write_to(&mut before)?;
        assert!(matches!(
            NewHistory::new(1, "Duplicate").insert(&mut conn),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert!(matches!(
            NewHistory::new(0, "Zero").insert(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        assert!(matches!(
            NewHistory::new(2, "Bad\0name").insert(&mut conn),
            Err(Error::EmbeddedNul)
        ));
        assert!(matches!(
            NewHistory::new(2, "x".repeat(5000)).insert(&mut conn),
            Err(Error::NoRowSpace { .. })
        ));
        assert!(matches!(
            NewHistoryContent::new(0, 1, 1).insert(&mut conn),
            Err(Error::InvalidModelField {
                field: "history_id",
                ..
            })
        ));
        assert!(matches!(
            NewHistoryContent::new(1, 0, 1).insert(&mut conn),
            Err(Error::InvalidModelField {
                field: "content_id",
                ..
            })
        ));
        assert_eq!((conn.page_count(), conn.modified_page_count()), state);
        let mut after = Vec::new();
        conn.write_to(&mut after)?;
        assert_eq!(after, before);
        let mut limited = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(0),
            ..Default::default()
        })?;
        assert!(matches!(
            NewHistory::new(1, "Limited").insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(matches!(
            NewHistoryContent::new(1, 2, 1).insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(limited.modified_page_count(), 0);
        assert!(History::all(&mut limited)?.is_empty());
        assert!(HistoryContent::all(&mut limited)?.is_empty());
        Ok(())
    }

    #[test]
    fn history_models_reject_extended_connections_and_regular_playlist_rows() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            History::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            History::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            History::find(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            History::id_exists(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            HistoryContent::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            HistoryContent::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            HistoryContent::by_history(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewHistory::new(1, "Wrong").insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewHistoryContent::new(1, 2, 1).insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert_eq!(conn.modified_page_count(), 0);
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let regular = NewPlaylist::new(1, 0, 1, "Regular").insert(&mut conn)?;
        let entry = rbox_devlib::models::NewPlaylistContent::new(1, 10, 2).insert(&mut conn)?;
        assert!(matches!(
            History::try_from(regular.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            HistoryContent::try_from(entry.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        Ok(())
    }

    #[test]
    fn explicit_history_transactions_and_invalid_text_keep_context() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = NewHistory::new(1, "Name").to_binary()?;
        row.name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let mut tx = conn.transaction();
        tx.insert(
            History::TABLE,
            &Row::Standard(StandardRow::HistoryPlaylist(row)),
        )?;
        tx.commit()?;
        assert!(History::id_exists(&mut conn, &1)?);
        assert!(
            matches!(History::find(&mut conn, &1), Err(Error::ModelText { field: "name", source, .. }) if matches!(*source, Error::InvalidUtf16))
        );
        let mut tx = conn.transaction();
        let handle = tx.insert(
            HistoryContent::TABLE,
            &Row::Standard(StandardRow::HistoryEntry(
                NewHistoryContent::new(1, 20, 9).to_binary()?,
            )),
        )?;
        let model = HistoryContent::try_from(tx.read(&handle)?)?;
        assert_eq!((model.history_id, model.content_id, model.seq), (1, 20, 9));
        tx.rollback();
        assert!(HistoryContent::all(&mut conn)?.is_empty());
        Ok(())
    }

    #[test]
    fn native_history_tables_match_binary_rows_including_empty_tables() -> Result<()> {
        let mut conn = demo()?;
        let raw = conn.iter_raw(History::TABLE)?.collect::<Result<Vec<_>>>()?;
        let models = History::all(&mut conn)?;
        assert_eq!(models.len(), raw.len());
        for (model, record) in models.into_iter().zip(raw) {
            let Row::Standard(StandardRow::HistoryPlaylist(row)) = record.decode()? else {
                panic!()
            };
            assert_eq!(model.id, row.id);
            assert_eq!(model.name, row.name.text()?);
            assert_eq!(model.raw_record(), &record);
            assert_eq!(History::find(&mut conn, &model.id)?, Some(model));
        }
        let raw = conn
            .iter_raw(HistoryContent::TABLE)?
            .collect::<Result<Vec<_>>>()?;
        let models = HistoryContent::all(&mut conn)?;
        assert_eq!(models.len(), raw.len());
        for (model, record) in models.into_iter().zip(raw) {
            let Row::Standard(StandardRow::HistoryEntry(row)) = record.decode()? else {
                panic!()
            };
            assert_eq!(
                (model.content_id, model.history_id, model.seq),
                (row.track_id, row.playlist_id, row.position)
            );
            assert_eq!(model.raw_record(), &record);
        }
        assert_eq!(conn.modified_page_count(), 0);
        Ok(())
    }
}

mod tags {
    //! Extended tags and cross-database associations, without example dependencies.

    use super::*;
    use rbox_devlib::binary::{ExtendedRow, MyTagHeader};
    use rbox_devlib::models::{MyTag, MyTagContent, NewMyTag, NewMyTagContent};

    #[test]
    fn fresh_tags_and_associations_round_trip_with_explicit_ids_and_positions() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let reads = conn.read_stats();
        drop(MyTag::iter(&mut conn)?);
        drop(MyTagContent::iter(&mut conn)?);
        assert_eq!(conn.read_stats(), reads);
        assert!(MyTag::all(&mut conn)?.is_empty());
        assert!(MyTag::find(&mut conn, &99)?.is_none());
        assert!(MyTagContent::all(&mut conn)?.is_empty());
        let category = NewMyTag::category(0xffff_ffff, 0, "Mood").insert(&mut conn)?;
        assert_eq!(category.is_category(), Some(true));
        let mut input = NewMyTag::new(2, category.id, 9, "Ã©Å¸Â³Ã¦Â¥Â½");
        input.unknown_04 = 0x12345678;
        input.unknown_08 = 32;
        input.unknown_18 = 17;
        input.unknown_1a = 5;
        let tag = input.insert(&mut conn)?;
        assert_eq!(tag.is_category(), Some(false));
        let Row::Extended(ExtendedRow::MyTag(row)) = tag.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(
            (
                row.header.unknown_04,
                row.header.unknown_08,
                row.header.unknown_18,
                row.header.unknown_1a
            ),
            (0x12345678, 32, 17, 5)
        );
        assert_eq!(row.header.subtype, MyTagHeader::NEAR_SUBTYPE);
        assert_eq!(row.unknown_string.text()?, "");
        let mut unknown = NewMyTag::new(3, category.id, 1, "Unknown attribute");
        unknown.attribute = 0x80;
        assert_eq!(unknown.insert(&mut conn)?.is_category(), None);
        NewMyTag::new(4, category.id, 1, "Tied").insert(&mut conn)?;
        assert_eq!(
            MyTag::by_parent(&mut conn, &category.id)?
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>(),
            vec![3, 4, 2]
        );
        assert_eq!(MyTag::by_parent(&mut conn, &0)?.len(), 1);
        assert!(MyTag::by_parent(&mut conn, &99)?.is_empty());
        assert_eq!(MyTag::find(&mut conn, &2)?.unwrap().name, "Ã©Å¸Â³Ã¦Â¥Â½");
        let reads = conn.read_stats();
        assert!(MyTag::id_exists(&mut conn, &2)?);
        assert!(!MyTag::id_exists(&mut conn, &99)?);
        assert_eq!(conn.read_stats(), reads);
        let mut detached = tag;
        detached.name = "Local only".into();
        assert_eq!(MyTag::find(&mut conn, &2)?.unwrap().name, "Ã©Å¸Â³Ã¦Â¥Â½");

        for (track, tag) in [(10, 2), (10, 3), (10, 2), (20, 2)] {
            NewMyTagContent::new(track, tag).insert(&mut conn)?;
        }
        assert_eq!(
            MyTagContent::by_content(&mut conn, &10)?
                .iter()
                .map(|r| r.my_tag_id)
                .collect::<Vec<_>>(),
            vec![2, 3, 2]
        );
        assert_eq!(
            MyTagContent::by_my_tag(&mut conn, &2)?
                .iter()
                .map(|r| r.content_id)
                .collect::<Vec<_>>(),
            vec![10, 10, 20]
        );
        assert!(MyTagContent::by_content(&mut conn, &99)?.is_empty());
        assert!(MyTagContent::by_my_tag(&mut conn, &99)?.is_empty());
        let tags = MyTag::all(&mut conn)?;
        let links = MyTagContent::all(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Extended)?;
        assert_eq!(MyTag::all(&mut reopened)?, tags);
        assert_eq!(MyTagContent::all(&mut reopened)?, links);
        Ok(())
    }

    #[test]
    fn native_tags_preserve_names_ids_attributes_and_reserved_bytes() -> Result<()> {
        let bytes = fs::read(export_dir(7).join("rekordbox/exportExt.pdb"))?;
        let mut conn =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Extended)?;
        let raw = conn.iter_raw(MyTag::TABLE)?.collect::<Result<Vec<_>>>()?;
        assert!(!raw.is_empty());
        let models = MyTag::all(&mut conn)?;
        assert_eq!(models.len(), raw.len());
        for (model, record) in models.into_iter().zip(&raw) {
            let Row::Extended(ExtendedRow::MyTag(row)) = record.decode()? else {
                panic!()
            };
            assert_eq!(
                (model.id, model.parent_id, model.seq, model.attribute),
                (
                    row.header.id,
                    row.header.parent_id,
                    row.header.position,
                    row.header.attribute
                )
            );
            assert_eq!(model.name, row.name.text()?);
            assert_eq!(model.is_category(), row.header.is_category());
            assert_eq!(model.raw_record(), record);
            assert_eq!(MyTag::find(&mut conn, &model.id)?, Some(model));
        }
        let raw = conn
            .iter_raw(MyTagContent::TABLE)?
            .collect::<Result<Vec<_>>>()?;
        // The demo exports the tag catalog but has no assigned track tags.
        assert!(raw.is_empty());
        assert!(MyTagContent::all(&mut conn)?.is_empty());
        assert_eq!(conn.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn invalid_tag_inputs_and_allocation_failures_do_not_publish_changes() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        NewMyTag::category(1, 0, "Original").insert(&mut conn)?;
        let mut before = Vec::new();
        conn.write_to(&mut before)?;
        let state = (conn.page_count(), conn.modified_page_count());
        assert!(matches!(
            NewMyTag::category(1, 0, "Duplicate").insert(&mut conn),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert!(matches!(
            NewMyTag::category(0, 0, "Zero").insert(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        assert!(matches!(
            NewMyTag::new(2, 1, 0, "Bad\0name").insert(&mut conn),
            Err(Error::EmbeddedNul)
        ));
        // The second string offset exceeds the qualified near-layout range.
        assert!(matches!(
            NewMyTag::new(2, 1, 0, "x".repeat(300)).insert(&mut conn),
            Err(Error::UnsupportedMutation { .. })
        ));
        assert!(matches!(
            NewMyTagContent::new(0, 1).insert(&mut conn),
            Err(Error::InvalidModelField {
                field: "content_id",
                ..
            })
        ));
        assert!(matches!(
            NewMyTagContent::new(1, 0).insert(&mut conn),
            Err(Error::InvalidModelField {
                field: "my_tag_id",
                ..
            })
        ));
        assert_eq!((conn.page_count(), conn.modified_page_count()), state);
        let mut after = Vec::new();
        conn.write_to(&mut after)?;
        assert_eq!(after, before);
        let mut limited = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(0),
            ..Default::default()
        })?;
        assert!(matches!(
            NewMyTag::category(1, 0, "Limited").insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(matches!(
            NewMyTagContent::new(1, 2).insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(limited.modified_page_count(), 0);
        assert!(MyTag::all(&mut limited)?.is_empty());
        assert!(MyTagContent::all(&mut limited)?.is_empty());
        Ok(())
    }

    #[test]
    fn tag_models_reject_standard_connections_and_other_row_types() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        assert!(matches!(
            MyTag::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            MyTag::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            MyTag::find(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            MyTag::id_exists(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            MyTag::by_parent(&mut conn, &0),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            MyTagContent::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            MyTagContent::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            MyTagContent::by_content(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            MyTagContent::by_my_tag(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewMyTag::category(1, 0, "Wrong").insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewMyTagContent::new(1, 2).insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert_eq!(conn.modified_page_count(), 0);
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let tag = NewMyTag::category(1, 0, "Tag").insert(&mut conn)?;
        let link = NewMyTagContent::new(1, 2).insert(&mut conn)?;
        assert!(matches!(
            MyTag::try_from(link.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            MyTagContent::try_from(tag.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        Ok(())
    }

    #[test]
    fn binary_transactions_preserve_reserved_values_and_report_bad_tag_text() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let mut row = NewMyTag::new(2, 1, 0, "Name").to_binary()?;
        row.name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let mut tx = conn.transaction();
        tx.insert(MyTag::TABLE, &Row::Extended(ExtendedRow::MyTag(row)))?;
        tx.commit()?;
        assert!(MyTag::id_exists(&mut conn, &2)?);
        assert!(
            matches!(MyTag::find(&mut conn, &2), Err(Error::ModelText { field: "name", source, .. }) if matches!(*source, Error::InvalidUtf16))
        );
        let mut input = NewMyTagContent::new(7, 2);
        input.unknown_00 = 0xabcdef;
        input.reserved_string_marker = 5;
        input.reserved_tail = [3, 2, 1];
        let row = input.to_binary()?;
        let mut tx = conn.transaction();
        let handle = tx.insert(
            MyTagContent::TABLE,
            &Row::Extended(ExtendedRow::TrackMyTag(row.clone())),
        )?;
        let model = MyTagContent::try_from(tx.read(&handle)?)?;
        assert_eq!(
            model.raw_record().decode()?,
            Row::Extended(ExtendedRow::TrackMyTag(row))
        );
        tx.rollback();
        assert!(MyTagContent::all(&mut conn)?.is_empty());
        Ok(())
    }
}

mod playlists {
    //! Playlist nodes, ordered occurrences, namespace checks and failure behavior.

    use super::*;
    use rbox_devlib::models::{NewPlaylist, NewPlaylistContent, Playlist, PlaylistContent};

    #[test]
    fn folders_and_playlists_keep_explicit_positions_flags_and_reserved_values() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let before = conn.read_stats();
        drop(Playlist::iter(&mut conn)?);
        drop(PlaylistContent::iter(&mut conn)?);
        assert_eq!(conn.read_stats(), before);
        assert!(Playlist::all(&mut conn)?.is_empty());
        assert!(Playlist::find(&mut conn, &1)?.is_none());
        assert!(Playlist::by_parent(&mut conn, &0)?.is_empty());
        assert!(PlaylistContent::all(&mut conn)?.is_empty());

        let mut folder = NewPlaylist::folder(1, 0, 50, "Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ");
        folder.attribute = 0x80;
        folder.unknown_04 = 0xaabbccdd;
        let folder = folder.insert(&mut conn)?;
        assert!(folder.is_folder());
        assert_eq!(folder.attribute, 0x80);
        assert_eq!(folder.name, "Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ");
        let Row::Standard(StandardRow::PlaylistTree(row)) = folder.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(row.unknown_04, 0xaabbccdd);
        let later = NewPlaylist::new(2, 1, 20, "Later").insert(&mut conn)?;
        NewPlaylist::new(3, 1, 4, "Earlier").insert(&mut conn)?;
        NewPlaylist::new(4, 1, 4, "Tied").insert(&mut conn)?;
        NewPlaylist::new(5, 0, 1, "Root playlist").insert(&mut conn)?;
        assert!(!later.is_folder());
        assert_eq!(
            Playlist::by_parent(&mut conn, &1)?
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            vec![3, 4, 2]
        );
        assert_eq!(
            Playlist::by_parent(&mut conn, &0)?
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            vec![5, 1]
        );
        assert!(Playlist::by_parent(&mut conn, &999)?.is_empty());
        assert_eq!(
            Playlist::all(&mut conn)?
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        let current_folder = Playlist::find(&mut conn, &1)?.unwrap();
        assert_eq!(
            current_folder.raw_record().as_bytes(),
            folder.raw_record().as_bytes()
        );
        let reads = conn.read_stats();
        assert!(Playlist::id_exists(&mut conn, &1)?);
        assert!(!Playlist::id_exists(&mut conn, &999)?);
        assert_eq!(conn.read_stats(), reads);
        let mut detached = folder;
        detached.name = "Local edit".into();
        assert_eq!(
            Playlist::find(&mut conn, &1)?.unwrap().name,
            "Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ"
        );
        Ok(())
    }

    #[test]
    fn repeated_occurrences_and_position_ties_survive_save_and_reopen() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewPlaylist::new(1, 0, 1, "Playlist").insert(&mut conn)?;
        NewPlaylist::new(2, 0, 2, "Other").insert(&mut conn)?;
        NewContent::new(10, "/Contents/One.wav").insert(&mut conn)?;
        NewContent::new(20, "/Contents/Two.wav").insert(&mut conn)?;
        let late = NewPlaylistContent::new(1, 10, 30).insert(&mut conn)?;
        let first_tie = NewPlaylistContent::new(1, 20, 4).insert(&mut conn)?;
        let second_tie = NewPlaylistContent::new(1, 10, 4).insert(&mut conn)?;
        let identical = NewPlaylistContent::new(1, 10, 4).insert(&mut conn)?;
        let zero = NewPlaylistContent::new(1, 10, 0).insert(&mut conn)?;
        let other = NewPlaylistContent::new(2, 10, 1).insert(&mut conn)?;
        // Later transactions can change touched flags on earlier snapshots.
        // Compare values and occurrence locations, not transient page metadata.
        let occurrence = |entry: &PlaylistContent| {
            (
                entry.playlist_id,
                entry.content_id,
                entry.seq,
                entry.raw_record().page(),
                entry.raw_record().slot().slot,
            )
        };
        assert_ne!(occurrence(&second_tie), occurrence(&identical));
        let expected = vec![
            occurrence(&zero),
            occurrence(&first_tie),
            occurrence(&second_tie),
            occurrence(&identical),
            occurrence(&late),
        ];
        assert_eq!(
            PlaylistContent::by_playlist(&mut conn, &1)?
                .iter()
                .map(occurrence)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            PlaylistContent::by_playlist(&mut conn, &2)?
                .iter()
                .map(occurrence)
                .collect::<Vec<_>>(),
            vec![occurrence(&other)]
        );
        assert!(PlaylistContent::by_playlist(&mut conn, &999)?.is_empty());
        assert_eq!(
            PlaylistContent::all(&mut conn)?
                .iter()
                .map(occurrence)
                .collect::<Vec<_>>(),
            [&late, &first_tie, &second_tie, &identical, &zero, &other]
                .into_iter()
                .map(occurrence)
                .collect::<Vec<_>>()
        );
        assert_eq!(Content::all(&mut conn)?.len(), 2);
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(
            PlaylistContent::by_playlist(&mut reopened, &1)?
                .iter()
                .map(occurrence)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(Playlist::all(&mut reopened)?.len(), 2);
        Ok(())
    }

    #[test]
    fn rejected_playlist_insertions_leave_existing_pages_unchanged() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewPlaylist::new(1, 0, 1, "Original").insert(&mut conn)?;
        let mut before = Vec::new();
        conn.write_to(&mut before)?;
        let pages = conn.page_count();
        let dirty = conn.modified_page_count();
        assert!(matches!(
            NewPlaylist::folder(1, 0, 2, "Duplicate").insert(&mut conn),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert!(matches!(
            NewPlaylist::new(0, 0, 1, "Zero").insert(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        assert!(matches!(
            NewPlaylist::new(2, 0, 1, "Bad\0name").insert(&mut conn),
            Err(Error::EmbeddedNul)
        ));
        assert!(matches!(
            NewPlaylist::new(2, 0, 1, "x".repeat(5000)).insert(&mut conn),
            Err(Error::NoRowSpace { .. })
        ));
        assert!(matches!(
            NewPlaylistContent::new(0, 1, 1).insert(&mut conn),
            Err(Error::InvalidModelField {
                field: "playlist_id",
                ..
            })
        ));
        assert!(matches!(
            NewPlaylistContent::new(1, 0, 1).insert(&mut conn),
            Err(Error::InvalidModelField {
                field: "content_id",
                ..
            })
        ));
        assert_eq!(
            (conn.page_count(), conn.modified_page_count()),
            (pages, dirty)
        );
        let mut after = Vec::new();
        conn.write_to(&mut after)?;
        assert_eq!(after, before);
        assert!(PlaylistContent::all(&mut conn)?.is_empty());

        let mut limited = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(0),
            ..Default::default()
        })?;
        assert!(matches!(
            NewPlaylist::new(1, 0, 1, "Limited").insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(matches!(
            NewPlaylistContent::new(1, 1, 1).insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(limited.modified_page_count(), 0);
        assert!(Playlist::all(&mut limited)?.is_empty());
        assert!(PlaylistContent::all(&mut limited)?.is_empty());
        Ok(())
    }

    #[test]
    fn native_playlist_models_preserve_raw_rows() -> Result<()> {
        let mut conn = demo()?;
        let nodes = conn
            .iter_raw(Playlist::TABLE)?
            .collect::<Result<Vec<_>>>()?;
        let entries = conn
            .iter_raw(PlaylistContent::TABLE)?
            .collect::<Result<Vec<_>>>()?;
        assert!(!nodes.is_empty());
        assert!(!entries.is_empty());
        let models = Playlist::all(&mut conn)?;
        assert_eq!(models.len(), nodes.len());
        for (model, record) in models.into_iter().zip(nodes) {
            let Row::Standard(StandardRow::PlaylistTree(binary)) = record.decode()? else {
                panic!()
            };
            assert_eq!(
                (model.id, model.parent_id, model.seq, model.attribute),
                (
                    binary.id,
                    binary.parent_id,
                    binary.position,
                    binary.folder_flag
                )
            );
            assert_eq!(model.name, binary.name.text()?);
            assert_eq!(model.raw_record(), &record);
            assert_eq!(Playlist::find(&mut conn, &model.id)?, Some(model));
        }
        let models = PlaylistContent::all(&mut conn)?;
        assert_eq!(models.len(), entries.len());
        for (model, record) in models.into_iter().zip(entries) {
            let Row::Standard(StandardRow::PlaylistEntry(binary)) = record.decode()? else {
                panic!()
            };
            assert_eq!(
                (model.playlist_id, model.content_id, model.seq),
                (binary.playlist_id, binary.track_id, binary.position)
            );
            assert_eq!(model.raw_record(), &record);
        }
        assert_eq!(conn.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn playlist_models_reject_extended_connections_and_wrong_row_types() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            Playlist::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Playlist::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Playlist::find(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Playlist::id_exists(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            Playlist::by_parent(&mut conn, &0),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            PlaylistContent::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            PlaylistContent::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            PlaylistContent::by_playlist(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewPlaylist::new(1, 0, 1, "Wrong namespace").insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewPlaylistContent::new(1, 1, 1).insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert_eq!(conn.modified_page_count(), 0);
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let node = NewPlaylist::new(1, 0, 1, "Node").insert(&mut conn)?;
        let entry = NewPlaylistContent::new(1, 1, 1).insert(&mut conn)?;
        assert!(matches!(
            Playlist::try_from(entry.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            PlaylistContent::try_from(node.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        Ok(())
    }

    #[test]
    fn binary_transactions_remain_explicit_and_bad_name_text_is_contextual() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = NewPlaylist::new(1, 0, 1, "Name").to_binary()?;
        row.name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let mut tx = conn.transaction();
        tx.insert(
            Playlist::TABLE,
            &Row::Standard(StandardRow::PlaylistTree(row)),
        )?;
        tx.commit()?;
        assert!(Playlist::id_exists(&mut conn, &1)?);
        assert!(
            matches!(Playlist::find(&mut conn, &1), Err(Error::ModelText { field: "name", source, .. }) if matches!(*source, Error::InvalidUtf16))
        );
        let mut tx = conn.transaction();
        let handle = tx.insert(
            PlaylistContent::TABLE,
            &Row::Standard(StandardRow::PlaylistEntry(
                NewPlaylistContent::new(1, 10, 7).to_binary()?,
            )),
        )?;
        let staged = PlaylistContent::try_from(tx.read(&handle)?)?;
        assert_eq!(
            (staged.playlist_id, staged.content_id, staged.seq),
            (1, 10, 7)
        );
        tx.rollback();
        assert!(PlaylistContent::all(&mut conn)?.is_empty());
        Ok(())
    }
}

mod catalog {
    //! Shared behavioral checks for the seven catalog model pairs.

    use super::*;
    use rbox_devlib::binary::{AlbumHeader, ArtistHeader, StringOffsets, TableKind};
    use rbox_devlib::models::{
        Album, Artist, Artwork, Color, Genre, Key, Label, NewAlbum, NewArtist, NewArtwork,
        NewGenre, NewKey, NewLabel,
    };

    struct Case {
        table: TableKind,
        insert: fn(&mut MemoryConnection, u32, &str) -> Result<RowRecord>,
        find: fn(&mut MemoryConnection, u32) -> Result<Option<RowRecord>>,
        all: fn(&mut MemoryConnection) -> Result<Vec<RowRecord>>,
        exists: fn(&mut MemoryConnection, u32) -> Result<bool>,
        lazy: fn(&mut MemoryConnection) -> Result<()>,
        decode: fn(RowRecord) -> Result<String>,
    }

    fn cases() -> Vec<Case> {
        vec![
            Case {
                table: Artist::TABLE,
                insert: |conn, id, value| {
                    Ok(NewArtist::new(id, value).insert(conn)?.raw_record().clone())
                },
                find: |conn, id| {
                    Ok(Artist::find(conn, &id)?.map(|model| model.raw_record().clone()))
                },
                all: |conn| {
                    Ok(Artist::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| Artist::id_exists(conn, &id),
                lazy: |conn| {
                    drop(Artist::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(Artist::try_from(record)?.name),
            },
            Case {
                table: Album::TABLE,
                insert: |conn, id, value| {
                    Ok(NewAlbum::new(id, 73, value)
                        .insert(conn)?
                        .raw_record()
                        .clone())
                },
                find: |conn, id| Ok(Album::find(conn, &id)?.map(|model| model.raw_record().clone())),
                all: |conn| {
                    Ok(Album::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| Album::id_exists(conn, &id),
                lazy: |conn| {
                    drop(Album::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(Album::try_from(record)?.name),
            },
            Case {
                table: Genre::TABLE,
                insert: |conn, id, value| {
                    Ok(NewGenre::new(id, value).insert(conn)?.raw_record().clone())
                },
                find: |conn, id| Ok(Genre::find(conn, &id)?.map(|model| model.raw_record().clone())),
                all: |conn| {
                    Ok(Genre::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| Genre::id_exists(conn, &id),
                lazy: |conn| {
                    drop(Genre::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(Genre::try_from(record)?.name),
            },
            Case {
                table: Label::TABLE,
                insert: |conn, id, value| {
                    Ok(NewLabel::new(id, value).insert(conn)?.raw_record().clone())
                },
                find: |conn, id| Ok(Label::find(conn, &id)?.map(|model| model.raw_record().clone())),
                all: |conn| {
                    Ok(Label::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| Label::id_exists(conn, &id),
                lazy: |conn| {
                    drop(Label::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(Label::try_from(record)?.name),
            },
            Case {
                table: Key::TABLE,
                insert: |conn, id, value| {
                    Ok(NewKey::new(id, 91, value)
                        .insert(conn)?
                        .raw_record()
                        .clone())
                },
                find: |conn, id| Ok(Key::find(conn, &id)?.map(|model| model.raw_record().clone())),
                all: |conn| {
                    Ok(Key::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| Key::id_exists(conn, &id),
                lazy: |conn| {
                    drop(Key::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(Key::try_from(record)?.name),
            },
            Case {
                table: Artwork::TABLE,
                insert: |conn, id, value| {
                    Ok(NewArtwork::new(id, value)
                        .insert(conn)?
                        .raw_record()
                        .clone())
                },
                find: |conn, id| {
                    Ok(Artwork::find(conn, &id)?.map(|model| model.raw_record().clone()))
                },
                all: |conn| {
                    Ok(Artwork::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| Artwork::id_exists(conn, &id),
                lazy: |conn| {
                    drop(Artwork::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(Artwork::try_from(record)?.path),
            },
        ]
    }

    #[test]
    fn fresh_catalog_models_round_trip_without_input_databases() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let value = format!("/Ã©Å¸Â³Ã¦Â¥Â½/{}.jpg", "Ã¥ÂÂ".repeat(150));
        for case in cases() {
            let before = conn.read_stats();
            (case.lazy)(&mut conn)?;
            assert_eq!(conn.read_stats(), before);
            assert!((case.all)(&mut conn)?.is_empty());
            assert!((case.find)(&mut conn, 7)?.is_none());

            let first = (case.insert)(&mut conn, 7, &value)?;
            assert_eq!(first.table(), case.table);
            assert_eq!((case.decode)(first.clone())?, value);
            assert_eq!((case.find)(&mut conn, 7)?, Some(first));
            let before = conn.read_stats();
            assert!((case.exists)(&mut conn, 7)?);
            assert!(!(case.exists)(&mut conn, 999)?);
            assert_eq!(conn.read_stats(), before);

            (case.insert)(&mut conn, 11, "Second")?;
            assert_eq!((case.all)(&mut conn)?.len(), 2);
            assert_eq!(
                (case.decode)((case.find)(&mut conn, 11)?.unwrap())?,
                "Second"
            );
        }
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        for case in cases() {
            assert_eq!((case.all)(&mut reopened)?.len(), 2);
            assert_eq!(
                (case.decode)((case.find)(&mut reopened, 7)?.unwrap())?,
                value
            );
        }
        Ok(())
    }

    #[test]
    fn duplicate_invalid_and_oversized_insertions_are_atomic() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        for case in cases() {
            (case.insert)(&mut conn, 7, "Existing")?;
            let mut before = Vec::new();
            conn.write_to(&mut before)?;
            let pages = conn.page_count();
            let dirty = conn.modified_page_count();
            assert!(matches!((case.insert)(&mut conn, 7, "Duplicate"),
                Err(Error::DuplicateId { table, id: 7 }) if table == case.table));
            assert!(matches!(
                (case.insert)(&mut conn, 0, "Zero"),
                Err(Error::InvalidModelField { field: "id", .. })
            ));
            assert!(matches!(
                (case.insert)(&mut conn, 8, "Bad\0text"),
                Err(Error::EmbeddedNul)
            ));
            assert!(matches!(
                (case.insert)(&mut conn, 8, &"x".repeat(5000)),
                Err(Error::NoRowSpace { .. })
            ));
            assert_eq!(
                (conn.page_count(), conn.modified_page_count()),
                (pages, dirty)
            );
            let mut after = Vec::new();
            conn.write_to(&mut after)?;
            assert_eq!(after, before);
            assert!(!(case.exists)(&mut conn, 8)?);
        }
        assert!(matches!(
            NewArtwork::new(8, "").insert(&mut conn),
            Err(Error::InvalidModelField { field: "path", .. })
        ));
        Ok(())
    }

    #[test]
    fn all_catalog_models_reject_extended_connections_and_other_tables() -> Result<()> {
        let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let track = NewContent::new(1, "/Contents/One.wav").insert(&mut standard)?;
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        for case in cases() {
            assert!(matches!(
                (case.lazy)(&mut extended),
                Err(Error::WrongDatabaseKind { .. })
            ));
            assert!(matches!(
                (case.all)(&mut extended),
                Err(Error::WrongDatabaseKind { .. })
            ));
            assert!(matches!(
                (case.find)(&mut extended, 1),
                Err(Error::WrongDatabaseKind { .. })
            ));
            assert!(matches!(
                (case.exists)(&mut extended, 1),
                Err(Error::WrongDatabaseKind { .. })
            ));
            assert!(matches!(
                (case.insert)(&mut extended, 1, "Name"),
                Err(Error::WrongDatabaseKind { .. })
            ));
            assert!(matches!(
                (case.decode)(track.raw_record().clone()),
                Err(Error::UnsupportedModelRow { .. })
            ));
        }
        assert_eq!(extended.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn native_catalog_rows_preserve_original_windows_and_namespaces() -> Result<()> {
        let mut conn = demo()?;
        for case in cases() {
            let records = conn.iter_raw(case.table)?.collect::<Result<Vec<_>>>()?;
            assert_eq!((case.all)(&mut conn)?, records);
            for record in records {
                let id = match record.decode()? {
                    Row::Standard(StandardRow::Artist(row)) => row.header.id,
                    Row::Standard(StandardRow::Album(row)) => row.header.id,
                    Row::Standard(
                        StandardRow::Genre(row)
                        | StandardRow::Label(row)
                        | StandardRow::Artwork(row),
                    ) => row.id,
                    Row::Standard(StandardRow::Key(row)) => row.id,
                    Row::Standard(StandardRow::Color(row)) => u32::from(row.id),
                    _ => panic!("unexpected catalog table"),
                };
                assert_eq!((case.find)(&mut conn, id)?, Some(record));
            }
        }
        assert_eq!(Artist::all(&mut conn)?.len(), 1);
        assert_eq!(Label::all(&mut conn)?.len(), 1);
        assert_eq!(Color::all(&mut conn)?.len(), 8);
        assert_eq!(conn.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn independent_ids_reserved_values_and_far_layouts_survive() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut artist = NewArtist::new(73, "Artist").to_binary()?;
        artist.header.subtype = ArtistHeader::FAR_SUBTYPE;
        artist.strings = StringOffsets::Far {
            marker: 5,
            offsets: [0],
        };
        let mut album = NewAlbum::new(9, 73, "Album");
        album.unknown_04 = 0xaabbccdd;
        album.unknown_10 = 0x11223344;
        let mut album = album.to_binary()?;
        album.header.subtype = AlbumHeader::FAR_SUBTYPE;
        album.strings = StringOffsets::Far {
            marker: 7,
            offsets: [0],
        };
        let mut tx = conn.transaction();
        tx.insert(Artist::TABLE, &Row::Standard(StandardRow::Artist(artist)))?;
        tx.insert(Album::TABLE, &Row::Standard(StandardRow::Album(album)))?;
        tx.commit()?;
        let artist = Artist::find(&mut conn, &73)?.unwrap();
        let mut album = Album::find(&mut conn, &9)?.unwrap();
        assert_eq!(artist.name, "Artist");
        assert_eq!(album.artist_id, artist.id);
        let Row::Standard(StandardRow::Album(binary)) = album.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(
            (binary.header.unknown_04, binary.header.unknown_10),
            (0xaabbccdd, 0x11223344)
        );
        assert!(matches!(
            binary.strings,
            StringOffsets::Far { marker: 7, .. }
        ));
        album.name = "Detached edit".into();
        assert_eq!(Album::find(&mut conn, &9)?.unwrap().name, "Album");

        let key = NewKey::new(5, 91, "Am").insert(&mut conn)?;
        assert_eq!((key.id, key.secondary_id), (5, 91));
        let mut color = rbox_devlib::binary::ColorRow {
            id: 0x1234,
            name: DeviceSqlString::encode("Custom color", StringEncoding::ShortAscii)?,
            unknown_00: 0,
            unknown_04: 0,
            unknown_07: 0,
        };
        color.unknown_00 = 0xabcdef01;
        color.unknown_04 = 0x78;
        color.unknown_07 = 0x56;
        let mut tx = conn.transaction();
        tx.insert(Color::TABLE, &Row::Standard(StandardRow::Color(color)))?;
        tx.commit()?;
        let color = Color::find(&mut conn, &0x1234)?.unwrap();
        assert_eq!(color.id, 0x1234);
        let Row::Standard(StandardRow::Color(binary)) = color.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(
            (binary.unknown_00, binary.unknown_04, binary.unknown_07),
            (0xabcdef01, 0x78, 0x56)
        );
        Ok(())
    }

    #[test]
    fn malformed_catalog_text_keeps_id_lookup_and_reports_field_context() -> Result<()> {
        for case in cases() {
            let mut source = DeviceSqlConnection::create(DatabaseKind::Standard)?;
            let original = (case.insert)(&mut source, 7, "Good text")?;
            let mut row = original.decode()?;
            let name = match &mut row {
                Row::Standard(StandardRow::Artist(row)) => &mut row.name,
                Row::Standard(StandardRow::Album(row)) => &mut row.name,
                Row::Standard(
                    StandardRow::Genre(row) | StandardRow::Label(row) | StandardRow::Artwork(row),
                ) => &mut row.name,
                Row::Standard(StandardRow::Key(row)) => &mut row.name,
                Row::Standard(StandardRow::Color(row)) => &mut row.name,
                _ => panic!("unexpected catalog table"),
            };
            *name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
            let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
            let mut tx = conn.transaction();
            tx.insert(case.table, &row)?;
            tx.commit()?;
            assert!((case.exists)(&mut conn, 7)?);
            assert!(
                matches!((case.find)(&mut conn, 7), Err(Error::ModelText { table, source, field, .. })
                if table == case.table && matches!(*source, Error::InvalidUtf16)
                && field == if table == Artwork::TABLE { "path" } else { "name" })
            );
        }
        Ok(())
    }

    #[test]
    fn tracks_can_reference_all_fresh_catalog_models() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let artist = NewArtist::new(1, "Artist").insert(&mut conn)?;
        let album = NewAlbum::new(2, artist.id, "Album").insert(&mut conn)?;
        let genre = NewGenre::new(3, "House").insert(&mut conn)?;
        let label = NewLabel::new(4, "Label").insert(&mut conn)?;
        let key = NewKey::new(5, 23, "Am").insert(&mut conn)?;
        let color = seed_color(&mut conn, 6, "Blue")?;
        let artwork = NewArtwork::new(7, "/PIONEER/Artwork/00001/a7.jpg").insert(&mut conn)?;
        let mut track = NewContent::new(8, "/Contents/Track.flac");
        track.artist_id = artist.id;
        track.album_id = album.id;
        track.genre_id = genre.id;
        track.label_id = label.id;
        track.key_id = key.id;
        track.color_id = color.id as u8;
        track.artwork_id = artwork.id;
        let model = track.insert(&mut conn)?;
        assert_eq!(Artist::find(&mut conn, &model.artist_id)?, Some(artist));
        assert_eq!(Album::find(&mut conn, &model.album_id)?, Some(album));
        assert_eq!(Genre::find(&mut conn, &model.genre_id)?, Some(genre));
        assert_eq!(Label::find(&mut conn, &model.label_id)?, Some(label));
        assert_eq!(Key::find(&mut conn, &model.key_id)?, Some(key));
        assert_eq!(
            Color::find(&mut conn, &u16::from(model.color_id))?,
            Some(color)
        );
        assert_eq!(Artwork::find(&mut conn, &model.artwork_id)?, Some(artwork));
        Ok(())
    }
}

type MemoryConnection = DeviceSqlConnection<Cursor<Vec<u8>>>;

fn demo() -> Result<MemoryConnection> {
    let bytes = fs::read(export_dir(7).join("rekordbox/export.pdb"))?;
    DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)
}

fn first_record(conn: &mut MemoryConnection) -> Result<RowRecord> {
    conn.iter_raw(Content::TABLE)?.next().expect("demo track")
}

fn track_row(record: &RowRecord) -> Result<TrackRow> {
    let Row::Standard(StandardRow::Track(row)) = record.decode()? else {
        panic!("demo track layout");
    };
    Ok(row)
}

#[test]
fn native_tracks_are_lazy_detached_snapshots_with_original_bytes() -> Result<()> {
    let mut conn = demo()?;
    let before = conn.read_stats();
    drop(Content::iter(&mut conn)?);
    assert_eq!(conn.read_stats(), before);
    let records = conn.iter_raw(Content::TABLE)?.collect::<Result<Vec<_>>>()?;
    let tracks = Content::all(&mut conn)?;
    assert_eq!(tracks.len(), 2);
    for (mut model, record) in tracks.into_iter().zip(records) {
        let row = track_row(&record)?;
        assert_eq!(model.id, row.header.id);
        assert_eq!(model.title, row.text.title.text()?);
        assert_eq!(model.path, row.text.file_path.text()?);
        assert_eq!(model.analysis_path, row.text.analysis_path.text()?);
        assert_eq!(model.bpmx100, row.header.tempo);
        assert_eq!(model.length, row.header.duration);
        assert_eq!(model.raw_record(), &record);
        assert_eq!(model.isrc()?, row.text.isrc.isrc_text()?);
        model.title = "detached edit".into();
        assert_eq!(model.raw_record(), &record);
        assert_ne!(
            Content::find(&mut conn, &model.id)?.unwrap().title,
            model.title
        );
    }
    assert_eq!(conn.modified_page_count(), 0);
    assert!(Content::find(&mut conn, &u32::MAX)?.is_none());
    assert!(!Content::id_exists(&mut conn, &u32::MAX)?);
    Ok(())
}

#[test]
fn known_fields_keep_units_unicode_raw_codes_and_source_ids() -> Result<()> {
    let mut conn = demo()?;
    let mut row = track_row(&first_record(&mut conn)?)?;
    row.header.id = 42;
    row.header.master_content_id = 1234;
    row.header.master_db_id = 5678;
    row.header.tempo = 12850;
    row.header.duration = 321;
    row.header.sample_rate = 48000;
    row.header.file_size = 1234567;
    row.header.audio_format = 65535;
    row.header.rating = 255;
    row.header.artist_id = 0;
    row.text.title = DeviceSqlString::encode("Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ", StringEncoding::Utf16Le)?;
    row.text.file_path =
        DeviceSqlString::encode("/Contents/Mixed Case.wav", StringEncoding::ShortAscii)?;
    row.text.release_date = DeviceSqlString::encode("unparsed date", StringEncoding::ShortAscii)?;
    row.text.isrc = DeviceSqlString::encode_isrc("USABC2400001")?;
    let mut fresh = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let mut tx = fresh.transaction();
    tx.insert(Content::TABLE, &Row::Standard(StandardRow::Track(row)))?;
    tx.commit()?;
    let model = Content::find(&mut fresh, &42)?.unwrap();
    assert_eq!(
        (model.id, model.master_content_id, model.master_db_id),
        (42, 1234, 5678)
    );
    assert_eq!((model.bpmx100, model.length), (12850, 321));
    assert_eq!((model.sample_rate, model.file_size), (48000, 1234567));
    assert_eq!(
        (model.file_type, model.rating, model.artist_id),
        (65535, 255, 0)
    );
    assert_eq!(model.title, "Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ");
    assert_eq!(model.path, "/Contents/Mixed Case.wav");
    assert_eq!(model.release_date, "unparsed date");
    assert_eq!(model.isrc()?, "USABC2400001");
    Ok(())
}

#[test]
fn index_reuse_and_committed_binary_edits_stay_consistent() -> Result<()> {
    let mut conn = demo()?;
    let mut row = track_row(&first_record(&mut conn)?)?;
    row.header.id = 42;
    let mut fresh = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    assert!(!Content::id_exists(&mut fresh, &42)?);
    let mut tx = fresh.transaction();
    tx.insert(Content::TABLE, &Row::Standard(StandardRow::Track(row)))?;
    tx.commit()?;
    let model = Content::find(&mut fresh, &42)?.unwrap();
    assert_eq!(fresh.resource_usage().indexed_rows, 1);
    let before = fresh.read_stats();
    assert!(Content::id_exists(&mut fresh, &42)?);
    assert_eq!(fresh.read_stats(), before);

    let mut changed = track_row(model.raw_record())?;
    changed.header.id = 84;
    let mut tx = fresh.transaction();
    let handle = tx.locate(
        Content::TABLE,
        model.raw_record().page(),
        model.raw_record().slot().slot,
    )?;
    tx.update(&handle, &Row::Standard(StandardRow::Track(changed)))?;
    tx.commit()?;
    assert!(!Content::id_exists(&mut fresh, &42)?);
    let updated = Content::find(&mut fresh, &84)?.unwrap();
    let mut tx = fresh.transaction();
    let handle = tx.locate(
        Content::TABLE,
        updated.raw_record().page(),
        updated.raw_record().slot().slot,
    )?;
    tx.delete(&handle)?;
    tx.commit()?;
    assert!(!Content::id_exists(&mut fresh, &84)?);
    assert!(Content::all(&mut fresh)?.is_empty());
    Ok(())
}

#[test]
fn empty_standard_and_extended_namespaces_are_distinct() -> Result<()> {
    let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    assert!(Content::all(&mut standard)?.is_empty());
    assert!(Content::find(&mut standard, &1)?.is_none());
    let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
    assert!(matches!(
        Content::iter(&mut extended),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        Content::find(&mut extended, &1),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        Content::id_exists(&mut extended, &1),
        Err(Error::WrongDatabaseKind { .. })
    ));
    Ok(())
}

#[test]
fn duplicate_ids_do_not_publish_a_partial_index() -> Result<()> {
    let mut conn = demo()?;
    let row = first_record(&mut conn)?.decode()?;
    let mut fresh = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let mut tx = fresh.transaction();
    tx.insert(Content::TABLE, &row)?;
    tx.insert(Content::TABLE, &row)?;
    tx.commit()?;
    for _ in 0..2 {
        assert!(matches!(
            Content::find(&mut fresh, &1),
            Err(Error::DuplicateId { .. })
        ));
        assert_eq!(fresh.resource_usage().indexed_rows, 0);
    }
    Ok(())
}

#[test]
fn malformed_text_is_contextual_but_id_queries_do_not_decode_it() -> Result<()> {
    let mut conn = demo()?;
    let record = first_record(&mut conn)?;
    let row = track_row(&record)?;
    let row_start = record.page().0 as usize * conn.header().prefix.page_size as usize
        + PAGE_HEADER_SIZE as usize
        + record.slot().heap_offset as usize;
    let title_start = row_start + row.strings.offset(17).unwrap() as usize;
    assert!(row.text.title.as_bytes().len() >= 6);
    let mut bytes = Vec::new();
    conn.write_to(&mut bytes)?;
    // A valid frame containing an unpaired UTF-16 high surrogate.
    bytes[title_start..title_start + 6].copy_from_slice(&[0x90, 6, 0, 0, 0, 0xd8]);
    let mut corrupt = DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
    assert!(Content::id_exists(&mut corrupt, &row.header.id)?);
    match Content::find(&mut corrupt, &row.header.id) {
        Err(Error::ModelText {
            field,
            page,
            slot,
            source,
            ..
        }) => {
            assert_eq!(field, "title");
            assert_eq!((page, slot), (record.page(), record.slot().slot));
            assert!(matches!(*source, Error::InvalidUtf16));
        }
        other => panic!("expected contextual text error, got {other:?}"),
    }
    Ok(())
}

#[test]
fn unknown_track_layout_is_reported_instead_of_skipped() -> Result<()> {
    let mut conn = demo()?;
    let record = first_record(&mut conn)?;
    let start = record.page().0 as usize * conn.header().prefix.page_size as usize
        + PAGE_HEADER_SIZE as usize
        + record.slot().heap_offset as usize;
    let mut bytes = Vec::new();
    conn.write_to(&mut bytes)?;
    bytes[start..start + 2].copy_from_slice(&0xffffu16.to_le_bytes());
    let mut unknown = DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
    assert!(matches!(
        Content::all(&mut unknown),
        Err(Error::UnsupportedModelRow { .. })
    ));
    assert!(matches!(
        Content::find(&mut unknown, &1),
        Err(Error::UnsupportedModelRow { .. })
    ));
    assert_eq!(unknown.resource_usage().indexed_rows, 0);
    Ok(())
}

fn property_counts(conn: &mut MemoryConnection) -> Result<Vec<u32>> {
    use rbox_devlib::binary::{PageType, TableKind};
    conn.iter_raw(TableKind::Standard(PageType::ExportProperties))?
        .map(|record| {
            let Row::Standard(StandardRow::ExportProperty(row)) = record?.decode()? else {
                panic!("expected export property");
            };
            Ok(row.track_count)
        })
        .collect()
}

#[test]
fn fresh_insertions_allocate_rows_and_maintain_counts_without_input_pdbs() -> Result<()> {
    let options = CreateOptions::standard("2026-10-25", "New database")?;
    let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
    let original_pages = conn.page_count();
    for id in 1..=24 {
        let mut input = NewContent::new(id, format!("/Contents/Artist/Track {id}.flac"));
        input.title = format!("Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ {id}");
        input.comment = "Long ASCII comment. ".repeat(10);
        input.bpmx100 = 12850;
        input.length = 321;
        input.file_size = 1234567;
        input.master_content_id = id + 1000;
        input.master_db_id = 123;
        input.analysis_path = format!("/PIONEER/USBANLZ/P000/{id:08X}/ANLZ0000.DAT");
        input.isrc = "USABC2400001".into();
        let inserted = input.clone().insert(&mut conn)?;
        assert_eq!(inserted.title, input.title);
        assert_eq!(inserted.comment, input.comment);
        assert_eq!(inserted.file_name, format!("Track {id}.flac"));
        assert_eq!(inserted.analysis_path, input.analysis_path);
        assert_eq!(inserted.isrc()?, input.isrc);
        assert_eq!((inserted.bpmx100, inserted.length), (12850, 321));
        assert_eq!(
            (inserted.master_content_id, inserted.master_db_id),
            (id + 1000, 123)
        );
        assert_eq!(inserted.file_size, 1234567);
        assert_eq!(Content::find(&mut conn, &id)?, Some(inserted));
    }
    assert!(conn.page_count() > original_pages);
    assert_eq!(property_counts(&mut conn)?, vec![24]);
    let mut bytes = Vec::new();
    conn.write_to(&mut bytes)?;
    let mut reopened =
        DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
    assert_eq!(Content::all(&mut reopened)?.len(), 24);
    assert_eq!(property_counts(&mut reopened)?, vec![24]);
    assert_eq!(
        Content::find(&mut reopened, &24)?.unwrap().title,
        "Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ 24"
    );
    Ok(())
}

#[test]
fn fresh_track_policy_is_explicit_and_overridable() -> Result<()> {
    let mut input = NewContent::new(7, "/Contents/Test.wav");
    let row = input.to_binary()?;
    assert_eq!(row.header.content_link, 0x000c0700);
    assert_eq!(row.header.raw_analysis, 0x29);
    assert_eq!(row.text.unknown_02.text()?, "1");
    assert_eq!(row.text.unknown_03.text()?, "1");
    assert_eq!(row.text.unknown_04.text()?, "1");
    for frame in [
        &row.text.unknown_08,
        &row.text.unknown_09,
        &row.text.unknown_13,
        &row.text.unknown_18,
    ] {
        assert_eq!(frame.text()?, "");
    }
    input.creation.content_link = 0x700;
    input.creation.raw_analysis = 0x1234;
    input.creation.string_marker = 5;
    input.creation.unknown_strings[0] = "opaque".into();
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let inserted = input.insert(&mut conn)?;
    let row = track_row(inserted.raw_record())?;
    assert_eq!(row.header.content_link, 0x700);
    assert_eq!(row.header.raw_analysis, 0x1234);
    assert_eq!(row.text.unknown_02.text()?, "opaque");
    assert!(matches!(
        row.strings,
        rbox_devlib::binary::StringOffsets::Far { marker: 5, .. }
    ));
    assert!(property_counts(&mut conn)?.is_empty());
    Ok(())
}

#[test]
fn rejected_new_tracks_leave_existing_rows_and_pages_unchanged() -> Result<()> {
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    NewContent::new(1, "/Contents/Existing.wav").insert(&mut conn)?;
    let mut before = Vec::new();
    conn.write_to(&mut before)?;
    let pages = conn.page_count();
    let dirty = conn.modified_page_count();
    assert!(matches!(
        NewContent::new(1, "/duplicate.wav").insert(&mut conn),
        Err(Error::DuplicateId { id: 1, .. })
    ));
    assert!(matches!(
        NewContent::new(0, "/invalid.wav").insert(&mut conn),
        Err(Error::InvalidModelField { field: "id", .. })
    ));
    assert!(matches!(
        NewContent::new(2, "").insert(&mut conn),
        Err(Error::InvalidModelField { field: "path", .. })
    ));
    let mut invalid = NewContent::new(2, "/Contents/Invalid.wav");
    invalid.title = "embedded\0NUL".into();
    assert!(matches!(
        invalid.clone().insert(&mut conn),
        Err(Error::EmbeddedNul)
    ));
    invalid.title.clear();
    invalid.isrc = "invalid".into();
    assert!(matches!(
        invalid.clone().insert(&mut conn),
        Err(Error::InvalidIsrc)
    ));
    invalid.isrc.clear();
    invalid.comment = "x".repeat(5000);
    assert!(matches!(
        invalid.insert(&mut conn),
        Err(Error::NoRowSpace { .. })
    ));
    assert_eq!(
        (conn.page_count(), conn.modified_page_count()),
        (pages, dirty)
    );
    let mut after = Vec::new();
    conn.write_to(&mut after)?;
    assert_eq!(after, before);
    assert_eq!(Content::all(&mut conn)?.len(), 1);

    let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
    assert!(matches!(
        NewContent::new(2, "/Contents/Invalid.wav").insert(&mut extended),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert_eq!(extended.modified_page_count(), 0);
    Ok(())
}

#[test]
fn property_failure_rolls_back_the_already_staged_track() -> Result<()> {
    use rbox_devlib::binary::{PageType, TableKind};
    let options = CreateOptions::standard("2026-10-25", "Rollback")?;
    let mut source = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
    let property = source
        .iter_raw(TableKind::Standard(PageType::ExportProperties))?
        .next()
        .unwrap()?;
    let start = property.page().0 as usize * source.header().prefix.page_size as usize
        + PAGE_HEADER_SIZE as usize
        + property.slot().heap_offset as usize;
    let mut bytes = Vec::new();
    source.write_to(&mut bytes)?;
    bytes[start..start + 2].copy_from_slice(&0xffffu16.to_le_bytes());
    let mut conn =
        DeviceSqlConnection::from_reader(Cursor::new(bytes.clone()), DatabaseKind::Standard)?;
    let pages = conn.page_count();
    assert!(matches!(
        NewContent::new(1, "/Contents/Track.wav").insert(&mut conn),
        Err(Error::UnsupportedModelRow { .. })
    ));
    assert!(Content::all(&mut conn)?.is_empty());
    assert_eq!(conn.page_count(), pages);
    assert_eq!(conn.modified_page_count(), 0);
    let mut after = Vec::new();
    conn.write_to(&mut after)?;
    assert_eq!(after, bytes);
    Ok(())
}

#[test]
fn fresh_tracks_persist_and_support_later_insertions() -> Result<()> {
    let temporary = create_temp_dir();
    let path = temporary.path().join("export.pdb");
    let mut fresh = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    NewContent::new(1, "/Contents/One.wav").insert(&mut fresh)?;
    fresh.save_as(&path)?;
    let mut reopened = DeviceSqlConnection::open(&path, DatabaseKind::Standard)?;
    assert!(Content::id_exists(&mut reopened, &1)?);
    NewContent::new(2, "/Contents/Two.wav").insert(&mut reopened)?;
    reopened.save_pages()?;
    drop(reopened);
    let mut reopened = DeviceSqlConnection::open(&path, DatabaseKind::Standard)?;
    assert_eq!(Content::all(&mut reopened)?.len(), 2);
    assert_eq!(
        Content::find(&mut reopened, &2)?.unwrap().file_name,
        "Two.wav"
    );
    Ok(())
}

#[test]
fn new_tracks_can_be_used_in_explicit_binary_transactions() -> Result<()> {
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let input = NewContent::new(1, "/Contents/Track.wav");
    let mut tx = conn.transaction();
    let handle = tx.insert(
        Content::TABLE,
        &Row::Standard(StandardRow::Track(input.to_binary()?)),
    )?;
    assert_eq!(Content::try_from(tx.read(&handle)?)?.id, 1);
    tx.rollback();
    assert!(Content::all(&mut conn)?.is_empty());
    Ok(())
}

#[test]
fn shared_track_conversion_resolves_names_and_preserves_metadata() -> Result<()> {
    use rbox_core::{Track, TrackSource};
    use rbox_devlib::models::{NewAlbum, NewArtist, NewArtwork};
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let artist = NewArtist::new(1, "Artist").insert(&mut conn)?;
    let album_artist = NewArtist::new(2, "Album artist").insert(&mut conn)?;
    let album = NewAlbum::new(3, album_artist.id, "Album");
    let album = album.insert(&mut conn)?;
    let artwork = NewArtwork::new(4, "/art.jpg").insert(&mut conn)?;
    let mut input = NewContent::new(10, "/Contents/track.wav");
    input.artist_id = artist.id;
    input.album_id = album.id;
    input.artwork_id = artwork.id;
    input.master_db_id = 7;
    input.master_content_id = 123;
    input.title = "Track".into();
    input.file_type = 11;
    input.file_size = u32::MAX;
    input.bpmx100 = 12835;
    input.length = 241;
    input.rating = 5;
    input.isrc = "USABC2600001".into();
    input.analysis_path = "/PIONEER/USBANLZ/test/ANLZ0000.DAT".into();
    input.analysis_date = "2026-10-26".into();
    let view = input.clone().insert(&mut conn)?.resolve(&mut conn)?;
    let track = Track::try_from(&view)?;
    assert_eq!(track.source, TrackSource::DeviceLib);
    assert_eq!(track.artist.as_deref(), Some("Artist"));
    assert_eq!(track.album_artist.as_deref(), Some("Album artist"));
    assert_eq!(track.album.as_deref(), Some("Album"));
    assert_eq!(track.artwork_path, Some("/art.jpg".into()));
    assert_eq!(track.master_content_id, Some(123));
    assert_eq!(track.bpm, Some(128.35));
    assert_eq!(track.length, Some(241.0));
    assert!(track.date_created.is_none());
    assert!(track.genre.is_none());
    let mapped = NewContent::try_from(track)?;
    assert_eq!(mapped.id, input.id);
    assert_eq!(mapped.title, input.title);
    assert_eq!(mapped.bpmx100, input.bpmx100);
    assert_eq!(mapped.file_size, input.file_size);
    assert_eq!(mapped.isrc, input.isrc);
    assert_eq!(mapped.analysis_path, input.analysis_path);
    assert_eq!(mapped.master_db_id, input.master_db_id);
    assert_eq!(mapped.artist_id, 0);
    assert_eq!(mapped.album_id, 0);
    assert_eq!(mapped.artwork_id, 0);
    assert!(mapped.analysis_date.is_empty());
    assert_eq!(Track::try_from(view)?.id, Some(10));
    Ok(())
}

#[test]
fn shared_track_insertion_checks_ranges_and_normalizes_paths() -> Result<()> {
    use rbox_core::Track;
    let mut track = Track::new(r"/Contents/Artist\track.wav");
    track.id = Some(1);
    track.bpm = Some(120.126);
    track.length = Some(12.6);
    let row = NewContent::try_from(&track)?;
    assert_eq!(row.path, "/Contents/Artist/track.wav");
    assert_eq!(row.file_name, "track.wav");
    assert_eq!(row.bpmx100, 12013);
    assert_eq!(row.length, 13);
    for value in [f64::NAN, f64::INFINITY, -1.0, f64::from(u32::MAX)] {
        let mut invalid = track.clone();
        invalid.bpm = Some(value);
        assert!(NewContent::try_from(invalid).is_err());
    }
    let mut invalid = track.clone();
    invalid.length = Some(65536.0);
    assert!(NewContent::try_from(invalid).is_err());
    let mut invalid = track.clone();
    invalid.file_size = Some(u64::from(u32::MAX) + 1);
    assert!(NewContent::try_from(invalid).is_err());
    let mut invalid = track.clone();
    invalid.play_count = Some(65536);
    assert!(NewContent::try_from(invalid).is_err());
    let mut invalid = track.clone();
    invalid.file_type = Some(-1);
    assert!(NewContent::try_from(invalid).is_err());
    let mut invalid = track.clone();
    invalid.rating = Some(6);
    assert!(NewContent::try_from(invalid).is_err());
    for id in [None, Some(0)] {
        let mut invalid = track.clone();
        invalid.id = id;
        assert!(NewContent::try_from(invalid).is_err());
    }
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let mut view = row.insert(&mut conn)?.resolve(&mut conn)?;
    view.content.rating = 6;
    assert!(Track::try_from(view).is_err());
    Ok(())
}
