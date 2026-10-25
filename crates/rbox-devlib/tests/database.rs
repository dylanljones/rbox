// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Paired database opening, initialization, persistence, and partial failure reporting.

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
fn content_deletion_cleans_optional_extended_database_and_requires_save() -> Result<()> {
    use rbox_devlib::models::{Content, NewContent, NewRecommendedLike, RecommendedLike};

    for with_extended in [false, true] {
        let temp = create_temp_dir();
        let mut library = DeviceLibrary::create(temp.path(), &options())?;
        if !with_extended {
            drop(library);
            fs::remove_file(temp.path().join("exportExt.pdb"))?;
            library = DeviceLibrary::new(temp.path())?;
        }
        NewContent::new(1, "/Contents/test.wav").insert(library.standard_mut())?;
        if with_extended {
            NewRecommendedLike::new(1, 1, 0, 0, 0).insert(library.extended_mut()?)?;
        }
        library.save()?;
        let path = temp.path().join("export.pdb");
        let before = fs::read(&path)?;
        let extended_path = temp.path().join("exportExt.pdb");
        let extended_before = with_extended
            .then(|| fs::read(&extended_path))
            .transpose()?;

        assert_eq!(
            library.delete_content(&1)?,
            if with_extended { 2 } else { 1 }
        );
        assert!(!Content::id_exists(library.standard_mut(), &1)?);
        assert_eq!(fs::read(&path)?, before);
        if let Some(before) = extended_before {
            assert_eq!(fs::read(&extended_path)?, before);
        }
        library.save()?;
        drop(library);
        let mut library = DeviceLibrary::new(temp.path())?;
        assert!(!Content::id_exists(library.standard_mut(), &1)?);
        assert_eq!(library.has_extended(), with_extended);
        if with_extended {
            assert!(RecommendedLike::all(library.extended_mut()?)?.is_empty());
        }
    }
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
