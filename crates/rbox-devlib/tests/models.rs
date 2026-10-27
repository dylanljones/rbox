// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Application models, lazy reads, provenance, ordering, and atomic mutations.

use std::fs;
use std::io::Cursor;

use rbox_devlib::binary::{
    DatabaseKind, DeviceSqlString, Row, RowRecord, StandardRow, StringEncoding, TrackRow,
    PAGE_HEADER_SIZE,
};
use rbox_devlib::models::{DevContent, NewDevContent};
use rbox_devlib::{CreateOptions, DeviceSqlConnection, Error, MyTagType, PlaylistType, Result};
use rbox_testing::data::{create_temp_dir, export_dir};

#[test]
fn insertions_allocate_table_local_ids() -> Result<()> {
    use rbox_devlib::models::*;
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
    macro_rules! check {
        ($model:ident, $new:expr) => {{
            let new = $new;
            assert!(matches!(
                new(1).insert(&mut extended),
                Err(Error::WrongDatabaseKind { .. })
            ));
            let first = new(1).insert(&mut conn)?;
            assert_eq!(first.id, 1);
            assert_eq!(new(2).insert(&mut conn)?.id, 2);
            let row = new(3).insert(&mut conn)?;
            assert_eq!(row.id, 3);
            let raw = row.raw_record();
            let mut tx = conn.transaction();
            let handle = tx.locate($model::TABLE, raw.page(), raw.slot().slot)?;
            tx.delete(&handle)?;
            tx.commit()?;
            assert_eq!(new(4).insert(&mut conn)?.id, 3);

            // Seed sparse imported IDs through the binary engine.
            let mut imported = first.raw_record().decode()?;
            set_row_id(&mut imported, u32::MAX - 1);
            let mut tx = conn.transaction();
            tx.insert($model::TABLE, &imported)?;
            tx.commit()?;
            assert_eq!(new(5).insert(&mut conn)?.id, i64::from(u32::MAX));
            let mut saved = Vec::new();
            conn.write_to(&mut saved)?;
            assert!(matches!(
                new(6).insert(&mut conn),
                Err(Error::InvalidModelField { field: "id", .. })
            ));
            let mut after = Vec::new();
            conn.write_to(&mut after)?;
            assert_eq!(after, saved);
            let mut reopened =
                DeviceSqlConnection::from_reader(Cursor::new(saved), DatabaseKind::Standard)?;
            assert!(matches!(
                new(6).insert(&mut reopened),
                Err(Error::InvalidModelField { field: "id", .. })
            ));
        }};
    }
    check!(DevArtist, |n| NewDevArtist::new(format!("Artist {n}")));
    check!(DevArtwork, |n| NewDevArtwork::new(format!("/Art/{n}.jpg")));
    check!(DevContent, |n| NewDevContent::new(format!(
        "/Contents/{n}.wav"
    )));
    check!(DevGenre, |n| NewDevGenre::new(format!("Genre {n}")));
    check!(DevHistory, |n| NewDevHistory::new(format!("History {n}")));
    check!(DevKey, |n| NewDevKey::new(format!("Key {n}"), 77));
    check!(DevLabel, |n| NewDevLabel::new(format!("Label {n}")));
    check!(DevPlaylist, |n| NewDevPlaylist::playlist(format!(
        "Playlist {n}"
    )));
    Ok(())
}

/// Set a binary fixture's identity without using model insertion rules.
fn set_row_id(row: &mut Row, id: u32) {
    match row {
        Row::Standard(StandardRow::Artist(row)) => row.header.id = id,
        Row::Standard(StandardRow::Album(row)) => row.header.id = id,
        Row::Standard(StandardRow::Track(row)) => row.header.id = id,
        Row::Standard(
            StandardRow::Genre(row)
            | StandardRow::Label(row)
            | StandardRow::Artwork(row)
            | StandardRow::HistoryPlaylist(row),
        ) => row.id = id,
        Row::Standard(StandardRow::Key(row)) => row.id = id,
        Row::Standard(StandardRow::PlaylistTree(row)) => row.id = id,
        _ => panic!("fixture must have an independent ID"),
    }
}

#[test]
fn automatic_ids_reject_duplicate_binary_ids() -> Result<()> {
    use rbox_devlib::models::{DevGenre, NewDevGenre};
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let genre = NewDevGenre::new("Genre").insert(&mut conn)?;
    let row = genre.raw_record().decode()?;
    let mut tx = conn.transaction();
    tx.insert(DevGenre::TABLE, &row)?;
    tx.commit()?;
    assert!(matches!(NewDevGenre::new("New").insert(&mut conn),
        Err(Error::DuplicateId { table, id: 1 }) if table == DevGenre::TABLE));
    Ok(())
}

/// Binary named-row fixture for imported IDs, malformed text and duplicate rows.
fn named_row(id: u32, name: &str) -> Result<rbox_devlib::binary::NamedRow> {
    Ok(rbox_devlib::binary::NamedRow {
        id,
        name: DeviceSqlString::encode(name, StringEncoding::Utf16Le)?,
    })
}

fn artist_row(id: u32, name: &str) -> Result<rbox_devlib::binary::ArtistRow> {
    use rbox_devlib::binary::{ArtistHeader, ArtistRow, StringOffsets};
    Ok(ArtistRow {
        header: ArtistHeader {
            subtype: ArtistHeader::NEAR_SUBTYPE,
            index_shift: 0,
            id,
        },
        strings: StringOffsets::Near {
            marker: 3,
            offsets: [0],
        },
        name: named_row(id, name)?.name,
    })
}

fn playlist_row(
    id: u32,
    input: &rbox_devlib::models::NewDevPlaylist,
) -> Result<rbox_devlib::binary::PlaylistTreeRow> {
    Ok(rbox_devlib::binary::PlaylistTreeRow {
        id,
        parent_id: input.parent_id.try_into().unwrap(),
        unknown_04: input.unknown_04,
        position: input.seq.unwrap_or(0),
        folder_flag: input.attribute as u32,
        name: named_row(id, &input.name)?.name,
    })
}

/// Obtain a valid track fixture through public insertion, then assign its imported ID.
fn content_row(id: u32, input: &NewDevContent) -> Result<TrackRow> {
    let mut scratch = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let model = input.clone().insert(&mut scratch)?;
    let mut row = track_row(model.raw_record())?;
    row.header.id = id;
    Ok(row)
}

/// Seed imported content IDs for lookup and association tests, bypassing model allocation.
fn seed_content(
    conn: &mut MemoryConnection,
    id: u32,
    path: impl Into<String>,
) -> Result<DevContent> {
    let row = content_row(id, &NewDevContent::new(path))?;
    let mut tx = conn.transaction();
    let handle = tx.insert(DevContent::TABLE, &Row::Standard(StandardRow::Track(row)))?;
    let model = DevContent::try_from(tx.read(&handle)?)?;
    tx.commit()?;
    Ok(model)
}

/// Binary fixtures deliberately bypass model validation, ordering and uniqueness.
fn my_tag_row(input: &rbox_devlib::models::NewDevMyTag) -> Result<rbox_devlib::binary::MyTagRow> {
    use rbox_devlib::binary::{MyTagHeader, MyTagRow, StringOffsets};
    Ok(MyTagRow {
        header: MyTagHeader {
            subtype: MyTagHeader::NEAR_SUBTYPE,
            index_shift: 0,
            unknown_04: input.unknown_04,
            unknown_08: input.unknown_08,
            parent_id: input.parent_id.try_into().unwrap(),
            position: input.seq.unwrap_or(0),
            id: input.id.try_into().unwrap(),
            unknown_18: input.unknown_18,
            unknown_1a: input.unknown_1a,
            attribute: input.attribute as u8,
        },
        strings: StringOffsets::Near {
            marker: 3,
            offsets: [0; 2],
        },
        name: named_row(input.id.try_into().unwrap(), &input.name)?.name,
        unknown_string: DeviceSqlString::encode("", StringEncoding::ShortAscii)?,
    })
}

fn tag_content_row(
    input: &rbox_devlib::models::NewDevMyTagContent,
) -> rbox_devlib::binary::TrackMyTagRow {
    rbox_devlib::binary::TrackMyTagRow {
        unknown_00: input.unknown_00,
        track_id: input.content_id.try_into().unwrap(),
        tag_id: input.my_tag_id.try_into().unwrap(),
        reserved_string_marker: input.reserved_string_marker,
        reserved_tail: input.reserved_tail,
    }
}

fn playlist_entry_row(
    input: &rbox_devlib::models::NewDevPlaylistContent,
) -> rbox_devlib::binary::PlaylistEntryRow {
    rbox_devlib::binary::PlaylistEntryRow {
        position: input.seq.unwrap_or(1),
        track_id: input.content_id.try_into().unwrap(),
        playlist_id: input.playlist_id.try_into().unwrap(),
    }
}

fn history_entry_row(
    input: &rbox_devlib::models::NewDevHistoryContent,
) -> rbox_devlib::binary::HistoryEntryRow {
    rbox_devlib::binary::HistoryEntryRow {
        position: input.seq,
        track_id: input.content_id.try_into().unwrap(),
        playlist_id: input.history_id.try_into().unwrap(),
    }
}

fn recommendation_row(
    input: &rbox_devlib::models::NewDevRecommendedLike,
) -> rbox_devlib::binary::RecommendLikeRow {
    rbox_devlib::binary::RecommendLikeRow {
        unknown_00: input.unknown_00,
        track_id_1: input.content_id_1.try_into().unwrap(),
        track_id_2: input.content_id_2.try_into().unwrap(),
        unknown_10: input.unknown_10,
        unknown_14: input.unknown_14,
    }
}

/// Explicit binary fixture for imported IDs and non-default album layouts.
fn album_row(id: u32, artist_id: u32, name: &str) -> Result<rbox_devlib::binary::AlbumRow> {
    use rbox_devlib::binary::{AlbumHeader, AlbumRow, StringOffsets};
    Ok(AlbumRow {
        header: AlbumHeader {
            subtype: AlbumHeader::NEAR_SUBTYPE,
            index_shift: 0,
            unknown_04: 0,
            artist_id,
            id,
            unknown_10: 0,
        },
        strings: StringOffsets::Near {
            marker: 3,
            offsets: [0],
        },
        name: DeviceSqlString::encode(name, StringEncoding::Utf16Le)?,
    })
}

#[test]
fn album_builder_allocates_ids_and_failed_insertions_do_not_consume_them() -> Result<()> {
    use rbox_devlib::models::{DevAlbum, NewDevAlbum};
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let first = NewDevAlbum::new("First").insert(&mut conn)?;
    assert_eq!((first.id, first.artist_id), (1, 0));
    let second = NewDevAlbum::new("Second").artist_id(73).insert(&mut conn)?;
    assert_eq!((second.id, second.artist_id), (2, 73));
    let mut before = Vec::new();
    conn.write_to(&mut before)?;
    assert!(matches!(
        NewDevAlbum::new("Second").artist_id(73).insert(&mut conn),
        Err(Error::UniqueConstraintViolation { .. })
    ));
    assert!(matches!(
        NewDevAlbum::new("Bad\0name").insert(&mut conn),
        Err(Error::EmbeddedNul)
    ));
    assert!(matches!(
        NewDevAlbum::new("x".repeat(5000)).insert(&mut conn),
        Err(Error::NoRowSpace { .. })
    ));
    let mut after = Vec::new();
    conn.write_to(&mut after)?;
    assert_eq!(after, before);
    assert_eq!(NewDevAlbum::new("Third").insert(&mut conn)?.id, 3);
    DevAlbum::delete(&mut conn, &3)?;
    assert_eq!(NewDevAlbum::new("Reused").insert(&mut conn)?.id, 3);

    // Imported IDs can have gaps; automatic allocation follows the live maximum.
    let mut tx = conn.transaction();
    tx.insert(
        DevAlbum::TABLE,
        &Row::Standard(StandardRow::Album(album_row(20, 0, "Imported")?)),
    )?;
    tx.commit()?;
    assert_eq!(NewDevAlbum::new("After import").insert(&mut conn)?.id, 21);
    let mut saved = Vec::new();
    conn.write_to(&mut saved)?;
    let mut conn = DeviceSqlConnection::from_reader(Cursor::new(saved), DatabaseKind::Standard)?;
    assert_eq!(NewDevAlbum::new("After reopen").insert(&mut conn)?.id, 22);
    let mut tx = conn.transaction();
    tx.insert(
        DevAlbum::TABLE,
        &Row::Standard(StandardRow::Album(album_row(u32::MAX - 1, 0, "High ID")?)),
    )?;
    tx.commit()?;
    assert_eq!(
        NewDevAlbum::new("Last ID").insert(&mut conn)?.id,
        i64::from(u32::MAX)
    );
    let dirty = conn.modified_page_count();
    assert!(matches!(
        NewDevAlbum::new("Overflow").insert(&mut conn),
        Err(Error::InvalidModelField { field: "id", .. })
    ));
    assert_eq!(conn.modified_page_count(), dirty);
    assert_eq!(DevAlbum::all(&mut conn)?.len(), 8);
    Ok(())
}

#[test]
fn unknown_tree_attributes_remain_binary_only() -> Result<()> {
    use rbox_devlib::binary::ExtendedRow;
    use rbox_devlib::models::{DevMyTag, DevPlaylist, NewDevMyTag, NewDevPlaylist};

    for flag in [4, 0x80, u32::MAX] {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = playlist_row(1, &NewDevPlaylist::playlist("Unknown"))?;
        row.folder_flag = flag;
        let mut tx = conn.transaction();
        let handle = tx.insert(
            DevPlaylist::TABLE,
            &Row::Standard(StandardRow::PlaylistTree(row)),
        )?;
        let record = tx.read(&handle)?;
        tx.commit()?;
        let Row::Standard(StandardRow::PlaylistTree(binary)) = record.decode()? else {
            panic!("expected playlist row");
        };
        assert_eq!(binary.folder_flag, flag);
        assert!(matches!(
            DevPlaylist::try_from(record),
            Err(Error::InvalidModelField {
                field: "attribute",
                ..
            })
        ));
        assert!(matches!(
            DevPlaylist::find(&mut conn, &1),
            Err(Error::InvalidModelField {
                field: "attribute",
                ..
            })
        ));
        assert!(NewDevPlaylist::playlist("Child")
            .parent_id(1)
            .insert(&mut conn)
            .is_err());
    }

    let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
    let mut row = my_tag_row(&NewDevMyTag::tag(1, "Unknown"))?;
    row.header.attribute = 0x80;
    let mut tx = conn.transaction();
    let handle = tx.insert(DevMyTag::TABLE, &Row::Extended(ExtendedRow::MyTag(row)))?;
    let record = tx.read(&handle)?;
    tx.commit()?;
    let Row::Extended(ExtendedRow::MyTag(binary)) = record.decode()? else {
        panic!("expected My Tag row");
    };
    assert_eq!(binary.header.attribute, 0x80);
    assert!(matches!(
        DevMyTag::try_from(record),
        Err(Error::InvalidModelField {
            field: "attribute",
            ..
        })
    ));
    assert!(matches!(
        DevMyTag::find(&mut conn, &1),
        Err(Error::InvalidModelField {
            field: "attribute",
            ..
        })
    ));
    Ok(())
}

#[test]
fn unsupported_playlist_types_cannot_be_written_by_models() -> Result<()> {
    use rbox_devlib::models::{DevPlaylist, NewDevPlaylist};

    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let dirty = conn.modified_page_count();
    for attribute in [PlaylistType::SmartList, PlaylistType::CloudLibrarySync] {
        let panic = std::panic::catch_unwind(|| NewDevPlaylist::new("Unsupported", attribute))
            .expect_err("unsupported playlist types must panic during construction");
        assert_eq!(
            panic.downcast_ref::<String>().map(String::as_str),
            Some(format!("Invalid playlist type: {attribute:?}").as_str())
        );

        // Public fields can still be changed after construction; encoding must validate them.
        let mut input = NewDevPlaylist::playlist("Unsupported");
        input.attribute = attribute;
        assert!(matches!(
            input.insert(&mut conn),
            Err(Error::InvalidModelField {
                field: "attribute",
                ..
            })
        ));
        assert_eq!(conn.modified_page_count(), dirty);
        assert!(DevPlaylist::all(&mut conn)?.is_empty());
    }
    let playlist = NewDevPlaylist::new("Playlist", PlaylistType::List).insert(&mut conn)?;
    assert_eq!(playlist.attribute, PlaylistType::List);
    assert_eq!(
        DevPlaylist::get_attribute(&mut conn, &1)?,
        Some(PlaylistType::List)
    );
    Ok(())
}

#[test]
fn content_lookup_helpers_follow_live_rows() -> Result<()> {
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    seed_content(&mut conn, 20, "/Contents/音.wav")?;
    seed_content(&mut conn, 10, "/Contents/other.wav")?;
    assert!(DevContent::by_ids(&mut conn, &[])?.is_empty());
    assert_eq!(
        DevContent::by_ids(&mut conn, &[20, 999, 10, 20])?
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        vec![10, 20]
    );
    assert_eq!(
        DevContent::find_by_path(&mut conn, "/Contents/音.wav")?
            .unwrap()
            .id,
        20
    );
    assert!(DevContent::path_exists(&mut conn, "/Contents/音.wav")?);
    assert!(!DevContent::path_exists(&mut conn, "/contents/音.wav")?);
    assert!(!DevContent::path_exists(&mut conn, r"\Contents\音.wav")?);
    assert!(DevContent::find_by_path(&mut conn, "/missing.wav")?.is_none());

    let mut edited = DevContent::find(&mut conn, &20)?.unwrap();
    edited.path = "/Contents/renamed.wav".into();
    edited.update(&mut conn)?;
    assert!(!DevContent::path_exists(&mut conn, "/Contents/音.wav")?);
    assert!(DevContent::path_exists(&mut conn, "/Contents/renamed.wav")?);
    DevContent::delete(&mut conn, &20)?;
    assert_eq!(DevContent::by_ids(&mut conn, &[10, 20])?.len(), 1);
    assert!(!DevContent::path_exists(
        &mut conn,
        "/Contents/renamed.wav"
    )?);

    let mut ext = DeviceSqlConnection::create(DatabaseKind::Extended)?;
    assert!(matches!(
        DevContent::by_ids(&mut ext, &[]),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        DevContent::find_by_path(&mut ext, ""),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        DevContent::find_by_master_content_id(&mut ext, 1, 2),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        DevContent::by_master_content_ids(&mut ext, &[], 0),
        Err(Error::WrongDatabaseKind { .. })
    ));
    Ok(())
}

#[test]
fn content_master_identity_lookup_is_scoped_to_source_library() -> Result<()> {
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let library = 3_499_321_917;
    for (id, source_content, source_library) in [
        (30, 123, library),
        (10, 123, library),
        (20, 123, 42),
        (40, 124, library),
        (50, 0, 0),
        (60, i64::from(u32::MAX), library),
    ] {
        let mut row = NewDevContent::new(format!("/Contents/{id}.wav"));
        row.master_content_id = source_content;
        row.master_db_id = source_library;
        let row = content_row(id, &row)?;
        let mut tx = conn.transaction();
        tx.insert(DevContent::TABLE, &Row::Standard(StandardRow::Track(row)))?;
        tx.commit()?;
    }
    assert_eq!(
        DevContent::find_by_master_content_id(&mut conn, 123, library)?
            .unwrap()
            .id,
        10
    );
    assert_eq!(
        DevContent::find_by_master_content_id(&mut conn, 123, 42)?
            .unwrap()
            .id,
        20
    );
    assert_eq!(
        DevContent::find_by_master_content_id(&mut conn, 124, library)?
            .unwrap()
            .id,
        40
    );
    assert_eq!(
        DevContent::find_by_master_content_id(&mut conn, i64::from(u32::MAX), library)?
            .unwrap()
            .id,
        60
    );
    assert!(DevContent::find_by_master_content_id(&mut conn, 123, 999)?.is_none());
    assert!(DevContent::find_by_master_content_id(&mut conn, 999, library)?.is_none());
    assert!(DevContent::find_by_master_content_id(&mut conn, 0, 0)?.is_none());
    let mut ids = vec![i64::from(u32::MAX), 124, 123, 0, 123];
    ids.extend(1000..2200);
    assert_eq!(
        DevContent::by_master_content_ids(&mut conn, &ids, library)?
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        vec![10, 30, 40, 60]
    );
    assert_eq!(
        DevContent::by_master_content_ids(&mut conn, &ids, 42)?
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        vec![20]
    );
    for (source_ids, source_library) in [
        (&[][..], library),
        (&[0][..], library),
        (&ids[..], 0),
        (&ids[..], 999),
    ] {
        assert!(
            DevContent::by_master_content_ids(&mut conn, source_ids, source_library)?.is_empty()
        );
    }
    let mut row = DevContent::find(&mut conn, &10)?.unwrap();
    row.master_db_id = 43;
    row.update(&mut conn)?;
    assert_eq!(
        DevContent::find_by_master_content_id(&mut conn, 123, library)?
            .unwrap()
            .id,
        30
    );
    DevContent::delete(&mut conn, &30)?;
    assert!(DevContent::find_by_master_content_id(&mut conn, 123, library)?.is_none());
    assert_eq!(
        DevContent::by_master_content_ids(&mut conn, &ids, library)?
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        vec![40, 60]
    );
    Ok(())
}

#[test]
fn content_isrc_edits_validate_preserve_unknown_frames_and_roundtrip() -> Result<()> {
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let mut input = NewDevContent::new("/Contents/isrc.wav");
    input.isrc = "USABC2400001".into();
    let original = input.insert(&mut conn)?;
    let mut edited = original.clone();
    assert!(matches!(
        edited.set_isrc("invalid"),
        Err(Error::InvalidIsrc)
    ));
    assert_eq!(edited, original);
    edited.set_isrc("GBAYE6700149")?;
    assert_eq!(edited.isrc()?, "GBAYE6700149");
    assert!(format!("{edited:?}").contains("GBAYE6700149"));
    assert_eq!(
        rbox_core::Track::try_from(edited.clone().resolve(&mut conn)?)?
            .isrc
            .as_deref(),
        Some("GBAYE6700149")
    );
    let edited = edited.update(&mut conn)?;
    let mut stale = original;
    stale.title = "Keep newer ISRC".into();
    assert_eq!(stale.update(&mut conn)?.isrc()?, "GBAYE6700149");

    let Row::Standard(StandardRow::Track(mut row)) = edited.raw_record().decode()? else {
        panic!()
    };
    // Syntactically valid string framing, but not a supported ISRC representation.
    row.text.isrc = DeviceSqlString::encode("opaque", StringEncoding::LongAscii)?;
    let opaque = row.text.isrc.clone();
    let current = DevContent::find(&mut conn, &1)?.unwrap();
    let mut tx = conn.transaction();
    let handle = tx.locate(
        DevContent::TABLE,
        current.raw_record().page(),
        current.raw_record().slot().slot,
    )?;
    tx.update(&handle, &Row::Standard(StandardRow::Track(row)))?;
    tx.commit()?;
    let mut unknown = DevContent::find(&mut conn, &1)?.unwrap();
    assert!(unknown.isrc().is_err());
    unknown.comment = "Unrelated edit".into();
    let mut unknown = unknown.update(&mut conn)?;
    let Row::Standard(StandardRow::Track(row)) = unknown.raw_record().decode()? else {
        panic!()
    };
    assert_eq!(row.text.isrc, opaque);
    unknown.set_isrc("USABC2400002")?;
    let mut repaired = unknown.update(&mut conn)?;
    assert_eq!(repaired.isrc()?, "USABC2400002");
    repaired.set_isrc("")?;
    repaired.update(&mut conn)?;
    let mut bytes = Vec::new();
    conn.write_to(&mut bytes)?;
    let mut reopened =
        DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
    assert_eq!(DevContent::find(&mut reopened, &1)?.unwrap().isrc()?, "");
    Ok(())
}

