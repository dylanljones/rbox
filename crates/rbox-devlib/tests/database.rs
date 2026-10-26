// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Database lifecycle, model operations, persistence, and partial failure reporting.

use std::fs::{self, OpenOptions};
use std::io::Write;

use rbox_devlib::binary::{
    DeviceSqlString, ExtPageType, ExtendedRow, NamedRow, PageType, RecommendLikeRow, Row,
    StandardRow, StringEncoding, TableKind,
};
use rbox_devlib::{DatabaseKind, DeviceLibrary, Error, LibraryCreateOptions, Result};
use rbox_testing::data::{create_temp_dir, export_dir};

fn options() -> LibraryCreateOptions {
    LibraryCreateOptions::new("2026-10-25", "Test USB", 42)
}

#[test]
fn playlist_tree_methods_preserve_hierarchy_order_and_explicit_saving() -> Result<()> {
    use rbox_core::PlaylistType;
    use rbox_devlib::models::{NewPlaylist, Playlist};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert!(library.get_playlists()?.is_empty());
        assert!(library.get_playlist_by_id(999)?.is_none());
        assert!(library.get_playlist_by_path(&[])?.is_none());
        assert!(library.get_playlist_tree()?.is_empty());
        assert_eq!(
            library.get_playlist_attribute(0)?,
            Some(PlaylistType::Folder)
        );
        assert_eq!(library.get_playlist_attribute(999)?, None);
        assert!(!library.is_playlist(0)?);
        let folder = library.create_playlist_folder("Sets", None, None)?;
        let other = library.create_playlist_folder("Other", None, None)?;
        let a = library.create_playlist("A", Some(folder.id), None)?;
        let b = library.insert_playlist(NewPlaylist::playlist("B").parent_id(folder.id))?;
        let c = library.create_playlist("C", Some(folder.id), Some(0))?;
        assert_eq!((folder.id, other.id, a.id, b.id, c.id), (1, 2, 3, 4, 5));
        assert!(library.is_playlist(a.id)?);
        assert_eq!(
            library.get_playlist_attribute(a.id)?,
            Some(PlaylistType::List)
        );
        assert_eq!(
            library.get_playlist_by_path(&["Sets", "B"])?.unwrap().id,
            b.id
        );
        assert!(library.get_playlist_by_path(&["sets", "B"])?.is_none());
        assert_eq!(library.count_playlist_children(folder.id)?, 3);
        assert_eq!(library.get_playlist_seq_numbers(folder.id)?, vec![0, 1, 2]);
        assert_eq!(
            library
                .get_playlist_children(folder.id)?
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            vec![c.id, a.id, b.id]
        );
        let tree = library.get_playlist_tree()?;
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].playlist.id, folder.id);
        assert_eq!(tree[0].children[0].playlist.id, c.id);
        let before_invalid = library.get_playlists()?;
        assert!(library
            .move_playlist(folder.id, Some(&folder.id), None)
            .is_err());
        assert!(library.move_playlist(a.id, Some(&b.id), None).is_err());
        assert_eq!(library.get_playlists()?, before_invalid);
        assert_eq!(library.move_playlist(c.id, Some(&other.id), None)?, 1);
        assert_eq!(library.move_playlists(&[a.id, b.id], other.id, Some(0))?, 2);
        let mut edited = library.get_playlist_by_id(c.id)?.unwrap();
        edited.parent_id = folder.id;
        edited.name = "Updated".into();
        let updated = library.update_playlist(edited)?;
        assert_eq!(
            (updated.parent_id, updated.seq),
            (folder.id, Playlist::START_SEQ)
        );
        assert_eq!(library.rename_playlist(a.id, "Renamed")?, 1);
        assert!(matches!(
            library.rename_playlist(999, "Missing"),
            Err(Error::ModelNotFound { .. })
        ));
        // Imported gaps can be inspected and explicitly normalized.
        let node = library.get_playlist_by_id(b.id)?.unwrap();
        let Row::Standard(StandardRow::PlaylistTree(mut row)) = node.raw_record().decode()? else {
            panic!()
        };
        row.position = 9;
        let mut tx = library.standard_mut().transaction();
        let handle = tx.locate(
            Playlist::TABLE,
            node.raw_record().page(),
            node.raw_record().slot().slot,
        )?;
        tx.update(&handle, &Row::Standard(StandardRow::PlaylistTree(row)))?;
        tx.commit()?;
        assert!(!library.validate_playlist_seq_numbers(other.id)?);
        assert_eq!(library.reset_playlist_seq(other.id)?, 1);
        assert!(library.validate_playlist_seq_numbers(other.id)?);
        let content = library.create_content("/Contents/keep.wav")?;
        library.create_playlist_content(c.id, content.id, None)?;
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        drop(library);
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert_eq!(
            library
                .get_playlist_by_path(&["Other", "Renamed"])?
                .unwrap()
                .id,
            a.id
        );
        assert_eq!(library.get_playlist_contents(c.id)?[0].id, content.id);
        let before_delete = fs::read(&path)?;
        assert_eq!(library.delete_playlist(999)?, 0);
        assert_eq!(library.delete_playlist(folder.id)?, 3);
        assert!(library.get_playlist_by_id(c.id)?.is_none());
        assert_eq!(
            library.get_playlist_children(0)?[0].seq,
            Playlist::START_SEQ
        );
        assert!(library.delete_playlists(&[other.id, 999]).is_err());
        assert!(library.get_playlist_by_id(other.id)?.is_some());
        assert_eq!(library.delete_playlists(&[other.id, a.id])?, 3);
        assert!(library.get_playlists()?.is_empty());
        assert_eq!(library.get_contents()?.len(), 1);
        assert_eq!(fs::read(&path)?, before_delete);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);
        let mut reopened = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert!(reopened.get_playlist_tree()?.is_empty());
        assert!(reopened.get_playlist_associations()?.is_empty());
        assert_eq!(reopened.get_contents()?.len(), 1);
    }
    Ok(())
}

#[test]
fn playlist_content_methods_preserve_order_uniqueness_and_imported_occurrences() -> Result<()> {
    use rbox_devlib::models::{NewPlaylistContent, PlaylistContent};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let playlist = library.create_playlist("Set", None, None)?;
        let folder = library.create_playlist_folder("Folder", None, None)?;
        let a = library.create_content("/Contents/a.wav")?;
        let b = library.create_content("/Contents/b.wav")?;
        let c = library.create_content("/Contents/c.wav")?;
        library.save()?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert!(library.get_playlist_entries(playlist.id)?.is_empty());
        assert!(library
            .get_playlist_content_by_id(playlist.id, a.id)?
            .is_none());
        assert!(library
            .create_playlist_content(folder.id, a.id, None)
            .is_err());
        assert!(library
            .create_playlist_content(playlist.id, a.id, Some(0))
            .is_err());
        let entry =
            library.insert_playlist_content(NewPlaylistContent::new(playlist.id, a.id, None))?;
        assert_eq!(entry.seq, PlaylistContent::START_SEQ);
        library.create_playlist_content(playlist.id, b.id, Some(1))?;
        library.create_playlist_content(playlist.id, c.id, Some(1))?;
        assert!(matches!(
            library.create_playlist_content(playlist.id, a.id, None),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert_eq!(
            library
                .get_playlist_contents(playlist.id)?
                .iter()
                .map(|row| row.id)
                .collect::<Vec<_>>(),
            vec![c.id, b.id, a.id]
        );
        assert_eq!(library.get_playlist_associations()?.len(), 3);
        assert_eq!(
            library.get_playlist_associations_by_content_id(a.id)?[0].playlist_id,
            playlist.id
        );
        assert_eq!(library.count_playlist_contents(playlist.id)?, 3);
        assert_eq!(
            library.get_playlist_content_seq_numbers(playlist.id)?,
            vec![1, 2, 3]
        );
        assert!(library.validate_playlist_content_seq_numbers(playlist.id)?);
        assert_eq!(
            library.move_playlist_content(playlist.id, a.id, Some(1))?,
            1
        );
        assert_eq!(
            library.move_playlist_contents(&[(playlist.id, b.id), (playlist.id, a.id)], Some(1))?,
            2
        );
        assert_eq!(
            library
                .get_playlist_contents(playlist.id)?
                .iter()
                .map(|row| row.id)
                .collect::<Vec<_>>(),
            vec![b.id, a.id, c.id]
        );
        let entry = library
            .get_playlist_content_by_id(playlist.id, c.id)?
            .unwrap();
        let Row::Standard(StandardRow::PlaylistEntry(mut row)) = entry.raw_record().decode()?
        else {
            panic!()
        };
        row.position = 9;
        let mut tx = library.standard_mut().transaction();
        let handle = tx.locate(
            PlaylistContent::TABLE,
            entry.raw_record().page(),
            entry.raw_record().slot().slot,
        )?;
        tx.update(&handle, &Row::Standard(StandardRow::PlaylistEntry(row)))?;
        tx.commit()?;
        assert!(!library.validate_playlist_content_seq_numbers(playlist.id)?);
        assert_eq!(library.reset_playlist_content_seq(playlist.id)?, 1);
        assert!(library.validate_playlist_content_seq_numbers(playlist.id)?);
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        drop(library);
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert_eq!(
            library
                .get_playlist_contents(playlist.id)?
                .iter()
                .map(|row| row.id)
                .collect::<Vec<_>>(),
            vec![b.id, a.id, c.id]
        );
        // Imported duplicate pairs remain readable, with precise occurrence deletion.
        let entry = library
            .get_playlist_content_by_id(playlist.id, a.id)?
            .unwrap();
        let row = entry.raw_record().decode()?;
        let mut tx = library.standard_mut().transaction();
        tx.insert(PlaylistContent::TABLE, &row)?;
        tx.commit()?;
        assert_eq!(library.get_playlist_contents(playlist.id)?.len(), 4);
        assert!(library
            .get_playlist_content_by_id(playlist.id, a.id)
            .is_err());
        assert!(library.delete_playlist_content(playlist.id, a.id).is_err());
        let entry = library
            .get_playlist_entries(playlist.id)?
            .into_iter()
            .find(|e| e.content_id == a.id)
            .unwrap();
        assert_eq!(library.delete_playlist_content_entry(entry)?, 1);
        assert_eq!(library.get_playlist_entries(playlist.id)?.len(), 3);
        assert_eq!(library.delete_playlist_content(playlist.id, b.id)?, 1);
        assert_eq!(library.delete_playlist_content(playlist.id, b.id)?, 0);
        assert!(library
            .delete_playlist_contents(&[(playlist.id, a.id), (playlist.id, 999)])
            .is_err());
        assert_eq!(library.count_playlist_contents(playlist.id)?, 2);
        assert_eq!(
            library.delete_playlist_contents(&[(playlist.id, a.id), (playlist.id, c.id)])?,
            2
        );
        assert!(library.get_playlist_contents(playlist.id)?.is_empty());
        assert!(library.get_playlist_by_id(playlist.id)?.is_some());
        assert_eq!(library.get_contents()?.len(), 3);
        library.save()?;
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);
        let mut reopened = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert!(reopened.get_playlist_entries(playlist.id)?.is_empty());
        assert_eq!(reopened.get_contents()?.len(), 3);
    }
    Ok(())
}

