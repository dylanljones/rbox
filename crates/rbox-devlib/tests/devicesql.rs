// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Lazy connection reads, database creation, transactions, and row allocation.

use binrw::{BinRead, BinWrite};
use rbox_devlib::binary::*;
use rbox_devlib::*;
use rbox_testing::data::{create_temp_dir, export_dir, testdata_dir};
use std::cell::Cell;
use std::fs;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::rc::Rc;

#[path = "support/empty_export.rs"]
#[allow(dead_code)]
mod empty_export;

mod support;

use support::decode_rows;

mod connection {
    //! Lazy engine reads, table namespaces, corruption boundaries, and native parity.

    use super::*;

    const PAGE_SIZE: usize = 512;
    const TRACKS: TableKind = TableKind::Standard(PageType::Tracks);
    const TAGS: TableKind = TableKind::Extended(ExtPageType::MyTags);

    // Independently specified little-endian bytes: index page 1 -> data page 3.
    // Unlinked page 2 and the last page's successor 4 must not be interpreted.
    fn database(kind: DatabaseKind) -> Vec<u8> {
        let code = if kind == DatabaseKind::Standard {
            0u32
        } else {
            3
        };
        let mut bytes = vec![0; 5 * PAGE_SIZE];
        bytes[4..8].copy_from_slice(&(PAGE_SIZE as u32).to_le_bytes());
        bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
        bytes[28..32].copy_from_slice(&code.to_le_bytes());
        bytes[36..40].copy_from_slice(&1u32.to_le_bytes());
        bytes[40..44].copy_from_slice(&3u32.to_le_bytes());
        bytes[PAGE_SIZE + 4..PAGE_SIZE + 8].copy_from_slice(&1u32.to_le_bytes());
        bytes[PAGE_SIZE + 8..PAGE_SIZE + 12].copy_from_slice(&code.to_le_bytes());
        bytes[PAGE_SIZE + 12..PAGE_SIZE + 16].copy_from_slice(&3u32.to_le_bytes());
        bytes[PAGE_SIZE + 27] = 0x64;
        let mut row = if kind == DatabaseKind::Standard {
            let mut row = vec![0; 137];
            row[..2].copy_from_slice(&0x24u16.to_le_bytes());
            row[56..60].copy_from_slice(&12850u32.to_le_bytes());
            row[72..76].copy_from_slice(&7u32.to_le_bytes());
            row[84..86].copy_from_slice(&90u16.to_le_bytes());
            row[92..94].copy_from_slice(&3u16.to_le_bytes());
            for offset in row[94..136].chunks_exact_mut(2) {
                offset.copy_from_slice(&136u16.to_le_bytes());
            }
            row[128..130].copy_from_slice(&137u16.to_le_bytes());
            row[136] = 3;
            row
        } else {
            let mut row = vec![0; 32];
            row[..2].copy_from_slice(&0x680u16.to_le_bytes());
            row[20..24].copy_from_slice(&7u32.to_le_bytes());
            row[27] = 1;
            row[28..32].copy_from_slice(&[3, 32, 31, 3]);
            row
        };
        row.extend_from_slice(&[9, b'A', b'B', b'C']);
        let page = &mut bytes[3 * PAGE_SIZE..4 * PAGE_SIZE];
        page[4..8].copy_from_slice(&3u32.to_le_bytes());
        page[8..12].copy_from_slice(&code.to_le_bytes());
        page[12..16].copy_from_slice(&4u32.to_le_bytes());
        page[24..28].copy_from_slice(&[1, 0x20, 0, 0x24]);
        page[30..32].copy_from_slice(&(row.len() as u16).to_le_bytes());
        page[40..40 + row.len()].copy_from_slice(&row);
        page[PAGE_SIZE - 6..].copy_from_slice(&[0, 0, 1, 0, 0, 0]);
        bytes[2 * PAGE_SIZE..3 * PAGE_SIZE].fill(0xff);
        bytes[4 * PAGE_SIZE..].fill(0xff);
        bytes
    }

    struct Counted {
        input: Cursor<Vec<u8>>,
        bytes: Rc<Cell<usize>>,
    }
    impl Read for Counted {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let count = self.input.read(output)?;
            self.bytes.set(self.bytes.get() + count);
            Ok(count)
        }
    }
    impl Seek for Counted {
        fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
            self.input.seek(from)
        }
    }

    #[test]
    fn opening_and_iterator_creation_read_only_page_zero_in_both_namespaces() -> Result<()> {
        for (kind, table) in [
            (DatabaseKind::Standard, TRACKS),
            (DatabaseKind::Extended, TAGS),
        ] {
            let bytes = Rc::new(Cell::new(0));
            let source = Counted {
                input: Cursor::new(database(kind)),
                bytes: bytes.clone(),
            };
            let mut db = DeviceSqlConnection::from_reader(source, kind)?;
            assert_eq!(db.kind(), kind);
            assert_eq!(bytes.get(), PAGE_SIZE);
            assert_eq!(db.read_stats().page_reads, 0);
            let mut rows = db.iter_raw(table)?;
            assert_eq!(bytes.get(), PAGE_SIZE);
            assert_eq!(rows.next().unwrap()?.slot().slot, 0);
            assert_eq!(bytes.get(), 3 * PAGE_SIZE);
            assert!(rows.next().is_none());
            assert!(rows.next().is_none());
            drop(rows);
            assert_eq!(db.read_stats().page_reads, 2);
            assert_eq!(db.iter_raw(table)?.count(), 1);
            assert_eq!(bytes.get(), 3 * PAGE_SIZE);
        }
        Ok(())
    }

    #[test]
    fn wrong_namespace_is_an_error_and_absent_table_is_empty() -> Result<()> {
        for (kind, wrong_table) in [
            (DatabaseKind::Standard, TAGS),
            (DatabaseKind::Extended, TRACKS),
        ] {
            let mut db = DeviceSqlConnection::from_reader(Cursor::new(database(kind)), kind)?;
            assert!(matches!(
                db.iter_raw(wrong_table),
                Err(Error::WrongDatabaseKind { .. })
            ));
            assert_eq!(db.read_stats().page_reads, 0);

            let mut bytes = database(kind);
            bytes[8..12].fill(0);
            let mut empty = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
            let table = if kind == DatabaseKind::Standard {
                TRACKS
            } else {
                TAGS
            };
            assert!(empty.iter_raw(table)?.next().is_none());
            assert_eq!(empty.read_stats().page_reads, 0);
        }
        Ok(())
    }

    #[test]
    fn cycles_and_invalid_directories_fail_lazily_and_fuse_iteration() -> Result<()> {
        for (kind, table) in [
            (DatabaseKind::Standard, TRACKS),
            (DatabaseKind::Extended, TAGS),
        ] {
            let mut bytes = database(kind);
            bytes[PAGE_SIZE + 12..PAGE_SIZE + 16].copy_from_slice(&1u32.to_le_bytes());
            let mut db = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
            let mut rows = db.iter_raw(table)?;
            assert!(matches!(rows.next().unwrap(), Err(Error::PageCycle { .. })));
            assert!(rows.next().is_none());

            let mut bytes = database(kind);
            bytes[3 * PAGE_SIZE + 30..3 * PAGE_SIZE + 32].copy_from_slice(&512u16.to_le_bytes());
            let mut db = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
            let mut rows = db.iter_raw(table)?;
            assert!(matches!(
                rows.next().unwrap(),
                Err(Error::InvalidRowDirectory { .. })
            ));
            assert!(rows.next().is_none());
        }
        Ok(())
    }

    #[test]
    fn demo_engine_reads_match_original_track_and_tag_windows() -> Result<()> {
        let temporary = create_temp_dir();
        for (name, kind, table, expected_count) in [
            ("export.pdb", DatabaseKind::Standard, TRACKS, 2),
            ("exportExt.pdb", DatabaseKind::Extended, TAGS, 28),
        ] {
            let path = temporary.path().join(name);
            fs::copy(export_dir(7).join("rekordbox").join(name), &path)?;
            let mut input = Cursor::new(fs::read(&path)?);
            let header = FileHeader::read_le(&mut input)?;
            let mut expected = Vec::new();
            for chain in read_table_chains(&mut input, &header)?
                .into_iter()
                .filter(|chain| chain.table.table_type == table.code())
            {
                for page in chain.pages.iter().filter(|page| !page.kind.is_index()) {
                    let directory = read_row_directory(&mut input, header.prefix.page_size, page)?;
                    for slot in directory.slots().filter(|slot| slot.live) {
                        expected.push(read_row(&mut input, kind, &directory, slot.slot)?.unwrap());
                    }
                }
            }
            let mut db = DeviceSqlConnection::open(&path, kind)?;
            assert_eq!(db.read_stats().page_reads, 0);
            let actual = db.iter_raw(table)?.collect::<Result<Vec<_>>>()?;
            assert_eq!(actual.len(), expected_count);
            assert_eq!(actual, expected);
        }
        Ok(())
    }
}