// Binary-only fixture for unusual color IDs used by read/relationship tests.
fn seed_color(
    conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>,
    id: u16,
    name: &str,
) -> Result<rbox_devlib::models::DevColor> {
    use rbox_devlib::{binary::ColorRow, models::DevColor};
    let mut tx = conn.transaction();
    let handle = tx.insert(
        DevColor::TABLE,
        &Row::Standard(StandardRow::Color(ColorRow {
            id,
            name: DeviceSqlString::encode(name, StringEncoding::ShortAscii)?,
            unknown_00: 0,
            unknown_04: 0,
            unknown_07: 0,
        })),
    )?;
    let model = DevColor::try_from(tx.read(&handle)?)?;
    tx.commit()?;
    Ok(model)
}

#[test]
fn color_updates_preserve_identity_enforce_unique_names_and_roundtrip() -> Result<()> {
    use rbox_devlib::models::DevColor;
    let mut conn = DeviceSqlConnection::create_with_options(
        DatabaseKind::Standard,
        &CreateOptions::standard("2026-10-25", "USB")?,
    )?;
    let colors = DevColor::all(&mut conn)?;
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
        DevColor::find(&mut reopened, &updated.id)?.unwrap().name,
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
        DevExportProperty, DevHistory, DevHistoryContent, DevMyTag, DevMyTagContent, DevPlaylist,
        DevPlaylistContent, DevRecommendedLike, NewDevHistory, NewDevHistoryContent, NewDevMyTag,
        NewDevMyTagContent, NewDevPlaylist, NewDevPlaylistContent, NewDevRecommendedLike,
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
            NewDevContent::new(format!("/Contents/{id}.wav")).insert(&mut standard)?;
        }
        NewDevPlaylist::playlist("Playlist")
            .seq(0)
            .insert(&mut standard)?;
        NewDevHistory::new("History").insert(&mut standard)?;
        NewDevMyTag::tag(30, "Tag").seq(0).insert(&mut extended)?;
        for (id, seq) in [(1, 1), (1, 3), (2, 8)] {
            // Seed existing binary duplicates to exercise cleanup of imported files.
            let mut tx = standard.transaction();
            tx.insert(
                DevPlaylistContent::TABLE,
                &Row::Standard(StandardRow::PlaylistEntry(playlist_entry_row(
                    &NewDevPlaylistContent::new(1, id, seq),
                ))),
            )?;
            tx.commit()?;
        }
        for (id, seq) in [(1, 1), (2, 9)] {
            NewDevHistoryContent::new(1, id, seq).insert(&mut standard)?;
        }
        for id in [1, 1, 2] {
            NewDevMyTagContent::new(id, 30).insert(&mut extended)?;
        }
        for (a, b) in [(1, 2), (2, 1), (1, 1), (2, 2)] {
            NewDevRecommendedLike::new(a, b, 0x1234, 7, 8).insert(&mut extended)?;
        }
        Ok((standard, extended))
    }

    #[test]
    fn updates_map_all_public_fields_preserve_unknowns_and_follow_relocations() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut input = NewDevContent::new("/Contents/original.wav");
        input.isrc = "USABC1234567".into();
        input.creation_mut().content_link = 0x12345678;
        input.creation_mut().raw_analysis = 0x9876;
        input.creation_mut().unknown_strings =
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
        let mut expected = content_row(original.id.try_into().unwrap(), &desired)?;
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
        assert_eq!(DevContent::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn deletion_cleans_both_files_and_preserves_parents_survivors_and_sequences() -> Result<()> {
        let (mut standard, mut extended) = pair()?;
        let survivor = DevContent::find(&mut standard, &2)?
            .unwrap()
            .raw_record()
            .decode()?;
        assert_eq!(
            DevContent::delete_with_extended(&mut standard, &mut extended, &1)?,
            9
        );
        assert_eq!(
            DevContent::delete_with_extended(&mut standard, &mut extended, &1)?,
            0
        );
        assert!(DevContent::find(&mut standard, &1)?.is_none());
        assert_eq!(
            DevContent::find(&mut standard, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            survivor
        );
        assert_eq!(DevExportProperty::all(&mut standard)?[0].track_count, 1);
        let playlists = DevPlaylistContent::all(&mut standard)?;
        assert_eq!(playlists.len(), 1);
        assert_eq!((playlists[0].content_id, playlists[0].seq), (2, 1));
        let histories = DevHistoryContent::all(&mut standard)?;
        assert_eq!(histories.len(), 1);
        assert_eq!((histories[0].content_id, histories[0].seq), (2, 9));
        assert!(DevPlaylist::find(&mut standard, &1)?.is_some());
        assert!(DevHistory::find(&mut standard, &1)?.is_some());
        assert!(DevMyTag::find(&mut extended, &30)?.is_some());
        let tags = DevMyTagContent::all(&mut extended)?;
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].content_id, 2);
        let recommendations = DevRecommendedLike::all(&mut extended)?;
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
        assert!(DevContent::find(&mut standard, &1)?.is_none());
        assert!(DevMyTagContent::by_content_id(&mut extended, &1)?.is_empty());
        assert!(DevRecommendedLike::by_content_id(&mut extended, &1)?.is_empty());
        assert_eq!(
            DevContent::delete_with_extended(&mut standard, &mut extended, &2)?,
            5
        );
        assert_eq!(DevExportProperty::all(&mut standard)?[0].track_count, 0);
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
                DevContent::delete_with_extended(&mut standard, &mut extended, &1),
                Err(Error::DirtyPageLimit { .. })
            ));
            assert_eq!(bytes(&mut standard)?, original_standard);
            assert_eq!(bytes(&mut extended)?, original_extended);
        }
        // Failure after track and occurrence deletions have been staged.
        let property = DevExportProperty::all(&mut standard)?.remove(0);
        let raw = property.raw_record();
        let offset = raw.page().0 as usize * standard.header().prefix.page_size as usize
            + PAGE_HEADER_SIZE
            + raw.slot().heap_offset as usize;
        let mut corrupt = original_standard;
        corrupt[offset..offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
        let mut standard =
            DeviceSqlConnection::from_reader(Cursor::new(corrupt.clone()), DatabaseKind::Standard)?;
        assert!(DevContent::delete_with_extended(&mut standard, &mut extended, &1).is_err());
        assert_eq!(bytes(&mut standard)?, corrupt);
        assert_eq!(bytes(&mut extended)?, original_extended);
        Ok(())
    }

    #[test]
    fn update_validation_missing_rows_and_standard_only_delete() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let content = NewDevContent::new("/Contents/track.wav").insert(&mut conn)?;
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
            DevContent::delete(&mut ext, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevContent::delete(&mut conn, &0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(DevContent::delete(&mut conn, &99)?, 0);
        assert_eq!(DevContent::delete(&mut conn, &1)?, 1);
        assert!(matches!(
            content.update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));
        Ok(())
    }
}

#[test]
fn album_uniqueness_uses_name_and_current_album_artist() -> Result<()> {
    use rbox_devlib::models::{DevAlbum, NewDevAlbum};
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let first = NewDevAlbum::new("Album").artist_id(10).insert(&mut conn)?;
    let second = NewDevAlbum::new("Album").artist_id(20).insert(&mut conn)?;
    let other = NewDevAlbum::new("Other").artist_id(10).insert(&mut conn)?;
    NewDevAlbum::new("Album").artist_id(0).insert(&mut conn)?;
    for artist in [0, 10, 20] {
        assert!(matches!(
            NewDevAlbum::new("Album")
                .artist_id(artist)
                .insert(&mut conn),
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
    NewDevAlbum::new("Taken").artist_id(30).insert(&mut conn)?;
    let mut stale = first;
    stale.name = "Taken".into();
    assert!(matches!(
        stale.update(&mut conn),
        Err(Error::UniqueConstraintViolation { .. })
    ));
    assert_eq!(DevAlbum::find(&mut conn, &1)?.unwrap().name, "Album");
    let mut bytes = Vec::new();
    conn.write_to(&mut bytes)?;
    let mut reopened =
        DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
    assert!(matches!(
        NewDevAlbum::new("Album")
            .artist_id(30)
            .insert(&mut reopened),
        Err(Error::UniqueConstraintViolation { .. })
    ));
    DevAlbum::delete(&mut reopened, &1)?;
    NewDevAlbum::new("Album")
        .artist_id(30)
        .insert(&mut reopened)?;
    Ok(())
}

#[test]
fn named_models_find_exact_names_without_resolving_relations() -> Result<()> {
    use rbox_devlib::models::{
        DevAlbum, DevArtist, DevColor, DevGenre, DevKey, DevLabel, NewDevAlbum, NewDevArtist,
        NewDevGenre, NewDevKey, NewDevLabel,
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
    check!(DevArtist, NewDevArtist::new("Name"));
    check!(DevGenre, NewDevGenre::new("Name"));
    check!(DevKey, NewDevKey::new("Name", 23));
    check!(DevLabel, NewDevLabel::new("Name"));
    check!(DevAlbum, NewDevAlbum::new("Name").artist_id(999));
    NewDevAlbum::new("\u{97f3}")
        .artist_id(1000)
        .insert(&mut conn)?;
    let first = DevAlbum::all(&mut conn)?
        .into_iter()
        .find(|a| a.name == "\u{97f3}")
        .unwrap();
    assert_eq!(
        DevAlbum::find_by_name(&mut conn, "\u{97f3}")?.unwrap().id,
        first.id
    );
    let mut colors = DeviceSqlConnection::create_with_options(
        DatabaseKind::Standard,
        &CreateOptions::standard("2026-10-25", "USB")?,
    )?;
    let mut color = DevColor::all(&mut colors)?.remove(0);
    color.name = "Custom".into();
    let color = color.update(&mut colors)?;
    assert_eq!(
        DevColor::find_by_name(&mut colors, "Custom")?.unwrap().id,
        color.id
    );
    assert!(DevColor::find_by_name(&mut colors, "custom")?.is_none());
    assert!(DevColor::name_exists(&mut colors, "Custom")?);
    assert!(!DevColor::name_exists(&mut colors, "custom")?);
    assert!(DevAlbum::name_exists(&mut conn, "\u{97f3}")?);
    Ok(())
}

mod name_constraints {
    //! Model name constraints compare decoded text and roll back conflicts.
    use super::*;
    use rbox_devlib::models::{
        DevArtist, DevGenre, DevKey, DevLabel, NewDevArtist, NewDevGenre, NewDevKey, NewDevLabel,
    };

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
                let first = new("Name").insert(&mut conn)?;
                let second = new("Other").insert(&mut conn)?;
                // Self updates and identical names in different tables are valid.
                first.clone().update(&mut conn)?;
                let before = bytes(&mut conn)?;
                assert!(matches!(new("Name").insert(&mut conn),
                    Err(Error::UniqueConstraintViolation { table, fields: &["name"] }) if table == $model::TABLE));
                let mut edited = second.clone();
                edited.name = "Name".into();
                assert!(matches!(edited.update(&mut conn), Err(Error::UniqueConstraintViolation { .. })));
                assert_eq!(bytes(&mut conn)?, before);
                new("name").insert(&mut conn)?;
                new("Name ").insert(&mut conn)?;
                new("").insert(&mut conn)?;
                assert!(matches!(new("").insert(&mut conn), Err(Error::UniqueConstraintViolation { .. })));
                let mut changed = second.clone();
                changed.name = "Taken".into();
                changed.update(&mut conn)?;
                let mut stale = first.clone();
                stale.name = "Taken".into();
                assert!(matches!(stale.update(&mut conn), Err(Error::UniqueConstraintViolation { .. })));
                $model::delete(&mut conn, &1)?;
                new("Name").insert(&mut conn)?;
                let mut reopened = DeviceSqlConnection::from_reader(Cursor::new(bytes(&mut conn)?), DatabaseKind::Standard)?;
                assert!(matches!(new("Name").insert(&mut reopened), Err(Error::UniqueConstraintViolation { .. })));
            }};
        }
        check!(DevArtist, |name: &str| NewDevArtist::new(name));
        check!(DevGenre, |name: &str| NewDevGenre::new(name));
        check!(DevKey, |name: &str| NewDevKey::new(name, 77));
        check!(DevLabel, |name: &str| NewDevLabel::new(name));
        Ok(())
    }

    #[test]
    fn equal_names_conflict_across_binary_string_encodings() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = named_row(3, "House")?;
        row.name = DeviceSqlString::encode("House", StringEncoding::Utf16Le)?;
        let mut tx = conn.transaction();
        tx.insert(DevGenre::TABLE, &Row::Standard(StandardRow::Genre(row)))?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            NewDevGenre::new("House").insert(&mut conn),
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
    use rbox_devlib::models::{DevRecommendedLike, NewDevRecommendedLike};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn rejects_duplicate_pairs_and_allows_reinsertion_after_deletion() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let first = NewDevRecommendedLike::new(1, 2, 7, 8, 9).insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        for value in [7, 99] {
            assert!(matches!(
                NewDevRecommendedLike::new(1, 2, value, 8, 9).insert(&mut conn),
                Err(Error::UniqueConstraintViolation {
                    fields: &["content_id_1", "content_id_2"],
                    ..
                })
            ));
        }
        assert_eq!(bytes(&mut conn)?, before);
        let reverse = NewDevRecommendedLike::new(2, 1, 17, 18, 19)
            .insert(&mut conn)?
            .raw_record()
            .decode()?;
        let same = NewDevRecommendedLike::new(1, 1, 0, 0, 0).insert(&mut conn)?;
        assert!(matches!(
            NewDevRecommendedLike::new(1, 1, 1, 2, 3).insert(&mut conn),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        let mut edited = first.clone();
        edited.content_id_1 = 99;
        edited.unknown_00 = 42;
        assert_eq!(edited.delete(&mut conn)?, 1);
        assert!(first.delete(&mut conn).is_err());
        NewDevRecommendedLike::new(1, 2, 7, 8, 9)
            .insert(&mut conn)?
            .delete(&mut conn)?;
        assert_eq!(same.delete(&mut conn)?, 1);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Extended,
        )?;
        let rows = DevRecommendedLike::all(&mut reopened)?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].raw_record().decode()?, reverse);
        assert!(matches!(
            NewDevRecommendedLike::new(2, 1, 0, 0, 0).insert(&mut reopened),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        Ok(())
    }

    #[test]
    fn rejects_stale_rows_wrong_namespaces_and_failed_writes() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let original = NewDevRecommendedLike::new(1, 2, 7, 8, 9).insert(&mut conn)?;
        let mut tx = conn.transaction();
        let handle = tx.locate(
            DevRecommendedLike::TABLE,
            original.raw_record().page(),
            original.raw_record().slot().slot,
        )?;
        tx.update(
            &handle,
            &Row::Extended(ExtendedRow::RecommendLike(recommendation_row(
                &NewDevRecommendedLike::new(1, 2, 77, 8, 9),
            ))),
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
        let current = DevRecommendedLike::all(&mut limited)?.remove(0);
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
    use rbox_devlib::models::{DevBrowseColumn, DevDatabaseProperty, DevExportProperty};
    let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
    assert!(DevBrowseColumn::find_by_name(&mut standard, "COLOR")?.is_none());
    assert!(!DevBrowseColumn::name_exists(&mut standard, "COLOR")?);
    assert!(DevExportProperty::first(&mut standard)?.is_none());
    assert!(DevDatabaseProperty::first(&mut extended)?.is_none());
    assert!(matches!(
        DevBrowseColumn::name_exists(&mut extended, "COLOR"),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        DevExportProperty::first(&mut extended),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        DevDatabaseProperty::first(&mut standard),
        Err(Error::WrongDatabaseKind { .. })
    ));
    let mut standard = DeviceSqlConnection::create_with_options(
        DatabaseKind::Standard,
        &CreateOptions::standard("2026-10-26", "USB")?,
    )?;
    let color = DevBrowseColumn::find_by_name(&mut standard, "COLOR")?.unwrap();
    assert_eq!(color.name, "COLOR");
    assert!(DevBrowseColumn::name_exists(&mut standard, "COLOR")?);
    assert!(!DevBrowseColumn::name_exists(&mut standard, "color")?);
    assert!(!DevBrowseColumn::name_exists(&mut standard, " COLOR")?);
    assert!(!DevBrowseColumn::name_exists(
        &mut standard,
        "\u{fffa}COLOR\u{fffb}"
    )?);
    assert_eq!(
        DevExportProperty::first(&mut standard)?,
        DevExportProperty::all(&mut standard)?.into_iter().next()
    );
    // Multiple properties retain physical order rather than requiring a singleton.
    let property = DevExportProperty::first(&mut standard)?.unwrap();
    let Row::Standard(StandardRow::ExportProperty(mut row)) = property.raw_record().decode()?
    else {
        panic!()
    };
    row.name = DeviceSqlString::encode("Second USB", StringEncoding::ShortAscii)?;
    let mut tx = standard.transaction();
    tx.insert(
        DevExportProperty::TABLE,
        &Row::Standard(StandardRow::ExportProperty(row)),
    )?;
    tx.commit()?;
    assert_eq!(DevExportProperty::all(&mut standard)?.len(), 2);
    assert_eq!(
        DevExportProperty::first(&mut standard)?,
        DevExportProperty::all(&mut standard)?.into_iter().next()
    );
    let mut extended = DeviceSqlConnection::create_with_options(
        DatabaseKind::Extended,
        &CreateOptions::extended(42)?,
    )?;
    let property = DevDatabaseProperty::first(&mut extended)?.unwrap();
    let Row::Extended(ExtendedRow::DatabaseProperty(mut row)) = property.raw_record().decode()?
    else {
        panic!()
    };
    row.header.my_tag_master_db_id = 99;
    let mut tx = extended.transaction();
    tx.insert(
        DevDatabaseProperty::TABLE,
        &Row::Extended(ExtendedRow::DatabaseProperty(row)),
    )?;
    tx.commit()?;
    assert_eq!(DevDatabaseProperty::all(&mut extended)?.len(), 2);
    assert_eq!(
        DevDatabaseProperty::first(&mut extended)?
            .unwrap()
            .my_tag_master_db_id,
        42
    );
    Ok(())
}

mod property_mutations {
    //! Named export properties and occurrence-based extended metadata preserve reserved fields.
    use super::*;
    use rbox_devlib::binary::ExtendedRow;
    use rbox_devlib::models::{DevDatabaseProperty, DevExportProperty};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn export_names_are_unique_keys_and_count_helpers_match_onelib() -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("date", "")?,
        )?;
        let first = DevExportProperty::find(&mut conn, "")?.unwrap();
        let Row::Standard(StandardRow::ExportProperty(mut row)) = first.raw_record().decode()?
        else {
            panic!()
        };
        row.name = DeviceSqlString::encode("USB", StringEncoding::ShortAscii)?;
        let mut tx = conn.transaction();
        tx.insert(
            DevExportProperty::TABLE,
            &Row::Standard(StandardRow::ExportProperty(row)),
        )?;
        tx.commit()?;
        assert!(DevExportProperty::id_exists(&mut conn, "")?);
        assert!(DevExportProperty::id_exists(&mut conn, "USB")?);
        assert!(!DevExportProperty::id_exists(&mut conn, "usb")?);
        assert!(DevExportProperty::find(&mut conn, "missing")?.is_none());
        assert_eq!(
            DevExportProperty::set_number_of_contents(&mut conn, "USB", 7)?,
            1
        );
        assert_eq!(
            DevExportProperty::set_number_of_contents(&mut conn, "missing", 7)?,
            0
        );
        assert_eq!(
            DevExportProperty::set_number_of_contents_default(&mut conn, 8)?,
            1
        );
        assert_eq!(
            DevExportProperty::find(&mut conn, "")?.unwrap().track_count,
            8
        );
        assert_eq!(
            DevExportProperty::find(&mut conn, "USB")?
                .unwrap()
                .track_count,
            7
        );
        assert_eq!(
            DevExportProperty::set_number_of_contents_all(&mut conn, 9)?,
            2
        );
        assert!(DevExportProperty::all(&mut conn)?
            .iter()
            .all(|row| row.track_count == 9));
        let mut renamed = DevExportProperty::find(&mut conn, "USB")?.unwrap();
        renamed.name = "".into();
        let before = bytes(&mut conn)?;
        assert!(matches!(
            renamed.clone().update(&mut conn),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert_eq!(bytes(&mut conn)?, before);
        renamed.name = "Renamed USB".into();
        renamed.update(&mut conn)?;
        assert!(!DevExportProperty::id_exists(&mut conn, "USB")?);
        assert!(DevExportProperty::id_exists(&mut conn, "Renamed USB")?);

        // The binary engine can read/write unqualified data; models reject ambiguity.
        let duplicate = DevExportProperty::find(&mut conn, "")?.unwrap();
        let mut tx = conn.transaction();
        tx.insert(DevExportProperty::TABLE, &duplicate.raw_record().decode()?)?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            DevExportProperty::find(&mut conn, ""),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert!(matches!(
            DevExportProperty::first(&mut conn),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert!(matches!(
            DevExportProperty::set_content_count(&mut conn, 10),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert!(matches!(
            NewDevContent::new("/Contents/test.wav").insert(&mut conn),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert!(!DevContent::id_exists(&mut conn, &1)?);
        assert_eq!(bytes(&mut conn)?, before);
        let mut empty = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        assert_eq!(
            DevExportProperty::set_number_of_contents_default(&mut empty, 1)?,
            0
        );
        assert_eq!(
            DevExportProperty::set_number_of_contents_all(&mut empty, 1)?,
            0
        );
        let mut ext = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            DevExportProperty::set_number_of_contents_all(&mut ext, 1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }

    #[test]
    fn export_updates_all_fields_and_rejects_stale_count_snapshots() -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("2026-10-25", "USB")?,
        )?;
        let original = DevExportProperty::all(&mut conn)?.remove(0);
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
        assert_eq!(DevExportProperty::all(&mut reopened)?[0], updated);
        NewDevContent::new("/Contents/test.wav").insert(&mut reopened)?;
        assert!(matches!(
            updated.update(&mut reopened),
            Err(Error::StaleModelRow { .. })
        ));
        assert_eq!(DevExportProperty::all(&mut reopened)?[0].track_count, 1);
        Ok(())
    }

    #[test]
    fn database_identity_updates_one_occurrence_and_preserves_reserved_data() -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Extended,
            &CreateOptions::extended(42)?,
        )?;
        let original = DevDatabaseProperty::all(&mut conn)?.remove(0);
        let Row::Extended(ExtendedRow::DatabaseProperty(mut row)) =
            original.raw_record().decode()?
        else {
            panic!()
        };
        row.header.unknown_words = [1, 2, 3, 4, 5];
        let mut tx = conn.transaction();
        let handle = tx.locate(
            DevDatabaseProperty::TABLE,
            original.raw_record().page(),
            original.raw_record().slot().slot,
        )?;
        tx.update(
            &handle,
            &Row::Extended(ExtendedRow::DatabaseProperty(row.clone())),
        )?;
        tx.insert(
            DevDatabaseProperty::TABLE,
            &Row::Extended(ExtendedRow::DatabaseProperty(row.clone())),
        )?;
        tx.commit()?;
        let original = DevDatabaseProperty::all(&mut conn)?.remove(0);
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
            DevDatabaseProperty::all(&mut reopened)?
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
        let mut edited = DevDatabaseProperty::all(&mut reopened)?.remove(0);
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
            DevExportProperty::all(&mut standard)?
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
    use rbox_devlib::models::{
        DevPlaylist, DevPlaylistContent, NewDevPlaylist, NewDevPlaylistContent,
    };

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn edits_validate_parents_preserve_reserved_fields_and_merge_stale_snapshots() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let root = NewDevPlaylist::folder("Root").seq(0).insert(&mut conn)?;
        NewDevPlaylist::folder("Nested")
            .parent_id(1)
            .seq(0)
            .insert(&mut conn)?;
        let mut input = NewDevPlaylist::playlist("List").parent_id(1).seq(1);
        input.unknown_04 = 42;
        let playlist = input.insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        playlist.clone().update(&mut conn)?;
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = playlist.clone();
        edited.name = "Long name".repeat(100);
        edited.parent_id = 2;
        edited.seq = 0;
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
        edited.attribute = PlaylistType::Folder;
        assert!(edited.update(&mut conn).is_err());
        let mut edited = playlist.clone();
        edited.id = 99;
        assert!(edited.update(&mut conn).is_err());
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = playlist;
        edited.name.clear();
        let updated = edited.update(&mut conn)?;
        assert_eq!((updated.parent_id, updated.seq), (2, 0));
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(DevPlaylist::find(&mut reopened, &3)?
            .unwrap()
            .name
            .is_empty());
        Ok(())
    }

    #[test]
    fn cascade_is_atomic_and_keeps_contents_and_other_playlists() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let root = NewDevPlaylist::folder("Root").seq(0).insert(&mut conn)?;
        NewDevPlaylist::folder("Nested")
            .parent_id(1)
            .seq(0)
            .insert(&mut conn)?;
        NewDevPlaylist::playlist("List")
            .parent_id(2)
            .seq(0)
            .insert(&mut conn)?;
        NewDevPlaylist::playlist("Keep").seq(1).insert(&mut conn)?;
        let content = seed_content(&mut conn, 99, "/Contents/test.wav")?
            .raw_record()
            .decode()?;
        for playlist in [3, 3, 4] {
            // Include an imported duplicate to verify cascading cleanup removes both.
            let mut tx = conn.transaction();
            tx.insert(
                DevPlaylistContent::TABLE,
                &Row::Standard(StandardRow::PlaylistEntry(playlist_entry_row(
                    &NewDevPlaylistContent::new(playlist, 99, 7),
                ))),
            )?;
            tx.commit()?;
        }
        let before = bytes(&mut conn)?;
        let mut limited =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(1),
            ..Default::default()
        })?;
        assert!(matches!(
            DevPlaylist::delete(&mut limited, &1),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut limited)?, before);
        assert_eq!(DevPlaylist::delete(&mut conn, &1)?, 5);
        assert_eq!(DevPlaylist::delete(&mut conn, &1)?, 0);
        assert!(matches!(
            root.update(&mut conn),
            Err(Error::ModelNotFound { .. })
        ));
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        let playlists = DevPlaylist::all(&mut reopened)?;
        assert_eq!(playlists.len(), 1);
        assert_eq!((playlists[0].id, playlists[0].seq), (4, 0));
        let entries = DevPlaylistContent::all(&mut reopened)?;
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].playlist_id, entries[0].seq), (4, 7));
        assert_eq!(
            DevContent::find(&mut reopened, &99)?
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
        NewDevPlaylist::playlist("List").insert(&mut conn)?;
        seed_content(&mut conn, 2, "/2.wav")?;
        seed_content(&mut conn, 3, "/3.wav")?;
        let first = NewDevPlaylistContent::new(1, 2, 1).insert(&mut conn)?;
        NewDevPlaylistContent::new(1, 3, 2).insert(&mut conn)?;
        for seq in [3, 9] {
            // Existing files may contain duplicates created outside the model API.
            let mut tx = conn.transaction();
            tx.insert(
                DevPlaylistContent::TABLE,
                &Row::Standard(StandardRow::PlaylistEntry(playlist_entry_row(
                    &NewDevPlaylistContent::new(1, 2, seq),
                ))),
            )?;
            tx.commit()?;
        }
        let mut edited = first.clone();
        edited.content_id = 99;
        assert_eq!(edited.delete(&mut conn)?, 1);
        assert!(first.delete(&mut conn).is_err());
        assert_eq!(DevPlaylistContent::remove(&mut conn, &2, &1)?, 2);
        assert_eq!(DevPlaylistContent::remove(&mut conn, &2, &1)?, 0);
        assert_eq!(DevPlaylistContent::all(&mut conn)?[0].seq, 1);
        let keep = DevPlaylistContent::find(&mut conn, &(1, 3))?.unwrap();
        let mut tx = conn.transaction();
        let handle = tx.locate(
            DevPlaylistContent::TABLE,
            keep.raw_record().page(),
            keep.raw_record().slot().slot,
        )?;
        tx.update(
            &handle,
            &Row::Standard(StandardRow::PlaylistEntry(playlist_entry_row(
                &NewDevPlaylistContent::new(1, 3, 15),
            ))),
        )?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            keep.clone().delete(&mut conn),
            Err(Error::StaleModelRow { .. })
        ));
        assert!(DevPlaylistContent::remove(&mut conn, &0, &1).is_err());
        assert!(DevPlaylist::delete(&mut conn, &0).is_err());
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            keep.delete(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylist::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylistContent::remove(&mut extended, &2, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }
}