#[test]
fn playlist_association_cleanup_retains_playlists_and_contents() -> Result<()> {
    let temp = create_temp_dir();
    let mut library = DeviceLibrary::create(temp.path(), &options())?;
    let p = library.create_playlist("P", None, None)?.id;
    let q = library.create_playlist("Q", None, None)?.id;
    let a = library.create_content("/Contents/a.wav")?.id;
    let b = library.create_content("/Contents/b.wav")?.id;
    for playlist in [p, q] {
        library.create_playlist_content(playlist, a, None)?;
        library.create_playlist_content(playlist, b, None)?;
    }
    assert_eq!(library.delete_playlist_contents_by_content_id(a)?, 2);
    assert_eq!(library.get_playlist_content_seq_numbers(p)?, vec![1]);
    assert_eq!(
        library.delete_playlist_contents_by_content_ids(&[b, b, 999])?,
        2
    );
    for playlist in [p, q] {
        library.create_playlist_content(playlist, a, None)?;
        library.create_playlist_content(playlist, b, None)?;
    }
    assert_eq!(library.delete_playlist_contents_by_playlist_id(p)?, 2);
    assert_eq!(
        library.delete_playlist_contents_by_playlist_ids(&[q, q, 999])?,
        2
    );
    let entry = library.create_playlist_content(p, a, None)?;
    let row = entry.raw_record().decode()?;
    let mut tx = library.standard_mut().transaction();
    tx.insert(rbox_devlib::models::PlaylistContent::TABLE, &row)?;
    tx.commit()?;
    assert_eq!(library.remove_playlist_content(p, a)?, 2);
    assert_eq!(library.remove_playlist_content(p, a)?, 0);
    assert_eq!(library.get_playlists()?.len(), 2);
    assert_eq!(library.get_contents()?.len(), 2);
    assert!(library.get_playlist_associations()?.is_empty());
    Ok(())
}

#[test]
fn my_tag_methods_preserve_ids_order_associations_and_explicit_saving() -> Result<()> {
    use rbox_devlib::models::{MyTag, NewMyTag, NewMyTagContent};

    let temp = create_temp_dir();
    let mut library = DeviceLibrary::create(temp.path(), &options())?;
    let first = library.create_content("/Contents/first.wav")?;
    let second = library.create_content("/Contents/second.wav")?;
    library.save()?;
    let standard_before = fs::read(temp.path().join("export.pdb"))?;
    let extended_before = fs::read(temp.path().join("exportExt.pdb"))?;
    assert!(library.get_my_tags()?.is_empty());
    assert!(library.get_my_tag_by_id(999)?.is_none());
    assert!(library.get_my_tag_contents(999)?.is_empty());
    assert!(library.get_my_tag_by_content_id(999)?.is_empty());
    library.create_my_tag_folder(100, "Mood", None, None)?;
    library.create_my_tag_folder(200, "Energy", None, None)?;
    library.create_my_tag(101, "Warm", Some(100), None)?;
    library.insert_my_tag(NewMyTag::tag(102, "Bright").parent_id(100))?;
    library.create_my_tag(103, "Calm", Some(100), Some(0))?;
    assert_eq!(library.count_my_tag_children(100)?, 3);
    assert_eq!(
        library
            .get_my_tag_children(100)?
            .iter()
            .map(|tag| tag.id)
            .collect::<Vec<_>>(),
        vec![103, 101, 102]
    );
    assert_eq!(library.get_my_tag_seq_numbers(100)?, vec![0, 1, 2]);
    assert!(library.validate_my_tag_seq_numbers(100)?);
    assert!(matches!(
        library.create_my_tag(101, "Duplicate", None, None),
        Err(Error::DuplicateId { .. })
    ));
    let before_invalid_move = library.get_my_tags()?;
    assert!(library.move_my_tag(100, Some(&100), None).is_err());
    assert!(library.move_my_tag(101, Some(&102), None).is_err());
    assert_eq!(library.get_my_tags()?, before_invalid_move);
    assert_eq!(library.move_my_tag(103, Some(&200), None)?, 1);
    assert_eq!(library.move_my_tags(&[101, 102], 200, Some(0))?, 2);
    assert_eq!(
        library
            .get_my_tag_children(200)?
            .iter()
            .map(|tag| tag.id)
            .collect::<Vec<_>>(),
        vec![101, 102, 103]
    );
    let mut edited = library.get_my_tag_by_id(103)?.unwrap();
    edited.parent_id = 100;
    edited.name = "Soft".into();
    let updated = library.update_my_tag(edited)?;
    assert_eq!(
        (
            updated.id,
            updated.parent_id,
            updated.seq,
            updated.name.as_str()
        ),
        (103, 100, 0, "Soft")
    );
    assert_eq!(library.rename_my_tag(101, "Hot")?, 1);
    assert_eq!(library.rename_my_tag(999, "Missing")?, 0);
    // Simulate an imported sequence gap; the handler explicitly repairs it.
    let tag = library.get_my_tag_by_id(102)?.unwrap();
    let Row::Extended(ExtendedRow::MyTag(mut row)) = tag.raw_record().decode()? else {
        panic!()
    };
    row.header.position = 9;
    let mut tx = library.extended_mut()?.transaction();
    let handle = tx.locate(
        MyTag::TABLE,
        tag.raw_record().page(),
        tag.raw_record().slot().slot,
    )?;
    tx.update(&handle, &Row::Extended(ExtendedRow::MyTag(row)))?;
    tx.commit()?;
    assert!(!library.validate_my_tag_seq_numbers(200)?);
    assert_eq!(library.reset_my_tag_seq(200)?, 1);
    assert_eq!(library.get_my_tag_seq_numbers(200)?, vec![0, 1]);
    assert!(library.validate_my_tag_seq_numbers(200)?);
    library.create_my_tag_content(101, first.id)?;
    library.insert_my_tag_content(NewMyTagContent::new(first.id, 101))?;
    library.create_my_tag_content(102, second.id)?;
    library.create_my_tag_content(103, second.id)?;
    assert_eq!(library.get_my_tag_associations()?.len(), 4);
    assert_eq!(
        library
            .get_my_tag_associations_by_content_id(first.id)?
            .len(),
        2
    );
    assert_eq!(library.get_my_tag_entries(101)?.len(), 2);
    assert_eq!(
        library
            .get_my_tag_contents(101)?
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        vec![first.id, first.id]
    );
    assert_eq!(
        library
            .get_my_tag_by_content_id(first.id)?
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        vec![101, 101]
    );
    assert_eq!(
        library
            .get_my_tag_by_content_id(second.id)?
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        vec![102, 103]
    );
    let entry = library.get_my_tag_entries(101)?.remove(0);
    assert_eq!(library.delete_my_tag_content_entry(entry)?, 1);
    assert_eq!(library.get_my_tag_entries(101)?.len(), 1);
    library.create_my_tag_content(101, first.id)?;
    assert_eq!(library.delete_my_tag_content(101, first.id)?, 2);
    assert_eq!(library.delete_my_tag_content(101, first.id)?, 0);
    library.create_my_tag_content(101, first.id)?;
    assert_eq!(
        fs::read(temp.path().join("exportExt.pdb"))?,
        extended_before
    );
    library.save()?;
    drop(library);

    let mut library = DeviceLibrary::new(temp.path())?;
    assert_eq!(library.get_my_tag_by_id(101)?.unwrap().name, "Hot");
    assert_eq!(library.get_my_tag_by_id(103)?.unwrap().name, "Soft");
    assert_eq!(
        library
            .get_my_tag_children(200)?
            .iter()
            .map(|tag| tag.id)
            .collect::<Vec<_>>(),
        vec![101, 102]
    );
    assert_eq!(library.get_my_tag_contents(101)?[0].id, first.id);
    assert_eq!(library.delete_my_tag(999)?, 0);
    assert_eq!(library.delete_my_tag(100)?, 3); // Category, child and association.
    assert!(library.get_my_tag_by_id(103)?.is_none());
    assert_eq!(library.get_my_tag_children(0)?[0].seq, MyTag::START_SEQ);
    assert!(library.delete_my_tags(&[200, 999]).is_err());
    assert!(library.get_my_tag_by_id(200)?.is_some());
    assert_eq!(library.delete_my_tags(&[200, 101])?, 5); // Overlapping subtree counted once.
    assert!(library.get_my_tags()?.is_empty());
    assert!(library.get_my_tag_associations()?.is_empty());
    assert_eq!(library.get_contents()?.len(), 2);
    library.save()?;
    assert_eq!(fs::read(temp.path().join("export.pdb"))?, standard_before);
    drop(library);
    let mut reopened = DeviceLibrary::new(temp.path())?;
    assert!(reopened.get_my_tags()?.is_empty());
    assert!(reopened.get_my_tag_associations()?.is_empty());
    assert_eq!(reopened.get_contents()?.len(), 2);
    Ok(())
}

#[test]
fn my_tag_relationship_queries_report_missing_targets() -> Result<()> {
    use rbox_devlib::models::{Content, MyTag};

    let temp = create_temp_dir();
    let mut library = DeviceLibrary::create(temp.path(), &options())?;
    library.create_my_tag_content(100, 200)?;
    assert!(matches!(
        library.get_my_tag_contents(100),
        Err(Error::ModelNotFound {
            table: Content::TABLE,
            id: 200
        })
    ));
    assert!(matches!(
        library.get_my_tag_by_content_id(200),
        Err(Error::ModelNotFound {
            table: MyTag::TABLE,
            id: 100
        })
    ));
    assert_eq!(library.delete_my_tag_content(100, 200)?, 1);
    Ok(())
}

#[test]
fn my_tag_methods_require_an_open_extended_database() -> Result<()> {
    use rbox_devlib::models::{NewMyTag, NewMyTagContent};

    let temp = create_temp_dir();
    let mut pair = DeviceLibrary::create(temp.path(), &options())?;
    let tag = pair.create_my_tag(100, "Tag", None, None)?;
    let entry = pair.create_my_tag_content(100, 1)?;
    pair.save()?;
    drop(pair);
    let mut library = DeviceLibrary::from_paths(temp.path().join("export.pdb"), None)?;
    macro_rules! missing {
        ($operation:expr) => {
            assert!(matches!($operation, Err(Error::MissingExtendedDatabase)));
        };
    }
    missing!(library.get_my_tags());
    missing!(library.get_my_tag_by_id(100));
    missing!(library.get_my_tag_children(0));
    missing!(library.get_my_tag_contents(100));
    missing!(library.get_my_tag_by_content_id(1));
    missing!(library.insert_my_tag(NewMyTag::tag(101, "Tag")));
    missing!(library.create_my_tag(101, "Tag", None, None));
    missing!(library.create_my_tag_folder(200, "Category", None, None));
    missing!(library.update_my_tag(tag));
    missing!(library.rename_my_tag(100, "Renamed"));
    missing!(library.count_my_tag_children(0));
    missing!(library.get_my_tag_seq_numbers(0));
    missing!(library.validate_my_tag_seq_numbers(0));
    missing!(library.reset_my_tag_seq(0));
    missing!(library.move_my_tag(100, None, None));
    missing!(library.move_my_tags(&[], 0, None));
    missing!(library.delete_my_tag(100));
    missing!(library.delete_my_tags(&[]));
    missing!(library.get_my_tag_associations());
    missing!(library.get_my_tag_entries(100));
    missing!(library.get_my_tag_associations_by_content_id(1));
    missing!(library.insert_my_tag_content(NewMyTagContent::new(1, 100)));
    missing!(library.create_my_tag_content(100, 1));
    missing!(library.delete_my_tag_content(100, 1));
    missing!(library.delete_my_tag_content_entry(entry));
    assert_eq!(library.standard().read_stats().page_reads, 0);
    assert_eq!(library.standard().modified_page_count(), 0);
    Ok(())
}