mod creation {
    //! Fresh container bytes, native index baselines, and create/edit/save lifecycles.

    use super::support::association;
    use super::*;

    #[test]
    fn standard_defaults_match_native_configuration_with_explicit_layout_overrides() -> Result<()> {
        let options = empty_export::standard()?;
        let mut db = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
        let mut bytes = Vec::new();
        db.write_to(&mut bytes)?;
        assert_eq!(
            bytes,
            fs::read(testdata_dir().join("export/empty/PIONEER/rekordbox/export.pdb"))?
        );
        Ok(())
    }

    #[test]
    fn standard_defaults_round_trip_device_names_and_leave_content_tables_empty() -> Result<()> {
        let temp = create_temp_dir();
        for (index, name) in ["My USB".to_owned(), "音楽 🎵".to_owned(), "x".repeat(127)]
            .iter()
            .enumerate()
        {
            let options = CreateOptions::standard("2026-10-24", name)?;
            let mut db =
                DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
            let path = temp.path().join(format!("standard-{index}.pdb"));
            db.save_as(&path)?;
            let mut saved = DeviceSqlConnection::open(path, DatabaseKind::Standard)?;
            assert_eq!(saved.header().prefix.unknown_10, 1);
            assert_eq!(saved.header().prefix.sequence, 2);
            for code in 0..20 {
                let rows = decode_rows(saved.iter_raw(DatabaseKind::Standard.table_kind(code))?)?;
                let count = match code {
                    6 => 8,
                    16 => 27,
                    17 => 22,
                    18 => 17,
                    19 => 1,
                    _ => 0,
                };
                assert_eq!(rows.len(), count, "table {code}");
                if code == 19 {
                    let Row::Standard(StandardRow::ExportProperty(property)) = &rows[0] else {
                        panic!("wrong property row")
                    };
                    assert_eq!(property.name.text()?, *name);
                    assert_eq!(property.date.text()?, "2026-10-24");
                    assert_eq!(property.track_count, 0);
                    assert_eq!(property.version.text()?, "1000");
                }
            }
            let mut input =
                Cursor::new(fs::read(temp.path().join(format!("standard-{index}.pdb")))?);
            let header = FileHeader::read_le(&mut input)?;
            for chain in read_table_chains(&mut input, &header)? {
                assert!(chain
                    .pages
                    .iter()
                    .all(|p| p.common.sequence < header.prefix.sequence));
            }
        }
        assert!(matches!(
            CreateOptions::standard("date", "bad\0name"),
            Err(Error::EmbeddedNul)
        ));
        assert!(matches!(
            CreateOptions::standard("é", "USB"),
            Err(Error::NonAscii)
        ));
        assert!(matches!(
            CreateOptions::standard("date", &"x".repeat(65_532)),
            Err(Error::StringTooLong)
        ));
        Ok(())
    }

