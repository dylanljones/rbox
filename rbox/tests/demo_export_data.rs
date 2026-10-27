// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Check consistency of the RB7 demo exports and their source master database.
//!
//! These assertions describe the native demo export, not a requirement that all
//! independently generated exports allocate identical local IDs.
//! The source identity comparison additionally requires the `master-db` feature.

#![cfg(all(feature = "one-library", feature = "device-library"))]

use std::collections::BTreeSet;

use anyhow::Context;
use rbox::db::Model;
use rbox::onelib::models::{MyTagContent, PlaylistContent};
use rbox::{DeviceLibrary, OneLibrary};
use rbox_testing::data::{create_temp_dir, export_dir, setup_one_library_path};
use tempfile::{NamedTempFile, TempDir};

struct DemoExports {
    onelib: OneLibrary,
    devlib: DeviceLibrary,
    // Drop connections before removing their fixture copies, including on Windows.
    onelib_file: NamedTempFile,
    _devlib_directory: TempDir,
}

impl DemoExports {
    fn open() -> anyhow::Result<Self> {
        let onelib_file = setup_one_library_path();
        let directory = create_temp_dir();
        for name in ["export.pdb", "exportExt.pdb"] {
            std::fs::copy(
                export_dir(7).join("rekordbox").join(name),
                directory.path().join(name),
            )?;
        }
        Ok(Self {
            onelib: OneLibrary::from_path(onelib_file.path())?,
            devlib: DeviceLibrary::new(directory.path())?,
            onelib_file,
            _devlib_directory: directory,
        })
    }
}

#[test]
fn demo_content_identities_and_references_match() -> anyhow::Result<()> {
    let mut exports = DemoExports::open()?;
    let (onelib, devlib) = (&exports.onelib, &mut exports.devlib);
    let mut one = onelib.get_contents()?;
    let mut device = devlib.get_contents()?;
    // Match by audio path independently of the IDs being tested.
    one.sort_by(|a, b| a.path.cmp(&b.path));
    device.sort_by(|a, b| a.path.cmp(&b.path));
    assert_eq!(one.len(), 2, "the demo contains two exported tracks");
    assert_eq!(device.len(), one.len());
    assert_eq!(
        one.iter()
            .map(|row| &row.path)
            .collect::<BTreeSet<_>>()
            .len(),
        one.len()
    );

    for (one, device) in one.iter().zip(&device) {
        assert_eq!(one.path, device.path);
        assert_eq!(one.title.as_deref(), Some(device.title.as_str()));
        assert_eq!(one.id, device.id, "local ID for {}", one.path);
        assert_ne!(device.id, device.master_content_id);
        assert_ne!(device.master_content_id, 0);
        assert_eq!(one.master_content_id, Some(device.master_content_id));

        // Both native files contain the positive value. Casting through i32
        // would hide the old SQL mapping bug by turning it into -795645379.
        assert_eq!(device.master_db_id, 3_499_321_917);
        assert_eq!(one.master_db_id, Some(device.master_db_id));
        assert!(one.master_db_id.unwrap() > i64::from(i32::MAX));

        // Absent metadata references are SQL NULL or zero in OneLibrary, and
        // zero in Device Library. Populated references must match verbatim.
        for (field, sql_id, binary_id) in [
            ("artist", one.artist_id, device.artist_id),
            ("album", one.album_id, device.album_id),
            ("genre", one.genre_id, device.genre_id),
            ("label", one.label_id, device.label_id),
            ("key", one.key_id, device.key_id),
            ("image", one.image_id, device.artwork_id),
            ("composer", one.composer_id, device.composer_id),
            (
                "original artist",
                one.original_artist_id,
                device.original_artist_id,
            ),
            ("remixer", one.remixer_id, device.remixer_id),
            ("color", one.color_id, device.color_id),
        ] {
            assert_eq!(
                sql_id.unwrap_or(0),
                i64::from(binary_id),
                "{field} reference for {}",
                one.path
            );
        }

        let sql_match = onelib
            .get_content_by_master_content_id(device.master_content_id, device.master_db_id)?
            .context("OneLibrary source identity lookup failed")?;
        let binary_match = devlib
            .get_content_by_master_content_id(device.master_content_id, device.master_db_id)?
            .context("Device Library source identity lookup failed")?;
        assert_eq!(sql_match.id, one.id);
        assert_eq!(binary_match.id, device.id);

        let sql_track = onelib
            .get_track_by_id(one.id)?
            .context("OneLibrary track missing")?;
        let binary_track = devlib
            .get_track_by_id(device.id)?
            .context("Device Library track missing")?;
        assert_eq!(
            sql_track.master_db_id,
            Some(u32::try_from(device.master_db_id)?)
        );
        assert_eq!(sql_track.master_db_id, binary_track.master_db_id);
        assert_eq!(sql_track.master_content_id, binary_track.master_content_id);
        assert_eq!(sql_track.artist.as_deref(), Some("Loopmasters"));
        assert_eq!(sql_track.artist, binary_track.artist);
        assert_eq!(sql_track.label, binary_track.label);
        assert_eq!(
            sql_track.analysis_data_path,
            binary_track.analysis_data_path
        );
        assert_eq!(sql_track.file_size, binary_track.file_size);
        assert_eq!(sql_track.bpm, binary_track.bpm);
        assert_eq!(sql_track.length, binary_track.length);
    }
    Ok(())
}