#[test]
fn property_methods_update_each_file_and_save_explicitly() -> Result<()> {
    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert_eq!(library.get_properties()?.len(), 1);
        assert!(library.get_property_by_device("test usb")?.is_none());
        let original = library.get_first_property()?.unwrap();
        assert_eq!(
            library.get_property_by_device("Test USB")?,
            Some(original.clone())
        );
        let mut edited = original;
        edited.name = "Renamed USB".into();
        edited.date = "2026-10-26".into();
        edited.background_color = 2;
        let updated = library.update_property(edited)?;
        assert_eq!(updated.name, "Renamed USB");
        assert!(library.get_property_by_device("Test USB")?.is_none());
        assert_eq!(
            library.get_property_by_device("Renamed USB")?.unwrap().date,
            "2026-10-26"
        );

        // A property snapshot must not overwrite a count changed by content insertion.
        library.create_content("/Contents/track.wav")?;
        assert!(matches!(
            library.update_property(updated),
            Err(Error::StaleModelRow { .. })
        ));
        assert_eq!(library.get_first_property()?.unwrap().track_count, 1);
        if with_extended {
            assert_eq!(library.get_database_properties()?.len(), 1);
            let mut property = library.get_first_database_property()?.unwrap();
            assert_eq!(property.my_tag_master_db_id, 42);
            let mut expected = property.raw_record().decode()?;
            let Row::Extended(ExtendedRow::DatabaseProperty(mut row)) = expected else {
                panic!("expected extended database property");
            };
            row.header.my_tag_master_db_id = u32::MAX;
            expected = Row::Extended(ExtendedRow::DatabaseProperty(row));
            property.my_tag_master_db_id = u32::MAX;
            let updated = library.update_database_property(property)?;
            assert_eq!(updated.raw_record().decode()?, expected);
        } else {
            assert!(matches!(
                library.get_database_properties(),
                Err(Error::MissingExtendedDatabase)
            ));
            assert!(matches!(
                library.get_first_database_property(),
                Err(Error::MissingExtendedDatabase)
            ));
        }
        assert_eq!(fs::read(&path)?, before);
        assert_eq!(fs::read(&extended)?, extended_before);
        library.save()?;
        drop(library);
        let mut reopened = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let property = reopened.get_first_property()?.unwrap();
        assert_eq!(
            (
                property.name.as_str(),
                property.date.as_str(),
                property.background_color,
                property.track_count
            ),
            ("Renamed USB", "2026-10-26", 2, 1)
        );
        if with_extended {
            assert_eq!(
                reopened
                    .get_first_database_property()?
                    .unwrap()
                    .my_tag_master_db_id,
                u32::MAX
            );
        } else {
            assert_eq!(fs::read(&extended)?, extended_before);
        }
    }
    Ok(())
}

#[test]
fn recommendation_methods_preserve_raw_values_constraints_and_occurrences() -> Result<()> {
    use rbox_devlib::models::NewRecommendedLike;

    let temp = create_temp_dir();
    let mut library = DeviceLibrary::create(temp.path(), &options())?;
    let first = library.create_content("/Contents/first.wav")?;
    let second = library.create_content("/Contents/second.wav")?;
    library.save()?;
    let standard_before = fs::read(temp.path().join("export.pdb"))?;
    let extended_before = fs::read(temp.path().join("exportExt.pdb"))?;
    assert!(library.get_recommend_likes()?.is_empty());
    assert!(library.get_recommend_like_by_content_id(99)?.is_empty());
    assert!(matches!(
        library.create_recommend_like(0, second.id, 0, 0, 0),
        Err(Error::InvalidModelField { .. })
    ));
    let input = NewRecommendedLike::new(first.id, second.id, u64::MAX, 0x12345678, u32::MAX);
    let inserted = library.insert_recommend_like(input)?;
    assert!(matches!(
        library.create_recommend_like(first.id, second.id, 0, 0, 0),
        Err(Error::UniqueConstraintViolation { .. })
    ));
    library.create_recommend_like(second.id, first.id, 11, 22, 33)?;
    assert_eq!(library.get_recommend_like_by_content_id(first.id)?.len(), 2);
    assert_eq!(
        library.get_recommend_like_by_content_id(second.id)?.len(),
        2
    );
    // Imported binary rows may repeat: reads and snapshot deletion retain occurrence identity.
    let row = inserted.raw_record().decode()?;
    let mut tx = library.extended_mut()?.transaction();
    tx.insert(rbox_devlib::models::RecommendedLike::TABLE, &row)?;
    tx.commit()?;
    assert_eq!(library.get_recommend_likes()?.len(), 3);
    let occurrence = library.get_recommend_likes()?.remove(0);
    assert_eq!(library.delete_recommend_like(occurrence)?, 1);
    assert_eq!(library.get_recommend_likes()?.len(), 2);
    assert_eq!(
        fs::read(temp.path().join("exportExt.pdb"))?,
        extended_before
    );
    library.save()?;
    assert_eq!(fs::read(temp.path().join("export.pdb"))?, standard_before);
    drop(library);
    let mut reopened = DeviceLibrary::new(temp.path())?;
    let rows = reopened.get_recommend_likes()?;
    assert_eq!(rows.len(), 2);
    let forward = rows
        .iter()
        .find(|row| row.content_id_1 == first.id)
        .unwrap();
    assert_eq!(
        (forward.unknown_00, forward.unknown_10, forward.unknown_14),
        (u64::MAX, 0x12345678, u32::MAX)
    );
    let reverse = rows
        .iter()
        .find(|row| row.content_id_1 == second.id)
        .unwrap();
    assert_eq!(
        (
            reverse.content_id_2,
            reverse.unknown_00,
            reverse.unknown_10,
            reverse.unknown_14
        ),
        (first.id, 11, 22, 33)
    );
    assert_eq!(reopened.get_contents()?.len(), 2);
    Ok(())
}

#[test]
fn extended_model_methods_require_an_open_extended_database() -> Result<()> {
    use rbox_devlib::models::NewRecommendedLike;

    let temp = create_temp_dir();
    let mut pair = DeviceLibrary::create(temp.path(), &options())?;
    let property = pair.get_first_database_property()?.unwrap();
    let recommendation = pair.create_recommend_like(1, 2, 0, 0, 0)?;
    pair.save()?;
    drop(pair);
    let extended_before = fs::read(temp.path().join("exportExt.pdb"))?;
    let mut library = DeviceLibrary::from_paths(temp.path().join("export.pdb"), None)?;
    assert!(matches!(
        library.get_database_properties(),
        Err(Error::MissingExtendedDatabase)
    ));
    assert!(matches!(
        library.get_first_database_property(),
        Err(Error::MissingExtendedDatabase)
    ));
    assert!(matches!(
        library.update_database_property(property),
        Err(Error::MissingExtendedDatabase)
    ));
    assert!(matches!(
        library.get_recommend_likes(),
        Err(Error::MissingExtendedDatabase)
    ));
    assert!(matches!(
        library.get_recommend_like_by_content_id(1),
        Err(Error::MissingExtendedDatabase)
    ));
    assert!(matches!(
        library.insert_recommend_like(NewRecommendedLike::new(3, 4, 0, 0, 0)),
        Err(Error::MissingExtendedDatabase)
    ));
    assert!(matches!(
        library.create_recommend_like(3, 4, 0, 0, 0),
        Err(Error::MissingExtendedDatabase)
    ));
    assert!(matches!(
        library.delete_recommend_like(recommendation),
        Err(Error::MissingExtendedDatabase)
    ));
    assert_eq!(library.standard().read_stats().page_reads, 0);
    assert_eq!(
        fs::read(temp.path().join("exportExt.pdb"))?,
        extended_before
    );
    Ok(())
}