    #[test]
    fn extended_defaults_preserve_explicit_identity_and_reserved_overrides() -> Result<()> {
        let temp = create_temp_dir();
        for id in [0, 1234, u32::MAX] {
            let mut options = CreateOptions::extended(id)?;
            let Row::Extended(ExtendedRow::DatabaseProperty(property)) =
                &mut options.tables[0].rows[0]
            else {
                panic!("wrong property row")
            };
            assert_eq!(property.header.unknown_words, [0; 5]);
            assert!(property
                .unknown_strings
                .iter()
                .all(|s| s.text().unwrap().is_empty()));
            // Creation defaults must not override an explicitly supplied raw value.
            property.header.unknown_words[2] = 0x12345678;
            options.unknown_10 = 5;
            let mut db =
                DeviceSqlConnection::create_with_options(DatabaseKind::Extended, &options)?;
            let path = temp.path().join(format!("extended-{id}.pdb"));
            db.save_as(&path)?;
            let mut saved = DeviceSqlConnection::open(&path, DatabaseKind::Extended)?;
            assert_eq!(saved.header().prefix.unknown_10, 5);
            assert_eq!(saved.header().prefix.sequence, 2);
            for code in 0..9 {
                let rows = decode_rows(saved.iter_raw(DatabaseKind::Extended.table_kind(code))?)?;
                assert_eq!(rows.len(), usize::from(code == 7));
                if code == 7 {
                    let Row::Extended(ExtendedRow::DatabaseProperty(property)) = &rows[0] else {
                        panic!("wrong property row")
                    };
                    assert_eq!(property.header.my_tag_master_db_id, id);
                    assert_eq!(property.header.unknown_words, [0, 0, 0x12345678, 0, 0]);
                    assert!(property
                        .unknown_strings
                        .iter()
                        .all(|s| s.text().unwrap().is_empty()));
                }
            }
        }
        assert!(DeviceSqlConnection::create_with_options(
            DatabaseKind::Extended,
            &CreateOptions::standard("2026-10-24", "USB")?
        )
        .is_err());
        assert!(DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &CreateOptions::extended(1234)?
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn empty_files_have_exact_header_directory_reservations_and_index_filler() -> Result<()> {
        for (kind, count) in [(DatabaseKind::Standard, 20u32), (DatabaseKind::Extended, 9)] {
            let mut db = DeviceSqlConnection::create(kind)?;
            assert_eq!(db.kind(), kind);
            assert_eq!(db.page_count(), u64::from(2 * count));
            assert_eq!(db.read_stats().page_reads, 0);
            assert_eq!(db.modified_page_count(), 0);
            let mut bytes = Vec::new();
            assert_eq!(db.write_to(&mut bytes)?, u64::from(2 * count * 4096));
            assert_eq!(db.read_stats().page_reads, 0);
            // Wire expectations are independent of the production header codecs.
            let mut prefix = Vec::new();
            for word in [0u32, 4096, count, 2 * count + 1, 1, 2, 0] {
                prefix.extend_from_slice(&word.to_le_bytes());
            }
            assert_eq!(&bytes[..28], prefix.as_slice());
            assert!(bytes[28 + count as usize * 16..4096]
                .iter()
                .all(|&b| b == 0));
            for code in 0..count {
                let first = 2 * code + 1;
                let at = 28 + code as usize * 16;
                for (field, expected) in [code, first + 1, first, first].into_iter().enumerate() {
                    assert_eq!(
                        &bytes[at + field * 4..at + field * 4 + 4],
                        &expected.to_le_bytes()
                    );
                }
                let page = &bytes[first as usize * 4096..(first as usize + 1) * 4096];
                assert_eq!(&page[4..8], &first.to_le_bytes());
                assert_eq!(&page[8..12], &code.to_le_bytes());
                assert_eq!(&page[12..16], &(first + 1).to_le_bytes());
                assert_eq!(&page[16..20], &1u32.to_le_bytes());
                assert_eq!(&page[24..32], &[0, 0, 0, 0x64, 0, 0, 0, 0]);
                assert_eq!(&page[32..40], &[0xff, 0x1f, 0xff, 0x1f, 0xec, 3, 0, 0]);
                assert_eq!(&page[40..44], &first.to_le_bytes());
                assert_eq!(
                    &page[44..60],
                    &[0xff, 0xff, 0xff, 3, 0xff, 0xff, 0xff, 3, 0, 0, 0, 0, 0, 0, 0xff, 0x1f]
                );
                assert!(page[60..4076]
                    .chunks_exact(4)
                    .all(|entry| entry == [0xf8, 0xff, 0xff, 0x1f]));
                assert_eq!(&page[4076..], &[0; 20]);
                if code + 1 < count {
                    assert!(
                        bytes[(first as usize + 1) * 4096..(first as usize + 2) * 4096]
                            .iter()
                            .all(|&b| b == 0)
                    );
                }
                assert!(db.iter_raw(kind.table_kind(code))?.next().is_none());
            }
            let mut input = Cursor::new(&bytes);
            let header = FileHeader::read_le(&mut input)?;
            let chains = read_table_chains(&mut input, &header)?;
            assert_eq!(chains.len(), count as usize);
            assert!(chains.iter().all(|c| c.pages.len() == 1));
            let mut another = DeviceSqlConnection::create(kind)?;
            let mut second = Vec::new();
            another.write_to(&mut second)?;
            assert_eq!(bytes, second); // No clock, random IDs, or fixture-dependent defaults.
        }
        Ok(())
    }

    #[test]
    fn created_index_pages_match_native_empty_pages_byte_for_byte() -> Result<()> {
        for (kind, name, page) in [
            (DatabaseKind::Standard, "export.pdb", 3usize), // empty Genres
            (DatabaseKind::Extended, "exportExt.pdb", 1),   // empty RecommendLike
        ] {
            let native = fs::read(export_dir(7).join("rekordbox").join(name))?;
            let mut db = DeviceSqlConnection::create(kind)?;
            let mut bytes = Vec::new();
            db.write_to(&mut bytes)?;
            let range = page * 4096..(page + 1) * 4096;
            assert_eq!(&bytes[range.clone()], &native[range]);
        }
        Ok(())
    }

    #[test]
    fn creation_supports_growth_rollback_saved_edits_and_destination_protection() -> Result<()> {
        let temp = create_temp_dir();
        for (kind, name) in [
            (DatabaseKind::Standard, "export.pdb"),
            (DatabaseKind::Extended, "exportExt.pdb"),
        ] {
            let mut db = DeviceSqlConnection::create(kind)?;
            let mut empty = Vec::new();
            db.write_to(&mut empty)?;
            let (table, _) = association(kind, 0);
            for commit in [false, true] {
                let mut tx = db.transaction();
                for id in 1..=700 {
                    let (_, row) = association(kind, id);
                    let handle = tx.insert(table, &row)?;
                    assert_eq!(tx.read(&handle)?.decode()?, row);
                }
                assert!(tx.page_count() > empty.len() as u64 / 4096);
                if commit {
                    tx.commit()?;
                } else {
                    tx.rollback();
                    let mut restored = Vec::new();
                    db.write_to(&mut restored)?;
                    assert_eq!(restored, empty);
                }
            }
            let output = temp.path().join(name);
            db.save_as(&output)?;
            let original = fs::read(&output)?;
            assert!(matches!(db.save_as(&output), Err(Error::Save { .. })));
            assert_eq!(fs::read(&output)?, original);
            let mut reopened = DeviceSqlConnection::open(&output, kind)?;
            assert_eq!(
                reopened.iter_raw(table)?.collect::<Result<Vec<_>>>()?.len(),
                700
            );
            let mut tx = reopened.transaction();
            let records = tx.iter_raw(table)?.take(2).collect::<Result<Vec<_>>>()?;
            let first = tx.locate(table, records[0].page(), records[0].slot().slot)?;
            let second = tx.locate(table, records[1].page(), records[1].slot().slot)?;
            let replacement = association(kind, 900).1;
            let updated = tx.update(&first, &replacement)?;
            tx.delete(&second)?;
            assert_eq!(tx.read(&updated)?.decode()?, replacement);
            tx.commit()?;
            let edited = temp.path().join(format!("edited-{name}"));
            reopened.save_as(&edited)?;
            let mut again = DeviceSqlConnection::open(&edited, kind)?;
            let rows = decode_rows(again.iter_raw(table)?)?;
            assert_eq!(rows.len(), 699);
            assert_eq!(rows[0], replacement);
            assert_eq!(fs::read(&output)?, original);
        }
        Ok(())
    }

    #[test]
    fn final_table_candidate_can_be_materialized_beyond_eof() -> Result<()> {
        for (kind, code, row) in [
            (
                DatabaseKind::Standard,
                19,
                Row::Standard(StandardRow::ExportProperty(
                    rbox_devlib::binary::ExportPropertyRow {
                        subtype: 0x280,
                        index_shift: 0,
                        track_count: 0,
                        unknown_08: 0,
                        background_color: 0,
                        unknown_0a: 0,
                        date: rbox_devlib::binary::DeviceSqlString::encode(
                            "2026-10-23",
                            rbox_devlib::binary::StringEncoding::ShortAscii,
                        )?,
                        separator: 0x1e19,
                        version: rbox_devlib::binary::DeviceSqlString::encode(
                            "1000",
                            rbox_devlib::binary::StringEncoding::ShortAscii,
                        )?,
                        name_padding: Vec::new(),
                        name: rbox_devlib::binary::DeviceSqlString::encode(
                            "New",
                            rbox_devlib::binary::StringEncoding::ShortAscii,
                        )?,
                    },
                )),
            ),
            // The final extended table is opaque: its reservation stays untouched
            // while allocating the supported property table immediately before it.
            (
                DatabaseKind::Extended,
                7,
                Row::Extended(ExtendedRow::DatabaseProperty(
                    rbox_devlib::binary::DatabasePropertyRow {
                        header: rbox_devlib::binary::DatabasePropertyHeader {
                            subtype: 0x700,
                            index_shift: 0,
                            unknown_words: [0; 5],
                            my_tag_master_db_id: 123,
                        },
                        strings: rbox_devlib::binary::StringOffsets::Near {
                            marker: 3,
                            offsets: [34, 35, 36, 37, 38],
                        },
                        unknown_strings: std::array::from_fn(|_| {
                            rbox_devlib::binary::DeviceSqlString::parse(&[3]).unwrap()
                        }),
                    },
                )),
            ),
        ] {
            let mut db = DeviceSqlConnection::create(kind)?;
            let original_count = db.page_count();
            let last_reservation = db.header().tables.last().unwrap().empty_candidate;
            assert_eq!(u64::from(last_reservation.0), original_count);
            let table = kind.table_kind(code);
            let mut tx = db.transaction();
            let handle = tx.insert(table, &row)?;
            tx.read(&handle)?.decode()?;
            if kind == DatabaseKind::Standard {
                assert_eq!(handle.page(), PageIndex(original_count as u32));
                assert_eq!(tx.page_count(), original_count + 1);
            } else {
                assert_eq!(tx.page_count(), original_count);
                assert_eq!(
                    tx.header().tables.last().unwrap().empty_candidate,
                    last_reservation
                );
            }
            tx.commit()?;
        }
        Ok(())
    }

    #[test]
    fn native_empty_export_is_reconstructed_without_using_it_as_a_template() -> Result<()> {
        let options = empty_export::standard()?;
        let mut db = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
        let mut bytes = Vec::new();
        db.write_to(&mut bytes)?;
        // This is the first access to the reference: neither constructor nor profile
        // reads it. Equality covers all headers, footers, slack and 75 encoded rows.
        let reference = fs::read(testdata_dir().join("export/empty/PIONEER/rekordbox/export.pdb"))?;
        assert_eq!(bytes.len(), 167936);
        assert_eq!(bytes, reference);
        let temp = create_temp_dir();
        let path = temp.path().join("empty.pdb");
        db.save_as(&path)?;
        assert_eq!(fs::read(&path)?, reference);
        let mut reopened = DeviceSqlConnection::open(&path, DatabaseKind::Standard)?;
        for seed in &options.tables {
            let actual = decode_rows(reopened.iter_raw(seed.table)?)?;
            assert_eq!(actual, seed.rows);
        }
        // The initialized result remains a normal editable connection. Dropping an
        // update restores exact bytes; committing it advances from the chosen sequence.
        let table = TableKind::Standard(PageType::ColorLabels);
        for commit in [false, true] {
            let mut tx = reopened.transaction();
            let record = tx.iter_raw(table)?.next().unwrap()?;
            let handle = tx.locate(table, record.page(), record.slot().slot)?;
            tx.delete(&handle)?;
            assert_eq!(tx.header().prefix.sequence, options.sequence + 1);
            if commit {
                tx.commit()?;
            } else {
                tx.rollback();
            }
            let mut edited = Vec::new();
            reopened.write_to(&mut edited)?;
            if commit {
                assert_eq!(
                    reopened.iter_raw(table)?.collect::<Result<Vec<_>>>()?.len(),
                    7
                );
                assert_ne!(edited, reference);
            } else {
                assert_eq!(edited, reference);
            }
        }
        assert_eq!(fs::read(&path)?, reference);
        Ok(())
    }

    #[test]
    fn explicit_initialization_supports_both_namespaces_and_multiple_pages() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let mut bare = DeviceSqlConnection::create(kind)?;
            let mut defaults =
                DeviceSqlConnection::create_with_options(kind, &CreateOptions::default())?;
            let mut first = Vec::new();
            let mut second = Vec::new();
            bare.write_to(&mut first)?;
            defaults.write_to(&mut second)?;
            assert_eq!(first, second);
            let (table, _) = association(kind, 0);
            let rows = (1..=700)
                .map(|i| association(kind, i).1)
                .collect::<Vec<_>>();
            let options = CreateOptions {
                unknown_10: 5,
                sequence: 30,
                tables: vec![InitialTable {
                    table,
                    first_sequence: 20,
                    rows: rows.clone(),
                }],
            };
            let mut db = DeviceSqlConnection::create_with_options(kind, &options)?;
            assert_eq!(decode_rows(db.iter_raw(table)?)?, rows);
            let mut bytes = Vec::new();
            db.write_to(&mut bytes)?;
            let mut input = Cursor::new(bytes);
            let header = FileHeader::read_le(&mut input)?;
            assert_eq!(header.prefix.unknown_10, 5);
            assert_eq!(header.prefix.sequence, 30);
            for chain in read_table_chains(&mut input, &header)? {
                assert_eq!(chain.pages[0].common.sequence, 1);
                if chain.table.table_type == table.code() {
                    assert!(chain.pages.len() > 2);
                    for (i, page) in chain.pages.iter().skip(1).enumerate() {
                        assert_eq!(page.common.sequence, 20 + i as u32);
                        assert_eq!(page.common.counts.live(), page.common.counts.allocated());
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn initialization_rejects_invalid_metadata_and_rows() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let (table, row) = association(kind, 1);
            let valid = CreateOptions {
                unknown_10: 5,
                sequence: 4,
                tables: vec![InitialTable {
                    table,
                    first_sequence: 2,
                    rows: vec![row],
                }],
            };
            for sequence in [0, 1, 2] {
                let options = CreateOptions {
                    sequence,
                    ..valid.clone()
                };
                assert!(matches!(
                    DeviceSqlConnection::create_with_options(kind, &options),
                    Err(Error::InvalidCreation { .. })
                ));
            }
            let mut options = valid.clone();
            options.tables[0].first_sequence = 0;
            assert!(matches!(
                DeviceSqlConnection::create_with_options(kind, &options),
                Err(Error::InvalidCreation { .. })
            ));
            options = valid.clone();
            options.tables.push(options.tables[0].clone());
            assert!(matches!(
                DeviceSqlConnection::create_with_options(kind, &options),
                Err(Error::InvalidCreation { .. })
            ));
            options = valid.clone();
            options.tables[0].table = kind.table_kind(100);
            assert!(matches!(
                DeviceSqlConnection::create_with_options(kind, &options),
                Err(Error::InvalidCreation { .. })
            ));
            options = valid.clone();
            options.tables[0].table = kind.table_kind(0); // Wrong binary row variant.
            assert!(matches!(
                DeviceSqlConnection::create_with_options(kind, &options),
                Err(Error::UnsupportedMutation { .. })
            ));
            options = valid.clone();
            let other = if kind == DatabaseKind::Standard {
                DatabaseKind::Extended
            } else {
                DatabaseKind::Standard
            };
            options.tables[0].table = other.table_kind(table.code());
            assert!(matches!(
                DeviceSqlConnection::create_with_options(kind, &options),
                Err(Error::WrongDatabaseKind { .. })
            ));
            // First sequence fits, but a later page would reach the committed sequence.
            options = valid.clone();
            options.sequence = 3;
            options.tables[0].rows = (1..=700).map(|i| association(kind, i).1).collect();
            assert!(matches!(
                DeviceSqlConnection::create_with_options(kind, &options),
                Err(Error::InvalidCreation { .. })
            ));
        }
        Ok(())
    }

    #[test]
    fn property_insertions_reserve_native_trailing_space() -> Result<()> {
        let options = empty_export::standard()?;
        let seed = &options.tables[0];
        let mut db = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let mut tx = db.transaction();
        for slot in 0..2 {
            let h = tx.insert(seed.table, &seed.rows[0])?;
            let r = tx.read(&h)?;
            assert_eq!(r.slot().heap_offset, slot * 40);
            assert_eq!(r.as_bytes().len(), 40);
            assert!(r.as_bytes()[31..].iter().all(|&b| b == 0));
        }
        Ok(())
    }
}

mod transactions {
    //! Binary CRUD, directory boundaries, handle lifetimes, and native saved edits.

    use super::support::{entry, entry_table};
    use super::*;

    fn encoded(row: &Row) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        match row {
            Row::Standard(StandardRow::PlaylistEntry(r)) => r.write_le(&mut bytes).unwrap(),
            Row::Extended(ExtendedRow::TrackMyTag(r)) => r.write_le(&mut bytes).unwrap(),
            Row::Standard(StandardRow::Genre(r)) => r.write_le(&mut bytes).unwrap(),
            _ => panic!("unexpected test row"),
        }
        bytes.into_inner()
    }

    // Independent wire layout: index page 1 -> data page 2, unlinked page 3.
    // Unused bytes are deliberately nonzero so preservation failures are visible.
    fn fixture(table: TableKind, rows: &[Row]) -> Vec<u8> {
        let mut bytes = vec![0xa5; 2048];
        bytes[..44].fill(0);
        bytes[4..8].copy_from_slice(&512u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
        bytes[20..24].copy_from_slice(&100u32.to_le_bytes());
        bytes[28..32].copy_from_slice(&table.code().to_le_bytes());
        bytes[36..40].copy_from_slice(&1u32.to_le_bytes());
        bytes[40..44].copy_from_slice(&2u32.to_le_bytes());
        for page in [1usize, 2] {
            let at = page * 512;
            bytes[at..at + 60].fill(0);
            bytes[at + 4..at + 8].copy_from_slice(&(page as u32).to_le_bytes());
            bytes[at + 8..at + 12].copy_from_slice(&table.code().to_le_bytes());
            bytes[at + 12..at + 16]
                .copy_from_slice(&(if page == 1 { 2u32 } else { 99 }).to_le_bytes());
            bytes[at + 16..at + 20].copy_from_slice(&90u32.to_le_bytes());
            bytes[at + 27] = if page == 1 { 0x64 } else { 0x24 };
        }
        let n = rows.len();
        let counts = n as u32 | ((n as u32) << 13);
        bytes[1048..1051].copy_from_slice(&counts.to_le_bytes()[..3]);
        let mut used = 0;
        for (slot, row) in rows.iter().enumerate() {
            let payload = encoded(row);
            bytes[1064 + used..1064 + used + payload.len()].copy_from_slice(&payload);
            let end = 1536 - slot / 16 * 36;
            bytes[end - 6 - 2 * (slot % 16)..end - 4 - 2 * (slot % 16)]
                .copy_from_slice(&(used as u16).to_le_bytes());
            used += (payload.len() + 3) & !3;
        }
        for group in 0..n.div_ceil(16) {
            let count = (n - group * 16).min(16);
            let mask = ((1u32 << count) - 1) as u16;
            let end = 1536 - group * 36;
            bytes[end - 4..end - 2].copy_from_slice(&mask.to_le_bytes());
            bytes[end - 2..end].copy_from_slice(&0xaaaau16.to_le_bytes());
        }
        let free = 512 - 40 - used - (n * 2 + n.div_ceil(16) * 4);
        bytes[1052..1054].copy_from_slice(&(free as u16).to_le_bytes());
        bytes[1054..1056].copy_from_slice(&(used as u16).to_le_bytes());
        bytes
    }

    #[test]
    fn crud_crosses_group_boundary_preserves_bytes_and_rejects_stale_handles() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let table = entry_table(kind);
            let rows: Vec<_> = (1..=16).map(|id| entry(kind, id)).collect();
            let original = fixture(table, &rows);
            let mut conn = DeviceSqlConnection::from_reader(Cursor::new(original.clone()), kind)?;
            let mut tx = conn.transaction();
            let first = tx.locate(table, PageIndex(2), 0)?;
            let same = tx.update(&first, &rows[0])?;
            assert_eq!(tx.modified_page_count(), 0);
            let updated = tx.update(&same, &entry(kind, 1000))?;
            assert!(matches!(tx.read(&first), Err(Error::InvalidRowHandle)));
            assert_eq!(tx.read(&updated)?.decode()?, entry(kind, 1000));
            let deleted = tx.locate(table, PageIndex(2), 1)?;
            tx.delete(&deleted)?;
            assert!(matches!(tx.delete(&deleted), Err(Error::InvalidRowHandle)));
            assert!(matches!(
                tx.locate(table, PageIndex(2), 1),
                Err(Error::RowNotFound { .. })
            ));
            let inserted = tx.insert(table, &entry(kind, 2000))?;
            assert_eq!(inserted.slot(), 16);
            assert_eq!(tx.read(&inserted)?.decode()?, entry(kind, 2000));
            assert_eq!(tx.iter_raw(table)?.count(), 16);
            assert_eq!(tx.header().prefix.sequence, 101); // one touched page
            tx.commit()?;
            let mut next = conn.transaction();
            assert!(matches!(next.read(&inserted), Err(Error::InvalidRowHandle)));
            next.rollback();

            let mut bytes = Vec::new();
            conn.write_to(&mut bytes)?;
            assert_eq!(&bytes[512..1024], &original[512..1024]); // index page
            assert_eq!(&bytes[1536..], &original[1536..]); // unlinked page
            let width = encoded(&rows[0]).len();
            assert_eq!(
                &bytes[1064 + width..1064 + 2 * width],
                &original[1064 + width..1064 + 2 * width]
            ); // retired payload
            let mut input = Cursor::new(&bytes);
            input.set_position(1024);
            let page = PageHeader::read_le(&mut input)?;
            assert_eq!(
                (page.common.counts.allocated(), page.common.counts.live()),
                (17, 16)
            );
            assert_eq!(page.common.sequence, 100);
            assert_eq!(page.common.flags.0, 0x34);
            let PageHeaderKind::Data(data) = page.kind else {
                panic!()
            };
            assert_eq!(
                (data.transaction_row_count, data.transaction_row_index),
                (3, 0)
            );
            let directory = read_row_directory(&mut input, 512, &page)?;
            assert_eq!(directory.groups()[0].transaction_flags, 3);
            assert_eq!(directory.groups()[1].transaction_flags, 1);
            assert_eq!(directory.slot(16).unwrap().heap_offset as usize, width * 16);
            let mut reopened = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
            let actual = decode_rows(reopened.iter_raw(table)?)?;
            assert_eq!(actual[0], entry(kind, 1000));
            assert_eq!(actual.last(), Some(&entry(kind, 2000)));
        }
        Ok(())
    }

    #[test]
    fn failed_operations_are_atomic_and_drop_rolls_back_all_successes() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let table = entry_table(kind);
            let n = if kind == DatabaseKind::Standard {
                32
            } else {
                25
            };
            let original = fixture(table, &(0..n).map(|id| entry(kind, id)).collect::<Vec<_>>());
            let mut conn = DeviceSqlConnection::from_reader(Cursor::new(original.clone()), kind)?;
            {
                let mut tx = conn.transaction();
                let handle = tx.locate(table, PageIndex(2), 0)?;
                tx.update(&handle, &entry(kind, 999))?;
                let before = tx.iter_raw(table)?.collect::<Result<Vec<_>>>()?;
                let header = tx.header().clone();
                assert!(matches!(
                    tx.insert(table, &entry(kind, 111)),
                    Err(Error::UnsupportedMutation { .. })
                ));
                let wrong = entry(
                    if kind == DatabaseKind::Standard {
                        DatabaseKind::Extended
                    } else {
                        DatabaseKind::Standard
                    },
                    1,
                );
                assert!(matches!(
                    tx.insert(table, &wrong),
                    Err(Error::UnsupportedMutation { .. })
                ));
                assert_eq!(tx.iter_raw(table)?.collect::<Result<Vec<_>>>()?, before);
                assert_eq!(tx.header(), &header);
            }
            let mut saved = Vec::new();
            conn.write_to(&mut saved)?;
            assert_eq!(saved, original);
        }
        Ok(())
    }

    #[test]
    fn native_crud_reopens_both_namespaces_without_modifying_sources() -> Result<()> {
        let temp = create_temp_dir();
        for (kind, name) in [
            (DatabaseKind::Standard, "export.pdb"),
            (DatabaseKind::Extended, "exportExt.pdb"),
        ] {
            let source = temp.path().join(name);
            fs::copy(export_dir(7).join("rekordbox").join(name), &source)?;
            let original = fs::read(&source)?;
            let table = entry_table(kind);
            let mut conn = DeviceSqlConnection::open(&source, kind)?;
            // The demo has playlist entries but no track-tag associations.
            if kind == DatabaseKind::Extended {
                assert_eq!(conn.iter_raw(table)?.count(), 0);
                let mut seed = conn.transaction();
                for id in 1..=2 {
                    seed.insert(table, &entry(kind, id))?;
                }
                seed.commit()?;
            }
            let old_count = conn.iter_raw(table)?.count();
            let mut tx = conn.transaction();
            let originals = tx.iter_raw(table)?.take(2).collect::<Result<Vec<_>>>()?;
            let first = tx.locate(table, originals[0].page(), originals[0].slot().slot)?;
            let second = tx.locate(table, originals[1].page(), originals[1].slot().slot)?;
            tx.update(&first, &entry(kind, 900001))?;
            tx.delete(&second)?;
            tx.insert(table, &entry(kind, 900002))?;
            let expected = decode_rows(tx.iter_raw(table)?)?;
            assert_eq!(expected.len(), old_count);
            tx.commit()?;
            let destination = temp.path().join(format!("modified-{name}"));
            conn.save_as(&destination)?;
            let mut reopened = DeviceSqlConnection::open(&destination, kind)?;
            assert_eq!(decode_rows(reopened.iter_raw(table)?)?, expected);
            assert_eq!(fs::read(&source)?, original);
        }
        Ok(())
    }

    #[test]
    fn native_track_and_tag_prefix_updates_preserve_strings_and_opaque_tails() -> Result<()> {
        let temp = create_temp_dir();
        for (kind, name, table) in [
            (
                DatabaseKind::Standard,
                "export.pdb",
                TableKind::Standard(PageType::Tracks),
            ),
            (
                DatabaseKind::Extended,
                "exportExt.pdb",
                TableKind::Extended(ExtPageType::MyTags),
            ),
        ] {
            let path = temp.path().join(name);
            fs::copy(export_dir(7).join("rekordbox").join(name), &path)?;
            let mut conn = DeviceSqlConnection::open(&path, kind)?;
            let mut tx = conn.transaction();
            let original = tx.iter_raw(table)?.next().unwrap()?;
            let handle = tx.locate(table, original.page(), original.slot().slot)?;
            let mut replacement = original.decode()?;
            let prefix_len = match &mut replacement {
                Row::Standard(StandardRow::Track(row)) => {
                    row.header.rating = (row.header.rating + 1) % 6;
                    92
                }
                Row::Extended(ExtendedRow::MyTag(row)) => {
                    row.header.position += 1;
                    28
                }
                _ => panic!(),
            };
            let handle = tx.update(&handle, &replacement)?;
            let edited = tx.read(&handle)?;
            assert_eq!(edited.decode()?, replacement);
            assert_eq!(
                &edited.as_bytes()[prefix_len..],
                &original.as_bytes()[prefix_len..]
            );
            tx.commit()?;
            let mut bytes = Vec::new();
            conn.write_to(&mut bytes)?;
            let mut reopened = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
            assert_eq!(
                reopened.iter_raw(table)?.next().unwrap()?.decode()?,
                replacement
            );
        }
        Ok(())
    }

    #[test]
    fn shrinking_inline_strings_preserves_tail_and_growth_relocates() -> Result<()> {
        let table = TableKind::Standard(PageType::Genres);
        let genre = |name: &str| {
            Row::Standard(StandardRow::Genre(NamedRow {
                id: 7,
                name: DeviceSqlString::encode(name, StringEncoding::ShortAscii).unwrap(),
            }))
        };
        let original = fixture(table, &[genre("Long genre")]);
        let mut conn =
            DeviceSqlConnection::from_reader(Cursor::new(original), DatabaseKind::Standard)?;
        let mut tx = conn.transaction();
        let handle = tx.locate(table, PageIndex(2), 0)?;
        let original = tx.read(&handle)?;
        let short = genre("DJ");
        let handle = tx.update(&handle, &short)?;
        let current = tx.read(&handle)?;
        assert_eq!(current.decode()?, short);
        let length = encoded(&short).len();
        assert_eq!(
            &current.as_bytes()[length..],
            &original.as_bytes()[length..]
        );
        let moved = tx.update(&handle, &genre("Larger"))?;
        assert_ne!(moved.slot(), handle.slot());
        assert!(matches!(tx.read(&handle), Err(Error::InvalidRowHandle)));
        assert_eq!(tx.read(&moved)?.decode()?, genre("Larger"));
        assert_eq!(tx.iter_raw(table)?.count(), 1);
        let inserted = tx.insert(table, &genre("A new longer name"))?;
        assert_eq!(tx.read(&inserted)?.decode()?, genre("A new longer name"));
        tx.commit()?;
        Ok(())
    }

    #[test]
    fn sequence_overflow_on_second_page_preserves_first_successful_operation() -> Result<()> {
        let kind = DatabaseKind::Extended;
        let table = entry_table(kind);
        let mut original = fixture(table, &[entry(kind, 1)]);
        let copy = original[1024..1536].to_vec();
        original[1536..].copy_from_slice(&copy);
        original[1540..1544].copy_from_slice(&3u32.to_le_bytes());
        original[1036..1040].copy_from_slice(&3u32.to_le_bytes());
        original[40..44].copy_from_slice(&3u32.to_le_bytes());
        original[20..24].copy_from_slice(&(u32::MAX - 1).to_le_bytes());
        let mut conn = DeviceSqlConnection::from_reader(Cursor::new(original), kind)?;
        let mut tx = conn.transaction();
        let a = tx.locate(table, PageIndex(2), 0)?;
        let b = tx.locate(table, PageIndex(3), 0)?;
        tx.update(&a, &entry(kind, 9))?;
        assert!(matches!(
            tx.update(&b, &entry(kind, 10)),
            Err(Error::MutationOverflow)
        ));
        assert_eq!(tx.read(&b)?.decode()?, entry(kind, 1));
        assert_eq!(tx.header().prefix.sequence, u32::MAX);
        tx.commit()?;
        let mut next = conn.transaction();
        let a = next.locate(table, PageIndex(2), 0)?;
        assert_eq!(next.read(&a)?.decode()?, entry(kind, 9));
        assert!(matches!(next.delete(&a), Err(Error::MutationOverflow)));
        next.rollback();
        Ok(())
    }

    #[test]
    fn deleting_opaque_rows_preserves_payload_and_does_not_reuse_slots() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let table = kind.table_kind(9001);
            let original = fixture(table, &[entry(kind, 1)]);
            let mut conn = DeviceSqlConnection::from_reader(Cursor::new(original.clone()), kind)?;
            let mut tx = conn.transaction();
            let handle = tx.locate(table, PageIndex(2), 0)?;
            tx.delete(&handle)?;
            assert_eq!(tx.iter_raw(table)?.count(), 0);
            tx.commit()?;
            let mut saved = Vec::new();
            conn.write_to(&mut saved)?;
            assert_eq!(&saved[1064..1530], &original[1064..1530]);
            let mut input = Cursor::new(&saved[1024..]);
            let header = PageHeader::read_le(&mut input)?;
            assert_eq!(header.common.counts.allocated(), 1);
            assert_eq!(header.common.counts.live(), 0);
        }
        Ok(())
    }

    // Explicit candidate-page profile, independent from the production allocator.
    fn growing_fixture(kind: DatabaseKind) -> Vec<u8> {
        let n = if kind == DatabaseKind::Standard {
            32
        } else {
            25
        };
        let mut bytes = fixture(
            entry_table(kind),
            &(0..n).map(|id| entry(kind, id)).collect::<Vec<_>>(),
        );
        bytes[12..16].copy_from_slice(&4u32.to_le_bytes());
        bytes[32..36].copy_from_slice(&3u32.to_le_bytes());
        bytes[1036..1040].copy_from_slice(&3u32.to_le_bytes());
        bytes[552..556].copy_from_slice(&1u32.to_le_bytes());
        bytes[556..560].copy_from_slice(&2u32.to_le_bytes());
        bytes[1536..].fill(0);
        bytes
    }

    #[test]
    fn allocation_grows_both_kinds_and_preserves_rollback_and_repeated_commits() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let table = entry_table(kind);
            let original = growing_fixture(kind);
            let mut conn = DeviceSqlConnection::from_reader(Cursor::new(original.clone()), kind)?;
            let original_rows = conn.iter_raw(table)?.count();
            for commit in [false, true] {
                let mut tx = conn.transaction();
                let mut last = None;
                for id in 100..180 {
                    last = Some(tx.insert(table, &entry(kind, id))?);
                }
                let handle = last.unwrap();
                assert!(handle.page().0 >= 5);
                assert_eq!(tx.read(&handle)?.decode()?, entry(kind, 179));
                assert_eq!(
                    tx.iter_raw(table)?.collect::<Result<Vec<_>>>()?.len(),
                    original_rows + 80
                );
                assert!(tx.header().tables[0].empty_candidate.0 > handle.page().0);
                if commit {
                    tx.commit()?;
                } else {
                    tx.rollback();
                    let mut bytes = Vec::new();
                    conn.write_to(&mut bytes)?;
                    assert_eq!(bytes, original);
                }
            }
            let mut tx = conn.transaction();
            for id in 180..220 {
                tx.insert(table, &entry(kind, id))?;
            }
            tx.commit()?;
            let mut bytes = Vec::new();
            conn.write_to(&mut bytes)?;
            assert!(bytes.len() > original.len());
            assert_eq!(&bytes[512..1024], &original[512..1024]); // index entries untouched
            let mut cursor = Cursor::new(&bytes);
            let header = FileHeader::read_le(&mut cursor)?;
            let chains = read_table_chains(&mut cursor, &header)?;
            for page in &chains[0].pages {
                assert!(page.common.sequence < header.prefix.sequence);
                if !page.kind.is_index() {
                    read_row_directory(&mut cursor, 512, page)?;
                }
            }
            let mut reopened = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
            assert_eq!(
                reopened.iter_raw(table)?.collect::<Result<Vec<_>>>()?.len(),
                original_rows + 120
            );
        }
        Ok(())
    }

    #[test]
    fn failed_allocation_is_atomic_after_link_preparation_and_rejects_occupied_candidates(
    ) -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let table = entry_table(kind);
            for occupied in [false, true] {
                let mut bytes = growing_fixture(kind);
                if occupied {
                    bytes[1536] = 0x5a;
                } else {
                    bytes[20..24].copy_from_slice(&(u32::MAX - 1).to_le_bytes());
                }
                let mut conn = DeviceSqlConnection::from_reader(Cursor::new(bytes.clone()), kind)?;
                let mut tx = conn.transaction();
                let handle = tx.locate(table, PageIndex(2), 0)?;
                let before = tx.read(&handle)?;
                assert!(tx.insert(table, &entry(kind, 111)).is_err());
                assert_eq!(tx.modified_page_count(), 0);
                assert_eq!(tx.read(&handle)?, before);
                tx.commit()?;
                let mut saved = Vec::new();
                conn.write_to(&mut saved)?;
                assert_eq!(saved, bytes);
            }
        }
        Ok(())
    }

    #[test]
    fn first_data_page_allocation_updates_empty_table_index() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let table = entry_table(kind);
            let mut bytes = growing_fixture(kind);
            bytes[40..44].copy_from_slice(&1u32.to_le_bytes());
            bytes[524..528].copy_from_slice(&3u32.to_le_bytes());
            bytes[556..560].copy_from_slice(&PageIndex::NONE.0.to_le_bytes());
            let unused = bytes[1024..1536].to_vec();
            let mut conn = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
            let mut tx = conn.transaction();
            let handle = tx.insert(table, &entry(kind, 9))?;
            assert_eq!(handle.page(), PageIndex(3));
            assert_eq!(tx.read(&handle)?.decode()?, entry(kind, 9));
            tx.commit()?;
            let mut saved = Vec::new();
            conn.write_to(&mut saved)?;
            assert_eq!(&saved[1024..1536], unused.as_slice());
            assert_eq!(&saved[556..560], &3u32.to_le_bytes());
            let mut reopened = DeviceSqlConnection::from_reader(Cursor::new(saved), kind)?;
            assert_eq!(reopened.iter_raw(table)?.count(), 1);
        }
        Ok(())
    }

    #[test]
    fn demo_and_synthetic_string_rows_insert_and_relocate_with_slot_fixups() -> Result<()> {
        for (kind, file, table) in [
            (
                DatabaseKind::Standard,
                "export.pdb",
                TableKind::Standard(PageType::Tracks),
            ),
            (
                DatabaseKind::Standard,
                "export.pdb",
                TableKind::Standard(PageType::Artists),
            ),
            (
                DatabaseKind::Standard,
                "export.pdb",
                TableKind::Standard(PageType::Albums),
            ),
            (
                DatabaseKind::Standard,
                "export.pdb",
                TableKind::Standard(PageType::ExportProperties),
            ),
            (
                DatabaseKind::Extended,
                "exportExt.pdb",
                TableKind::Extended(ExtPageType::MyTags),
            ),
            (
                DatabaseKind::Extended,
                "exportExt.pdb",
                TableKind::Extended(ExtPageType::DatabasePropertyOptions),
            ),
        ] {
            let bytes = fs::read(export_dir(7).join("rekordbox").join(file))?;
            let mut conn = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
            if table == TableKind::Standard(PageType::Albums) {
                assert_eq!(conn.iter_raw(table)?.count(), 0);
                let mut seed = conn.transaction();
                seed.insert(
                    table,
                    &Row::Standard(StandardRow::Album(AlbumRow {
                        header: AlbumHeader {
                            subtype: AlbumHeader::NEAR_SUBTYPE,
                            index_shift: 0,
                            unknown_04: 0,
                            artist_id: 1,
                            id: 1,
                            unknown_10: 0,
                        },
                        strings: StringOffsets::Near {
                            marker: 3,
                            offsets: [22],
                        },
                        name: DeviceSqlString::encode("Test album", StringEncoding::ShortAscii)?,
                    })),
                )?;
                seed.commit()?;
            }
            let mut tx = conn.transaction();
            let original = tx.iter_raw(table)?.next().unwrap()?;
            let old = tx.locate(table, original.page(), original.slot().slot)?;
            let mut row = original.decode()?;
            let text = DeviceSqlString::encode(&"longer name ".repeat(8), StringEncoding::Utf16Le)?;
            match &mut row {
                Row::Standard(StandardRow::Track(r)) => r.text.title = text.clone(),
                Row::Standard(StandardRow::Artist(r)) => r.name = text.clone(),
                Row::Standard(StandardRow::Album(r)) => r.name = text.clone(),
                Row::Standard(StandardRow::ExportProperty(r)) => r.name = text.clone(),
                Row::Extended(ExtendedRow::MyTag(r)) => r.name = text.clone(),
                Row::Extended(ExtendedRow::DatabaseProperty(r)) => {
                    r.header.my_tag_master_db_id ^= 1
                }
                _ => panic!(),
            }
            let handle = tx.update(&old, &row)?;
            assert!(matches!(tx.read(&old), Err(Error::InvalidRowHandle)));
            let updated = tx.read(&handle)?.decode()?;
            let inserted = tx.insert(table, &updated)?;
            let decoded = tx.read(&inserted)?.decode()?;
            let shift = match &decoded {
                Row::Standard(StandardRow::Track(r)) => {
                    assert_eq!(r.text.title, text);
                    r.header.index_shift
                }
                Row::Standard(StandardRow::Artist(r)) => {
                    assert_eq!(r.name, text);
                    r.header.index_shift
                }
                Row::Standard(StandardRow::Album(r)) => {
                    assert_eq!(r.name, text);
                    r.header.index_shift
                }
                Row::Standard(StandardRow::ExportProperty(r)) => {
                    assert_eq!(r.name, text);
                    r.index_shift
                }
                Row::Extended(ExtendedRow::MyTag(r)) => {
                    assert_eq!(r.name, text);
                    r.header.index_shift
                }
                Row::Extended(ExtendedRow::DatabaseProperty(r)) => r.header.index_shift,
                _ => panic!(),
            };
            assert_eq!(shift >> 5, inserted.slot());
            tx.commit()?;
            let mut saved = Vec::new();
            conn.write_to(&mut saved)?;
            let mut reopened = DeviceSqlConnection::from_reader(Cursor::new(saved), kind)?;
            let mut tx = reopened.transaction();
            let handle = tx.locate(table, inserted.page(), inserted.slot())?;
            assert_eq!(tx.read(&handle)?.decode()?, decoded);
        }
        Ok(())
    }

    #[test]
    fn relocation_failure_after_insertion_does_not_publish_replacement() -> Result<()> {
        let table = TableKind::Standard(PageType::Genres);
        let row = |text: &str| {
            Row::Standard(StandardRow::Genre(NamedRow {
                id: 7,
                name: DeviceSqlString::encode(text, StringEncoding::ShortAscii).unwrap(),
            }))
        };
        let mut bytes = fixture(table, &[row("a")]);
        // Add a linked second data page. Relocation will append there first, then
        // overflow while marking the original page's row deleted.
        let page = bytes[1024..1536].to_vec();
        bytes[1536..].copy_from_slice(&page);
        bytes[1540..1544].copy_from_slice(&3u32.to_le_bytes());
        bytes[1036..1040].copy_from_slice(&3u32.to_le_bytes());
        bytes[40..44].copy_from_slice(&3u32.to_le_bytes());
        bytes[20..24].copy_from_slice(&(u32::MAX - 1).to_le_bytes());
        let mut conn =
            DeviceSqlConnection::from_reader(Cursor::new(bytes.clone()), DatabaseKind::Standard)?;
        let mut tx = conn.transaction();
        let handle = tx.locate(table, PageIndex(2), 0)?;
        let original = tx.read(&handle)?;
        assert!(matches!(
            tx.update(&handle, &row("a much longer name")),
            Err(Error::MutationOverflow)
        ));
        assert_eq!(tx.read(&handle)?, original);
        assert_eq!(tx.modified_page_count(), 0);
        assert_eq!(tx.iter_raw(table)?.count(), 2);
        tx.commit()?;
        let mut saved = Vec::new();
        conn.write_to(&mut saved)?;
        assert_eq!(saved, bytes);
        Ok(())
    }

    #[test]
    fn shared_candidate_is_rejected_even_when_zero_or_beyond_eof() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            for candidate in [3u32, 5] {
                let mut bytes = growing_fixture(kind);
                bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
                bytes[12..16].copy_from_slice(&6u32.to_le_bytes());
                bytes[32..36].copy_from_slice(&candidate.to_le_bytes());
                bytes[1036..1040].copy_from_slice(&candidate.to_le_bytes());
                bytes[44..48].copy_from_slice(&999u32.to_le_bytes());
                bytes[48..52].copy_from_slice(&candidate.to_le_bytes());
                bytes[52..56].copy_from_slice(&4u32.to_le_bytes());
                bytes[56..60].copy_from_slice(&4u32.to_le_bytes());
                let mut index = bytes[512..1024].to_vec();
                index[4..8].copy_from_slice(&4u32.to_le_bytes());
                index[8..12].copy_from_slice(&999u32.to_le_bytes());
                bytes.extend_from_slice(&index);
                let mut conn = DeviceSqlConnection::from_reader(Cursor::new(bytes.clone()), kind)?;
                let mut tx = conn.transaction();
                assert!(matches!(
                    tx.insert(entry_table(kind), &entry(kind, 123)),
                    Err(Error::UnsupportedMutation { .. })
                ));
                assert_eq!(tx.modified_page_count(), 0);
                tx.commit()?;
                let mut saved = Vec::new();
                conn.write_to(&mut saved)?;
                assert_eq!(saved, bytes);
            }
        }
        Ok(())
    }

    #[test]
    fn native_empty_tables_allocate_their_first_data_page_in_both_namespaces() -> Result<()> {
        for (kind, name, table, row) in [
            (
                DatabaseKind::Standard,
                "export.pdb",
                TableKind::Standard(PageType::Genres),
                Row::Standard(StandardRow::Genre(NamedRow {
                    id: 7,
                    name: DeviceSqlString::encode("New", StringEncoding::ShortAscii)?,
                })),
            ),
            (
                DatabaseKind::Extended,
                "exportExt.pdb",
                entry_table(DatabaseKind::Extended),
                entry(DatabaseKind::Extended, 7),
            ),
        ] {
            let original = fs::read(
                rbox_testing::data::export_dir(7)
                    .join("rekordbox")
                    .join(name),
            )?;
            let mut conn = DeviceSqlConnection::from_reader(Cursor::new(original), kind)?;
            let mut tx = conn.transaction();
            assert_eq!(tx.iter_raw(table)?.count(), 0);
            for _ in 0..500 {
                tx.insert(table, &row)?;
            }
            assert_eq!(tx.iter_raw(table)?.collect::<Result<Vec<_>>>()?.len(), 500);
            tx.commit()?;
            let mut bytes = Vec::new();
            conn.write_to(&mut bytes)?;
            let mut reopened = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
            let rows = decode_rows(reopened.iter_raw(table)?)?;
            assert_eq!(rows, vec![row; 500]);
        }
        Ok(())
    }

    #[test]
    fn allocation_respects_index_references_and_rejects_out_of_bounds_arrays() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            for reference in [3u32, 4, 2] {
                let mut bytes = growing_fixture(kind);
                bytes[568..570].copy_from_slice(&1u16.to_le_bytes());
                bytes[572..576].copy_from_slice(&((reference << 3) | 5).to_le_bytes());
                let mut db = DeviceSqlConnection::from_reader(Cursor::new(bytes.clone()), kind)?;
                let mut tx = db.transaction();
                let result = tx.insert(entry_table(kind), &entry(kind, 99));
                if reference == 2 {
                    result?; // A reference to an owned existing page is preserved.
                    tx.commit()?;
                    let mut saved = Vec::new();
                    db.write_to(&mut saved)?;
                    assert_eq!(&saved[512..1024], &bytes[512..1024]);
                } else {
                    assert!(matches!(result, Err(Error::UnsupportedMutation { .. })));
                    assert_eq!(tx.modified_page_count(), 0);
                    tx.commit()?;
                    let mut saved = Vec::new();
                    db.write_to(&mut saved)?;
                    assert_eq!(saved, bytes);
                }
            }
            let mut bytes = growing_fixture(kind);
            bytes[568..570].copy_from_slice(&u16::MAX.to_le_bytes());
            let mut db = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
            let mut tx = db.transaction();
            assert!(matches!(
                tx.insert(entry_table(kind), &entry(kind, 99)),
                Err(Error::InvalidIndexEntries { .. })
            ));
            assert_eq!(tx.modified_page_count(), 0);
        }
        Ok(())
    }

    #[test]
    fn fitting_offset_strings_preserve_slots_offsets_and_unmodeled_bytes() -> Result<()> {
        // Independent wire frames, deliberately containing nonzero unmodeled gaps.
        // These prove bounded editing, not allocation rules for new rows.
        for table in [
            TableKind::Standard(PageType::Artists),
            TableKind::Extended(ExtPageType::MyTags),
            TableKind::Extended(ExtPageType::DatabasePropertyOptions),
        ] {
            let kind = match table {
                TableKind::Standard(_) => DatabaseKind::Standard,
                TableKind::Extended(_) => DatabaseKind::Extended,
            };
            let mut payload = vec![0xa5; 128];
            let starts: Vec<usize> = match table {
                TableKind::Standard(_) => {
                    payload[..8].fill(0);
                    payload[0..2].copy_from_slice(&0x64u16.to_le_bytes());
                    payload[8..12].copy_from_slice(&[3, 0, 36, 0]);
                    vec![36]
                }
                TableKind::Extended(ExtPageType::MyTags) => {
                    payload[..28].fill(0);
                    payload[0..2].copy_from_slice(&0x684u16.to_le_bytes());
                    payload[28..34].copy_from_slice(&[3, 0, 36, 0, 52, 0]);
                    vec![36, 52]
                }
                _ => {
                    payload[..28].fill(0);
                    payload[0..2].copy_from_slice(&0x700u16.to_le_bytes());
                    payload[28..34].copy_from_slice(&[3, 36, 52, 68, 84, 100]);
                    vec![36, 52, 68, 84, 100]
                }
            };
            let frame = DeviceSqlString::encode("abcdef", StringEncoding::LongAscii)?;
            for &start in &starts {
                payload[start..start + frame.as_bytes().len()].copy_from_slice(frame.as_bytes());
            }
            let mut bytes = fixture(table, &[]);
            bytes[1048..1051].copy_from_slice(&(1u32 | (1 << 13)).to_le_bytes()[..3]);
            bytes[1052..1054].copy_from_slice(&338u16.to_le_bytes());
            bytes[1054..1056].copy_from_slice(&128u16.to_le_bytes());
            bytes[1064..1192].copy_from_slice(&payload);
            bytes[1530..1536].copy_from_slice(&[0, 0, 1, 0, 0, 0]);
            let mut db = DeviceSqlConnection::from_reader(Cursor::new(bytes.clone()), kind)?;
            let mut tx = db.transaction();
            let handle = tx.locate(table, PageIndex(2), 0)?;
            let mut edited = tx.read(&handle)?.decode()?;
            let replacement = DeviceSqlString::encode("é", StringEncoding::Utf16Le)?;
            match &mut edited {
                Row::Standard(StandardRow::Artist(r)) => r.name = replacement.clone(),
                Row::Extended(ExtendedRow::MyTag(r)) => {
                    r.name = replacement.clone();
                    r.unknown_string = replacement.clone();
                    r.header.position = 7;
                }
                Row::Extended(ExtendedRow::DatabaseProperty(r)) => {
                    r.unknown_strings.fill(replacement.clone());
                }
                _ => unreachable!(),
            }
            let updated = tx.update(&handle, &edited)?;
            assert_eq!(
                (updated.page(), updated.slot()),
                (handle.page(), handle.slot())
            );
            assert_eq!(tx.read(&updated)?.decode()?, edited);
            assert!(matches!(tx.read(&handle), Err(Error::InvalidRowHandle)));
            let current = tx.read(&updated)?;
            for &start in &starts {
                payload[start..start + frame.as_bytes().len()].fill(0);
                payload[start..start + replacement.as_bytes().len()]
                    .copy_from_slice(replacement.as_bytes());
            }
            if matches!(table, TableKind::Extended(ExtPageType::MyTags)) {
                payload[16..20].copy_from_slice(&7u32.to_le_bytes());
            }
            assert_eq!(current.as_bytes(), payload);
            // No tail/gap can be claimed for growth. Nonzero gaps prevent relocation.
            let long = DeviceSqlString::encode(&"x".repeat(200), StringEncoding::LongAscii)?;
            match &mut edited {
                Row::Standard(StandardRow::Artist(r)) => r.name = long,
                Row::Extended(ExtendedRow::MyTag(r)) => r.name = long,
                Row::Extended(ExtendedRow::DatabaseProperty(r)) => r.unknown_strings[0] = long,
                _ => unreachable!(),
            }
            assert!(matches!(
                tx.update(&updated, &edited),
                Err(Error::UnsupportedMutation { .. })
            ));
            assert_eq!(tx.read(&updated)?, current);
            tx.commit()?;
            let mut saved = Vec::new();
            db.write_to(&mut saved)?;
            assert_eq!(&saved[512..1024], &bytes[512..1024]);
            let mut reopened = DeviceSqlConnection::from_reader(Cursor::new(saved), kind)?;
            assert_eq!(
                reopened.iter_raw(table)?.next().unwrap()?.as_bytes(),
                payload
            );
        }
        Ok(())
    }

    #[test]
    fn repacking_uses_known_content_for_reserved_strings_and_bounded_far_promotion() -> Result<()> {
        // These synthetic rows establish edit bounds, not fresh allocation formulas.
        for variant in 0..3 {
            let table = TableKind::Extended(if variant == 2 {
                ExtPageType::DatabasePropertyOptions
            } else {
                ExtPageType::MyTags
            });
            let mut payload = vec![0; 400];
            payload[2..4].copy_from_slice(&7u16.to_le_bytes());
            let end = if variant == 2 {
                payload[..2].copy_from_slice(&0x700u16.to_le_bytes());
                payload[28..34].copy_from_slice(&[3, 34, 40, 104, 168, 232]);
                let first = DeviceSqlString::encode("small", StringEncoding::ShortAscii)?;
                payload[34..40].copy_from_slice(first.as_bytes());
                let long = DeviceSqlString::encode(&"x".repeat(60), StringEncoding::LongAscii)?;
                for start in [40, 104, 168, 232] {
                    payload[start..start + 64].copy_from_slice(long.as_bytes());
                }
                296
            } else {
                let name = DeviceSqlString::encode("originalname", StringEncoding::ShortAscii)?;
                let (start, reserved) = if variant == 0 {
                    payload[..2].copy_from_slice(&0x680u16.to_le_bytes());
                    payload[28..31].copy_from_slice(&[3, 31, 44]);
                    (31, 44)
                } else {
                    payload[..2].copy_from_slice(&0x684u16.to_le_bytes());
                    payload[28..34].copy_from_slice(&[3, 0, 34, 0, 48, 0]);
                    (34, 48)
                };
                payload[start..start + name.as_bytes().len()].copy_from_slice(name.as_bytes());
                let long = DeviceSqlString::encode(&"r".repeat(300), StringEncoding::LongAscii)?;
                payload[reserved..reserved + long.as_bytes().len()]
                    .copy_from_slice(long.as_bytes());
                reserved + long.as_bytes().len()
            };
            payload[end..].fill(0xa5);
            let mut bytes = fixture(table, &[]);
            bytes[1048..1051].copy_from_slice(&(1u32 | (1 << 13)).to_le_bytes()[..3]);
            bytes[1052..1054].copy_from_slice(&66u16.to_le_bytes());
            bytes[1054..1056].copy_from_slice(&400u16.to_le_bytes());
            bytes[1064..1464].copy_from_slice(&payload);
            bytes[1530..1536].copy_from_slice(&[0, 0, 1, 0, 0, 0]);
            let mut db =
                DeviceSqlConnection::from_reader(Cursor::new(bytes), DatabaseKind::Extended)?;
            let mut tx = db.transaction();
            let handle = tx.locate(table, PageIndex(2), 0)?;
            let mut edited = tx.read(&handle)?.decode()?;
            let empty = DeviceSqlString::encode("", StringEncoding::ShortAscii)?;
            let text = if variant == 2 {
                "new".repeat(30)
            } else {
                "n".repeat(270)
            };
            match &mut edited {
                Row::Extended(ExtendedRow::MyTag(r)) => {
                    r.name = DeviceSqlString::encode(&text, StringEncoding::LongAscii)?;
                    r.unknown_string = empty;
                }
                Row::Extended(ExtendedRow::DatabaseProperty(r)) => {
                    r.unknown_strings.fill(empty);
                    r.unknown_strings[0] =
                        DeviceSqlString::encode(&text, StringEncoding::LongAscii)?;
                }
                _ => unreachable!(),
            }
            let updated = tx.update(&handle, &edited)?;
            assert_eq!(
                (updated.page(), updated.slot()),
                (handle.page(), handle.slot())
            );
            let result = tx.read(&updated)?;
            assert_eq!(&result.as_bytes()[end..], &payload[end..]);
            match result.decode()? {
                Row::Extended(ExtendedRow::MyTag(r)) => {
                    assert_eq!(r.header.subtype, 0x684);
                    assert_eq!(r.header.index_shift, 7);
                    assert!(r.strings.is_far());
                    assert_eq!(r.name.text()?, text);
                    assert_eq!(r.unknown_string.text()?, "");
                }
                Row::Extended(ExtendedRow::DatabaseProperty(r)) => {
                    assert_eq!(r.header.index_shift, 7);
                    assert_eq!(r.unknown_strings[0].text()?, text);
                    assert!(r.unknown_strings[1..].iter().all(|s| s.as_bytes() == [3]));
                }
                _ => unreachable!(),
            }
            tx.commit()?;
            let mut saved = Vec::new();
            db.write_to(&mut saved)?;
            let mut reopened =
                DeviceSqlConnection::from_reader(Cursor::new(saved), DatabaseKind::Extended)?;
            assert_eq!(
                reopened.iter_raw(table)?.next().unwrap()?.decode()?,
                result.decode()?
            );
        }
        Ok(())
    }
}