#[test]
fn demo_my_tag_library_identity_matches_without_signed_conversion() -> anyhow::Result<()> {
    let mut exports = DemoExports::open()?;
    let one = exports.onelib.get_properties()?;
    let device = exports.devlib.get_database_properties()?;
    assert_eq!(one.len(), 1);
    assert_eq!(device.len(), 1);
    assert_eq!(device[0].my_tag_master_db_id, 3_499_321_918);
    assert_eq!(
        one[0].my_tag_master_dbid,
        i64::from(device[0].my_tag_master_db_id)
    );
    assert!(one[0].my_tag_master_dbid > i64::from(i32::MAX));
    // The demo's My Tag library identity is distinct from its track library ID.
    for track in exports.onelib.get_contents()? {
        assert_ne!(Some(one[0].my_tag_master_dbid), track.master_db_id);
    }
    Ok(())
}

#[test]
fn demo_my_tag_ids_hierarchy_and_empty_associations_match() -> anyhow::Result<()> {
    let mut exports = DemoExports::open()?;
    let one = exports.onelib.get_my_tags()?;
    let device = exports.devlib.get_my_tags()?;
    assert!(!one.is_empty());
    assert!(one.iter().any(|row| row.id > i64::from(i32::MAX)));
    let mut sql_rows: Vec<_> = one
        .iter()
        .map(|row| {
            (
                row.id,
                row.parent_id,
                row.name.as_str(),
                i64::from(row.seq),
                row.attribute,
            )
        })
        .collect();
    let mut binary_rows: Vec<_> = device
        .iter()
        .map(|row| {
            (
                i64::from(row.id),
                i64::from(row.parent_id),
                row.name.as_str(),
                i64::from(row.seq),
                row.attribute,
            )
        })
        .collect();
    sql_rows.sort_by_key(|row| row.0);
    binary_rows.sort_by_key(|row| row.0);
    assert_eq!(
        sql_rows, binary_rows,
        "My Tag identities, parents, names, order and types"
    );

    for tag in &device {
        let id = i64::from(tag.id);
        let found = exports
            .onelib
            .get_my_tag_by_id(id)?
            .context("native My Tag lookup failed")?;
        assert_eq!(found.id, id);
        assert_eq!(found.name, tag.name);
        let sql_children: Vec<_> = exports
            .onelib
            .get_my_tag_children(id)?
            .into_iter()
            .map(|row| row.id)
            .collect();
        let binary_children: Vec<_> = exports
            .devlib
            .get_my_tag_children(tag.id)?
            .into_iter()
            .map(|row| i64::from(row.id))
            .collect();
        assert_eq!(sql_children, binary_children, "children of {}", tag.name);
    }

    // The unmodified demo defines tags but assigns none to its two tracks.
    // Assert the actual empty tables rather than claiming nonempty link coverage.
    let mut conn =
        rbox::onelib::establish_connection(exports.onelib_file.path(), &Default::default())?;
    assert!(MyTagContent::all(&mut conn)?.is_empty());
    assert!(exports.devlib.get_my_tag_associations()?.is_empty());
    for track in exports.devlib.get_contents()? {
        assert!(exports
            .onelib
            .get_my_tag_by_content_id(i64::from(track.id))?
            .is_empty());
        assert!(exports
            .devlib
            .get_my_tag_by_content_id(track.id)?
            .is_empty());
    }
    Ok(())
}