#[test]
fn key_methods_preserve_secondary_ids_references_and_explicit_saving() -> Result<()> {
    use rbox_devlib::models::{ExportProperty, NewContent, NewKey};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert!(library.get_keys()?.is_empty());
        assert!(library.get_key_by_id(99)?.is_none());
        assert!(library.get_key_by_name("Missing")?.is_none());
        let key = library.insert_key(NewKey::new("Am", 23))?;
        assert!(matches!(
            library.create_key("Am", 42),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        // Secondary IDs are independent values, not unique keys.
        let keep = library.create_key("Cm", 23)?;
        assert_eq!((key.id, keep.id), (1, 2));
        assert_eq!(library.get_keys()?.len(), 2);
        assert_eq!(library.get_key_by_id(key.id)?.unwrap().secondary_id, 23);
        assert_eq!(library.get_key_by_name("Am")?.unwrap().id, key.id);
        assert!(library.get_key_by_name("am")?.is_none());
        let mut invalid = key.clone();
        invalid.id = 99;
        assert!(matches!(
            library.update_key(invalid),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut duplicate = key.clone();
        duplicate.name = keep.name.clone();
        duplicate.secondary_id = 42;
        assert!(matches!(
            library.update_key(duplicate),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        let unchanged = library.get_key_by_id(key.id)?.unwrap();
        assert_eq!(
            (unchanged.name.as_str(), unchanged.secondary_id),
            ("Am", 23)
        );
        let mut edited = key;
        edited.name = "A minor".into();
        let updated = library.update_key(edited)?;
        assert_eq!(updated.secondary_id, 23);
        assert!(library.get_key_by_name("Am")?.is_none());
        let mut edited = updated;
        edited.secondary_id = u32::MAX;
        let updated = library.update_key(edited)?;
        assert_eq!(
            (updated.name.as_str(), updated.secondary_id),
            ("A minor", u32::MAX)
        );
        assert_eq!(library.get_key_by_name("A minor")?.unwrap().id, updated.id);

        let mut input = NewContent::new("/Contents/first.wav");
        input.key_id = updated.id;
        let content = library.insert_content(input)?;
        let mut input = NewContent::new("/Contents/keep.wav");
        input.key_id = keep.id;
        let other = library.insert_content(input)?;
        assert_eq!(
            library.get_track_by_id(content.id)?.unwrap().key.as_deref(),
            Some("A minor")
        );
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        drop(library);

        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let saved = library.get_key_by_id(updated.id)?.unwrap();
        assert_eq!(
            (saved.name.as_str(), saved.secondary_id),
            ("A minor", u32::MAX)
        );
        let before_delete = fs::read(&path)?;
        assert_eq!(library.delete_key(99)?, 0);
        assert!(matches!(
            library.delete_key(0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(library.delete_key(updated.id)?, 2);
        assert_eq!(library.delete_key(updated.id)?, 0);
        assert!(library.get_key_by_id(updated.id)?.is_none());
        assert_eq!(library.get_content_by_id(content.id)?.unwrap().key_id, 0);
        assert_eq!(
            library.get_content_by_id(other.id)?.unwrap().key_id,
            keep.id
        );
        assert!(library.get_track_by_id(content.id)?.unwrap().key.is_none());
        assert_eq!(
            ExportProperty::first(library.standard_mut())?
                .unwrap()
                .track_count,
            2
        );
        assert_eq!(fs::read(&path)?, before_delete);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);

        let mut reopened = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert_eq!(reopened.get_keys()?.len(), 1);
        assert!(reopened.get_key_by_id(updated.id)?.is_none());
        let saved = reopened.get_key_by_id(keep.id)?.unwrap();
        assert_eq!((saved.name.as_str(), saved.secondary_id), ("Cm", 23));
        assert_eq!(reopened.get_content_by_id(content.id)?.unwrap().key_id, 0);
        assert_eq!(
            reopened.get_content_by_id(other.id)?.unwrap().key_id,
            keep.id
        );
        assert_eq!(reopened.get_contents()?.len(), 2);
        assert_eq!(
            ExportProperty::first(reopened.standard_mut())?
                .unwrap()
                .track_count,
            2
        );
    }
    Ok(())
}

#[test]
fn history_methods_preserve_occurrences_order_and_explicit_saving() -> Result<()> {
    use rbox_devlib::models::{ExportProperty, NewHistory, NewHistoryContent};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert!(library.get_histories()?.is_empty());
        assert!(library.get_history_by_id(99)?.is_none());
        assert!(library.get_history_contents(99)?.is_empty());
        assert!(library.get_history_entries(99)?.is_empty());
        let history = library.insert_history(NewHistory::new("Set"))?;
        let keep = library.create_history("Other set")?;
        assert_eq!((history.id, keep.id), (1, 2));
        assert_eq!(library.get_histories()?.len(), 2);
        let mut invalid = history.clone();
        invalid.id = 99;
        assert!(matches!(
            library.update_history(invalid),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut edited = history;
        edited.name = "Renamed set".into();
        let history = library.update_history(edited)?;
        assert_eq!(history.name, "Renamed set");
        assert_eq!(
            library.get_history_by_id(history.id)?.unwrap().name,
            history.name
        );
        let first = library.create_content("/Contents/first.wav")?;
        let second = library.create_content("/Contents/second.wav")?;
        assert!(matches!(
            library.create_history_content(0, first.id, 1),
            Err(Error::InvalidModelField {
                field: "history_id",
                ..
            })
        ));
        assert!(matches!(
            library.create_history_content(history.id, 0, 1),
            Err(Error::InvalidModelField {
                field: "content_id",
                ..
            })
        ));
        library.insert_history_content(NewHistoryContent::new(history.id, first.id, 9))?;
        library.create_history_content(history.id, second.id, 1)?;
        library.create_history_content(history.id, first.id, 1)?;
        library.create_history_content(history.id, first.id, 9)?;
        library.create_history_content(keep.id, second.id, 4)?;
        assert_eq!(
            library
                .get_history_entries(history.id)?
                .iter()
                .map(|row| (row.content_id, row.seq))
                .collect::<Vec<_>>(),
            vec![(second.id, 1), (first.id, 1), (first.id, 9), (first.id, 9)]
        );
        assert_eq!(
            library
                .get_history_contents(history.id)?
                .iter()
                .map(|row| row.id)
                .collect::<Vec<_>>(),
            vec![second.id, first.id, first.id, first.id]
        );
        let entry = library.get_history_entries(history.id)?.remove(2);
        assert_eq!(library.delete_history_content(entry)?, 1);
        assert_eq!(
            library
                .get_history_entries(history.id)?
                .iter()
                .map(|row| (row.content_id, row.seq))
                .collect::<Vec<_>>(),
            vec![(second.id, 1), (first.id, 1), (first.id, 9)]
        );
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        drop(library);

        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert_eq!(
            library.get_history_by_id(history.id)?.unwrap().name,
            "Renamed set"
        );
        assert_eq!(library.get_history_entries(history.id)?.len(), 3);
        assert_eq!(library.get_history_contents(history.id)?.len(), 3);
        let before_delete = fs::read(&path)?;
        assert_eq!(library.delete_history(99)?, 0);
        assert!(matches!(
            library.delete_history(0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(library.delete_history(history.id)?, 4);
        assert_eq!(library.delete_history(history.id)?, 0);
        assert!(library.get_history_by_id(history.id)?.is_none());
        assert!(library.get_history_entries(history.id)?.is_empty());
        assert!(library.get_history_contents(history.id)?.is_empty());
        assert_eq!(library.get_history_contents(keep.id)?[0].id, second.id);
        assert_eq!(library.get_contents()?.len(), 2);
        assert_eq!(
            ExportProperty::first(library.standard_mut())?
                .unwrap()
                .track_count,
            2
        );
        assert_eq!(fs::read(&path)?, before_delete);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);

        let mut reopened = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert!(reopened.get_history_by_id(history.id)?.is_none());
        assert!(reopened.get_history_entries(history.id)?.is_empty());
        assert_eq!(reopened.get_histories()?.len(), 1);
        assert_eq!(reopened.get_history_entries(keep.id)?[0].seq, 4);
        assert_eq!(reopened.get_history_contents(keep.id)?[0].id, second.id);
        assert_eq!(reopened.get_contents()?.len(), 2);
    }
    Ok(())
}

#[test]
fn history_contents_report_dangling_references() -> Result<()> {
    use rbox_devlib::models::Content;

    let temp = create_temp_dir();
    let mut library = DeviceLibrary::create(temp.path(), &options())?;
    let history = library.create_history("Set")?;
    library.create_history_content(history.id, 999, 1)?;
    assert_eq!(library.get_history_entries(history.id)?.len(), 1);
    assert!(matches!(
        library.get_history_contents(history.id),
        Err(Error::ModelNotFound {
            table: Content::TABLE,
            id: 999
        })
    ));
    Ok(())
}

#[test]
fn label_methods_preserve_constraints_references_and_explicit_saving() -> Result<()> {
    use rbox_devlib::models::{ExportProperty, NewContent, NewLabel};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert!(library.get_labels()?.is_empty());
        assert!(library.get_label_by_id(99)?.is_none());
        assert!(library.get_label_by_name("Missing")?.is_none());
        let label = library.insert_label(NewLabel::new("First Records"))?;
        assert!(matches!(
            library.create_label("First Records"),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        let keep = library.create_label("Other Records")?;
        assert_eq!((label.id, keep.id), (1, 2));
        assert_eq!(library.get_labels()?.len(), 2);
        assert_eq!(
            library.get_label_by_id(label.id)?.unwrap().name,
            "First Records"
        );
        assert_eq!(
            library.get_label_by_name("First Records")?.unwrap().id,
            label.id
        );
        assert!(library.get_label_by_name("first records")?.is_none());

        let mut invalid = label.clone();
        invalid.id = 99;
        assert!(matches!(
            library.update_label(invalid),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut duplicate = label.clone();
        duplicate.name = keep.name.clone();
        assert!(matches!(
            library.update_label(duplicate),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert_eq!(
            library.get_label_by_id(label.id)?.unwrap().name,
            "First Records"
        );
        let mut edited = label;
        edited.name = "Renamed Records".into();
        let updated = library.update_label(edited)?;
        assert_eq!(updated.name, "Renamed Records");
        assert_eq!(
            library.get_label_by_name("Renamed Records")?.unwrap().id,
            updated.id
        );
        assert!(library.get_label_by_name("First Records")?.is_none());

        let mut input = NewContent::new("/Contents/first records.wav");
        input.label_id = updated.id;
        let content = library.insert_content(input)?;
        let mut input = NewContent::new("/Contents/techno.wav");
        input.label_id = keep.id;
        let other = library.insert_content(input)?;
        assert_eq!(
            library
                .get_track_by_id(content.id)?
                .unwrap()
                .label
                .as_deref(),
            Some("Renamed Records")
        );
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        drop(library);

        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert_eq!(
            library.get_label_by_id(updated.id)?.unwrap().name,
            "Renamed Records"
        );
        let before_delete = fs::read(&path)?;
        assert_eq!(library.delete_label(99)?, 0);
        assert!(matches!(
            library.delete_label(0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(library.delete_label(updated.id)?, 2);
        assert_eq!(library.delete_label(updated.id)?, 0);
        assert!(library.get_label_by_id(updated.id)?.is_none());
        assert_eq!(library.get_content_by_id(content.id)?.unwrap().label_id, 0);
        assert_eq!(
            library.get_content_by_id(other.id)?.unwrap().label_id,
            keep.id
        );
        assert!(library
            .get_track_by_id(content.id)?
            .unwrap()
            .label
            .is_none());
        assert_eq!(
            ExportProperty::first(library.standard_mut())?
                .unwrap()
                .track_count,
            2
        );
        assert_eq!(fs::read(&path)?, before_delete);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);

        let mut reopened = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert_eq!(reopened.get_labels()?.len(), 1);
        assert!(reopened.get_label_by_id(updated.id)?.is_none());
        assert_eq!(
            reopened.get_label_by_id(keep.id)?.unwrap().name,
            "Other Records"
        );
        assert_eq!(reopened.get_content_by_id(content.id)?.unwrap().label_id, 0);
        assert_eq!(
            reopened.get_content_by_id(other.id)?.unwrap().label_id,
            keep.id
        );
        assert_eq!(reopened.get_contents()?.len(), 2);
        assert_eq!(
            ExportProperty::first(reopened.standard_mut())?
                .unwrap()
                .track_count,
            2
        );
    }
    Ok(())
}

#[test]
fn genre_methods_preserve_constraints_references_and_explicit_saving() -> Result<()> {
    use rbox_devlib::models::{ExportProperty, NewContent, NewGenre};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert!(library.get_genres()?.is_empty());
        assert!(library.get_genre_by_id(99)?.is_none());
        assert!(library.get_genre_by_name("Missing")?.is_none());
        let genre = library.insert_genre(NewGenre::new("House"))?;
        assert!(matches!(
            library.create_genre("House"),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        let keep = library.create_genre("Techno")?;
        assert_eq!((genre.id, keep.id), (1, 2));
        assert_eq!(library.get_genres()?.len(), 2);
        assert_eq!(library.get_genre_by_id(genre.id)?.unwrap().name, "House");
        assert_eq!(library.get_genre_by_name("House")?.unwrap().id, genre.id);
        assert!(library.get_genre_by_name("house")?.is_none());

        let mut invalid = genre.clone();
        invalid.id = 99;
        assert!(matches!(
            library.update_genre(invalid),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut duplicate = genre.clone();
        duplicate.name = keep.name.clone();
        assert!(matches!(
            library.update_genre(duplicate),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert_eq!(library.get_genre_by_id(genre.id)?.unwrap().name, "House");
        let mut edited = genre;
        edited.name = "Deep House".into();
        let updated = library.update_genre(edited)?;
        assert_eq!(updated.name, "Deep House");
        assert_eq!(
            library.get_genre_by_name("Deep House")?.unwrap().id,
            updated.id
        );
        assert!(library.get_genre_by_name("House")?.is_none());

        let mut input = NewContent::new("/Contents/house.wav");
        input.genre_id = updated.id;
        let content = library.insert_content(input)?;
        let mut input = NewContent::new("/Contents/techno.wav");
        input.genre_id = keep.id;
        let other = library.insert_content(input)?;
        assert_eq!(
            library
                .get_track_by_id(content.id)?
                .unwrap()
                .genre
                .as_deref(),
            Some("Deep House")
        );
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        drop(library);

        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert_eq!(
            library.get_genre_by_id(updated.id)?.unwrap().name,
            "Deep House"
        );
        let before_delete = fs::read(&path)?;
        assert_eq!(library.delete_genre(99)?, 0);
        assert!(matches!(
            library.delete_genre(0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(library.delete_genre(updated.id)?, 2);
        assert_eq!(library.delete_genre(updated.id)?, 0);
        assert!(library.get_genre_by_id(updated.id)?.is_none());
        assert_eq!(library.get_content_by_id(content.id)?.unwrap().genre_id, 0);
        assert_eq!(
            library.get_content_by_id(other.id)?.unwrap().genre_id,
            keep.id
        );
        assert!(library
            .get_track_by_id(content.id)?
            .unwrap()
            .genre
            .is_none());
        assert_eq!(
            ExportProperty::first(library.standard_mut())?
                .unwrap()
                .track_count,
            2
        );
        assert_eq!(fs::read(&path)?, before_delete);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);

        let mut reopened = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert_eq!(reopened.get_genres()?.len(), 1);
        assert!(reopened.get_genre_by_id(updated.id)?.is_none());
        assert_eq!(reopened.get_genre_by_id(keep.id)?.unwrap().name, "Techno");
        assert_eq!(reopened.get_content_by_id(content.id)?.unwrap().genre_id, 0);
        assert_eq!(
            reopened.get_content_by_id(other.id)?.unwrap().genre_id,
            keep.id
        );
        assert_eq!(reopened.get_contents()?.len(), 2);
        assert_eq!(
            ExportProperty::first(reopened.standard_mut())?
                .unwrap()
                .track_count,
            2
        );
    }
    Ok(())
}

#[test]
fn content_queries_updates_and_tracks_resolve_metadata_and_save_explicitly() -> Result<()> {
    use rbox_core::TrackSource;
    use rbox_devlib::models::{ExportProperty, NewContent};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert!(library.get_tracks_by_ids(&[])?.is_empty());
        assert!(library
            .get_contents_by_master_content_ids(&[], 42)?
            .is_empty());
        assert_eq!(library.standard().read_stats().page_reads, 0);
        assert!(library.get_contents()?.is_empty());
        assert!(library.get_tracks()?.is_empty());
        assert!(library.get_content_by_id(99)?.is_none());
        assert!(library.get_content_with_relations(99)?.is_none());
        assert!(library.get_track_by_id(99)?.is_none());
        assert!(library.get_track_by_path("/Missing.wav")?.is_none());
        assert!(matches!(
            library.create_content(""),
            Err(Error::InvalidModelField { .. })
        ));

        let artist = library.create_artist("Artist")?;
        let album_artist = library.create_artist("Album artist")?;
        let album = library.create_album("Album", Some(album_artist.id))?;
        let artwork = library.create_artwork("/Artwork/cover.jpg")?;
        let mut input = NewContent::new("/Contents/first.wav");
        input.artist_id = artist.id;
        input.album_id = album.id;
        input.artwork_id = artwork.id;
        input.genre_id = 999; // Missing metadata remains unresolved.
        input.master_content_id = u32::MAX;
        input.master_db_id = 3_499_321_917;
        input.title = "First".into();
        input.bpmx100 = 12835;
        input.length = 241;
        input.file_size = 1234567;
        input.analysis_path = "/PIONEER/USBANLZ/P000/00000001/ANLZ0000.DAT".into();
        input.isrc = "USABC2400001".into();
        let first = library.insert_content(input.clone())?;
        let second = library.create_content("/Contents/second.wav")?;
        let duplicate = library.insert_content(input.clone())?;
        input.master_db_id = 42;
        let other_library = library.insert_content(input)?;
        assert_eq!(
            (first.id, second.id, duplicate.id, other_library.id),
            (1, 2, 3, 4)
        );
        assert_eq!(library.get_contents()?.len(), 4);
        assert_eq!(
            ExportProperty::first(library.standard_mut())?
                .unwrap()
                .track_count,
            4
        );
        assert_eq!(
            library
                .get_content_by_id(first.id)?
                .unwrap()
                .raw_record()
                .decode()?,
            first.raw_record().decode()?
        );
        assert_eq!(
            library
                .get_content_by_master_content_id(u32::MAX, first.master_db_id)?
                .unwrap()
                .id,
            first.id
        );
        assert_eq!(
            library
                .get_content_by_master_content_id(u32::MAX, 42)?
                .unwrap()
                .id,
            other_library.id
        );
        assert!(library
            .get_content_by_master_content_id(u32::MAX, 43)?
            .is_none());
        assert!(library
            .get_content_by_master_content_id(0, first.master_db_id)?
            .is_none());
        assert!(library
            .get_content_by_master_content_id(u32::MAX, 0)?
            .is_none());
        assert_eq!(
            library
                .get_contents_by_master_content_ids(
                    &[u32::MAX, 0, 123, u32::MAX],
                    first.master_db_id
                )?
                .iter()
                .map(|row| row.id)
                .collect::<Vec<_>>(),
            vec![first.id, duplicate.id]
        );
        assert!(library
            .get_contents_by_master_content_ids(&[u32::MAX], 0)?
            .is_empty());

        let view = library.get_content_with_relations(first.id)?.unwrap();
        assert_eq!(view.artist.unwrap().name, "Artist");
        assert_eq!(view.album.unwrap().name, "Album");
        assert_eq!(view.album_artist.unwrap().name, "Album artist");
        assert!(view.genre.is_none());
        assert_eq!(library.get_contents_with_relations()?.len(), 4);
        let track = library.get_track_by_id(first.id)?.unwrap();
        assert_eq!(track.source, TrackSource::DeviceLib);
        assert_eq!(track.artist.as_deref(), Some("Artist"));
        assert_eq!(track.album_artist.as_deref(), Some("Album artist"));
        assert_eq!(track.album.as_deref(), Some("Album"));
        assert_eq!(track.artwork_path, Some("/Artwork/cover.jpg".into()));
        assert_eq!(
            track.analysis_data_path,
            Some(first.analysis_path.clone().into())
        );
        assert_eq!(
            (track.master_content_id, track.master_db_id),
            (Some(u32::MAX), Some(first.master_db_id))
        );
        assert_eq!(
            (track.bpm, track.length, track.file_size),
            (Some(128.35), Some(241.0), Some(1234567))
        );
        assert_eq!(track.isrc.as_deref(), Some("USABC2400001"));
        assert!(track.genre.is_none());
        assert_eq!(
            library
                .get_track_by_path(r"\Contents\first.wav")?
                .unwrap()
                .id,
            Some(first.id)
        );
        assert!(library.get_track_by_path("/contents/first.wav")?.is_none());
        assert_eq!(library.get_tracks()?.len(), 4);
        assert_eq!(
            library
                .get_tracks_by_ids(&[duplicate.id, first.id, 999, first.id])?
                .iter()
                .map(|row| row.id)
                .collect::<Vec<_>>(),
            vec![Some(first.id), Some(duplicate.id)]
        );

        let mut invalid = first.clone();
        invalid.id = 99;
        assert!(matches!(
            library.update_content(invalid),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut edited = first;
        edited.title = "Updated title ".repeat(80);
        edited.set_isrc("GBAYE6700149")?;
        let updated = library.update_content(edited)?;
        assert_eq!(
            library.get_track_by_id(updated.id)?.unwrap().title,
            Some(updated.title.clone())
        );
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);
        let mut reopened = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let saved = reopened.get_content_by_id(updated.id)?.unwrap();
        assert_eq!(saved.title, updated.title);
        assert_eq!(saved.isrc()?, "GBAYE6700149");
        assert_eq!(
            ExportProperty::first(reopened.standard_mut())?
                .unwrap()
                .track_count,
            4
        );
        assert_eq!(
            reopened
                .get_track_by_id(updated.id)?
                .unwrap()
                .album_artist
                .as_deref(),
            Some("Album artist")
        );
    }
    Ok(())
}

#[test]
fn track_helpers_propagate_conversion_errors_without_hiding_raw_content() -> Result<()> {
    use rbox_devlib::models::NewContent;

    let temp = create_temp_dir();
    let mut library = DeviceLibrary::create(temp.path(), &options())?;
    let mut input = NewContent::new("/Contents/unknown-rating.wav");
    input.rating = 6;
    let row = library.insert_content(input)?;
    assert_eq!(library.get_content_by_id(row.id)?.unwrap().rating, 6);
    assert!(library.get_track_by_id(row.id).is_err());
    assert!(library.get_track_by_path(&row.path).is_err());
    assert!(library.get_tracks_by_ids(&[row.id]).is_err());
    assert!(library.get_tracks().is_err());
    assert!(library.get_tracks_by_ids(&[999])?.is_empty());
    Ok(())
}

#[test]
fn content_deletion_cleans_available_files_and_preserves_resources() -> Result<()> {
    use rbox_devlib::models::{
        ExportProperty, History, HistoryContent, MyTag, MyTagContent, NewHistory,
        NewHistoryContent, NewMyTag, NewMyTagContent, NewPlaylist, NewPlaylistContent,
        NewRecommendedLike, Playlist, PlaylistContent, RecommendedLike,
    };

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let mut library = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        let resource = temp.path().join("keep.wav");
        fs::write(&resource, b"audio resource")?;
        let removed = library.create_content(resource.to_string_lossy().replace('\\', "/"))?;
        let keep = library.create_content("/Contents/keep.wav")?;
        let playlist = NewPlaylist::playlist("Playlist").insert(library.standard_mut())?;
        let history = NewHistory::new("History").insert(library.standard_mut())?;
        for id in [removed.id, keep.id] {
            NewPlaylistContent::new(playlist.id, id, None).insert(library.standard_mut())?;
            NewHistoryContent::new(history.id, id, id).insert(library.standard_mut())?;
        }
        if with_extended {
            NewMyTag::tag(100, "Tag").insert(library.extended_mut()?)?;
            for id in [removed.id, keep.id] {
                NewMyTagContent::new(id, 100).insert(library.extended_mut()?)?;
            }
            for (a, b) in [
                (removed.id, keep.id),
                (keep.id, removed.id),
                (keep.id, keep.id),
            ] {
                NewRecommendedLike::new(a, b, 0, 0, 0).insert(library.extended_mut()?)?;
            }
        }
        library.save()?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert_eq!(library.delete_content(999)?, 0);
        assert!(matches!(
            library.delete_content(0),
            Err(Error::InvalidModelField { .. })
        ));
        assert_eq!(
            library.delete_content(removed.id)?,
            if with_extended { 6 } else { 3 }
        );
        assert_eq!(library.delete_content(removed.id)?, 0);
        assert!(library.get_content_by_id(removed.id)?.is_none());
        assert_eq!(library.get_content_by_id(keep.id)?.unwrap().path, keep.path);
        assert_eq!(
            ExportProperty::first(library.standard_mut())?
                .unwrap()
                .track_count,
            1
        );
        assert_eq!(fs::read(&path)?, before);
        assert_eq!(fs::read(&extended)?, extended_before);
        library.save()?;
        drop(library);

        let mut reopened = DeviceLibrary::from_paths(&path, with_extended.then_some(&extended))?;
        assert!(reopened.get_content_by_id(removed.id)?.is_none());
        assert_eq!(reopened.get_contents()?.len(), 1);
        assert_eq!(
            ExportProperty::first(reopened.standard_mut())?
                .unwrap()
                .track_count,
            1
        );
        let entries = PlaylistContent::all(reopened.standard_mut())?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content_id, keep.id);
        let entries = HistoryContent::all(reopened.standard_mut())?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content_id, keep.id);
        assert!(Playlist::find(reopened.standard_mut(), &playlist.id)?.is_some());
        assert!(History::find(reopened.standard_mut(), &history.id)?.is_some());
        if with_extended {
            let tags = MyTagContent::all(reopened.extended_mut()?)?;
            assert_eq!(tags.len(), 1);
            assert_eq!(tags[0].content_id, keep.id);
            assert!(MyTag::find(reopened.extended_mut()?, &100)?.is_some());
            let pairs = RecommendedLike::all(reopened.extended_mut()?)?;
            assert_eq!(pairs.len(), 1);
            assert_eq!(
                (pairs[0].content_id_1, pairs[0].content_id_2),
                (keep.id, keep.id)
            );
        } else {
            assert_eq!(fs::read(&extended)?, extended_before);
        }
        assert_eq!(fs::read(&resource)?, b"audio resource");
    }
    Ok(())
}

#[test]
fn color_methods_preserve_fixed_ids_and_save_explicitly() -> Result<()> {
    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let extended_path = with_extended.then_some(extended.as_path());
        let mut library = DeviceLibrary::from_paths(&path, extended_path)?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        let colors = library.get_colors()?;
        assert_eq!(colors.len(), 8);
        assert!(library.get_color_by_id(u16::MAX)?.is_none());
        assert!(library.get_color_by_name("Missing color")?.is_none());
        let color = colors[0].clone();
        assert_eq!(library.get_color_by_id(color.id)?, Some(color.clone()));
        assert_eq!(library.get_color_by_name(&color.name)?, Some(color.clone()));

        let mut invalid = color.clone();
        invalid.id = u16::MAX;
        assert!(matches!(
            library.update_color(invalid),
            Err(Error::InvalidModelField { field: "id", .. })
        ));
        let mut duplicate = color.clone();
        duplicate.name = colors[1].name.clone();
        assert!(matches!(
            library.update_color(duplicate),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert_eq!(library.get_colors()?, colors);

        let mut edited = color.clone();
        edited.name = "Custom color".into();
        let updated = library.update_color(edited)?;
        assert_eq!(updated.id, color.id);
        assert_eq!(updated.name, "Custom color");
        assert_eq!(library.get_color_by_name("Custom color")?, Some(updated));
        assert!(library.get_color_by_name("custom color")?.is_none());
        assert!(library.get_color_by_name(&color.name)?.is_none());
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);

        let mut reopened = DeviceLibrary::from_paths(&path, extended_path)?;
        let saved = reopened.get_colors()?;
        assert_eq!(saved.len(), colors.len());
        for original in colors {
            let saved = reopened.get_color_by_id(original.id)?.unwrap();
            let expected_name = if original.id == color.id {
                "Custom color"
            } else {
                &original.name
            };
            assert_eq!(saved.name, expected_name);
        }
    }
    Ok(())
}

#[test]
fn browse_methods_update_initialized_settings_and_save_explicitly() -> Result<()> {
    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let extended_path = with_extended.then_some(extended.as_path());
        let mut library = DeviceLibrary::from_paths(&path, extended_path)?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        let columns = library.get_browse_columns()?;
        let categories = library.get_browse_categories()?;
        let sorts = library.get_browse_sorts()?;
        assert!(!columns.is_empty() && !categories.is_empty() && !sorts.is_empty());
        assert!(library.get_browse_column_by_id(u16::MAX)?.is_none());
        assert!(library
            .get_browse_column_by_name("Missing label")?
            .is_none());
        assert!(library
            .get_browse_categories_by_column(u16::MAX)?
            .is_empty());
        assert!(library.get_browse_sorts_by_column(u16::MAX)?.is_empty());

        let column = columns[0].clone();
        assert_eq!(
            library.get_browse_column_by_id(column.id)?,
            Some(column.clone())
        );
        assert_eq!(
            library.get_browse_column_by_name(&column.name)?.unwrap().id,
            column.id
        );
        let mut invalid = column.clone();
        invalid.code ^= 1;
        assert!(matches!(
            library.update_browse_column(invalid),
            Err(Error::InvalidModelField { field: "code", .. })
        ));
        let mut edited = column;
        edited.name = "Custom label".into();
        let column = library.update_browse_column(edited)?;
        assert_eq!(
            library
                .get_browse_column_by_name("Custom label")?
                .unwrap()
                .id,
            column.id
        );
        assert!(library.get_browse_column_by_name("custom label")?.is_none());

        let category = categories[0].clone();
        assert_eq!(
            library
                .get_browse_categories_by_column(category.column_id)?
                .len(),
            categories
                .iter()
                .filter(|row| row.column_id == category.column_id)
                .count()
        );
        let mut invalid = category.clone();
        invalid.column_id = u16::MAX;
        assert!(matches!(
            library.update_browse_category(invalid),
            Err(Error::InvalidModelField {
                field: "column_id",
                ..
            })
        ));
        let mut edited = category;
        edited.state ^= 1;
        edited.position = 42;
        let category = library.update_browse_category(edited)?;
        assert_eq!(
            library.get_browse_categories_by_column(category.column_id)?[0].state,
            category.state
        );

        let sort = sorts[0].clone();
        assert_eq!(
            library.get_browse_sorts_by_column(sort.column_id)?.len(),
            sorts
                .iter()
                .filter(|row| row.column_id == sort.column_id)
                .count()
        );
        let mut invalid = sort.clone();
        invalid.selector ^= 1;
        assert!(matches!(
            library.update_browse_sort(invalid),
            Err(Error::InvalidModelField {
                field: "selector",
                ..
            })
        ));
        let mut edited = sort;
        // Exercise opaque-byte forwarding without assigning a meaning to the bits.
        edited.raw_tail ^= 0x8000_0000;
        let sort = library.update_browse_sort(edited)?;
        assert_eq!(
            library.get_browse_sorts_by_column(sort.column_id)?[0].raw_tail,
            sort.raw_tail
        );

        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);
        let mut reopened = DeviceLibrary::from_paths(&path, extended_path)?;
        assert_eq!(reopened.get_browse_columns()?.len(), columns.len());
        assert_eq!(reopened.get_browse_categories()?.len(), categories.len());
        assert_eq!(reopened.get_browse_sorts()?.len(), sorts.len());
        assert_eq!(
            reopened.get_browse_column_by_id(column.id)?.unwrap().name,
            "Custom label"
        );
        let saved_category = reopened
            .get_browse_categories_by_column(category.column_id)?
            .remove(0);
        assert_eq!(
            (saved_category.state, saved_category.position),
            (category.state, 42)
        );
        assert_eq!(
            reopened.get_browse_sorts_by_column(sort.column_id)?[0].raw_tail,
            sort.raw_tail
        );
    }
    Ok(())
}

#[test]
fn artwork_methods_preserve_references_and_save_explicitly() -> Result<()> {
    use rbox_devlib::models::{Content, NewArtwork, NewContent};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let extended_path = with_extended.then_some(extended.as_path());
        let mut library = DeviceLibrary::from_paths(&path, extended_path)?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert!(library.get_artworks()?.is_empty());
        assert!(library.get_artwork_by_id(99)?.is_none());
        assert!(library.get_artwork_by_path("/Missing.jpg")?.is_none());
        assert!(matches!(
            library.create_artwork(""),
            Err(Error::InvalidModelField { field: "path", .. })
        ));

        let artwork = library.insert_artwork(NewArtwork::new("/Artwork/cover.jpg"))?;
        let keep = library.create_artwork("/Artwork/cover.jpg")?;
        assert_eq!((artwork.id, keep.id), (1, 2));
        assert_eq!(library.get_artworks()?.len(), 2);
        assert_eq!(
            library.get_artwork_by_id(artwork.id)?.unwrap().path,
            artwork.path
        );
        // Duplicate paths are valid; lookup returns the first occurrence.
        assert_eq!(
            library.get_artwork_by_path(&artwork.path)?.unwrap().id,
            artwork.id
        );
        assert!(library.get_artwork_by_path("/artwork/cover.jpg")?.is_none());
        assert!(library
            .get_artwork_by_path(r"\Artwork\cover.jpg")?
            .is_none());
        let mut invalid = artwork.clone();
        invalid.path.clear();
        assert!(matches!(
            library.update_artwork(invalid),
            Err(Error::InvalidModelField { field: "path", .. })
        ));

        let artwork_id = artwork.id;
        let mut edited = artwork;
        edited.path = "/Artwork/renamed.jpg".into();
        let updated = library.update_artwork(edited)?;
        assert_eq!(
            library.get_artwork_by_path(&updated.path)?,
            Some(updated.clone())
        );
        assert_eq!(
            library.get_artwork_by_path(&keep.path)?.unwrap().id,
            keep.id
        );
        let mut input = NewContent::new("/Contents/track.wav");
        input.artwork_id = artwork_id;
        let content = input.insert(library.standard_mut())?;
        let mut input = NewContent::new("/Contents/keep.wav");
        input.artwork_id = keep.id;
        let other = input.insert(library.standard_mut())?;
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        drop(library);

        let mut library = DeviceLibrary::from_paths(&path, extended_path)?;
        assert_eq!(
            library.get_artwork_by_id(artwork_id)?.unwrap().path,
            updated.path
        );
        let saved = fs::read(&path)?;
        assert_eq!(library.delete_artwork(artwork_id)?, 2);
        assert_eq!(library.delete_artwork(artwork_id)?, 0);
        assert!(library.get_artwork_by_id(artwork_id)?.is_none());
        assert_eq!(
            Content::find(library.standard_mut(), &content.id)?
                .unwrap()
                .artwork_id,
            0
        );
        assert_eq!(
            Content::find(library.standard_mut(), &other.id)?
                .unwrap()
                .artwork_id,
            keep.id
        );
        assert_eq!(fs::read(&path)?, saved);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);
        let mut reopened = DeviceLibrary::from_paths(&path, extended_path)?;
        assert!(reopened.get_artwork_by_path(&updated.path)?.is_none());
        assert_eq!(reopened.get_artworks()?.len(), 1);
        assert_eq!(
            reopened.get_artwork_by_id(keep.id)?.unwrap().path,
            keep.path
        );
        assert_eq!(
            Content::find(reopened.standard_mut(), &content.id)?
                .unwrap()
                .artwork_id,
            0
        );
    }
    Ok(())
}

#[test]
fn artist_methods_preserve_constraints_references_and_explicit_saving() -> Result<()> {
    use rbox_devlib::models::{Content, NewArtist, NewContent};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let extended_path = with_extended.then_some(extended.as_path());
        let mut library = DeviceLibrary::from_paths(&path, extended_path)?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert!(library.get_artists()?.is_empty());
        assert!(library.get_artist_by_id(99)?.is_none());
        assert!(library.get_artist_by_name("Missing")?.is_none());

        let artist = library.insert_artist(NewArtist::new("Artist"))?;
        let keep = library.create_artist("Other")?;
        assert_eq!((artist.id, keep.id), (1, 2));
        assert_eq!(library.get_artists()?.len(), 2);
        assert_eq!(library.get_artist_by_id(artist.id)?.unwrap().name, "Artist");
        assert_eq!(library.get_artist_by_name("Artist")?.unwrap().id, artist.id);
        assert!(library.get_artist_by_name("artist")?.is_none());
        assert!(matches!(
            library.create_artist("Artist"),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert!(matches!(
            library.insert_artist(NewArtist::new("Other")),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        let mut conflict = artist.clone();
        conflict.name = keep.name.clone();
        assert!(matches!(
            library.update_artist(conflict),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        let artist_id = artist.id;
        let mut edited = artist;
        edited.name = "Renamed".into();
        let updated = library.update_artist(edited)?;
        assert_eq!(library.get_artist_by_name("Renamed")?, Some(updated));
        assert!(library.get_artist_by_name("Artist")?.is_none());

        let album = library.create_album("Album", Some(artist_id))?;
        let mut input = NewContent::new("/Contents/track.wav");
        input.album_id = album.id;
        input.artist_id = artist_id;
        input.composer_id = artist_id;
        input.original_artist_id = artist_id;
        input.remixer_id = artist_id;
        let content = input.insert(library.standard_mut())?;
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        drop(library);

        let mut library = DeviceLibrary::from_paths(&path, extended_path)?;
        assert_eq!(
            library.get_artist_by_id(artist_id)?.unwrap().name,
            "Renamed"
        );
        let saved = fs::read(&path)?;
        assert_eq!(library.delete_artist(artist_id)?, 3); // Artist, album, content.
        assert_eq!(library.delete_artist(artist_id)?, 0);
        assert!(library.get_artist_by_id(artist_id)?.is_none());
        assert_eq!(library.get_artist_by_id(keep.id)?.unwrap().name, "Other");
        assert_eq!(library.get_album_by_id(album.id)?.unwrap().artist_id, 0);
        let cleared = Content::find(library.standard_mut(), &content.id)?.unwrap();
        assert_eq!(
            [
                cleared.artist_id,
                cleared.composer_id,
                cleared.original_artist_id,
                cleared.remixer_id
            ],
            [0; 4]
        );
        assert_eq!(cleared.album_id, album.id);
        assert_eq!(fs::read(&path)?, saved);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);
        let mut reopened = DeviceLibrary::from_paths(&path, extended_path)?;
        assert!(reopened.get_artist_by_id(artist_id)?.is_none());
        assert_eq!(reopened.get_artists()?.len(), 1);
        assert_eq!(reopened.get_album_by_id(album.id)?.unwrap().artist_id, 0);
        assert_eq!(
            Content::find(reopened.standard_mut(), &content.id)?
                .unwrap()
                .artist_id,
            0
        );
    }
    Ok(())
}

#[test]
fn album_methods_preserve_model_rules_and_save_explicitly() -> Result<()> {
    use rbox_devlib::models::{Content, NewAlbum, NewArtist, NewContent};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        drop(DeviceLibrary::create(temp.path(), &options())?);
        let path = temp.path().join("export.pdb");
        let extended = temp.path().join("exportExt.pdb");
        let mut library = DeviceLibrary::from_paths(
            &path,
            if with_extended {
                Some(extended.as_path())
            } else {
                None
            },
        )?;
        let before = fs::read(&path)?;
        let extended_before = fs::read(&extended)?;
        assert!(library.get_albums()?.is_empty());
        assert!(library.get_albums_with_relations()?.is_empty());
        assert!(library.get_album_by_id(99)?.is_none());
        assert!(library.get_album_with_relations(99)?.is_none());
        assert!(library.get_album_by_name("Missing")?.is_none());

        let artist = NewArtist::new("Album artist").insert(library.standard_mut())?;
        let album = library.insert_album(NewAlbum::new("Album").artist_id(artist.id))?;
        let unassigned = library.create_album("Album", None)?;
        let album_id = album.id;
        let unassigned_id = unassigned.id;
        assert_eq!((album_id, unassigned_id), (1, 2));
        assert_eq!(library.get_albums()?.len(), 2);
        let fetched = library.get_album_by_id(album_id)?.unwrap();
        assert_eq!(
            (fetched.id, fetched.artist_id, fetched.name.as_str()),
            (album_id, artist.id, "Album")
        );
        assert_eq!(library.get_album_by_name("Album")?.unwrap().id, album_id);
        assert!(library.get_album_by_name("album")?.is_none());
        let view = library.get_album_with_relations(album_id)?.unwrap();
        assert_eq!(view.artist.as_ref().map(|row| row.id), Some(artist.id));
        assert_eq!(
            view.artist.as_ref().map(|row| row.name.as_str()),
            Some("Album artist")
        );
        assert_eq!(view.album.id, album.id);
        let views = library.get_albums_with_relations()?;
        assert_eq!(views.len(), 2);
        assert!(views
            .iter()
            .find(|view| view.id == unassigned_id)
            .unwrap()
            .artist
            .is_none());

        assert!(matches!(
            library.create_album("Album", Some(artist.id)),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert!(matches!(
            library.create_album("Album", Some(0)),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        let mut conflicting = unassigned;
        conflicting.artist_id = 1;
        assert!(matches!(
            library.update_album(conflicting),
            Err(Error::UniqueConstraintViolation { .. })
        ));
        assert_eq!(
            library.get_album_by_id(unassigned_id)?.unwrap().artist_id,
            0
        );

        let mut edited = album;
        edited.name = "Renamed".into();
        let updated = library.update_album(edited)?;
        assert_eq!(library.get_album_by_name("Renamed")?, Some(updated));
        let mut content = NewContent::new("/Contents/track.wav");
        content.album_id = album_id;
        content.insert(library.standard_mut())?;
        assert_eq!(fs::read(&path)?, before);
        library.save()?;
        drop(library);

        let mut library = DeviceLibrary::from_paths(
            &path,
            if with_extended {
                Some(extended.as_path())
            } else {
                None
            },
        )?;
        assert_eq!(library.get_album_by_id(album_id)?.unwrap().name, "Renamed");
        let saved = fs::read(&path)?;
        assert_eq!(library.delete_album(album_id)?, 2);
        assert!(library.get_album_by_id(album_id)?.is_none());
        assert_eq!(
            Content::find(library.standard_mut(), &1)?.unwrap().album_id,
            0
        );
        assert_eq!(library.delete_album(album_id)?, 0);
        assert_eq!(fs::read(&path)?, saved);
        library.save()?;
        if with_extended {
            assert_eq!(library.extended()?.read_stats().page_reads, 0);
        }
        assert_eq!(fs::read(&extended)?, extended_before);
        drop(library);
        let mut reopened = DeviceLibrary::from_paths(&path, None)?;
        assert!(reopened.get_album_by_id(album_id)?.is_none());
        assert_eq!(reopened.get_albums()?.len(), 1);
        assert_eq!(
            Content::find(reopened.standard_mut(), &1)?
                .unwrap()
                .album_id,
            0
        );
    }
    Ok(())
}

#[test]
fn content_count_can_be_set_and_recalculated_without_saving_implicitly() -> Result<()> {
    use rbox_devlib::models::{ExportProperty, NewContent};
    let temp = create_temp_dir();
    let mut library = DeviceLibrary::create(temp.path(), &options())?;
    let path = temp.path().join("export.pdb");
    let before = fs::read(&path)?;
    ExportProperty::set_content_count(library.standard_mut(), 99)?;
    assert_eq!(
        ExportProperty::first(library.standard_mut())?
            .unwrap()
            .track_count,
        99
    );
    assert_eq!(
        ExportProperty::update_content_count(library.standard_mut())?,
        0
    );
    NewContent::new("/Contents/test.wav").insert(library.standard_mut())?;
    ExportProperty::set_content_count(library.standard_mut(), 99)?;
    assert_eq!(
        ExportProperty::update_content_count(library.standard_mut())?,
        1
    );
    assert_eq!(
        ExportProperty::first(library.standard_mut())?
            .unwrap()
            .track_count,
        1
    );
    assert_eq!(fs::read(&path)?, before);
    library.save()?;
    drop(library);
    let mut library = DeviceLibrary::new(temp.path())?;
    assert_eq!(
        ExportProperty::first(library.standard_mut())?
            .unwrap()
            .track_count,
        1
    );
    Ok(())
}

fn insert_rows(library: &mut DeviceLibrary) -> Result<()> {
    let mut tx = library.standard_mut().transaction();
    tx.insert(
        TableKind::Standard(PageType::Genres),
        &Row::Standard(StandardRow::Genre(NamedRow {
            id: 7,
            name: DeviceSqlString::encode("House", StringEncoding::ShortAscii)?,
        })),
    )?;
    tx.commit()?;
    let mut tx = library.extended_mut()?.transaction();
    tx.insert(
        TableKind::Extended(ExtPageType::RecommendLike),
        &Row::Extended(ExtendedRow::RecommendLike(RecommendLikeRow {
            unknown_00: 0,
            track_id_1: 1,
            track_id_2: 2,
            unknown_10: 0,
            unknown_14: 0,
        })),
    )?;
    tx.commit()
}

fn check_rows(library: &mut DeviceLibrary) -> Result<()> {
    let rows = library
        .standard_mut()
        .iter_raw(TableKind::Standard(PageType::Genres))?
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(rows.len(), 1);
    let Row::Standard(StandardRow::Genre(row)) = rows[0].decode()? else {
        panic!("expected genre");
    };
    assert_eq!(row.id, 7);
    assert_eq!(row.name.text()?, "House");
    let rows = library
        .extended_mut()?
        .iter_raw(TableKind::Extended(ExtPageType::RecommendLike))?
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(rows.len(), 1);
    let Row::Extended(ExtendedRow::RecommendLike(row)) = rows[0].decode()? else {
        panic!("expected recommendation");
    };
    assert_eq!((row.track_id_1, row.track_id_2), (1, 2));
    Ok(())
}

#[test]
fn creation_initializes_both_profiles_and_opens_lazily() -> Result<()> {
    let temp = create_temp_dir();
    let directory = temp.path().join("PIONEER/rekordbox");
    let mut library = DeviceLibrary::create(&directory, &options())?;
    assert!(library.has_extended());
    assert_eq!(library.standard().kind(), DatabaseKind::Standard);
    assert_eq!(library.extended()?.kind(), DatabaseKind::Extended);
    assert_eq!(library.standard().read_stats().page_reads, 0);
    assert_eq!(library.extended()?.read_stats().page_reads, 0);
    let stats = library.save()?;
    assert_eq!(stats.standard.bytes_written, 0);
    assert_eq!(stats.extended.unwrap().bytes_written, 0);
    let stats = library.save_pages()?;
    assert_eq!(stats.standard.bytes_written, 0);
    assert_eq!(stats.extended.unwrap().bytes_written, 0);

    let properties = library
        .extended_mut()?
        .iter_raw(TableKind::Extended(ExtPageType::DatabasePropertyOptions))?
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(properties.len(), 1);
    let Row::Extended(ExtendedRow::DatabaseProperty(property)) = properties[0].decode()? else {
        panic!("expected database property");
    };
    assert_eq!(property.header.my_tag_master_db_id, 42);
    assert_eq!(
        library
            .standard_mut()
            .iter_raw(TableKind::Standard(PageType::ColorLabels))?
            .count(),
        8
    );
    Ok(())
}

#[test]
fn missing_extended_is_optional_but_invalid_or_explicitly_missing_is_an_error() -> Result<()> {
    let temp = create_temp_dir();
    drop(DeviceLibrary::create(temp.path(), &options())?);
    let standard = temp.path().join("export.pdb");
    let extended = temp.path().join("exportExt.pdb");
    fs::remove_file(&extended)?;
    let mut library = DeviceLibrary::new(temp.path())?;
    assert!(!library.has_extended());
    assert!(matches!(
        library.extended(),
        Err(Error::MissingExtendedDatabase)
    ));
    assert!(matches!(
        library.extended_mut(),
        Err(Error::MissingExtendedDatabase)
    ));
    assert!(library.save()?.extended.is_none());
    assert!(library.save_pages()?.extended.is_none());
    assert!(DeviceLibrary::from_paths(&standard, Some(&extended)).is_err());
    assert!(matches!(
        DeviceLibrary::from_paths(&standard, Some(&standard)),
        Err(Error::AliasedDatabaseFiles(_))
    ));
    fs::write(&extended, b"invalid")?;
    assert!(DeviceLibrary::new(temp.path()).is_err());
    assert!(DeviceLibrary::from_paths(&standard, None).is_ok());
    let destination = temp.path().join("copy");
    library.save_as(&destination)?;
    assert!(!destination.join("exportExt.pdb").exists());
    assert!(!DeviceLibrary::new(destination)?.has_extended());
    Ok(())
}

#[test]
fn both_save_modes_persist_edits_and_clear_pending_pages() -> Result<()> {
    for selective in [false, true] {
        let temp = create_temp_dir();
        let mut library = DeviceLibrary::create(temp.path(), &options())?;
        let before = fs::read(temp.path().join("export.pdb"))?;
        insert_rows(&mut library)?;
        assert_eq!(fs::read(temp.path().join("export.pdb"))?, before);
        let dirty_standard = library.standard().modified_page_count() as u64;
        let dirty_extended = library.extended()?.modified_page_count() as u64;
        if selective {
            let stats = library.save_pages()?;
            assert_eq!(stats.standard.pages_written, dirty_standard);
            assert_eq!(stats.extended.unwrap().pages_written, dirty_extended);
        } else {
            let stats = library.save()?;
            assert_eq!(
                stats.standard.pages_written,
                library.standard().page_count()
            );
            assert_eq!(
                stats.extended.unwrap().pages_written,
                library.extended()?.page_count()
            );
        }
        assert_eq!(library.standard().modified_page_count(), 0);
        assert_eq!(library.extended()?.modified_page_count(), 0);
        assert_eq!(library.save()?.standard.bytes_written, 0);
        check_rows(&mut DeviceLibrary::new(temp.path())?)?;
    }
    Ok(())
}

#[test]
fn saved_copy_includes_edits_without_changing_sources_or_pending_state() -> Result<()> {
    let temp = create_temp_dir();
    let source = temp.path().join("source");
    let destination = temp.path().join("destination");
    let mut library = DeviceLibrary::create(&source, &options())?;
    let before_standard = fs::read(source.join("export.pdb"))?;
    let before_extended = fs::read(source.join("exportExt.pdb"))?;
    insert_rows(&mut library)?;
    library.save_as(&destination)?;
    check_rows(&mut DeviceLibrary::new(&destination)?)?;
    assert!(library.standard().modified_page_count() > 0);
    assert!(library.extended()?.modified_page_count() > 0);
    drop(library);
    assert_eq!(fs::read(source.join("export.pdb"))?, before_standard);
    assert_eq!(fs::read(source.join("exportExt.pdb"))?, before_extended);
    Ok(())
}

#[test]
fn creation_and_copy_refuse_either_existing_destination_before_publication() -> Result<()> {
    let temp = create_temp_dir();
    let mut source = DeviceLibrary::create(temp.path().join("source"), &options())?;
    for existing in ["export.pdb", "exportExt.pdb"] {
        let destination = temp.path().join(existing);
        fs::create_dir(&destination)?;
        fs::write(destination.join(existing), b"keep me")?;
        for result in [
            DeviceLibrary::create(&destination, &options()).map(|_| ()),
            source.save_as(&destination),
        ] {
            assert!(matches!(
                result,
                Err(Error::LibraryWrite {
                    standard_completed: false,
                    ..
                })
            ));
        }
        assert_eq!(fs::read(destination.join(existing))?, b"keep me");
        assert_eq!(fs::read_dir(destination)?.count(), 1);
    }
    Ok(())
}

#[test]
fn invalid_metadata_does_not_publish_database_files() -> Result<()> {
    let temp = create_temp_dir();
    let destination = temp.path().join("new");
    let mut options = options();
    options.device_name = "invalid\0name".into();
    assert!(matches!(
        DeviceLibrary::create(&destination, &options),
        Err(Error::EmbeddedNul)
    ));
    assert!(!destination.exists());
    Ok(())
}

#[test]
fn save_failure_reports_which_file_completed() -> Result<()> {
    for selective in [false, true] {
        for fail_extended in [false, true] {
            let temp = create_temp_dir();
            let mut library = DeviceLibrary::create(temp.path(), &options())?;
            insert_rows(&mut library)?;
            let filename = if fail_extended {
                "exportExt.pdb"
            } else {
                "export.pdb"
            };
            // Alter a temporary file's length to deterministically trigger the
            // engine's source-change guard, without changing any real fixture.
            OpenOptions::new()
                .append(true)
                .open(temp.path().join(filename))?
                .write_all(&[0])?;
            let result = if selective {
                library.save_pages().map(|_| ())
            } else {
                library.save().map(|_| ())
            };
            let Err(Error::LibraryWrite {
                database,
                standard_completed,
                source,
                ..
            }) = result
            else {
                panic!("expected a contextual write failure");
            };
            assert_eq!(
                database,
                if fail_extended {
                    DatabaseKind::Extended
                } else {
                    DatabaseKind::Standard
                }
            );
            assert_eq!(standard_completed, fail_extended);
            assert!(matches!(*source, Error::SourceChanged(_)));
            assert_eq!(library.standard().modified_page_count() == 0, fail_extended);
            assert!(library.extended()?.modified_page_count() > 0);
        }
    }
    Ok(())
}

#[test]
fn demo_files_copy_losslessly_without_loading_rows() -> Result<()> {
    let temp = create_temp_dir();
    let source = temp.path().join("source");
    let destination = temp.path().join("copy");
    fs::create_dir(&source)?;
    for name in ["export.pdb", "exportExt.pdb"] {
        fs::copy(
            export_dir(7).join("rekordbox").join(name),
            source.join(name),
        )?;
    }
    let mut library = DeviceLibrary::new(&source)?;
    library.save_as(&destination)?;
    for name in ["export.pdb", "exportExt.pdb"] {
        assert_eq!(
            fs::read(source.join(name))?,
            fs::read(destination.join(name))?
        );
    }
    assert_eq!(library.standard().read_stats().page_reads, 0);
    assert_eq!(library.extended()?.read_stats().page_reads, 0);
    assert!(DeviceLibrary::new(destination)?.has_extended());
    Ok(())
}

#[test]
fn explicit_paths_support_custom_names_and_independent_directories() -> Result<()> {
    let temp = create_temp_dir();
    let original = temp.path().join("original");
    drop(DeviceLibrary::create(&original, &options())?);
    let standard = temp.path().join("standard.db");
    let extended_directory = temp.path().join("extended");
    fs::create_dir(&extended_directory)?;
    let extended = extended_directory.join("extended.db");
    fs::copy(original.join("export.pdb"), &standard)?;
    fs::copy(original.join("exportExt.pdb"), &extended)?;
    let mut library = DeviceLibrary::from_paths(&standard, Some(&extended))?;
    insert_rows(&mut library)?;
    library.save()?;
    check_rows(&mut DeviceLibrary::from_paths(&standard, Some(&extended))?)?;
    assert!(DeviceLibrary::new(temp.path()).is_err());
    Ok(())
}

#[test]
fn copy_failure_reports_a_published_standard_file_without_rebasing_sources() -> Result<()> {
    let temp = create_temp_dir();
    let source = temp.path().join("source");
    let destination = temp.path().join("copy");
    let mut library = DeviceLibrary::create(&source, &options())?;
    insert_rows(&mut library)?;
    OpenOptions::new()
        .append(true)
        .open(source.join("exportExt.pdb"))?
        .write_all(&[0])?;
    assert!(matches!(
        library.save_as(&destination),
        Err(Error::LibraryWrite {
            operation: "save_as",
            database: DatabaseKind::Extended,
            standard_completed: true,
            ..
        })
    ));
    assert!(destination.join("export.pdb").is_file());
    assert!(!destination.join("exportExt.pdb").exists());
    assert!(library.standard().modified_page_count() > 0);
    assert!(library.extended()?.modified_page_count() > 0);
    Ok(())
}