mod my_tag_mutations {
    //! Hierarchy validation, cascading cleanup and duplicate associations.
    use super::*;
    use rbox_devlib::binary::ExtendedRow;
    use rbox_devlib::models::{DevMyTag, DevMyTagContent, NewDevMyTag, NewDevMyTagContent};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    fn children(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>, parent: i64) -> Result<Vec<i64>> {
        Ok(DevMyTag::by_parent_id(conn, &parent)?
            .iter()
            .map(|tag| tag.id)
            .collect())
    }

    #[test]
    fn builders_and_ordered_mutations_maintain_each_sibling_list() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let root = NewDevMyTag::folder(1, "Source");
        assert_eq!(
            (root.parent_id, root.seq, root.attribute),
            (0, None, MyTagType::Folder)
        );
        root.insert(&mut conn)?;
        NewDevMyTag::folder(2, "Destination").insert(&mut conn)?;
        NewDevMyTag::tag(10, "First")
            .parent_id(1)
            .insert(&mut conn)?;
        NewDevMyTag::tag(11, "Last")
            .parent_id(1)
            .insert(&mut conn)?;
        NewDevMyTag::new(12, "Middle", MyTagType::List)
            .parent_id(1)
            .seq(1)
            .insert(&mut conn)?;
        assert_eq!(children(&mut conn, 1)?, [10, 12, 11]);
        assert_eq!(DevMyTag::count_children(&mut conn, &1)?, 3);
        assert_eq!(DevMyTag::get_seq_numbers(&mut conn, &1)?, [0, 1, 2]);
        assert!(DevMyTag::validate_seq_numbers(&mut conn, &1)?);
        assert_eq!(DevMyTag::move_to(&mut conn, &11, None, Some(0))?, 1);
        assert_eq!(children(&mut conn, 1)?, [11, 10, 12]);
        assert_eq!(DevMyTag::move_all_to(&mut conn, &[12, 11], &2, None)?, 2);
        assert_eq!(children(&mut conn, 1)?, [10]);
        assert_eq!(children(&mut conn, 2)?, [12, 11]);
        assert_eq!(DevMyTag::get_seq_numbers(&mut conn, &2)?, [0, 1]);
        let mut edited = DevMyTag::find(&mut conn, &10)?.unwrap();
        edited.parent_id = 2; // An unchanged seq means append when changing parents.
        assert_eq!(edited.update(&mut conn)?.seq, 2);
        assert!(children(&mut conn, 1)?.is_empty());
        let mut edited = DevMyTag::find(&mut conn, &10)?.unwrap();
        edited.seq = 0;
        edited.update(&mut conn)?;
        assert_eq!(children(&mut conn, 2)?, [10, 12, 11]);
        NewDevMyTagContent::new(99, 12).insert(&mut conn)?;
        assert_eq!(DevMyTag::delete_all(&mut conn, &[12, 1])?, 3);
        assert_eq!(children(&mut conn, 0)?, [2]);
        assert_eq!(children(&mut conn, 2)?, [10, 11]);
        assert_eq!(DevMyTag::get_seq_numbers(&mut conn, &2)?, [0, 1]);
        assert!(DevMyTagContent::all(&mut conn)?.is_empty());
        assert_eq!(DevMyTag::delete_all(&mut conn, &[2, 10])?, 3);
        assert!(DevMyTag::all(&mut conn)?.is_empty());
        Ok(())
    }

    #[test]
    fn invalid_ordering_rolls_back_and_noop_moves_leave_pages_clean() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        NewDevMyTag::folder(1, "Root").insert(&mut conn)?;
        NewDevMyTag::folder(2, "Nested")
            .parent_id(1)
            .insert(&mut conn)?;
        NewDevMyTag::tag(3, "Tag").parent_id(1).insert(&mut conn)?;
        let mut unknown = my_tag_row(&NewDevMyTag::tag(4, "Unknown"))?;
        unknown.header.attribute = 0x80;
        let mut tx = conn.transaction();
        tx.insert(DevMyTag::TABLE, &Row::Extended(ExtendedRow::MyTag(unknown)))?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        for parent in [2, 3, 4, 99] {
            assert!(DevMyTag::move_to(&mut conn, &1, Some(&parent), None).is_err());
        }
        for parent in [3, 4, 5, 99] {
            assert!(NewDevMyTag::tag(5, "Invalid")
                .parent_id(parent)
                .insert(&mut conn)
                .is_err());
        }
        assert!(NewDevMyTag::tag(5, "Invalid")
            .seq(99)
            .insert(&mut conn)
            .is_err());
        assert!(DevMyTag::move_to(&mut conn, &2, None, Some(99)).is_err());
        assert!(DevMyTag::move_all_to(&mut conn, &[2, 2], &0, None).is_err());
        assert!(DevMyTag::move_all_to(&mut conn, &[2, 99], &0, None).is_err());
        assert!(DevMyTag::delete_all(&mut conn, &[2, 99]).is_err());
        assert!(DevMyTag::delete_all(&mut conn, &[2, 2]).is_err());
        assert!(DevMyTag::reset_seq(&mut conn, &3).is_err());
        assert_eq!(bytes(&mut conn)?, before);
        let mut conn =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Extended)?;
        assert_eq!(DevMyTag::move_to(&mut conn, &2, None, Some(0))?, 0);
        assert_eq!(DevMyTag::move_all_to(&mut conn, &[2, 3], &1, Some(0))?, 0);
        assert_eq!(DevMyTag::move_all_to(&mut conn, &[], &0, None)?, 0);
        assert_eq!(DevMyTag::reset_seq(&mut conn, &1)?, 0);
        assert_eq!(DevMyTag::delete_all(&mut conn, &[])?, 0);
        assert_eq!(conn.modified_page_count(), 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        assert!(matches!(
            DevMyTag::move_all_to(&mut standard, &[], &0, None),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTag::reset_seq(&mut standard, &0),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTag::delete_all(&mut standard, &[]),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTag::count_children(&mut standard, &0),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }

    #[test]
    fn reset_preserves_imported_ties_and_binary_reserved_fields() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let mut input =
            my_tag_row(&NewDevMyTag::new(1, "Reserved fields", MyTagType::List).seq(9))?;
        input.header.unknown_04 = 0x12345678;
        input.header.unknown_08 = 42;
        input.header.unknown_18 = 17;
        input.header.unknown_1a = 5;
        let mut tx = conn.transaction();
        tx.insert(DevMyTag::TABLE, &Row::Extended(ExtendedRow::MyTag(input)))?;
        for id in [2, 3] {
            tx.insert(
                DevMyTag::TABLE,
                &Row::Extended(ExtendedRow::MyTag(my_tag_row(
                    &NewDevMyTag::tag(id, "Tied").seq(4),
                )?)),
            )?;
        }
        tx.commit()?;
        let original = DevMyTag::find(&mut conn, &1)?
            .unwrap()
            .raw_record()
            .decode()?;
        assert!(!DevMyTag::validate_seq_numbers(&mut conn, &0)?);
        assert_eq!(DevMyTag::reset_seq(&mut conn, &0)?, 3);
        assert_eq!(children(&mut conn, 0)?, [2, 3, 1]);
        assert!(DevMyTag::validate_seq_numbers(&mut conn, &0)?);
        let Row::Extended(ExtendedRow::MyTag(mut updated)) = DevMyTag::find(&mut conn, &1)?
            .unwrap()
            .raw_record()
            .decode()?
        else {
            panic!()
        };
        updated.header.position = 9;
        assert_eq!(Row::Extended(ExtendedRow::MyTag(updated)), original);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Extended,
        )?;
        assert_eq!(children(&mut reopened, 0)?, [2, 3, 1]);
        assert_eq!(DevMyTag::get_seq_numbers(&mut reopened, &0)?, [0, 1, 2]);
        Ok(())
    }

    #[test]
    fn multi_page_ordering_rolls_back_and_does_not_rewrite_associations() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let association = NewDevMyTagContent::new(99, 1).insert(&mut conn)?;
        let mut tx = conn.transaction();
        for id in 1..=200 {
            tx.insert(
                DevMyTag::TABLE,
                &Row::Extended(ExtendedRow::MyTag(my_tag_row(
                    &NewDevMyTag::tag(id, "Tag").seq((id - 1).try_into().unwrap()),
                )?)),
            )?;
        }
        tx.commit()?;
        let before = bytes(&mut conn)?;
        let mut conn =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Extended)?;
        conn.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(1),
            ..Default::default()
        })?;
        assert!(matches!(
            DevMyTag::move_to(&mut conn, &200, None, Some(0)),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(matches!(
            NewDevMyTag::tag(201, "First").seq(0).insert(&mut conn),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(conn.modified_page_count(), 0);
        assert_eq!(bytes(&mut conn)?, before);
        conn.set_resource_limits(Default::default())?;
        assert_eq!(DevMyTag::move_to(&mut conn, &200, None, Some(0))?, 1);
        assert!(conn.modified_page_count() > 1);
        let saved = bytes(&mut conn)?;
        let page_size = conn.header().prefix.page_size as usize;
        let start = association.raw_record().page().0 as usize * page_size;
        assert_eq!(
            &saved[start..start + page_size],
            &before[start..start + page_size]
        );
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(saved), DatabaseKind::Extended)?;
        assert_eq!(
            children(&mut reopened, 0)?,
            std::iter::once(200).chain(1..200).collect::<Vec<_>>()
        );
        assert!(DevMyTag::validate_seq_numbers(&mut reopened, &0)?);
        Ok(())
    }

    #[test]
    fn updates_preserve_reserved_data_and_validate_current_ancestry() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let category = NewDevMyTag::folder(1, "Category")
            .seq(0)
            .insert(&mut conn)?;
        NewDevMyTag::folder(2, "Other").insert(&mut conn)?;
        let mut input = NewDevMyTag::tag(3, "Tag").parent_id(1).seq(7);
        input.unknown_04 = 42;
        let row = my_tag_row(&input)?;
        let mut tx = conn.transaction();
        let handle = tx.insert(
            DevMyTag::TABLE,
            &Row::Extended(ExtendedRow::MyTag(row.clone())),
        )?;
        let tag = DevMyTag::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        tag.clone().update(&mut conn)?;
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = tag.clone();
        edited.name = "Long tag".repeat(12);
        edited.parent_id = 2;
        edited.seq = 0;
        let updated = edited.update(&mut conn)?;
        assert_eq!(tag.clone().update(&mut conn)?, updated);
        let Row::Extended(ExtendedRow::MyTag(updated_row)) = updated.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(updated_row.header.unknown_04, 42);
        assert_eq!(updated_row.unknown_string, row.unknown_string);
        assert_eq!((updated.parent_id, updated.seq), (2, 0));
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
        let mut child_category = NewDevMyTag::folder(4, "Child").seq(0);
        child_category.parent_id = 1;
        child_category.insert(&mut conn)?;
        let before = bytes(&mut conn)?;
        let mut edited = category.clone();
        edited.parent_id = 4;
        assert!(edited.update(&mut conn).is_err());
        let mut edited = tag.clone();
        edited.attribute = MyTagType::Folder;
        assert!(edited.update(&mut conn).is_err());
        let mut edited = tag.clone();
        edited.id = 5;
        assert!(edited.update(&mut conn).is_err());
        assert_eq!(bytes(&mut conn)?, before);
        let mut edited = tag;
        edited.name.clear();
        let updated = edited.update(&mut conn)?;
        assert_eq!((updated.parent_id, updated.seq), (2, 0));
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Extended,
        )?;
        assert!(DevMyTag::find(&mut reopened, &3)?.unwrap().name.is_empty());
        Ok(())
    }

    #[test]
    fn category_deletion_cascades_and_rolls_back_on_resource_failure() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let root = NewDevMyTag::folder(1, "Root").seq(0).insert(&mut conn)?;
        let mut nested = NewDevMyTag::folder(2, "Nested").seq(0);
        nested.parent_id = 1;
        nested.insert(&mut conn)?;
        NewDevMyTag::tag(3, "Tag").parent_id(2).insert(&mut conn)?;
        NewDevMyTag::folder(4, "Keep").insert(&mut conn)?;
        for tag in [1, 2, 3, 3, 4] {
            NewDevMyTagContent::new(99, tag).insert(&mut conn)?;
        }
        let before = bytes(&mut conn)?;
        let mut limited =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Extended)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(1),
            ..Default::default()
        })?;
        assert!(matches!(
            DevMyTag::delete(&mut limited, &1),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut limited)?, before);
        assert_eq!(DevMyTag::delete(&mut conn, &1)?, 7);
        assert_eq!(DevMyTag::delete(&mut conn, &1)?, 0);
        assert!(matches!(
            root.update(&mut conn),
            Err(Error::ModelNotFound { .. })
        ));
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Extended,
        )?;
        let tags = DevMyTag::all(&mut reopened)?;
        assert_eq!(tags.len(), 1);
        assert_eq!((tags[0].id, tags[0].seq), (4, 0));
        let links = DevMyTagContent::all(&mut reopened)?;
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].my_tag_id, 4);
        Ok(())
    }

    #[test]
    fn associations_support_single_occurrences_pair_removal_and_stale_checks() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let first = NewDevMyTagContent::new(1, 2).insert(&mut conn)?;
        for _ in 0..2 {
            NewDevMyTagContent::new(1, 2).insert(&mut conn)?;
        }
        let keep = NewDevMyTagContent::new(2, 2).insert(&mut conn)?;
        let mut edited = first.clone();
        edited.content_id = 99;
        assert_eq!(edited.delete(&mut conn)?, 1);
        assert!(first.delete(&mut conn).is_err());
        assert_eq!(DevMyTagContent::remove(&mut conn, &1, &2)?, 2);
        assert_eq!(DevMyTagContent::remove(&mut conn, &1, &2)?, 0);
        let mut tx = conn.transaction();
        let handle = tx.locate(
            DevMyTagContent::TABLE,
            keep.raw_record().page(),
            keep.raw_record().slot().slot,
        )?;
        tx.update(
            &handle,
            &Row::Extended(ExtendedRow::TrackMyTag(tag_content_row(
                &NewDevMyTagContent::new(3, 2),
            ))),
        )?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            keep.clone().delete(&mut conn),
            Err(Error::StaleModelRow { .. })
        ));
        assert!(DevMyTagContent::remove(&mut conn, &0, &2).is_err());
        assert!(DevMyTag::delete(&mut conn, &0).is_err());
        assert_eq!(bytes(&mut conn)?, before);
        let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        assert!(matches!(
            keep.delete(&mut standard),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTag::delete(&mut standard, &2),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTagContent::remove(&mut standard, &1, &2),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }
}