#[test]
fn demo_playlist_ids_order_and_content_references_match() -> anyhow::Result<()> {
    let mut exports = DemoExports::open()?;
    let one = exports.onelib.get_playlists()?;
    let device = exports.devlib.get_playlists()?;
    assert_eq!(one.len(), 1);
    assert_eq!(device.len(), 1);
    assert_eq!(one[0].name, "Demo Tracks");
    assert_eq!(one[0].name, device[0].name);
    assert_eq!(one[0].id, i64::from(device[0].id));
    assert_eq!(one[0].parent_id, i64::from(device[0].parent_id));
    assert_eq!(i64::from(one[0].seq), i64::from(device[0].seq));
    assert_eq!(one[0].attribute, device[0].attribute);

    let mut conn =
        rbox::onelib::establish_connection(exports.onelib_file.path(), &Default::default())?;
    let mut sql_entries: Vec<_> = PlaylistContent::all(&mut conn)?
        .into_iter()
        .map(|row| (row.playlist_id, row.content_id, i64::from(row.seq)))
        .collect();
    let mut binary_entries: Vec<_> = exports
        .devlib
        .get_playlist_associations()?
        .into_iter()
        .map(|row| {
            (
                i64::from(row.playlist_id),
                i64::from(row.content_id),
                i64::from(row.seq),
            )
        })
        .collect();
    sql_entries.sort();
    binary_entries.sort();
    assert_eq!(sql_entries, binary_entries);
    assert_eq!(sql_entries, [(one[0].id, 1, 1), (one[0].id, 2, 2)]);

    let sql_tracks = exports.onelib.get_playlist_contents(one[0].id)?;
    let binary_tracks = exports.devlib.get_playlist_contents(device[0].id)?;
    assert_eq!(sql_tracks.len(), 2);
    assert_eq!(sql_tracks.len(), binary_tracks.len());
    for (sql_track, binary_track) in sql_tracks.iter().zip(&binary_tracks) {
        assert_eq!(sql_track.id, i64::from(binary_track.id));
        assert_eq!(sql_track.path, binary_track.path);
        assert_eq!(
            sql_track.master_content_id,
            Some(i64::from(binary_track.master_content_id))
        );
    }
    Ok(())
}

#[cfg(feature = "master-db")]
mod master_identity {
    use std::collections::BTreeMap;

    use anyhow::{ensure, Context};
    use rbox::{core::Track, MasterDb};
    use rbox_testing::data::setup_master_db_path;

    use super::DemoExports;

    // Match metadata independently of the identifiers being tested. Reject ambiguous
    // fixture metadata rather than silently choosing one track or matching by ID.
    fn by_metadata(tracks: Vec<Track>) -> anyhow::Result<BTreeMap<(String, String), Track>> {
        let mut result = BTreeMap::new();
        for track in tracks {
            let key = (
                track.title.clone().context("demo track has no title")?,
                track.artist.clone().unwrap_or_default(),
            );
            ensure!(
                !result.contains_key(&key),
                "ambiguous demo track metadata: {key:?}"
            );
            result.insert(key, track);
        }
        ensure!(!result.is_empty(), "demo database contains no tracks");
        Ok(result)
    }

    #[test]
    fn demo_master_identities_match_both_exports() -> anyhow::Result<()> {
        let master_file = setup_master_db_path(7);
        let master = MasterDb::new(master_file.path())?;
        let mut exports = DemoExports::open()?;
        let master_tracks = master.get_tracks()?;
        let onelib_tracks = by_metadata(exports.onelib.get_tracks()?)?;
        let devlib_tracks = by_metadata(exports.devlib.get_tracks()?)?;

        assert_eq!(
            devlib_tracks.keys().collect::<Vec<_>>(),
            onelib_tracks.keys().collect::<Vec<_>>(),
            "both demo exports must contain the same tracks"
        );
        for (format, exported) in [
            ("Device Library", &devlib_tracks),
            ("OneLibrary", &onelib_tracks),
        ] {
            for (metadata, track) in exported {
                let candidates: Vec<_> = master_tracks
                    .iter()
                    .filter(|original| {
                        original.title.as_deref() == Some(metadata.0.as_str())
                            && original.artist.as_deref().unwrap_or_default() == metadata.1
                    })
                    .collect();
                ensure!(
                    candidates.len() == 1,
                    "expected one masterdb match for {format} {metadata:?}, found {}",
                    candidates.len()
                );
                let original = candidates[0];
                ensure!(
                    original.master_db_id.is_some(),
                    "master library ID is absent"
                );
                assert_eq!(original.master_content_id, original.id);
                assert_eq!(
                    track.master_db_id, original.master_db_id,
                    "{format} library identity differs for {metadata:?}"
                );
                assert_eq!(
                    track.master_content_id, original.master_content_id,
                    "{format} content identity differs for {metadata:?}"
                );
            }
        }
        Ok(())
    }
}