mod history_mutations {
    //! History name edits and deletion of histories or individual occurrences.
    use super::*;
    use rbox_devlib::models::{DevHistory, DevHistoryContent, NewDevHistory, NewDevHistoryContent};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn history_updates_follow_original_id_and_preserve_unedited_names() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let history = NewDevHistory::new("Set").insert(&mut conn)?;
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
        assert!(DevHistory::find(&mut reopened, &1)?
            .unwrap()
            .name
            .is_empty());
        assert_eq!(DevHistory::delete(&mut reopened, &1)?, 1);
        assert!(matches!(
            history.update(&mut reopened),
            Err(Error::ModelNotFound { .. })
        ));
        Ok(())
    }

    #[test]
    fn deletion_preserves_duplicates_other_histories_and_contents() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevHistory::new("First").insert(&mut conn)?;
        NewDevHistory::new("Second").insert(&mut conn)?;
        NewDevContent::new("/Contents/test.wav").insert(&mut conn)?;
        let first = NewDevHistoryContent::new(1, 1, 3).insert(&mut conn)?;
        NewDevHistoryContent::new(1, 1, 3).insert(&mut conn)?;
        NewDevHistoryContent::new(1, 1, 9).insert(&mut conn)?;
        NewDevHistoryContent::new(2, 1, 12).insert(&mut conn)?;
        let content = DevContent::find(&mut conn, &1)?
            .unwrap()
            .raw_record()
            .decode()?;
        let mut edited_snapshot = first.clone();
        edited_snapshot.history_id = 2;
        edited_snapshot.seq = 99;
        assert_eq!(edited_snapshot.delete(&mut conn)?, 1);
        assert!(first.delete(&mut conn).is_err());
        assert_eq!(
            DevHistoryContent::by_history_id(&mut conn, &1)?
                .iter()
                .map(|r| r.seq)
                .collect::<Vec<_>>(),
            [3, 9]
        );
        assert_eq!(DevHistory::delete(&mut conn, &1)?, 3);
        assert_eq!(DevHistory::delete(&mut conn, &1)?, 0);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(DevHistory::find(&mut reopened, &1)?.is_none());
        assert!(DevHistory::find(&mut reopened, &2)?.is_some());
        let entries = DevHistoryContent::all(&mut reopened)?;
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].history_id, entries[0].seq), (2, 12));
        assert_eq!(
            DevContent::find(&mut reopened, &1)?
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
        let history = NewDevHistory::new("Set").insert(&mut conn)?;
        let entry = NewDevHistoryContent::new(1, 1, 1).insert(&mut conn)?;
        let mut tx = conn.transaction();
        let handle = tx.locate(
            DevHistoryContent::TABLE,
            entry.raw_record().page(),
            entry.raw_record().slot().slot,
        )?;
        tx.update(
            &handle,
            &Row::Standard(StandardRow::HistoryEntry(history_entry_row(
                &NewDevHistoryContent::new(1, 2, 1),
            ))),
        )?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            entry.clone().delete(&mut conn),
            Err(Error::StaleModelRow { .. })
        ));
        assert!(DevHistory::delete(&mut conn, &0).is_err());
        assert_eq!(DevHistory::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            history.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevHistory::delete(&mut extended, &1),
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
            DevHistory::delete(&mut limited, &1),
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
    use rbox_devlib::models::{DevExportProperty, DevKey, NewDevKey};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_and_support_growth_stale_snapshots_and_empty_names() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = Ok::<_, Error>(rbox_devlib::binary::KeyRow {
            id: 1,
            secondary_id: (NewDevKey::new("House", 17))
                .secondary_id
                .try_into()
                .unwrap(),
            name: named_row(1, &(NewDevKey::new("House", 17)).name)?.name,
        })?;
        row.name = DeviceSqlString::encode("House", StringEncoding::Utf16Le)?;
        let mut tx = conn.transaction();
        let handle = tx.insert(DevKey::TABLE, &Row::Standard(StandardRow::Key(row)))?;
        let key = DevKey::try_from(tx.read(&handle)?)?;
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
        assert_eq!(DevKey::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn deletion_clears_only_matching_references_and_preserves_counts_and_binary_fields(
    ) -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("2026-10-25", "Test")?,
        )?;
        NewDevKey::new("House", 17).insert(&mut conn)?;
        let keep = NewDevKey::new("Techno", 29).insert(&mut conn)?;
        let mut expected = Vec::new();
        for (id, key_id) in [(1, 1), (2, 2), (3, 1), (4, 0)] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
            content.key_id = key_id;
            content.creation_mut().content_link = 12345;
            let content = content.insert(&mut conn)?;
            let Row::Standard(StandardRow::Track(mut row)) = content.raw_record().decode()? else {
                panic!()
            };
            if key_id == 1 {
                row.header.key_id = 0;
            }
            expected.push(Row::Standard(StandardRow::Track(row)));
        }
        let properties = DevExportProperty::all(&mut conn)?;
        assert_eq!(DevKey::delete(&mut conn, &1)?, 3);
        assert_eq!(DevKey::delete(&mut conn, &1)?, 0);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(DevKey::find(&mut reopened, &1)?.is_none());
        assert_eq!(
            DevKey::find(&mut reopened, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            keep.raw_record().decode()?
        );
        assert_eq!(DevExportProperty::all(&mut reopened)?, properties);
        let contents = DevContent::all(&mut reopened)?;
        assert_eq!(contents.len(), expected.len());
        for (content, expected) in contents.iter().zip(expected) {
            assert_eq!(content.raw_record().decode()?, expected);
        }
        Ok(())
    }

    #[test]
    fn invalid_edits_and_failed_reference_cleanup_leave_database_unchanged() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let key = NewDevKey::new("House", 17).insert(&mut conn)?;
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
        assert!(DevKey::delete(&mut conn, &0).is_err());
        assert_eq!(DevKey::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            key.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevKey::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
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
        assert!(DevKey::delete(&mut broken, &1).is_err());
        assert_eq!(bytes(&mut broken)?, before);
        assert_eq!(broken.modified_page_count(), 0);
        assert_eq!(DevKey::delete(&mut conn, &1)?, 3);
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
    use rbox_devlib::models::{DevExportProperty, DevLabel, NewDevLabel};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_and_support_growth_stale_snapshots_and_empty_names() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = named_row(1, "Label A")?;
        row.name = DeviceSqlString::encode("Label A", StringEncoding::Utf16Le)?;
        let mut tx = conn.transaction();
        let handle = tx.insert(DevLabel::TABLE, &Row::Standard(StandardRow::Label(row)))?;
        let label = DevLabel::try_from(tx.read(&handle)?)?;
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
        assert_eq!(DevLabel::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn deletion_clears_only_matching_references_and_preserves_counts_and_binary_fields(
    ) -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("2026-10-25", "Test")?,
        )?;
        NewDevLabel::new("Label A").insert(&mut conn)?;
        let keep = NewDevLabel::new("Label B").insert(&mut conn)?;
        let mut expected = Vec::new();
        for (id, label_id) in [(1, 1), (2, 2), (3, 1), (4, 0)] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
            content.label_id = label_id;
            content.creation_mut().content_link = 12345;
            let content = content.insert(&mut conn)?;
            let Row::Standard(StandardRow::Track(mut row)) = content.raw_record().decode()? else {
                panic!()
            };
            if label_id == 1 {
                row.header.label_id = 0;
            }
            expected.push(Row::Standard(StandardRow::Track(row)));
        }
        let properties = DevExportProperty::all(&mut conn)?;
        assert_eq!(DevLabel::delete(&mut conn, &1)?, 3);
        assert_eq!(DevLabel::delete(&mut conn, &1)?, 0);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(DevLabel::find(&mut reopened, &1)?.is_none());
        assert_eq!(
            DevLabel::find(&mut reopened, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            keep.raw_record().decode()?
        );
        assert_eq!(DevExportProperty::all(&mut reopened)?, properties);
        let contents = DevContent::all(&mut reopened)?;
        assert_eq!(contents.len(), expected.len());
        for (content, expected) in contents.iter().zip(expected) {
            assert_eq!(content.raw_record().decode()?, expected);
        }
        Ok(())
    }

    #[test]
    fn invalid_edits_and_failed_reference_cleanup_leave_database_unchanged() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let label = NewDevLabel::new("Label A").insert(&mut conn)?;
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
        assert!(DevLabel::delete(&mut conn, &0).is_err());
        assert_eq!(DevLabel::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            label.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevLabel::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
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
        assert!(DevLabel::delete(&mut broken, &1).is_err());
        assert_eq!(bytes(&mut broken)?, before);
        assert_eq!(broken.modified_page_count(), 0);
        assert_eq!(DevLabel::delete(&mut conn, &1)?, 3);
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
    use rbox_devlib::models::{DevExportProperty, DevGenre, NewDevGenre};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_and_support_growth_stale_snapshots_and_empty_names() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = named_row(1, "House")?;
        row.name = DeviceSqlString::encode("House", StringEncoding::Utf16Le)?;
        let mut tx = conn.transaction();
        let handle = tx.insert(DevGenre::TABLE, &Row::Standard(StandardRow::Genre(row)))?;
        let genre = DevGenre::try_from(tx.read(&handle)?)?;
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
        assert_eq!(DevGenre::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn deletion_clears_only_matching_references_and_preserves_counts_and_binary_fields(
    ) -> Result<()> {
        let mut conn = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::standard("2026-10-25", "Test")?,
        )?;
        NewDevGenre::new("House").insert(&mut conn)?;
        let keep = NewDevGenre::new("Techno").insert(&mut conn)?;
        let mut expected = Vec::new();
        for (id, genre_id) in [(1, 1), (2, 2), (3, 1), (4, 0)] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
            content.genre_id = genre_id;
            content.creation_mut().content_link = 12345;
            let content = content.insert(&mut conn)?;
            let Row::Standard(StandardRow::Track(mut row)) = content.raw_record().decode()? else {
                panic!()
            };
            if genre_id == 1 {
                row.header.genre_id = 0;
            }
            expected.push(Row::Standard(StandardRow::Track(row)));
        }
        let properties = DevExportProperty::all(&mut conn)?;
        assert_eq!(DevGenre::delete(&mut conn, &1)?, 3);
        assert_eq!(DevGenre::delete(&mut conn, &1)?, 0);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(DevGenre::find(&mut reopened, &1)?.is_none());
        assert_eq!(
            DevGenre::find(&mut reopened, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            keep.raw_record().decode()?
        );
        assert_eq!(DevExportProperty::all(&mut reopened)?, properties);
        let contents = DevContent::all(&mut reopened)?;
        assert_eq!(contents.len(), expected.len());
        for (content, expected) in contents.iter().zip(expected) {
            assert_eq!(content.raw_record().decode()?, expected);
        }
        Ok(())
    }

    #[test]
    fn invalid_edits_and_failed_reference_cleanup_leave_database_unchanged() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let genre = NewDevGenre::new("House").insert(&mut conn)?;
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
        assert!(DevGenre::delete(&mut conn, &0).is_err());
        assert_eq!(DevGenre::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            genre.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevGenre::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
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
        assert!(DevGenre::delete(&mut broken, &1).is_err());
        assert_eq!(bytes(&mut broken)?, before);
        assert_eq!(broken.modified_page_count(), 0);
        assert_eq!(DevGenre::delete(&mut conn, &1)?, 3);
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
    use rbox_devlib::models::{DevArtwork, DevExportProperty, NewDevArtwork};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_handle_growth_and_refresh_stale_snapshots() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = named_row(1, "/Art/original.jpg")?;
        row.name = DeviceSqlString::encode("/Art/original.jpg", StringEncoding::Utf16Le)?;
        let mut tx = conn.transaction();
        let handle = tx.insert(DevArtwork::TABLE, &Row::Standard(StandardRow::Artwork(row)))?;
        let artwork = DevArtwork::try_from(tx.read(&handle)?)?;
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
        assert_eq!(DevArtwork::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn invalid_paths_ids_namespaces_and_resource_failures_are_atomic() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let artwork = NewDevArtwork::new("/Art/cover.jpg").insert(&mut conn)?;
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
            DevArtwork::delete(&mut conn, &0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(DevArtwork::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            artwork.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevArtwork::delete(&mut extended, &1),
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
            DevArtwork::delete(&mut limited, &1),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut limited)?, before);
        assert_eq!(DevArtwork::delete(&mut conn, &1)?, 1);
        assert!(matches!(
            artwork.clone().update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));
        let mut tx = conn.transaction();
        for _ in 0..2 {
            tx.insert(
                DevArtwork::TABLE,
                &Row::Standard(StandardRow::Artwork(named_row(1, "/Art/duplicate.jpg")?)),
            )?;
        }
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            artwork.update(&mut conn),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert!(matches!(
            DevArtwork::delete(&mut conn, &1),
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
        let artwork = NewDevArtwork::new(image.to_string_lossy()).insert(&mut conn)?;
        NewDevArtwork::new("/Art/keep.jpg").insert(&mut conn)?;
        let mut expected = Vec::new();
        for (id, artwork_id) in [(1, 1), (2, 2), (3, 1), (4, 0)] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
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
        let properties = DevExportProperty::all(&mut conn)?;
        let keep = DevArtwork::find(&mut conn, &2)?
            .unwrap()
            .raw_record()
            .decode()?;
        let mut edited = artwork;
        let new_path = dir.path().join("new.jpg");
        edited.path = new_path.to_string_lossy().into_owned();
        edited.update(&mut conn)?;
        assert!(!new_path.exists());
        fs::write(&new_path, b"keep replacement image")?;
        assert_eq!(DevArtwork::delete(&mut conn, &1)?, 3);
        assert_eq!(DevArtwork::delete(&mut conn, &1)?, 0);
        assert!(DevArtwork::find(&mut conn, &1)?.is_none());
        assert_eq!(
            DevArtwork::find(&mut conn, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            keep
        );
        assert_eq!(DevExportProperty::all(&mut conn)?, properties);
        assert_eq!(fs::read(&image)?, b"keep image bytes");
        assert_eq!(fs::read(&new_path)?, b"keep replacement image");
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        let contents = DevContent::all(&mut reopened)?;
        assert_eq!(contents.len(), expected.len());
        for (content, expected) in contents.into_iter().zip(expected) {
            assert_eq!(content.raw_record().decode()?, expected);
        }
        assert!(DevArtwork::find(&mut reopened, &1)?.is_none());
        Ok(())
    }

    #[test]
    fn malformed_content_rolls_back_artwork_and_prior_reference_updates() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevArtwork::new("/Art/keep.jpg").insert(&mut conn)?;
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
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
        assert!(DevArtwork::delete(&mut conn, &1).is_err());
        assert_eq!(conn.modified_page_count(), 0);
        assert!(DevArtwork::find(&mut conn, &1)?.is_some());
        assert_eq!(DevContent::find(&mut conn, &1)?.unwrap().artwork_id, 1);
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }
}

mod artist_mutations {
    //! Artist mutations preserve binary data and clear all artist roles atomically.

    use super::*;
    use rbox_devlib::binary::TrackHeader;
    use rbox_devlib::models::{DevAlbum, DevArtist, DevExportProperty, NewDevAlbum, NewDevArtist};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_and_slot_bits_and_resolve_relocated_snapshots() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = artist_row(1, "Original")?;
        row.name = DeviceSqlString::encode("Original", StringEncoding::Utf16Le)?;
        row.header.index_shift = 0x0b;
        let mut tx = conn.transaction();
        let handle = tx.insert(DevArtist::TABLE, &Row::Standard(StandardRow::Artist(row)))?;
        let artist = DevArtist::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert_eq!(artist.clone().update(&mut conn)?, artist);
        assert_eq!(bytes(&mut conn)?, before);

        // Intervening low-level changes must survive a model rename.
        let mut tx = conn.transaction();
        let raw = artist.raw_record();
        let handle = tx.locate(DevArtist::TABLE, raw.page(), raw.slot().slot)?;
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
        assert_eq!(DevArtist::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn invalid_missing_duplicate_and_wrong_namespace_mutations_are_rejected() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let artist = NewDevArtist::new("Artist").insert(&mut conn)?;
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
            DevArtist::delete(&mut conn, &0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(DevArtist::delete(&mut conn, &99)?, 0);
        assert_eq!(bytes(&mut conn)?, before);
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            artist.clone().update(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevArtist::delete(&mut extended, &1),
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
            DevArtist::delete(&mut limited, &1),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(bytes(&mut limited)?, before);

        assert_eq!(DevArtist::delete(&mut conn, &1)?, 1);
        assert!(matches!(
            artist.clone().update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));
        let mut tx = conn.transaction();
        for _ in 0..2 {
            tx.insert(
                DevArtist::TABLE,
                &Row::Standard(StandardRow::Artist(artist_row(1, "Duplicate")?)),
            )?;
        }
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            artist.update(&mut conn),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert!(matches!(
            DevArtist::delete(&mut conn, &1),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }

    #[test]
    fn deletion_clears_all_roles_once_per_row_and_preserves_other_binary_fields() -> Result<()> {
        let options = CreateOptions::standard("2026-10-25", "Test")?;
        let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
        NewDevArtist::new("Delete").insert(&mut conn)?;
        NewDevArtist::new("Keep").insert(&mut conn)?;
        let mut expected_albums = Vec::new();
        for (id, artist_id) in [(10, 1), (11, 1), (12, 2), (13, 0)] {
            let mut album = NewDevAlbum::new(format!("Album {id}")).artist_id(artist_id);
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
            let mut content = NewDevContent::new(format!("/Contents/{i}.wav"));
            [
                content.artist_id,
                content.original_artist_id,
                content.composer_id,
                content.remixer_id,
            ] = roles;
            content.album_id = 1;
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
        let properties = DevExportProperty::all(&mut conn)?;
        let keep = DevArtist::find(&mut conn, &2)?
            .unwrap()
            .raw_record()
            .decode()?;
        assert_eq!(DevArtist::delete(&mut conn, &1)?, 8); // Artist + two albums + five tracks.
        assert_eq!(DevArtist::delete(&mut conn, &1)?, 0);
        assert!(DevArtist::find(&mut conn, &1)?.is_none());
        assert_eq!(
            DevArtist::find(&mut conn, &2)?
                .unwrap()
                .raw_record()
                .decode()?,
            keep
        );
        assert_eq!(DevExportProperty::all(&mut conn)?, properties);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        let albums = DevAlbum::all(&mut reopened)?;
        let tracks = DevContent::all(&mut reopened)?;
        assert_eq!(albums.len(), expected_albums.len());
        assert_eq!(tracks.len(), expected_tracks.len());
        for (album, expected) in albums.into_iter().zip(expected_albums) {
            assert_eq!(album.raw_record().decode()?, expected);
        }
        for (track, expected) in tracks.into_iter().zip(expected_tracks) {
            assert_eq!(track.raw_record().decode()?, expected);
        }
        assert!(DevArtist::find(&mut reopened, &1)?.is_none());
        Ok(())
    }

    #[test]
    fn malformed_content_rolls_back_artist_album_and_prior_content_changes() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevArtist::new("Keep on failure").insert(&mut conn)?;
        NewDevAlbum::new("Album").artist_id(1).insert(&mut conn)?;
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
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
        assert!(DevArtist::delete(&mut conn, &1).is_err());
        assert_eq!(conn.modified_page_count(), 0);
        assert!(DevArtist::find(&mut conn, &1)?.is_some());
        assert_eq!(DevAlbum::find(&mut conn, &1)?.unwrap().artist_id, 1);
        assert_eq!(DevContent::find(&mut conn, &1)?.unwrap().composer_id, 1);
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }
}

mod album_mutations {
    //! Album updates preserve current binary data; deletions clear references atomically.

    use super::*;
    use rbox_devlib::binary::TrackHeader;
    use rbox_devlib::models::{DevAlbum, DevExportProperty, NewDevAlbum, NewDevArtist};

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn updates_preserve_frames_and_current_reserved_fields_and_resolve_relocations() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut binary = album_row(1, 0, "Original")?;
        binary.name = DeviceSqlString::encode("Original", StringEncoding::Utf16Le)?;
        binary.header.unknown_04 = 0x12345678;
        let mut tx = conn.transaction();
        let handle = tx.insert(DevAlbum::TABLE, &Row::Standard(StandardRow::Album(binary)))?;
        let album = DevAlbum::try_from(tx.read(&handle)?)?;
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
        let handle = tx.locate(DevAlbum::TABLE, raw.page(), raw.slot().slot)?;
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
        assert_eq!(DevAlbum::find(&mut reopened, &1)?, Some(updated));
        Ok(())
    }

    #[test]
    fn invalid_updates_and_missing_or_ambiguous_ids_leave_the_database_unchanged() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let album = NewDevAlbum::new("Album").artist_id(0).insert(&mut conn)?;
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
            DevAlbum::delete(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevAlbum::delete(&mut conn, &0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(DevAlbum::delete(&mut conn, &99)?, 0);
        assert_eq!(DevAlbum::delete(&mut conn, &1)?, 1);
        assert!(matches!(
            album.clone().update(&mut conn),
            Err(Error::ModelNotFound { id: 1, .. })
        ));

        let mut tx = conn.transaction();
        for _ in 0..2 {
            tx.insert(
                DevAlbum::TABLE,
                &Row::Standard(StandardRow::Album(album_row(1, 0, "Duplicate")?)),
            )?;
        }
        tx.commit()?;
        let before = bytes(&mut conn)?;
        assert!(matches!(
            album.update(&mut conn),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert!(matches!(
            DevAlbum::delete(&mut conn, &1),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }

    #[test]
    fn deletion_clears_only_matching_references_and_keeps_counts_and_artists() -> Result<()> {
        let options = CreateOptions::standard("2026-10-25", "Test")?;
        let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
        let artist = NewDevArtist::new("Artist").insert(&mut conn)?;
        let deleted = NewDevAlbum::new("Delete")
            .artist_id(artist.id)
            .insert(&mut conn)?;
        let other = NewDevAlbum::new("Keep")
            .artist_id(artist.id)
            .insert(&mut conn)?;
        let mut expected = Vec::new();
        for (id, album_id) in [(1, deleted.id), (2, other.id), (3, deleted.id), (4, 0)] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
            content.album_id = album_id;
            let model = content.insert(&mut conn)?;
            let Row::Standard(StandardRow::Track(mut row)) = model.raw_record().decode()? else {
                panic!()
            };
            if album_id == deleted.id {
                row.header.album_id = 0;
            }
            expected.push(row);
        }
        let properties = DevExportProperty::all(&mut conn)?;
        assert_eq!(DevAlbum::delete(&mut conn, &deleted.id)?, 3);
        assert_eq!(DevAlbum::delete(&mut conn, &deleted.id)?, 0);
        assert!(DevAlbum::find(&mut conn, &deleted.id)?.is_none());
        let kept = DevAlbum::find(&mut conn, &other.id)?.unwrap();
        assert_eq!(kept.raw_record().as_bytes(), other.raw_record().as_bytes());
        assert_eq!(
            rbox_devlib::models::DevArtist::find(&mut conn, &1)?,
            Some(artist)
        );
        assert_eq!(DevExportProperty::all(&mut conn)?, properties);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        let contents = DevContent::all(&mut reopened)?;
        assert_eq!(contents.len(), expected.len());
        for (model, expected) in contents.into_iter().zip(expected) {
            assert_eq!(
                model.raw_record().decode()?,
                Row::Standard(StandardRow::Track(expected))
            );
        }
        assert!(DevAlbum::find(&mut reopened, &deleted.id)?.is_none());
        Ok(())
    }

    #[test]
    fn deletion_rolls_back_album_and_earlier_reference_updates_on_decode_failure() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevAlbum::new("Keep on failure")
            .artist_id(0)
            .insert(&mut conn)?;
        let mut records = Vec::new();
        for id in [1, 2] {
            let mut content = NewDevContent::new(format!("/Contents/{id}.wav"));
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
        assert!(DevAlbum::delete(&mut conn, &1).is_err());
        assert_eq!(conn.modified_page_count(), 0);
        assert!(DevAlbum::find(&mut conn, &1)?.is_some());
        assert_eq!(DevContent::find(&mut conn, &1)?.unwrap().album_id, 1);
        assert_eq!(bytes(&mut conn)?, before);
        Ok(())
    }
}

mod album_relations {
    //! Explicit album artist resolution, detached views and batch reuse.

    use super::*;
    use rbox_devlib::models::{DevAlbum, DevArtist, NewDevAlbum, NewDevArtist};

    #[test]
    fn album_views_keep_original_rows_local_edits_and_input_order() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevArtist::new("First artist").insert(&mut conn)?;
        NewDevArtist::new("Second artist").insert(&mut conn)?;
        NewDevAlbum::new("First album")
            .artist_id(1)
            .insert(&mut conn)?;
        NewDevAlbum::new("Second album")
            .artist_id(2)
            .insert(&mut conn)?;
        let albums = DevAlbum::all(&mut conn)?;
        let views = DevAlbum::all_with_relations(&mut conn)?;
        assert_eq!(views.len(), 2);
        for (view, album) in views.iter().zip(&albums) {
            assert_eq!(&view.album, album);
            assert_eq!(view.raw_record(), album.raw_record());
        }
        assert_eq!(views[0].name, "First album");
        assert_eq!(views[0].artist.as_ref().unwrap().name, "First artist");
        assert_eq!(views[1].artist.as_ref().unwrap().name, "Second artist");
        assert_eq!(
            DevAlbum::find_with_relations(&mut conn, &albums[0].id)?,
            Some(views[0].clone())
        );
        assert!(!format!("{:?}", views[0]).contains("raw:"));
        assert!(!format!("{:#?}", views[0]).contains("raw:"));
        let mut edited = albums[0].clone();
        edited.name = "Local name".into();
        edited.artist_id = 2;
        let batch =
            DevAlbum::resolve_many(&mut conn, vec![albums[1].clone(), edited.clone(), edited])?;
        assert_eq!(
            batch.iter().map(|view| view.id).collect::<Vec<_>>(),
            vec![albums[1].id, albums[0].id, albums[0].id]
        );
        assert_eq!(batch[1], batch[2]);
        assert_eq!(batch[1].name, "Local name");
        assert_eq!(batch[1].artist.as_ref().unwrap().name, "Second artist");
        assert_eq!(
            DevAlbum::find(&mut conn, &albums[0].id)?,
            Some(albums[0].clone())
        );
        Ok(())
    }

    #[test]
    fn missing_and_zero_artists_are_optional_and_wrong_namespaces_are_rejected() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let reads = conn.read_stats();
        assert!(DevAlbum::resolve_many(&mut conn, Vec::new())?.is_empty());
        assert_eq!(conn.read_stats(), reads);
        assert!(DevAlbum::find_with_relations(&mut conn, &99)?.is_none());
        let album = NewDevAlbum::new("Unassigned")
            .artist_id(0)
            .insert(&mut conn)?;
        let reads = conn.read_stats();
        assert!(album.clone().resolve(&mut conn)?.artist.is_none());
        assert_eq!(conn.read_stats(), reads);
        let mut missing = album.clone();
        missing.artist_id = 99;
        let view = missing.resolve(&mut conn)?;
        assert_eq!(view.artist_id, 99);
        assert!(view.artist.is_none());
        let artist = NewDevArtist::new("Now present").insert(&mut conn)?;
        let mut resolved = view.album;
        resolved.artist_id = artist.id;
        assert_eq!(
            resolved.resolve(&mut conn)?.artist.unwrap().name,
            "Now present"
        );
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            album.resolve(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevAlbum::resolve_many(&mut extended, Vec::new()),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevAlbum::find_with_relations(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevAlbum::all_with_relations(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }

    #[test]
    fn batch_resolution_reuses_artist_reads_without_a_page_cache() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevArtist::new("Shared artist").insert(&mut conn)?;
        NewDevAlbum::new("Album").artist_id(1).insert(&mut conn)?;
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
        let album = DevAlbum::find(&mut single, &1)?.unwrap();
        let repeated = DevAlbum::find(&mut batch, &1)?.unwrap();
        let before_single = single.read_stats().page_reads;
        let before_batch = batch.read_stats().page_reads;
        album.resolve(&mut single)?;
        let views = DevAlbum::resolve_many(&mut batch, vec![repeated; 50])?;
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
        NewDevArtist::new("Original").insert(&mut conn)?;
        let album = NewDevAlbum::new("Album").artist_id(1).insert(&mut conn)?;
        let mut bad = artist_row(2, "Bad")?;
        bad.name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let mut tx = conn.transaction();
        tx.insert(DevArtist::TABLE, &Row::Standard(StandardRow::Artist(bad)))?;
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
        let handle = tx.locate(DevArtist::TABLE, raw.page(), raw.slot().slot)?;
        tx.update(
            &handle,
            &Row::Standard(StandardRow::Artist(artist_row(1, "Updated")?)),
        )?;
        tx.commit()?;
        assert_eq!(
            album.clone().resolve(&mut conn)?.artist.unwrap().name,
            "Updated"
        );
        assert_eq!(old.artist.unwrap().name, "Original");
        let mut tx = conn.transaction();
        tx.insert(
            DevArtist::TABLE,
            &Row::Standard(StandardRow::Artist(artist_row(1, "Duplicate")?)),
        )?;
        tx.commit()?;
        assert!(
            matches!(album.resolve(&mut conn), Err(Error::DuplicateId { table, id: 1 }) if table == DevArtist::TABLE)
        );
        Ok(())
    }
}

mod content_relations {
    //! Explicit metadata views, shared targets, missing references and decoding errors.

    use super::*;
    use rbox_devlib::models::{
        DevArtist, NewDevAlbum, NewDevArtist, NewDevArtwork, NewDevGenre, NewDevKey, NewDevLabel,
    };

    #[test]
    fn resolves_all_roles_album_artist_and_metadata_without_losing_content() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        for (_id, name) in [
            (1, "Main"),
            (2, "Remixer"),
            (3, "Original"),
            (4, "Composer"),
            (5, "Album artist"),
        ] {
            NewDevArtist::new(name).insert(&mut conn)?;
        }
        let album = NewDevAlbum::new("Album").artist_id(5).insert(&mut conn)?;
        let genre = NewDevGenre::new("Genre").insert(&mut conn)?;
        let label = NewDevLabel::new("Label").insert(&mut conn)?;
        let key = NewDevKey::new("Am", 23).insert(&mut conn)?;
        seed_color(&mut conn, 255, "Custom color")?;
        let artwork = NewDevArtwork::new("/PIONEER/Artwork/image.jpg").insert(&mut conn)?;
        let mut input = NewDevContent::new("/Contents/Track.wav");
        input.artist_id = 1;
        input.remixer_id = 2;
        input.original_artist_id = 3;
        input.composer_id = 4;
        input.album_id = album.id;
        input.genre_id = genre.id;
        input.label_id = label.id;
        input.key_id = key.id;
        input.color_id = 255;
        input.artwork_id = artwork.id;
        let content = input.insert(&mut conn)?;
        let raw = content.raw_record().clone();
        let view = content.resolve(&mut conn)?;
        assert_eq!(view.id, 1);
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
            DevContent::find_with_relations(&mut conn, &1)?,
            Some(view.clone())
        );
        assert_eq!(
            DevContent::all_with_relations(&mut conn)?,
            vec![view.clone()]
        );
        assert!(!format!("{view:?}").contains("raw:"));
        let mut edited = view.content.clone();
        edited.title = "Local title".into();
        edited.artist_id = 2;
        let batch = DevContent::resolve_many(
            &mut conn,
            vec![edited.clone(), view.content.clone(), edited],
        )?;
        assert_eq!(batch[0].title, "Local title");
        assert_eq!(batch[0].artist.as_ref().unwrap().name, "Remixer");
        assert_eq!(batch[1], view);
        assert_eq!(batch[0], batch[2]);
        assert_eq!(DevContent::find(&mut conn, &1)?.unwrap().artist_id, 1);
        Ok(())
    }

    #[test]
    fn missing_targets_remain_none_and_empty_resolution_reads_nothing() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let reads = conn.read_stats();
        assert!(DevContent::resolve_many(&mut conn, Vec::new())?.is_empty());
        assert_eq!(conn.read_stats(), reads);
        assert!(DevContent::find_with_relations(&mut conn, &123)?.is_none());
        let content = NewDevContent::new("/Contents/Track.wav").insert(&mut conn)?;
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
        let album = NewDevAlbum::new("Missing artist")
            .artist_id(777)
            .insert(&mut conn)?;
        let mut missing = view.content;
        missing.album_id = album.id;
        let view = missing.resolve(&mut conn)?;
        assert!(view.album.is_some() && view.album_artist.is_none());
        let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            content.resolve(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevContent::resolve_many(&mut extended, Vec::new()),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevContent::find_with_relations(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevContent::all_with_relations(&mut extended),
            Err(Error::WrongDatabaseKind { .. })
        ));
        Ok(())
    }

    #[test]
    fn shared_targets_are_loaded_once_per_batch_even_without_page_cache() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevArtist::new("Shared").insert(&mut conn)?;
        NewDevAlbum::new("Album").artist_id(1).insert(&mut conn)?;
        let mut input = NewDevContent::new("/Contents/Track.wav");
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
        let content = DevContent::find(&mut single, &1)?.unwrap();
        let repeated = DevContent::find(&mut batch, &1)?.unwrap();
        let before_single = single.read_stats().page_reads;
        let before_batch = batch.read_stats().page_reads;
        content.resolve(&mut single)?;
        let views = DevContent::resolve_many(&mut batch, vec![repeated; 50])?;
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
        NewDevArtist::new("Good").insert(&mut conn)?;
        let mut bad = artist_row(2, "Bad")?;
        bad.name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let mut tx = conn.transaction();
        tx.insert(DevArtist::TABLE, &Row::Standard(StandardRow::Artist(bad)))?;
        tx.commit()?;
        let mut input = NewDevContent::new("/Contents/Track.wav");
        input.artist_id = 1;
        let content = input.insert(&mut conn)?;
        let old = content.clone().resolve(&mut conn)?;
        assert_eq!(old.artist.as_ref().unwrap().name, "Good");
        let mut invalid = content.clone();
        invalid.artist_id = 2;
        assert!(
            matches!(invalid.resolve(&mut conn), Err(Error::ModelText { table, field: "name", source, .. }) if table == DevArtist::TABLE && matches!(*source, Error::InvalidUtf16))
        );
        let raw = old.artist.as_ref().unwrap().raw_record();
        let mut tx = conn.transaction();
        let h = tx.locate(DevArtist::TABLE, raw.page(), raw.slot().slot)?;
        tx.update(
            &h,
            &Row::Standard(StandardRow::Artist(artist_row(1, "Updated")?)),
        )?;
        tx.commit()?;
        assert_eq!(
            content.clone().resolve(&mut conn)?.artist.unwrap().name,
            "Updated"
        );
        assert_eq!(old.artist.unwrap().name, "Good");
        let mut tx = conn.transaction();
        tx.insert(
            DevArtist::TABLE,
            &Row::Standard(StandardRow::Artist(artist_row(1, "Duplicate")?)),
        )?;
        tx.commit()?;
        assert!(
            matches!(content.resolve(&mut conn), Err(Error::DuplicateId { table, id: 1 }) if table == DevArtist::TABLE)
        );
        Ok(())
    }

    #[test]
    fn native_demo_content_resolves_catalog_rows() -> Result<()> {
        let mut conn = demo()?;
        let rows = DevContent::all(&mut conn)?;
        let views = DevContent::all_with_relations(&mut conn)?;
        assert_eq!(views.len(), rows.len());
        assert!(views.iter().any(|view| view.artist.is_some()));
        for (view, content) in views.into_iter().zip(rows) {
            assert_eq!(view.content, content);
            assert_eq!(view.artist, DevArtist::find(&mut conn, &content.artist_id)?);
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
        DevBrowseCategory, DevBrowseColumn, DevBrowseSort, DevDatabaseProperty, DevExportProperty,
        DevRecommendedLike, NewDevRecommendedLike,
    };

    // Seed binary rows independently of the model API under test.
    fn seed_browse(
        conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>,
        row: StandardRow,
    ) -> Result<RowRecord> {
        let table = match &row {
            StandardRow::BrowseColumn(_) => DevBrowseColumn::TABLE,
            StandardRow::BrowseCategory(_) => DevBrowseCategory::TABLE,
            StandardRow::BrowseSort(_) => DevBrowseSort::TABLE,
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
            let column = DevBrowseColumn::try_from(seed_browse(
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
        let expected = DevBrowseColumn::all(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(DevBrowseColumn::all(&mut reopened)?, expected);
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
        let categories = DevBrowseCategory::all(&mut conn)?;
        let sorts = DevBrowseSort::all(&mut conn)?;
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
        let expected_categories = DevBrowseCategory::all(&mut conn)?;
        let expected_sorts = DevBrowseSort::all(&mut conn)?;
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
        assert_eq!(DevBrowseCategory::all(&mut reopened)?, expected_categories);
        assert_eq!(DevBrowseSort::all(&mut reopened)?, expected_sorts);
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
        let column = DevBrowseColumn::all(&mut conn)?.remove(0);
        let category = DevBrowseCategory::all(&mut conn)?.remove(0);
        let sort = DevBrowseSort::all(&mut conn)?.remove(0);
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
                DevBrowseColumn::TABLE,
                &Row::Standard(StandardRow::BrowseColumn(binary)),
            )?;
            let record = tx.read(&handle)?;
            let model = DevBrowseColumn::try_from(record.clone())?;
            assert_eq!(model.name, plain);
            assert_eq!(model.raw_record(), &record);
            tx.commit()?;
        }
        let models = DevBrowseColumn::all(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(DevBrowseColumn::all(&mut reopened)?, models);
        Ok(())
    }

    #[test]
    fn browse_models_preserve_raw_settings_framing_and_repeated_rows() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let reads = conn.read_stats();
        drop(DevBrowseColumn::iter(&mut conn)?);
        drop(DevBrowseCategory::iter(&mut conn)?);
        drop(DevBrowseSort::iter(&mut conn)?);
        drop(DevExportProperty::iter(&mut conn)?);
        assert_eq!(conn.read_stats(), reads);
        assert!(DevBrowseColumn::find(&mut conn, &1)?.is_none());
        let mut column = DevBrowseColumn::try_from(seed_browse(
            &mut conn,
            StandardRow::BrowseColumn(BrowseColumnRow {
                id: u16::MAX,
                code: 0xabcd,
                name: DeviceSqlString::encode("Ã©Å¸Â³Ã¦Â¥Â½", StringEncoding::Utf16Le)?,
            }),
        )?)?;
        assert_eq!((column.id, column.code), (i64::from(u16::MAX), 0xabcd));
        column.name = "Local edit".into();
        assert_eq!(
            DevBrowseColumn::find(&mut conn, &i64::from(u16::MAX))?
                .unwrap()
                .name,
            "Ã©Å¸Â³Ã¦Â¥Â½"
        );
        let reads = conn.read_stats();
        assert!(DevBrowseColumn::id_exists(&mut conn, &i64::from(u16::MAX))?);
        assert!(!DevBrowseColumn::id_exists(&mut conn, &1)?);
        assert_eq!(conn.read_stats(), reads);
        DevBrowseColumn::try_from(seed_browse(
            &mut conn,
            StandardRow::BrowseColumn(BrowseColumnRow {
                id: 1,
                code: 0,
                name: DeviceSqlString::encode("", StringEncoding::Utf16Le)?,
            }),
        )?)?;
        assert!(DevBrowseColumn::id_exists(&mut conn, &1)?);
        for position in [9, 1, 1] {
            DevBrowseCategory::try_from(seed_browse(
                &mut conn,
                StandardRow::BrowseCategory(BrowseCategoryRow {
                    column_id: u16::MAX,
                    selector: 0,
                    unknown_04: 0xee,
                    state: 0x80,
                    position: position,
                }),
            )?)?;
            DevBrowseSort::try_from(seed_browse(
                &mut conn,
                StandardRow::BrowseSort(BrowseSortRow {
                    column_id: u16::MAX,
                    selector: 0,
                    raw_tail: 0x1234abcd,
                }),
            )?)?;
        }
        let categories = DevBrowseCategory::by_column(&mut conn, &i64::from(u16::MAX))?;
        assert_eq!(
            categories
                .iter()
                .map(|c| (c.unknown_04, c.state, c.position))
                .collect::<Vec<_>>(),
            vec![(0xee, 0x80, 9), (0xee, 0x80, 1), (0xee, 0x80, 1)]
        );
        let sorts = DevBrowseSort::by_column(&mut conn, &i64::from(u16::MAX))?;
        assert_eq!(
            sorts
                .iter()
                .map(|s| (s.selector, s.raw_tail))
                .collect::<Vec<_>>(),
            vec![(0, 0x1234abcd); 3]
        );
        assert!(DevBrowseCategory::by_column(&mut conn, &1)?.is_empty());
        assert!(DevBrowseSort::by_column(&mut conn, &1)?.is_empty());
        let columns = DevBrowseColumn::all(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(DevBrowseColumn::all(&mut reopened)?, columns);
        assert_eq!(DevBrowseCategory::all(&mut reopened)?, categories);
        assert_eq!(DevBrowseSort::all(&mut reopened)?, sorts);
        Ok(())
    }

    #[test]
    fn initialized_properties_support_binary_updates_and_track_count_maintenance() -> Result<()> {
        let options = CreateOptions::standard("2026-10-25", "Library Ã©Å¸Â³Ã¦Â¥Â½")?;
        let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
        let properties = DevExportProperty::all(&mut conn)?;
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
        let first = DevExportProperty::try_from(tx.read(&handle)?)?;
        // Multiple live properties must have distinct device names.
        let Row::Standard(StandardRow::ExportProperty(mut second)) = binary.clone() else {
            panic!()
        };
        second.name = DeviceSqlString::encode("Second USB", StringEncoding::ShortAscii)?;
        tx.insert(
            DevExportProperty::TABLE,
            &Row::Standard(StandardRow::ExportProperty(second)),
        )?;
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
        NewDevContent::new("/Contents/Track.wav").insert(&mut conn)?;
        let properties = DevExportProperty::all(&mut conn)?;
        assert_eq!(properties.len(), 2);
        assert!(properties
            .iter()
            .all(|p| p.track_count == 1 && p.version == "custom"));
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(DevExportProperty::all(&mut reopened)?, properties);
        Ok(())
    }

    #[test]
    fn extended_properties_and_pairs_preserve_explicit_unknowns_and_duplicates() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let reads = conn.read_stats();
        drop(DevDatabaseProperty::iter(&mut conn)?);
        drop(DevRecommendedLike::iter(&mut conn)?);
        assert_eq!(conn.read_stats(), reads);
        assert!(DevDatabaseProperty::all(&mut conn)?.is_empty());
        assert!(DevRecommendedLike::all(&mut conn)?.is_empty());
        let mut options = CreateOptions::extended(0)?;
        let table = options
            .tables
            .iter_mut()
            .find(|t| t.table == DevDatabaseProperty::TABLE)
            .unwrap();
        let Row::Extended(ExtendedRow::DatabaseProperty(row)) = &mut table.rows[0] else {
            panic!()
        };
        row.header.unknown_words = [1, 2, 3, 4, 5];
        table.rows.push(table.rows[0].clone());
        let mut conn = DeviceSqlConnection::create_with_options(DatabaseKind::Extended, &options)?;
        let model = DevDatabaseProperty::all(&mut conn)?.remove(0);
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
        let updated = DevDatabaseProperty::try_from(tx.read(&handle)?)?;
        assert_eq!(updated.my_tag_master_db_id, 42);
        assert_eq!(updated.raw_record().decode()?, binary);
        tx.commit()?;
        assert_eq!(model.my_tag_master_db_id, 0);
        for (first, second) in [(10, 20), (20, 10), (10, 20), (10, 10), (30, 40)] {
            // Existing binary files can contain duplicates even though model
            // insertion now rejects them. Reads must continue to preserve them.
            let row = recommendation_row(&NewDevRecommendedLike::new(
                first,
                second,
                u64::MAX,
                0x12345678,
                0xabcdef01,
            ));
            let mut tx = conn.transaction();
            tx.insert(
                DevRecommendedLike::TABLE,
                &Row::Extended(ExtendedRow::RecommendLike(row)),
            )?;
            tx.commit()?;
        }
        assert_eq!(DevRecommendedLike::by_content_id(&mut conn, &10)?.len(), 4);
        assert!(DevRecommendedLike::by_content_id(&mut conn, &99)?.is_empty());
        let pairs = DevRecommendedLike::all(&mut conn)?;
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
        let properties = DevDatabaseProperty::all(&mut conn)?;
        assert_eq!(properties.len(), 2);
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Extended)?;
        assert_eq!(DevRecommendedLike::all(&mut reopened)?, pairs);
        assert_eq!(DevDatabaseProperty::all(&mut reopened)?, properties);
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
        check!(standard, DevBrowseColumn, Standard, BrowseColumn, |m, r| {
            assert_eq!((m.id, m.code), (i64::from(r.id), r.code));
            assert_eq!(r.name.text()?, format!("\u{fffa}{}\u{fffb}", m.name));
            assert!(!m.name.contains(['\u{fffa}', '\u{fffb}']));
            assert_eq!(DevBrowseColumn::find(&mut standard, &m.id)?, Some(m));
        });
        check!(
            standard,
            DevBrowseCategory,
            Standard,
            BrowseCategory,
            |m, r| {
                assert_eq!(
                    (m.column_id, m.selector, m.unknown_04, m.state, m.position),
                    (
                        i64::from(r.column_id),
                        r.selector,
                        r.unknown_04,
                        r.state,
                        r.position
                    )
                );
            }
        );
        check!(standard, DevBrowseSort, Standard, BrowseSort, |m, r| {
            assert_eq!(
                (m.column_id, m.selector, m.raw_tail),
                (i64::from(r.column_id), r.selector, r.raw_tail)
            );
        });
        check!(
            standard,
            DevExportProperty,
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
            DevDatabaseProperty,
            Extended,
            DatabaseProperty,
            |m, r| {
                assert_eq!(
                    m.my_tag_master_db_id,
                    i64::from(r.header.my_tag_master_db_id)
                );
            }
        );
        check!(
            extended,
            DevRecommendedLike,
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
                        i64::from(r.track_id_1),
                        i64::from(r.track_id_2),
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
        wrong!(extended, DevBrowseColumn);
        wrong!(extended, DevBrowseCategory);
        wrong!(extended, DevBrowseSort);
        wrong!(extended, DevExportProperty);
        wrong!(standard, DevDatabaseProperty);
        wrong!(
            standard,
            DevRecommendedLike,
            NewDevRecommendedLike::new(1, 2, 3, 4, 5)
        );
        assert!(matches!(
            DevBrowseColumn::find(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevBrowseColumn::id_exists(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevBrowseCategory::by_column(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevBrowseSort::by_column(&mut extended, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevRecommendedLike::by_content_id(&mut standard, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        let column = DevBrowseColumn::try_from(seed_browse(
            &mut standard,
            StandardRow::BrowseColumn(BrowseColumnRow {
                id: 1,
                code: 0x80,
                name: DeviceSqlString::encode("Genre", StringEncoding::Utf16Le)?,
            }),
        )?)?;
        let record = column.raw_record().clone();
        assert!(matches!(
            DevBrowseCategory::try_from(record.clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            DevBrowseSort::try_from(record.clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            DevExportProperty::try_from(record.clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            DevDatabaseProperty::try_from(record.clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            DevRecommendedLike::try_from(record),
            Err(Error::UnsupportedModelRow { .. })
        ));
        let category = DevBrowseCategory::try_from(seed_browse(
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
            DevBrowseColumn::try_from(category.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        let mut before = Vec::new();
        standard.write_to(&mut before)?;
        let state = (standard.page_count(), standard.modified_page_count());
        assert!(matches!(
            NewDevRecommendedLike::new(0, 2, 3, 4, 5).insert(&mut extended),
            Err(Error::InvalidModelField {
                field: "content_id_1",
                ..
            })
        ));
        assert!(matches!(
            NewDevRecommendedLike::new(1, 0, 3, 4, 5).insert(&mut extended),
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
            NewDevRecommendedLike::new(1, 2, 3, 4, 5).insert(&mut extended),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(DevRecommendedLike::all(&mut extended)?.is_empty());
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
            DevBrowseColumn::TABLE,
            &Row::Standard(StandardRow::BrowseColumn(column)),
        )?;
        tx.commit()?;
        assert!(DevBrowseColumn::id_exists(&mut conn, &1)?);
        assert!(
            matches!(DevBrowseColumn::find(&mut conn, &1), Err(Error::ModelText { field: "name", source, .. }) if matches!(*source, Error::InvalidUtf16))
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
                DevExportProperty::TABLE,
                &Row::Standard(StandardRow::ExportProperty(row)),
            )?;
            assert!(
                matches!(DevExportProperty::try_from(tx.read(&handle)?), Err(Error::ModelText { field: actual, source, .. }) if actual == field && matches!(*source, Error::InvalidUtf16))
            );
            tx.rollback();
        }
        assert!(DevExportProperty::all(&mut conn)?.is_empty());
        let mut tx = conn.transaction();
        let h = tx.insert(
            DevBrowseSort::TABLE,
            &Row::Standard(StandardRow::BrowseSort(BrowseSortRow {
                column_id: 1,
                selector: 0,
                raw_tail: u32::MAX,
            })),
        )?;
        assert_eq!(DevBrowseSort::try_from(tx.read(&h)?)?.raw_tail, u32::MAX);
        tx.rollback();
        assert!(DevBrowseSort::all(&mut conn)?.is_empty());
        Ok(())
    }
}

mod history {
    //! History tables remain distinct from regular playlists and retain every play.

    use super::*;
    use rbox_devlib::models::{
        DevHistory, DevHistoryContent, DevPlaylistContent, NewDevHistory, NewDevHistoryContent,
        NewDevPlaylist,
    };

    #[test]
    fn history_round_trip_preserves_occurrences_order_and_separate_playlist_ids() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let reads = conn.read_stats();
        drop(DevHistory::iter(&mut conn)?);
        drop(DevHistoryContent::iter(&mut conn)?);
        assert_eq!(conn.read_stats(), reads);
        assert!(DevHistory::all(&mut conn)?.is_empty());
        assert!(DevHistory::find(&mut conn, &1)?.is_none());
        assert!(DevHistoryContent::by_history_id(&mut conn, &1)?.is_empty());
        NewDevPlaylist::playlist("Regular playlist")
            .seq(0)
            .insert(&mut conn)?;
        let mut detached = NewDevHistory::new("Session Ã©Å¸Â³Ã¦Â¥Â½").insert(&mut conn)?;
        detached.name = "Local edit".into();
        assert_eq!(
            DevHistory::find(&mut conn, &1)?.unwrap().name,
            "Session Ã©Å¸Â³Ã¦Â¥Â½"
        );
        let reads = conn.read_stats();
        assert!(DevHistory::id_exists(&mut conn, &1)?);
        assert!(!DevHistory::id_exists(&mut conn, &2)?);
        assert_eq!(conn.read_stats(), reads);
        NewDevHistory::new("").insert(&mut conn)?;
        assert!(DevHistory::id_exists(&mut conn, &2)?);
        NewDevContent::new("/Contents/One.wav").insert(&mut conn)?;
        NewDevContent::new("/Contents/Two.wav").insert(&mut conn)?;
        for (playlist, track, position) in [
            (1, 10, 30),
            (1, 20, 4),
            (1, 10, 4),
            (1, 10, 4),
            (1, 10, 0),
            (2, 20, 1),
        ] {
            NewDevHistoryContent::new(playlist, track, position).insert(&mut conn)?;
        }
        let ordered = DevHistoryContent::by_history_id(&mut conn, &1)?;
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
        assert_eq!(DevHistoryContent::by_history_id(&mut conn, &2)?.len(), 1);
        assert!(DevHistoryContent::by_history_id(&mut conn, &99)?.is_empty());
        assert!(DevPlaylistContent::all(&mut conn)?.is_empty());
        let all = DevHistoryContent::all(&mut conn)?;
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
        let playlists = DevHistory::all(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(DevHistory::all(&mut reopened)?, playlists);
        assert_eq!(DevHistoryContent::all(&mut reopened)?, all);
        assert_eq!(
            DevHistoryContent::by_history_id(&mut reopened, &1)?,
            ordered
        );
        Ok(())
    }

    #[test]
    fn history_insert_errors_preserve_the_connection() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevHistory::new("Original").insert(&mut conn)?;
        let state = (conn.page_count(), conn.modified_page_count());
        let mut before = Vec::new();
        conn.write_to(&mut before)?;
        assert!(matches!(
            NewDevHistory::new("Bad\0name").insert(&mut conn),
            Err(Error::EmbeddedNul)
        ));
        assert!(matches!(
            NewDevHistory::new("x".repeat(5000)).insert(&mut conn),
            Err(Error::NoRowSpace { .. })
        ));
        assert!(matches!(
            NewDevHistoryContent::new(0, 1, 1).insert(&mut conn),
            Err(Error::InvalidModelField {
                field: "history_id",
                ..
            })
        ));
        assert!(matches!(
            NewDevHistoryContent::new(1, 0, 1).insert(&mut conn),
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
            NewDevHistory::new("Limited").insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(matches!(
            NewDevHistoryContent::new(1, 2, 1).insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(limited.modified_page_count(), 0);
        assert!(DevHistory::all(&mut limited)?.is_empty());
        assert!(DevHistoryContent::all(&mut limited)?.is_empty());
        Ok(())
    }

    #[test]
    fn history_models_reject_extended_connections_and_regular_playlist_rows() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            DevHistory::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevHistory::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevHistory::find(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevHistory::id_exists(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevHistoryContent::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevHistoryContent::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevHistoryContent::by_history_id(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewDevHistory::new("Wrong").insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewDevHistoryContent::new(1, 2, 1).insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert_eq!(conn.modified_page_count(), 0);
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let regular = NewDevPlaylist::playlist("Regular")
            .seq(0)
            .insert(&mut conn)?;
        seed_content(&mut conn, 10, "/test.wav")?;
        let entry = rbox_devlib::models::NewDevPlaylistContent::new(1, 10, 1).insert(&mut conn)?;
        assert!(matches!(
            DevHistory::try_from(regular.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            DevHistoryContent::try_from(entry.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        Ok(())
    }

    #[test]
    fn explicit_history_transactions_and_invalid_text_keep_context() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = named_row(1, "Name")?;
        row.name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let mut tx = conn.transaction();
        tx.insert(
            DevHistory::TABLE,
            &Row::Standard(StandardRow::HistoryPlaylist(row)),
        )?;
        tx.commit()?;
        assert!(DevHistory::id_exists(&mut conn, &1)?);
        assert!(
            matches!(DevHistory::find(&mut conn, &1), Err(Error::ModelText { field: "name", source, .. }) if matches!(*source, Error::InvalidUtf16))
        );
        let mut tx = conn.transaction();
        let handle = tx.insert(
            DevHistoryContent::TABLE,
            &Row::Standard(StandardRow::HistoryEntry(history_entry_row(
                &NewDevHistoryContent::new(1, 20, 9),
            ))),
        )?;
        let model = DevHistoryContent::try_from(tx.read(&handle)?)?;
        assert_eq!((model.history_id, model.content_id, model.seq), (1, 20, 9));
        tx.rollback();
        assert!(DevHistoryContent::all(&mut conn)?.is_empty());
        Ok(())
    }

    #[test]
    fn native_history_tables_match_binary_rows_including_empty_tables() -> Result<()> {
        let mut conn = demo()?;
        let raw = conn
            .iter_raw(DevHistory::TABLE)?
            .collect::<Result<Vec<_>>>()?;
        let models = DevHistory::all(&mut conn)?;
        assert_eq!(models.len(), raw.len());
        for (model, record) in models.into_iter().zip(raw) {
            let Row::Standard(StandardRow::HistoryPlaylist(row)) = record.decode()? else {
                panic!()
            };
            assert_eq!(model.id, i64::from(row.id));
            assert_eq!(model.name, row.name.text()?);
            assert_eq!(model.raw_record(), &record);
            assert_eq!(DevHistory::find(&mut conn, &model.id)?, Some(model));
        }
        let raw = conn
            .iter_raw(DevHistoryContent::TABLE)?
            .collect::<Result<Vec<_>>>()?;
        let models = DevHistoryContent::all(&mut conn)?;
        assert_eq!(models.len(), raw.len());
        for (model, record) in models.into_iter().zip(raw) {
            let Row::Standard(StandardRow::HistoryEntry(row)) = record.decode()? else {
                panic!()
            };
            assert_eq!(
                (model.content_id, model.history_id, model.seq),
                (
                    i64::from(row.track_id),
                    i64::from(row.playlist_id),
                    row.position
                )
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
    use rbox_devlib::models::{DevMyTag, DevMyTagContent, NewDevMyTag, NewDevMyTagContent};

    fn raw_tag(
        conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>,
        input: NewDevMyTag,
    ) -> Result<DevMyTag> {
        let mut tx = conn.transaction();
        let handle = tx.insert(
            DevMyTag::TABLE,
            &Row::Extended(ExtendedRow::MyTag(my_tag_row(&input)?)),
        )?;
        let model = DevMyTag::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    #[test]
    fn fresh_tags_and_associations_round_trip_with_explicit_ids_and_positions() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let reads = conn.read_stats();
        drop(DevMyTag::iter(&mut conn)?);
        drop(DevMyTagContent::iter(&mut conn)?);
        assert_eq!(conn.read_stats(), reads);
        assert!(DevMyTag::all(&mut conn)?.is_empty());
        assert!(DevMyTag::find(&mut conn, &99)?.is_none());
        assert!(DevMyTagContent::all(&mut conn)?.is_empty());
        let category = NewDevMyTag::folder(0xffff_ffff, "Mood")
            .seq(0)
            .insert(&mut conn)?;
        assert!(category.is_category());
        let mut input = NewDevMyTag::tag(2, "Ã©Å¸Â³Ã¦Â¥Â½")
            .parent_id(category.id)
            .seq(9);
        input.unknown_04 = 0x12345678;
        input.unknown_08 = 32;
        input.unknown_18 = 17;
        input.unknown_1a = 5;
        let tag = raw_tag(&mut conn, input)?;
        assert!(!tag.is_category());
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
        let second = NewDevMyTag::new(3, "Second tag", MyTagType::List)
            .parent_id(category.id)
            .seq(1);
        assert!(!raw_tag(&mut conn, second)?.is_category());
        raw_tag(
            &mut conn,
            NewDevMyTag::tag(4, "Tied").parent_id(category.id).seq(1),
        )?;
        assert_eq!(
            DevMyTag::by_parent_id(&mut conn, &category.id)?
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>(),
            vec![3, 4, 2]
        );
        assert_eq!(DevMyTag::by_parent_id(&mut conn, &0)?.len(), 1);
        assert!(DevMyTag::by_parent_id(&mut conn, &99)?.is_empty());
        assert_eq!(DevMyTag::find(&mut conn, &2)?.unwrap().name, "Ã©Å¸Â³Ã¦Â¥Â½");
        let reads = conn.read_stats();
        assert!(DevMyTag::id_exists(&mut conn, &2)?);
        assert!(!DevMyTag::id_exists(&mut conn, &99)?);
        assert_eq!(conn.read_stats(), reads);
        let mut detached = tag;
        detached.name = "Local only".into();
        assert_eq!(DevMyTag::find(&mut conn, &2)?.unwrap().name, "Ã©Å¸Â³Ã¦Â¥Â½");

        for (track, tag) in [(10, 2), (10, 3), (10, 2), (20, 2)] {
            NewDevMyTagContent::new(track, tag).insert(&mut conn)?;
        }
        assert_eq!(
            DevMyTagContent::by_content_id(&mut conn, &10)?
                .iter()
                .map(|r| r.my_tag_id)
                .collect::<Vec<_>>(),
            vec![2, 3, 2]
        );
        assert_eq!(
            DevMyTagContent::by_my_tag_id(&mut conn, &2)?
                .iter()
                .map(|r| r.content_id)
                .collect::<Vec<_>>(),
            vec![10, 10, 20]
        );
        assert!(DevMyTagContent::by_content_id(&mut conn, &99)?.is_empty());
        assert!(DevMyTagContent::by_my_tag_id(&mut conn, &99)?.is_empty());
        let tags = DevMyTag::all(&mut conn)?;
        let links = DevMyTagContent::all(&mut conn)?;
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Extended)?;
        assert_eq!(DevMyTag::all(&mut reopened)?, tags);
        assert_eq!(DevMyTagContent::all(&mut reopened)?, links);
        Ok(())
    }

    #[test]
    fn native_tags_preserve_names_ids_attributes_and_reserved_bytes() -> Result<()> {
        let bytes = fs::read(export_dir(7).join("rekordbox/exportExt.pdb"))?;
        let mut conn =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Extended)?;
        let raw = conn
            .iter_raw(DevMyTag::TABLE)?
            .collect::<Result<Vec<_>>>()?;
        assert!(!raw.is_empty());
        let models = DevMyTag::all(&mut conn)?;
        assert_eq!(models.len(), raw.len());
        for (model, record) in models.into_iter().zip(&raw) {
            let Row::Extended(ExtendedRow::MyTag(row)) = record.decode()? else {
                panic!()
            };
            assert_eq!(
                (model.id, model.parent_id, model.seq, model.attribute as u8),
                (
                    i64::from(row.header.id),
                    i64::from(row.header.parent_id),
                    row.header.position,
                    row.header.attribute
                )
            );
            assert_eq!(model.name, row.name.text()?);
            assert_eq!(Some(model.is_category()), row.header.is_category());
            assert_eq!(model.raw_record(), record);
            assert_eq!(DevMyTag::find(&mut conn, &model.id)?, Some(model));
        }
        let raw = conn
            .iter_raw(DevMyTagContent::TABLE)?
            .collect::<Result<Vec<_>>>()?;
        // The demo exports the tag catalog but has no assigned track tags.
        assert!(raw.is_empty());
        assert!(DevMyTagContent::all(&mut conn)?.is_empty());
        assert_eq!(conn.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn invalid_tag_inputs_and_allocation_failures_do_not_publish_changes() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        NewDevMyTag::folder(1, "Original")
            .seq(0)
            .insert(&mut conn)?;
        let mut before = Vec::new();
        conn.write_to(&mut before)?;
        let state = (conn.page_count(), conn.modified_page_count());
        assert!(matches!(
            NewDevMyTag::folder(1, "Duplicate").seq(0).insert(&mut conn),
            Err(Error::DuplicateId { id: 1, .. })
        ));
        assert!(matches!(
            NewDevMyTag::folder(0, "Zero").seq(0).insert(&mut conn),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        assert!(matches!(
            NewDevMyTag::tag(2, "Bad\0name")
                .parent_id(1)
                .seq(0)
                .insert(&mut conn),
            Err(Error::EmbeddedNul)
        ));
        // The second string offset exceeds the qualified near-layout range.
        assert!(matches!(
            NewDevMyTag::tag(2, "x".repeat(300))
                .parent_id(1)
                .seq(0)
                .insert(&mut conn),
            Err(Error::UnsupportedMutation { .. })
        ));
        assert!(matches!(
            NewDevMyTagContent::new(0, 1).insert(&mut conn),
            Err(Error::InvalidModelField {
                field: "content_id",
                ..
            })
        ));
        assert!(matches!(
            NewDevMyTagContent::new(1, 0).insert(&mut conn),
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
            NewDevMyTag::folder(1, "Limited")
                .seq(0)
                .insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(matches!(
            NewDevMyTagContent::new(1, 2).insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(limited.modified_page_count(), 0);
        assert!(DevMyTag::all(&mut limited)?.is_empty());
        assert!(DevMyTagContent::all(&mut limited)?.is_empty());
        Ok(())
    }

    #[test]
    fn tag_models_reject_standard_connections_and_other_row_types() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        assert!(matches!(
            DevMyTag::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTag::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTag::find(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTag::id_exists(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTag::by_parent_id(&mut conn, &0),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTagContent::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTagContent::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTagContent::by_content_id(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevMyTagContent::by_my_tag_id(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewDevMyTag::folder(1, "Wrong").seq(0).insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewDevMyTagContent::new(1, 2).insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert_eq!(conn.modified_page_count(), 0);
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let tag = NewDevMyTag::folder(1, "Tag").seq(0).insert(&mut conn)?;
        let link = NewDevMyTagContent::new(1, 2).insert(&mut conn)?;
        assert!(matches!(
            DevMyTag::try_from(link.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            DevMyTagContent::try_from(tag.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        Ok(())
    }

    #[test]
    fn binary_transactions_preserve_reserved_values_and_report_bad_tag_text() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        let mut row = my_tag_row(&NewDevMyTag::tag(2, "Name").parent_id(1).seq(0))?;
        row.name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let mut tx = conn.transaction();
        tx.insert(DevMyTag::TABLE, &Row::Extended(ExtendedRow::MyTag(row)))?;
        tx.commit()?;
        assert!(DevMyTag::id_exists(&mut conn, &2)?);
        assert!(
            matches!(DevMyTag::find(&mut conn, &2), Err(Error::ModelText { field: "name", source, .. }) if matches!(*source, Error::InvalidUtf16))
        );
        let mut input = NewDevMyTagContent::new(7, 2);
        input.unknown_00 = 0xabcdef;
        input.reserved_string_marker = 5;
        input.reserved_tail = [3, 2, 1];
        let row = tag_content_row(&input);
        let mut tx = conn.transaction();
        let handle = tx.insert(
            DevMyTagContent::TABLE,
            &Row::Extended(ExtendedRow::TrackMyTag(row.clone())),
        )?;
        let model = DevMyTagContent::try_from(tx.read(&handle)?)?;
        assert_eq!(
            model.raw_record().decode()?,
            Row::Extended(ExtendedRow::TrackMyTag(row))
        );
        tx.rollback();
        assert!(DevMyTagContent::all(&mut conn)?.is_empty());
        Ok(())
    }
}

mod playlists {
    //! Playlist nodes, ordered occurrences, namespace checks and failure behavior.

    use super::*;
    use rbox_devlib::models::{
        DevPlaylist, DevPlaylistContent, NewDevPlaylist, NewDevPlaylistContent,
    };

    fn bytes(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        Ok(bytes)
    }

    fn children(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>, parent: i64) -> Result<Vec<i64>> {
        Ok(DevPlaylist::by_parent_id(conn, &parent)?
            .iter()
            .map(|p| p.id)
            .collect())
    }

    fn contents(conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>, parent: i64) -> Result<Vec<i64>> {
        Ok(DevPlaylistContent::by_playlist_id(conn, &parent)?
            .iter()
            .map(|p| p.content_id)
            .collect())
    }

    #[test]
    fn ordered_tree_insertions_moves_queries_and_bulk_deletion() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevPlaylist::folder("Root").insert(&mut conn)?;
        NewDevPlaylist::folder("Other").insert(&mut conn)?;
        NewDevPlaylist::folder("Nested")
            .parent_id(1)
            .insert(&mut conn)?;
        NewDevPlaylist::playlist("Last")
            .parent_id(1)
            .insert(&mut conn)?;
        let mut input = NewDevPlaylist::playlist("Middle").parent_id(1).seq(1);
        input.unknown_04 = 0xaabbccdd;
        input.insert(&mut conn)?;
        assert_eq!(children(&mut conn, 1)?, [3, 5, 4]);
        assert_eq!(DevPlaylist::count_children(&mut conn, &1)?, 3);
        assert_eq!(DevPlaylist::get_seq_numbers(&mut conn, &1)?, [0, 1, 2]);
        assert!(DevPlaylist::validate_seq_numbers(&mut conn, &1)?);
        assert!(DevPlaylist::is_playlist(&mut conn, &5)?);
        assert!(!DevPlaylist::is_playlist(&mut conn, &1)?);
        assert_eq!(
            DevPlaylist::get_attribute(&mut conn, &0)?,
            Some(PlaylistType::Folder)
        );
        assert_eq!(DevPlaylist::get_attribute(&mut conn, &99)?, None);
        assert_eq!(
            DevPlaylist::find_by_path(&mut conn, &["Root", "Middle"])?
                .unwrap()
                .id,
            5
        );
        assert!(DevPlaylist::find_by_path(&mut conn, &[])?.is_none());
        assert!(DevPlaylist::find_by_path(&mut conn, &["Missing"])?.is_none());
        let tree = DevPlaylist::tree(&mut conn)?;
        assert_eq!(
            tree.iter().map(|n| n.playlist.id).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            tree[0]
                .children
                .iter()
                .map(|n| n.playlist.id)
                .collect::<Vec<_>>(),
            [3, 5, 4]
        );
        assert_eq!(DevPlaylist::move_to(&mut conn, &4, None, Some(0))?, 1);
        assert_eq!(children(&mut conn, 1)?, [4, 3, 5]);
        assert_eq!(DevPlaylist::move_all_to(&mut conn, &[5, 4], &2, None)?, 2);
        assert_eq!(children(&mut conn, 1)?, [3]);
        assert_eq!(children(&mut conn, 2)?, [5, 4]);
        assert_eq!(DevPlaylist::get_seq_numbers(&mut conn, &2)?, [0, 1]);
        assert_eq!(DevPlaylist::rename(&mut conn, &5, "Renamed")?, 1);
        let row = DevPlaylist::find(&mut conn, &5)?.unwrap();
        let Row::Standard(StandardRow::PlaylistTree(binary)) = row.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(binary.unknown_04, 0xaabbccdd);
        assert_eq!(
            (row.parent_id, row.seq, row.name.as_str()),
            (2, 0, "Renamed")
        );
        // Moving an ancestor and its descendant to another folder flattens the selection.
        assert_eq!(DevPlaylist::move_all_to(&mut conn, &[1, 3], &2, None)?, 2);
        assert_eq!(children(&mut conn, 0)?, [2]);
        assert!(children(&mut conn, 1)?.is_empty());
        assert_eq!(children(&mut conn, 2)?, [5, 4, 1, 3]);
        assert_eq!(DevPlaylist::delete_all(&mut conn, &[2, 3])?, 5);
        assert!(DevPlaylist::all(&mut conn)?.is_empty());
        Ok(())
    }

    #[test]
    fn invalid_tree_mutations_are_atomic_and_paths_report_ambiguity() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevPlaylist::folder("Root").insert(&mut conn)?;
        NewDevPlaylist::folder("Nested")
            .parent_id(1)
            .insert(&mut conn)?;
        NewDevPlaylist::playlist("List")
            .parent_id(1)
            .insert(&mut conn)?;
        NewDevPlaylist::playlist("List")
            .parent_id(1)
            .insert(&mut conn)?;
        assert!(DevPlaylist::find_by_path(&mut conn, &["Root", "List"])?.is_none());
        let before = bytes(&mut conn)?;
        assert!(DevPlaylist::move_to(&mut conn, &1, Some(&2), None).is_err());
        assert!(DevPlaylist::move_to(&mut conn, &2, Some(&3), None).is_err());
        assert!(DevPlaylist::move_to(&mut conn, &2, Some(&99), None).is_err());
        assert!(DevPlaylist::move_to(&mut conn, &2, None, Some(99)).is_err());
        assert!(DevPlaylist::move_all_to(&mut conn, &[2, 2], &0, None).is_err());
        assert!(DevPlaylist::delete_all(&mut conn, &[2, 99]).is_err());
        assert!(DevPlaylist::delete_all(&mut conn, &[2, 2]).is_err());
        assert!(NewDevPlaylist::playlist("Invalid")
            .parent_id(3)
            .insert(&mut conn)
            .is_err());
        assert!(NewDevPlaylist::playlist("Cycle")
            .parent_id(5)
            .insert(&mut conn)
            .is_err());
        assert_eq!(bytes(&mut conn)?, before);
        let mut conn =
            DeviceSqlConnection::from_reader(Cursor::new(before.clone()), DatabaseKind::Standard)?;
        assert_eq!(DevPlaylist::move_to(&mut conn, &2, None, Some(0))?, 0);
        assert_eq!(DevPlaylist::reset_seq(&mut conn, &1)?, 0);
        assert_eq!(conn.modified_page_count(), 0);
        assert_eq!(bytes(&mut conn)?, before);
        raw_node(
            &mut conn,
            9,
            NewDevPlaylist::folder("Cycle").parent_id(9).seq(0),
        )?;
        assert!(DevPlaylist::tree(&mut conn).is_err());
        assert!(DevPlaylist::find_by_path(&mut conn, &["Cycle"]).is_err());
        assert_eq!(DevPlaylist::delete(&mut conn, &9)?, 1);
        raw_node(
            &mut conn,
            9,
            NewDevPlaylist::playlist("Orphan").parent_id(99).seq(0),
        )?;
        assert!(DevPlaylist::tree(&mut conn).is_err());
        Ok(())
    }

    #[test]
    fn ordered_contents_queries_moves_and_bulk_deletion() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        for _ in [1, 2] {
            NewDevPlaylist::playlist("List").insert(&mut conn)?;
        }
        for id in 10..14 {
            seed_content(&mut conn, id, format!("/{id}.wav"))?;
        }
        for id in [10, 11, 12] {
            NewDevPlaylistContent::new(1, id, None).insert(&mut conn)?;
        }
        NewDevPlaylistContent::new(1, 13, 2).insert(&mut conn)?;
        NewDevPlaylistContent::new(2, 10, None).insert(&mut conn)?;
        assert_eq!(contents(&mut conn, 1)?, [10, 13, 11, 12]);
        assert_eq!(DevPlaylistContent::count(&mut conn, &1)?, 4);
        assert_eq!(
            DevPlaylistContent::get_seq_numbers(&mut conn, &1)?,
            [1, 2, 3, 4]
        );
        assert!(DevPlaylistContent::validate_seq_numbers(&mut conn, &1)?);
        assert!(DevPlaylistContent::id_exists(&mut conn, &(1, 13))?);
        assert!(!DevPlaylistContent::id_exists(&mut conn, &(1, 99))?);
        assert_eq!(DevPlaylistContent::by_content_id(&mut conn, &10)?.len(), 2);
        assert_eq!(
            DevPlaylist::get_contents(&mut conn, &1)?
                .iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            [10, 13, 11, 12]
        );
        assert_eq!(
            DevPlaylistContent::move_to(&mut conn, &(1, 12), Some(1))?,
            1
        );
        assert_eq!(contents(&mut conn, 1)?, [12, 10, 13, 11]);
        assert_eq!(
            DevPlaylistContent::move_all_to(&mut conn, &[(1, 11), (1, 13)], Some(2))?,
            1
        );
        assert_eq!(contents(&mut conn, 1)?, [12, 11, 13, 10]);
        assert_eq!(DevPlaylistContent::move_to(&mut conn, &(1, 12), None)?, 1);
        assert_eq!(contents(&mut conn, 1)?, [11, 13, 10, 12]);
        let before = bytes(&mut conn)?;
        assert!(DevPlaylistContent::move_to(&mut conn, &(1, 11), Some(0)).is_err());
        assert!(DevPlaylistContent::move_to(&mut conn, &(1, 11), Some(5)).is_err());
        assert!(DevPlaylistContent::move_all_to(&mut conn, &[(1, 11), (2, 10)], None).is_err());
        assert!(DevPlaylistContent::move_all_to(&mut conn, &[(1, 11), (1, 11)], None).is_err());
        assert!(DevPlaylistContent::delete_all(&mut conn, &[(1, 11), (1, 99)]).is_err());
        assert!(DevPlaylistContent::delete_all(&mut conn, &[(1, 11), (1, 11)]).is_err());
        assert!(NewDevPlaylistContent::new(1, 99, None)
            .insert(&mut conn)
            .is_err());
        assert!(NewDevPlaylistContent::new(99, 10, None)
            .insert(&mut conn)
            .is_err());
        assert_eq!(bytes(&mut conn)?, before);
        assert_eq!(
            DevPlaylistContent::delete_all(&mut conn, &[(1, 13), (2, 10)])?,
            2
        );
        assert_eq!(DevPlaylistContent::delete_by_id(&mut conn, &(1, 99))?, 0);
        assert_eq!(DevPlaylistContent::delete_by_id(&mut conn, &(1, 11))?, 1);
        assert_eq!(contents(&mut conn, 1)?, [10, 12]);
        assert_eq!(DevPlaylistContent::get_seq_numbers(&mut conn, &1)?, [1, 2]);
        for id in [10, 11] {
            NewDevPlaylistContent::new(2, id, None).insert(&mut conn)?;
        }
        assert_eq!(DevPlaylistContent::delete_by_content_id(&mut conn, &10)?, 2);
        assert_eq!(
            DevPlaylistContent::delete_by_content_ids(&mut conn, &[12, 999])?,
            1
        );
        assert_eq!(DevPlaylistContent::delete_by_playlist_id(&mut conn, &2)?, 1);
        assert_eq!(
            DevPlaylistContent::delete_by_playlist_ids(&mut conn, &[1, 2, 999])?,
            0
        );
        assert_eq!(DevPlaylist::all(&mut conn)?.len(), 2);
        assert_eq!(DevContent::all(&mut conn)?.len(), 4);
        let mut reopened = DeviceSqlConnection::from_reader(
            Cursor::new(bytes(&mut conn)?),
            DatabaseKind::Standard,
        )?;
        assert!(DevPlaylistContent::all(&mut reopened)?.is_empty());
        Ok(())
    }

    #[test]
    fn imported_positions_can_be_reset_without_losing_duplicate_occurrences() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        raw_node(&mut conn, 1, NewDevPlaylist::playlist("Later").seq(9))?;
        raw_node(&mut conn, 2, NewDevPlaylist::playlist("Earlier").seq(4))?;
        raw_node(&mut conn, 3, NewDevPlaylist::playlist("Tied").seq(4))?;
        assert!(!DevPlaylist::validate_seq_numbers(&mut conn, &0)?);
        assert_eq!(DevPlaylist::reset_seq(&mut conn, &0)?, 3);
        assert_eq!(children(&mut conn, 0)?, [2, 3, 1]);
        assert!(DevPlaylist::validate_seq_numbers(&mut conn, &0)?);
        for (id, seq) in [(10, 9), (10, 0), (20, 0)] {
            raw_entry(&mut conn, NewDevPlaylistContent::new(1, id, seq))?;
        }
        assert!(!DevPlaylistContent::validate_seq_numbers(&mut conn, &1)?);
        assert_eq!(DevPlaylistContent::reset_seq(&mut conn, &1)?, 3);
        assert_eq!(contents(&mut conn, 1)?, [10, 20, 10]);
        assert!(DevPlaylistContent::validate_seq_numbers(&mut conn, &1)?);
        assert!(matches!(
            DevPlaylistContent::find(&mut conn, &(1, 10)),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert!(matches!(
            DevPlaylistContent::move_to(&mut conn, &(1, 20), Some(1)),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert_eq!(DevPlaylistContent::remove(&mut conn, &10, &1)?, 2);
        assert_eq!(DevPlaylistContent::get_seq_numbers(&mut conn, &1)?, [1]);
        Ok(())
    }

    #[test]
    fn multi_page_moves_rollback_on_resource_limits_and_preserve_other_tables() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevPlaylist::playlist("List").insert(&mut conn)?;
        NewDevContent::new("/keep.wav").insert(&mut conn)?;
        let content = DevContent::find(&mut conn, &1)?.unwrap();
        let mut tx = conn.transaction();
        // Binary setup isolates ordering from content insertion and allocation costs.
        for id in 1..=600 {
            tx.insert(
                DevPlaylistContent::TABLE,
                &Row::Standard(StandardRow::PlaylistEntry(playlist_entry_row(
                    &NewDevPlaylistContent::new(1, id, u32::try_from(id).unwrap()),
                ))),
            )?;
        }
        tx.commit()?;
        let original = bytes(&mut conn)?;
        let mut conn = DeviceSqlConnection::from_reader(
            Cursor::new(original.clone()),
            DatabaseKind::Standard,
        )?;
        assert_eq!(DevPlaylistContent::move_to(&mut conn, &(1, 1), Some(1))?, 0);
        assert_eq!(DevPlaylistContent::reset_seq(&mut conn, &1)?, 0);
        assert_eq!(conn.modified_page_count(), 0);
        assert_eq!(bytes(&mut conn)?, original);
        conn.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(1),
            ..Default::default()
        })?;
        assert!(matches!(
            DevPlaylistContent::move_to(&mut conn, &(1, 600), Some(1)),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(conn.modified_page_count(), 0);
        assert_eq!(bytes(&mut conn)?, original);
        conn.set_resource_limits(Default::default())?;
        assert_eq!(
            DevPlaylistContent::move_to(&mut conn, &(1, 600), Some(1))?,
            1
        );
        assert!(conn.modified_page_count() > 1);
        let saved = bytes(&mut conn)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(saved.clone()), DatabaseKind::Standard)?;
        assert_eq!(
            contents(&mut reopened, 1)?,
            std::iter::once(600).chain(1..600).collect::<Vec<_>>()
        );
        assert!(DevPlaylistContent::validate_seq_numbers(&mut reopened, &1)?);
        assert_eq!(
            DevContent::find(&mut reopened, &1)?
                .unwrap()
                .raw_record()
                .decode()?,
            content.raw_record().decode()?
        );
        let entry_pages: std::collections::HashSet<_> = DevPlaylistContent::all(&mut reopened)?
            .iter()
            .map(|r| r.raw_record().page())
            .collect();
        assert!(entry_pages.len() > 1);
        // Sequence edits must not rewrite unrelated content pages.
        let content_page = content.raw_record().page().0 as usize;
        let page_size = reopened.header().prefix.page_size as usize;
        assert_eq!(
            &saved[content_page * page_size..(content_page + 1) * page_size],
            &original[content_page * page_size..(content_page + 1) * page_size]
        );
        Ok(())
    }

    #[test]
    fn ordering_helpers_reject_the_extended_database() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Extended)?;
        assert!(matches!(
            DevPlaylist::tree(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylist::get_attribute(&mut conn, &0),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylist::reset_seq(&mut conn, &0),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylist::move_all_to(&mut conn, &[], &0, None),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylist::delete_all(&mut conn, &[]),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylistContent::move_all_to(&mut conn, &[], None),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylistContent::delete_all(&mut conn, &[]),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylistContent::delete_by_content_ids(&mut conn, &[]),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert_eq!(conn.modified_page_count(), 0);
        Ok(())
    }

    fn raw_node(
        conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>,
        id: u32,
        input: NewDevPlaylist,
    ) -> Result<DevPlaylist> {
        let mut tx = conn.transaction();
        let handle = tx.insert(
            DevPlaylist::TABLE,
            &Row::Standard(StandardRow::PlaylistTree(playlist_row(id, &input)?)),
        )?;
        let row = DevPlaylist::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(row)
    }

    fn raw_entry(
        conn: &mut DeviceSqlConnection<Cursor<Vec<u8>>>,
        input: NewDevPlaylistContent,
    ) -> Result<DevPlaylistContent> {
        let mut tx = conn.transaction();
        let handle = tx.insert(
            DevPlaylistContent::TABLE,
            &Row::Standard(StandardRow::PlaylistEntry(playlist_entry_row(&input))),
        )?;
        let row = DevPlaylistContent::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(row)
    }

    #[test]
    fn content_pairs_are_unique_regardless_of_sequence() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevPlaylist::playlist("First").insert(&mut conn)?;
        NewDevPlaylist::playlist("Second").insert(&mut conn)?;
        seed_content(&mut conn, 10, "/10.wav")?;
        seed_content(&mut conn, 20, "/20.wav")?;
        let first = NewDevPlaylistContent::new(1, 10, 1).insert(&mut conn)?;
        NewDevPlaylistContent::new(2, 10, 1).insert(&mut conn)?;
        NewDevPlaylistContent::new(1, 20, 2).insert(&mut conn)?;
        let mut before = Vec::new();
        conn.write_to(&mut before)?;
        for seq in [1, 99] {
            assert!(matches!(
                NewDevPlaylistContent::new(1, 10, seq).insert(&mut conn),
                Err(Error::UniqueConstraintViolation { table, fields })
                    if table == DevPlaylistContent::TABLE
                        && fields == &["playlist_id", "content_id"]
            ));
        }
        let mut after = Vec::new();
        conn.write_to(&mut after)?;
        assert_eq!(after, before);
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(before), DatabaseKind::Standard)?;
        assert!(matches!(
            NewDevPlaylistContent::new(1, 10, 2).insert(&mut reopened),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert_eq!(first.delete(&mut conn)?, 1);
        NewDevPlaylistContent::new(1, 10, 2).insert(&mut conn)?;
        assert_eq!(DevPlaylistContent::all(&mut conn)?.len(), 3);
        Ok(())
    }

    #[test]
    fn folders_and_playlists_keep_explicit_positions_flags_and_reserved_values() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let before = conn.read_stats();
        drop(DevPlaylist::iter(&mut conn)?);
        drop(DevPlaylistContent::iter(&mut conn)?);
        assert_eq!(conn.read_stats(), before);
        assert!(DevPlaylist::all(&mut conn)?.is_empty());
        assert!(DevPlaylist::find(&mut conn, &1)?.is_none());
        assert!(DevPlaylist::by_parent_id(&mut conn, &0)?.is_empty());
        assert!(DevPlaylistContent::all(&mut conn)?.is_empty());

        let mut folder = NewDevPlaylist::folder("Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ").seq(50);
        folder.unknown_04 = 0xaabbccdd;
        let folder = raw_node(&mut conn, 1, folder)?;
        assert!(folder.is_folder());
        assert_eq!(folder.attribute, PlaylistType::Folder);
        assert_eq!(folder.name, "Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ");
        let Row::Standard(StandardRow::PlaylistTree(row)) = folder.raw_record().decode()? else {
            panic!()
        };
        assert_eq!(row.unknown_04, 0xaabbccdd);
        let later = raw_node(
            &mut conn,
            2,
            NewDevPlaylist::playlist("Later").parent_id(1).seq(20),
        )?;
        raw_node(
            &mut conn,
            3,
            NewDevPlaylist::playlist("Earlier").parent_id(1).seq(4),
        )?;
        raw_node(
            &mut conn,
            4,
            NewDevPlaylist::playlist("Tied").parent_id(1).seq(4),
        )?;
        raw_node(
            &mut conn,
            5,
            NewDevPlaylist::playlist("Root playlist").seq(1),
        )?;
        assert!(!later.is_folder());
        assert_eq!(
            DevPlaylist::by_parent_id(&mut conn, &1)?
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            vec![3, 4, 2]
        );
        assert_eq!(
            DevPlaylist::by_parent_id(&mut conn, &0)?
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            vec![5, 1]
        );
        assert!(DevPlaylist::by_parent_id(&mut conn, &999)?.is_empty());
        assert_eq!(
            DevPlaylist::all(&mut conn)?
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        let current_folder = DevPlaylist::find(&mut conn, &1)?.unwrap();
        assert_eq!(
            current_folder.raw_record().as_bytes(),
            folder.raw_record().as_bytes()
        );
        let reads = conn.read_stats();
        assert!(DevPlaylist::id_exists(&mut conn, &1)?);
        assert!(!DevPlaylist::id_exists(&mut conn, &999)?);
        assert_eq!(conn.read_stats(), reads);
        let mut detached = folder;
        detached.name = "Local edit".into();
        assert_eq!(
            DevPlaylist::find(&mut conn, &1)?.unwrap().name,
            "Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ"
        );
        Ok(())
    }

    #[test]
    fn repeated_occurrences_and_position_ties_survive_save_and_reopen() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevPlaylist::playlist("Playlist")
            .seq(0)
            .insert(&mut conn)?;
        NewDevPlaylist::playlist("Other").seq(1).insert(&mut conn)?;
        seed_content(&mut conn, 10, "/Contents/One.wav")?;
        seed_content(&mut conn, 20, "/Contents/Two.wav")?;
        let late = raw_entry(&mut conn, NewDevPlaylistContent::new(1, 10, 30))?;
        let first_tie = raw_entry(&mut conn, NewDevPlaylistContent::new(1, 20, 4))?;
        // Bypass the model constraint to test lossless reads of existing duplicates.
        let mut tx = conn.transaction();
        let mut duplicates = Vec::new();
        for seq in [4, 4, 0] {
            let handle = tx.insert(
                DevPlaylistContent::TABLE,
                &Row::Standard(StandardRow::PlaylistEntry(playlist_entry_row(
                    &NewDevPlaylistContent::new(1, 10, seq),
                ))),
            )?;
            duplicates.push(DevPlaylistContent::try_from(tx.read(&handle)?)?);
        }
        tx.commit()?;
        let [second_tie, identical, zero]: [DevPlaylistContent; 3] = duplicates.try_into().unwrap();
        let other = NewDevPlaylistContent::new(2, 10, 1).insert(&mut conn)?;
        // Later transactions can change touched flags on earlier snapshots.
        // Compare values and occurrence locations, not transient page metadata.
        let occurrence = |entry: &DevPlaylistContent| {
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
            DevPlaylistContent::by_playlist_id(&mut conn, &1)?
                .iter()
                .map(occurrence)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            DevPlaylistContent::by_playlist_id(&mut conn, &2)?
                .iter()
                .map(occurrence)
                .collect::<Vec<_>>(),
            vec![occurrence(&other)]
        );
        assert!(DevPlaylistContent::by_playlist_id(&mut conn, &999)?.is_empty());
        assert_eq!(
            DevPlaylistContent::all(&mut conn)?
                .iter()
                .map(occurrence)
                .collect::<Vec<_>>(),
            [&late, &first_tie, &second_tie, &identical, &zero, &other]
                .into_iter()
                .map(occurrence)
                .collect::<Vec<_>>()
        );
        assert_eq!(DevContent::all(&mut conn)?.len(), 2);
        let mut bytes = Vec::new();
        conn.write_to(&mut bytes)?;
        let mut reopened =
            DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
        assert_eq!(
            DevPlaylistContent::by_playlist_id(&mut reopened, &1)?
                .iter()
                .map(occurrence)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(DevPlaylist::all(&mut reopened)?.len(), 2);
        Ok(())
    }

    #[test]
    fn rejected_playlist_insertions_leave_existing_pages_unchanged() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevPlaylist::playlist("Original")
            .seq(0)
            .insert(&mut conn)?;
        let mut before = Vec::new();
        conn.write_to(&mut before)?;
        let pages = conn.page_count();
        let dirty = conn.modified_page_count();
        assert!(matches!(
            NewDevPlaylist::playlist("Bad\0name")
                .seq(1)
                .insert(&mut conn),
            Err(Error::EmbeddedNul)
        ));
        assert!(matches!(
            NewDevPlaylist::playlist("x".repeat(5000))
                .seq(1)
                .insert(&mut conn),
            Err(Error::NoRowSpace { .. })
        ));
        assert!(matches!(
            NewDevPlaylistContent::new(0, 1, 1).insert(&mut conn),
            Err(Error::InvalidModelField {
                field: "playlist_id",
                ..
            })
        ));
        assert!(matches!(
            NewDevPlaylistContent::new(1, 0, 1).insert(&mut conn),
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
        assert!(DevPlaylistContent::all(&mut conn)?.is_empty());

        let mut limited = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        NewDevPlaylist::playlist("Existing").insert(&mut limited)?;
        NewDevContent::new("/test.wav").insert(&mut limited)?;
        let mut saved = Vec::new();
        limited.write_to(&mut saved)?;
        let mut limited =
            DeviceSqlConnection::from_reader(Cursor::new(saved), DatabaseKind::Standard)?;
        limited.set_resource_limits(rbox_devlib::devicesql::ResourceLimits {
            dirty_pages: Some(0),
            ..Default::default()
        })?;
        assert!(matches!(
            NewDevPlaylist::playlist("Limited")
                .seq(0)
                .insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert!(matches!(
            NewDevPlaylistContent::new(1, 1, 1).insert(&mut limited),
            Err(Error::DirtyPageLimit { .. })
        ));
        assert_eq!(limited.modified_page_count(), 0);
        assert_eq!(DevPlaylist::all(&mut limited)?.len(), 1);
        assert!(DevPlaylistContent::all(&mut limited)?.is_empty());
        Ok(())
    }

    #[test]
    fn native_playlist_models_preserve_raw_rows() -> Result<()> {
        let mut conn = demo()?;
        let nodes = conn
            .iter_raw(DevPlaylist::TABLE)?
            .collect::<Result<Vec<_>>>()?;
        let entries = conn
            .iter_raw(DevPlaylistContent::TABLE)?
            .collect::<Result<Vec<_>>>()?;
        assert!(!nodes.is_empty());
        assert!(!entries.is_empty());
        let models = DevPlaylist::all(&mut conn)?;
        assert_eq!(models.len(), nodes.len());
        for (model, record) in models.into_iter().zip(nodes) {
            let Row::Standard(StandardRow::PlaylistTree(binary)) = record.decode()? else {
                panic!()
            };
            assert_eq!(
                (model.id, model.parent_id, model.seq, model.attribute as u32),
                (
                    i64::from(binary.id),
                    i64::from(binary.parent_id),
                    binary.position,
                    binary.folder_flag
                )
            );
            assert_eq!(model.name, binary.name.text()?);
            assert_eq!(model.raw_record(), &record);
            assert_eq!(DevPlaylist::find(&mut conn, &model.id)?, Some(model));
        }
        let models = DevPlaylistContent::all(&mut conn)?;
        assert_eq!(models.len(), entries.len());
        for (model, record) in models.into_iter().zip(entries) {
            let Row::Standard(StandardRow::PlaylistEntry(binary)) = record.decode()? else {
                panic!()
            };
            assert_eq!(
                (model.playlist_id, model.content_id, model.seq),
                (
                    i64::from(binary.playlist_id),
                    i64::from(binary.track_id),
                    binary.position
                )
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
            DevPlaylist::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylist::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylist::find(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylist::id_exists(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylist::by_parent_id(&mut conn, &0),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylistContent::iter(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylistContent::all(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            DevPlaylistContent::by_playlist_id(&mut conn, &1),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewDevPlaylist::playlist("Wrong namespace")
                .seq(1)
                .insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert!(matches!(
            NewDevPlaylistContent::new(1, 1, 1).insert(&mut conn),
            Err(Error::WrongDatabaseKind { .. })
        ));
        assert_eq!(conn.modified_page_count(), 0);
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let node = NewDevPlaylist::playlist("Node").seq(0).insert(&mut conn)?;
        NewDevContent::new("/test.wav").insert(&mut conn)?;
        let entry = NewDevPlaylistContent::new(1, 1, 1).insert(&mut conn)?;
        assert!(matches!(
            DevPlaylist::try_from(entry.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        assert!(matches!(
            DevPlaylistContent::try_from(node.raw_record().clone()),
            Err(Error::UnsupportedModelRow { .. })
        ));
        Ok(())
    }

    #[test]
    fn binary_transactions_remain_explicit_and_bad_name_text_is_contextual() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut row = playlist_row(1, &NewDevPlaylist::playlist("Name").seq(1))?;
        row.name = DeviceSqlString::parse(&[0x90, 6, 0, 0, 0, 0xd8])?;
        let mut tx = conn.transaction();
        tx.insert(
            DevPlaylist::TABLE,
            &Row::Standard(StandardRow::PlaylistTree(row)),
        )?;
        tx.commit()?;
        assert!(DevPlaylist::id_exists(&mut conn, &1)?);
        assert!(
            matches!(DevPlaylist::find(&mut conn, &1), Err(Error::ModelText { field: "name", source, .. }) if matches!(*source, Error::InvalidUtf16))
        );
        let mut tx = conn.transaction();
        let handle = tx.insert(
            DevPlaylistContent::TABLE,
            &Row::Standard(StandardRow::PlaylistEntry(playlist_entry_row(
                &NewDevPlaylistContent::new(1, 10, 7),
            ))),
        )?;
        let staged = DevPlaylistContent::try_from(tx.read(&handle)?)?;
        assert_eq!(
            (staged.playlist_id, staged.content_id, staged.seq),
            (1, 10, 7)
        );
        tx.rollback();
        assert!(DevPlaylistContent::all(&mut conn)?.is_empty());
        Ok(())
    }
}

mod catalog {
    //! Shared behavioral checks for the seven catalog model pairs.

    use super::*;
    use rbox_devlib::binary::{AlbumHeader, ArtistHeader, StringOffsets, TableKind};
    use rbox_devlib::models::{
        DevAlbum, DevArtist, DevArtwork, DevColor, DevGenre, DevKey, DevLabel, NewDevAlbum,
        NewDevArtist, NewDevArtwork, NewDevGenre, NewDevKey, NewDevLabel,
    };

    struct Case {
        table: TableKind,
        insert: fn(&mut MemoryConnection, &str) -> Result<RowRecord>,
        find: fn(&mut MemoryConnection, i64) -> Result<Option<RowRecord>>,
        all: fn(&mut MemoryConnection) -> Result<Vec<RowRecord>>,
        exists: fn(&mut MemoryConnection, i64) -> Result<bool>,
        lazy: fn(&mut MemoryConnection) -> Result<()>,
        decode: fn(RowRecord) -> Result<String>,
    }

    fn cases() -> Vec<Case> {
        vec![
            Case {
                table: DevArtist::TABLE,
                insert: |conn, value| {
                    Ok(NewDevArtist::new(value).insert(conn)?.raw_record().clone())
                },
                find: |conn, id| {
                    Ok(DevArtist::find(conn, &id)?.map(|model| model.raw_record().clone()))
                },
                all: |conn| {
                    Ok(DevArtist::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| DevArtist::id_exists(conn, &id),
                lazy: |conn| {
                    drop(DevArtist::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(DevArtist::try_from(record)?.name),
            },
            Case {
                table: DevAlbum::TABLE,
                insert: |conn, value| {
                    Ok(NewDevAlbum::new(value)
                        .artist_id(73)
                        .insert(conn)?
                        .raw_record()
                        .clone())
                },
                find: |conn, id| {
                    Ok(DevAlbum::find(conn, &id)?.map(|model| model.raw_record().clone()))
                },
                all: |conn| {
                    Ok(DevAlbum::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| DevAlbum::id_exists(conn, &id),
                lazy: |conn| {
                    drop(DevAlbum::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(DevAlbum::try_from(record)?.name),
            },
            Case {
                table: DevGenre::TABLE,
                insert: |conn, value| {
                    Ok(NewDevGenre::new(value).insert(conn)?.raw_record().clone())
                },
                find: |conn, id| {
                    Ok(DevGenre::find(conn, &id)?.map(|model| model.raw_record().clone()))
                },
                all: |conn| {
                    Ok(DevGenre::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| DevGenre::id_exists(conn, &id),
                lazy: |conn| {
                    drop(DevGenre::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(DevGenre::try_from(record)?.name),
            },
            Case {
                table: DevLabel::TABLE,
                insert: |conn, value| {
                    Ok(NewDevLabel::new(value).insert(conn)?.raw_record().clone())
                },
                find: |conn, id| {
                    Ok(DevLabel::find(conn, &id)?.map(|model| model.raw_record().clone()))
                },
                all: |conn| {
                    Ok(DevLabel::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| DevLabel::id_exists(conn, &id),
                lazy: |conn| {
                    drop(DevLabel::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(DevLabel::try_from(record)?.name),
            },
            Case {
                table: DevKey::TABLE,
                insert: |conn, value| {
                    Ok(NewDevKey::new(value, 91).insert(conn)?.raw_record().clone())
                },
                find: |conn, id| {
                    Ok(DevKey::find(conn, &id)?.map(|model| model.raw_record().clone()))
                },
                all: |conn| {
                    Ok(DevKey::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| DevKey::id_exists(conn, &id),
                lazy: |conn| {
                    drop(DevKey::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(DevKey::try_from(record)?.name),
            },
            Case {
                table: DevArtwork::TABLE,
                insert: |conn, value| {
                    Ok(NewDevArtwork::new(value).insert(conn)?.raw_record().clone())
                },
                find: |conn, id| {
                    Ok(DevArtwork::find(conn, &id)?.map(|model| model.raw_record().clone()))
                },
                all: |conn| {
                    Ok(DevArtwork::all(conn)?
                        .into_iter()
                        .map(|model| model.raw_record().clone())
                        .collect())
                },
                exists: |conn, id| DevArtwork::id_exists(conn, &id),
                lazy: |conn| {
                    drop(DevArtwork::iter(conn)?);
                    Ok(())
                },
                decode: |record| Ok(DevArtwork::try_from(record)?.path),
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
            assert!((case.find)(&mut conn, 1)?.is_none());

            let first = (case.insert)(&mut conn, &value)?;
            assert_eq!(first.table(), case.table);
            assert_eq!((case.decode)(first.clone())?, value);
            assert_eq!((case.find)(&mut conn, 1)?, Some(first));
            let before = conn.read_stats();
            assert!((case.exists)(&mut conn, 1)?);
            assert!(!(case.exists)(&mut conn, 999)?);
            assert_eq!(conn.read_stats(), before);

            (case.insert)(&mut conn, "Second")?;
            assert_eq!((case.all)(&mut conn)?.len(), 2);
            assert_eq!(
                (case.decode)((case.find)(&mut conn, 2)?.unwrap())?,
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
                (case.decode)((case.find)(&mut reopened, 1)?.unwrap())?,
                value
            );
        }
        Ok(())
    }

    #[test]
    fn duplicate_invalid_and_oversized_insertions_are_atomic() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        for case in cases() {
            (case.insert)(&mut conn, "Existing")?;
            let mut before = Vec::new();
            conn.write_to(&mut before)?;
            let pages = conn.page_count();
            let dirty = conn.modified_page_count();
            if case.table != DevArtwork::TABLE {
                assert!(matches!(
                    (case.insert)(&mut conn, "Existing"),
                    Err(Error::UniqueConstraintViolation { .. })
                ));
            }
            assert!(matches!(
                (case.insert)(&mut conn, "Bad\0text"),
                Err(Error::EmbeddedNul)
            ));
            assert!(matches!(
                (case.insert)(&mut conn, &"x".repeat(5000)),
                Err(Error::NoRowSpace { .. })
            ));
            assert_eq!(
                (conn.page_count(), conn.modified_page_count()),
                (pages, dirty)
            );
            let mut after = Vec::new();
            conn.write_to(&mut after)?;
            assert_eq!(after, before);
            let next_id = 2;
            assert!(!(case.exists)(&mut conn, next_id)?);
        }
        assert!(matches!(
            NewDevArtwork::new("").insert(&mut conn),
            Err(Error::InvalidModelField { field: "path", .. })
        ));
        Ok(())
    }

    #[test]
    fn all_catalog_models_reject_extended_connections_and_other_tables() -> Result<()> {
        let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let track = NewDevContent::new("/Contents/One.wav").insert(&mut standard)?;
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
                (case.insert)(&mut extended, "Name"),
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
                assert_eq!((case.find)(&mut conn, i64::from(id))?, Some(record));
            }
        }
        assert_eq!(DevArtist::all(&mut conn)?.len(), 1);
        assert_eq!(DevLabel::all(&mut conn)?.len(), 1);
        assert_eq!(DevColor::all(&mut conn)?.len(), 8);
        assert_eq!(conn.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn independent_ids_reserved_values_and_far_layouts_survive() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut artist = artist_row(73, "Artist")?;
        artist.header.subtype = ArtistHeader::FAR_SUBTYPE;
        artist.strings = StringOffsets::Far {
            marker: 5,
            offsets: [0],
        };
        let mut album = album_row(9, 73, "Album")?;
        album.header.unknown_04 = 0xaabbccdd;
        album.header.unknown_10 = 0x11223344;
        album.header.subtype = AlbumHeader::FAR_SUBTYPE;
        album.strings = StringOffsets::Far {
            marker: 7,
            offsets: [0],
        };
        let mut tx = conn.transaction();
        tx.insert(
            DevArtist::TABLE,
            &Row::Standard(StandardRow::Artist(artist)),
        )?;
        tx.insert(DevAlbum::TABLE, &Row::Standard(StandardRow::Album(album)))?;
        tx.commit()?;
        let artist = DevArtist::find(&mut conn, &73)?.unwrap();
        let mut album = DevAlbum::find(&mut conn, &9)?.unwrap();
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
        assert_eq!(DevAlbum::find(&mut conn, &9)?.unwrap().name, "Album");

        let key = NewDevKey::new("Am", 91).insert(&mut conn)?;
        assert_eq!((key.id, key.secondary_id), (1, 91));
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
        tx.insert(DevColor::TABLE, &Row::Standard(StandardRow::Color(color)))?;
        tx.commit()?;
        let color = DevColor::find(&mut conn, &0x1234)?.unwrap();
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
            let original = (case.insert)(&mut source, "Good text")?;
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
            assert!((case.exists)(&mut conn, 1)?);
            assert!(
                matches!((case.find)(&mut conn, 1), Err(Error::ModelText { table, source, field, .. })
                if table == case.table && matches!(*source, Error::InvalidUtf16)
                && field == if table == DevArtwork::TABLE { "path" } else { "name" })
            );
        }
        Ok(())
    }

    #[test]
    fn tracks_can_reference_all_fresh_catalog_models() -> Result<()> {
        let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let artist = NewDevArtist::new("Artist").insert(&mut conn)?;
        let album = NewDevAlbum::new("Album")
            .artist_id(artist.id)
            .insert(&mut conn)?;
        let genre = NewDevGenre::new("House").insert(&mut conn)?;
        let label = NewDevLabel::new("Label").insert(&mut conn)?;
        let key = NewDevKey::new("Am", 23).insert(&mut conn)?;
        let color = seed_color(&mut conn, 6, "Blue")?;
        let artwork = NewDevArtwork::new("/PIONEER/Artwork/00001/a7.jpg").insert(&mut conn)?;
        let mut track = NewDevContent::new("/Contents/Track.flac");
        track.artist_id = artist.id;
        track.album_id = album.id;
        track.genre_id = genre.id;
        track.label_id = label.id;
        track.key_id = key.id;
        track.color_id = color.id;
        track.artwork_id = artwork.id;
        let model = track.insert(&mut conn)?;
        assert_eq!(DevArtist::find(&mut conn, &model.artist_id)?, Some(artist));
        assert_eq!(DevAlbum::find(&mut conn, &model.album_id)?, Some(album));
        assert_eq!(DevGenre::find(&mut conn, &model.genre_id)?, Some(genre));
        assert_eq!(DevLabel::find(&mut conn, &model.label_id)?, Some(label));
        assert_eq!(DevKey::find(&mut conn, &model.key_id)?, Some(key));
        assert_eq!(DevColor::find(&mut conn, &model.color_id)?, Some(color));
        assert_eq!(
            DevArtwork::find(&mut conn, &model.artwork_id)?,
            Some(artwork)
        );
        Ok(())
    }
}

type MemoryConnection = DeviceSqlConnection<Cursor<Vec<u8>>>;

fn demo() -> Result<MemoryConnection> {
    let bytes = fs::read(export_dir(7).join("rekordbox/export.pdb"))?;
    DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)
}

fn first_record(conn: &mut MemoryConnection) -> Result<RowRecord> {
    conn.iter_raw(DevContent::TABLE)?
        .next()
        .expect("demo track")
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
    drop(DevContent::iter(&mut conn)?);
    assert_eq!(conn.read_stats(), before);
    let records = conn
        .iter_raw(DevContent::TABLE)?
        .collect::<Result<Vec<_>>>()?;
    let tracks = DevContent::all(&mut conn)?;
    assert_eq!(tracks.len(), 2);
    for (mut model, record) in tracks.into_iter().zip(records) {
        let row = track_row(&record)?;
        assert_eq!(model.id, i64::from(row.header.id));
        assert_eq!(model.title, row.text.title.text()?);
        assert_eq!(model.path, row.text.file_path.text()?);
        assert_eq!(model.analysis_path, row.text.analysis_path.text()?);
        assert_eq!(model.bpmx100, i64::from(row.header.tempo));
        assert_eq!(model.length, i32::from(row.header.duration));
        assert_eq!(model.raw_record(), &record);
        assert_eq!(model.isrc()?, row.text.isrc.isrc_text()?);
        model.title = "detached edit".into();
        assert_eq!(model.raw_record(), &record);
        assert_ne!(
            DevContent::find(&mut conn, &model.id)?.unwrap().title,
            model.title
        );
    }
    assert_eq!(conn.modified_page_count(), 0);
    assert!(DevContent::find(&mut conn, &i64::from(u32::MAX))?.is_none());
    assert!(!DevContent::id_exists(&mut conn, &i64::from(u32::MAX))?);
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
    tx.insert(DevContent::TABLE, &Row::Standard(StandardRow::Track(row)))?;
    tx.commit()?;
    let model = DevContent::find(&mut fresh, &42)?.unwrap();
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
    assert!(!DevContent::id_exists(&mut fresh, &42)?);
    let mut tx = fresh.transaction();
    tx.insert(DevContent::TABLE, &Row::Standard(StandardRow::Track(row)))?;
    tx.commit()?;
    let model = DevContent::find(&mut fresh, &42)?.unwrap();
    assert_eq!(fresh.resource_usage().indexed_rows, 1);
    let before = fresh.read_stats();
    assert!(DevContent::id_exists(&mut fresh, &42)?);
    assert_eq!(fresh.read_stats(), before);

    let mut changed = track_row(model.raw_record())?;
    changed.header.id = 84;
    let mut tx = fresh.transaction();
    let handle = tx.locate(
        DevContent::TABLE,
        model.raw_record().page(),
        model.raw_record().slot().slot,
    )?;
    tx.update(&handle, &Row::Standard(StandardRow::Track(changed)))?;
    tx.commit()?;
    assert!(!DevContent::id_exists(&mut fresh, &42)?);
    let updated = DevContent::find(&mut fresh, &84)?.unwrap();
    let mut tx = fresh.transaction();
    let handle = tx.locate(
        DevContent::TABLE,
        updated.raw_record().page(),
        updated.raw_record().slot().slot,
    )?;
    tx.delete(&handle)?;
    tx.commit()?;
    assert!(!DevContent::id_exists(&mut fresh, &84)?);
    assert!(DevContent::all(&mut fresh)?.is_empty());
    Ok(())
}

#[test]
fn empty_standard_and_extended_namespaces_are_distinct() -> Result<()> {
    let mut standard = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    assert!(DevContent::all(&mut standard)?.is_empty());
    assert!(DevContent::find(&mut standard, &1)?.is_none());
    let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
    assert!(matches!(
        DevContent::iter(&mut extended),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        DevContent::find(&mut extended, &1),
        Err(Error::WrongDatabaseKind { .. })
    ));
    assert!(matches!(
        DevContent::id_exists(&mut extended, &1),
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
    tx.insert(DevContent::TABLE, &row)?;
    tx.insert(DevContent::TABLE, &row)?;
    tx.commit()?;
    for _ in 0..2 {
        assert!(matches!(
            DevContent::find(&mut fresh, &1),
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
    assert!(DevContent::id_exists(
        &mut corrupt,
        &i64::from(row.header.id)
    )?);
    match DevContent::find(&mut corrupt, &i64::from(row.header.id)) {
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
        DevContent::all(&mut unknown),
        Err(Error::UnsupportedModelRow { .. })
    ));
    assert!(matches!(
        DevContent::find(&mut unknown, &1),
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
        let mut input = NewDevContent::new(format!("/Contents/Artist/Track {id}.flac"));
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
        assert_eq!(DevContent::find(&mut conn, &id)?, Some(inserted));
    }
    assert!(conn.page_count() > original_pages);
    assert_eq!(property_counts(&mut conn)?, vec![24]);
    let mut bytes = Vec::new();
    conn.write_to(&mut bytes)?;
    let mut reopened =
        DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Standard)?;
    assert_eq!(DevContent::all(&mut reopened)?.len(), 24);
    assert_eq!(property_counts(&mut reopened)?, vec![24]);
    assert_eq!(
        DevContent::find(&mut reopened, &24)?.unwrap().title,
        "Ã©Å¸Â³Ã¦Â¥Â½ Ã°Å¸Å½Âµ 24"
    );
    Ok(())
}

#[test]
fn fresh_track_policy_is_explicit_and_overridable() -> Result<()> {
    let mut input = NewDevContent::new("/Contents/Test.wav");
    let row = content_row(1, &input)?;
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
    input.creation_mut().content_link = 0x700;
    input.creation_mut().raw_analysis = 0x1234;
    input.creation_mut().string_marker = 5;
    input.creation_mut().unknown_strings[0] = "opaque".into();
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
    NewDevContent::new("/Contents/Existing.wav").insert(&mut conn)?;
    let mut before = Vec::new();
    conn.write_to(&mut before)?;
    let pages = conn.page_count();
    let dirty = conn.modified_page_count();
    assert!(matches!(
        NewDevContent::new("").insert(&mut conn),
        Err(Error::InvalidModelField { field: "path", .. })
    ));
    let mut invalid = NewDevContent::new("/Contents/Invalid.wav");
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
    assert_eq!(DevContent::all(&mut conn)?.len(), 1);

    let mut extended = DeviceSqlConnection::create(DatabaseKind::Extended)?;
    assert!(matches!(
        NewDevContent::new("/Contents/Invalid.wav").insert(&mut extended),
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
        NewDevContent::new("/Contents/Track.wav").insert(&mut conn),
        Err(Error::UnsupportedModelRow { .. })
    ));
    assert!(DevContent::all(&mut conn)?.is_empty());
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
    NewDevContent::new("/Contents/One.wav").insert(&mut fresh)?;
    fresh.save_as(&path)?;
    let mut reopened = DeviceSqlConnection::open(&path, DatabaseKind::Standard)?;
    assert!(DevContent::id_exists(&mut reopened, &1)?);
    NewDevContent::new("/Contents/Two.wav").insert(&mut reopened)?;
    reopened.save_pages()?;
    drop(reopened);
    let mut reopened = DeviceSqlConnection::open(&path, DatabaseKind::Standard)?;
    assert_eq!(DevContent::all(&mut reopened)?.len(), 2);
    assert_eq!(
        DevContent::find(&mut reopened, &2)?.unwrap().file_name,
        "Two.wav"
    );
    Ok(())
}

#[test]
fn binary_track_rows_support_explicit_transactions() -> Result<()> {
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let input = NewDevContent::new("/Contents/Track.wav");
    let mut tx = conn.transaction();
    let handle = tx.insert(
        DevContent::TABLE,
        &Row::Standard(StandardRow::Track(content_row(1, &input)?)),
    )?;
    assert_eq!(DevContent::try_from(tx.read(&handle)?)?.id, 1);
    tx.rollback();
    assert!(DevContent::all(&mut conn)?.is_empty());
    Ok(())
}

#[test]
fn shared_track_conversion_resolves_names_and_preserves_metadata() -> Result<()> {
    use rbox_core::{Track, TrackSource};
    use rbox_devlib::models::{NewDevAlbum, NewDevArtist, NewDevArtwork};
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let artist = NewDevArtist::new("Artist").insert(&mut conn)?;
    let album_artist = NewDevArtist::new("Album artist").insert(&mut conn)?;
    let album = NewDevAlbum::new("Album").artist_id(album_artist.id);
    let album = album.insert(&mut conn)?;
    let artwork = NewDevArtwork::new("/art.jpg").insert(&mut conn)?;
    let mut input = NewDevContent::new("/Contents/track.wav");
    input.artist_id = artist.id;
    input.album_id = album.id;
    input.artwork_id = artwork.id;
    input.master_db_id = 7;
    input.master_content_id = 123;
    input.title = "Track".into();
    input.file_type = 11;
    input.file_size = i64::from(u32::MAX);
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
    let mapped = NewDevContent::try_from(track)?;
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
    assert_eq!(Track::try_from(view)?.id, Some(1));
    Ok(())
}

#[test]
fn shared_track_insertion_checks_ranges_and_normalizes_paths() -> Result<()> {
    use rbox_core::Track;
    let mut track = Track::new(r"/Contents/Artist\track.wav");
    track.id = Some(1);
    track.bpm = Some(120.126);
    track.length = Some(12.6);
    let row = NewDevContent::try_from(&track)?;
    assert_eq!(row.path, "/Contents/Artist/track.wav");
    assert_eq!(row.file_name, "track.wav");
    assert_eq!(row.bpmx100, 12013);
    assert_eq!(row.length, 13);
    for value in [f64::NAN, f64::INFINITY, -1.0, f64::from(u32::MAX)] {
        let mut invalid = track.clone();
        invalid.bpm = Some(value);
        assert!(NewDevContent::try_from(invalid).is_err());
    }
    let mut invalid = track.clone();
    invalid.length = Some(65536.0);
    assert!(NewDevContent::try_from(invalid).is_err());
    let mut invalid = track.clone();
    invalid.file_size = Some(u64::from(u32::MAX) + 1);
    assert!(NewDevContent::try_from(invalid).is_err());
    let mut invalid = track.clone();
    invalid.play_count = Some(65536);
    assert!(NewDevContent::try_from(invalid).is_err());
    let mut invalid = track.clone();
    invalid.file_type = Some(-1);
    assert!(NewDevContent::try_from(invalid).is_err());
    let mut invalid = track.clone();
    invalid.rating = Some(6);
    assert!(NewDevContent::try_from(invalid).is_err());
    for id in [None, Some(0), Some(u32::MAX)] {
        let mut invalid = track.clone();
        invalid.id = id;
        assert_eq!(NewDevContent::try_from(invalid)?, row);
    }
    let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
    let mut view = row.insert(&mut conn)?.resolve(&mut conn)?;
    view.content.rating = 6;
    assert!(Track::try_from(view).is_err());
    Ok(())
}
