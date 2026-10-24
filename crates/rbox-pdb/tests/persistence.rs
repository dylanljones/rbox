// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Lossless copies, selective saves, resource limits, conflict detection, and compaction.

use binrw::BinRead;
use rbox_pdb::binary::{
    read_index_entries, read_row_directory, read_table_chains, DatabaseKind, FileHeader,
    PageHeaderKind,
};
use rbox_pdb::{DeviceSqlConnection, Error, ResourceLimits, Result, SaveStats};
use rbox_testing::data::{create_temp_dir, export_dir};
use std::cell::Cell;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::rc::Rc;

mod support;

use support::decode_rows;

mod copies {
    //! Lossless output, no-clobber publication, and failure recovery for both kinds.

    use super::*;

    #[test]
    fn native_copies_are_byte_identical_and_reopen_in_both_namespaces() -> Result<()> {
        let temporary = create_temp_dir();
        for (kind, name, expected_rows) in [
            (DatabaseKind::Standard, "export.pdb", 82),
            (DatabaseKind::Extended, "exportExt.pdb", 29),
        ] {
            let source = temporary.path().join(name);
            fs::copy(export_dir(7).join("rekordbox").join(name), &source)?;
            let original = fs::read(&source)?;
            let mut connection = DeviceSqlConnection::open(&source, kind)?;
            let destination = temporary.path().join(format!("copy-{name}"));
            connection.save_as(&destination)?;
            assert_eq!(fs::read(&destination)?, original);
            assert_eq!(connection.read_stats().page_reads, 0);

            // Transaction construction/empty commit do not load pages or alter bytes.
            let tx = connection.transaction();
            assert_eq!(tx.kind(), kind);
            assert_eq!(tx.read_stats().page_reads, 0);
            assert_eq!(tx.modified_page_count(), 0);
            tx.commit()?;
            let mut output = vec![11, 22];
            assert_eq!(connection.write_to(&mut output)?, original.len() as u64);
            assert_eq!(&output[2..], original);
            assert_eq!(&output[..2], &[11, 22]);
            assert_eq!(connection.read_stats().page_reads, 0);

            let mut reopened = DeviceSqlConnection::open(&destination, kind)?;
            assert_eq!(reopened.header(), connection.header());
            let mut count = 0;
            for table in connection.header().tables.clone() {
                let table = kind.table_kind(table.table_type);
                let expected = connection.iter_raw(table)?.collect::<Result<Vec<_>>>()?;
                let mut tx = reopened.transaction();
                let actual = tx.iter_raw(table)?.collect::<Result<Vec<_>>>()?;
                assert_eq!(actual, expected);
                for row in &actual {
                    row.decode()?;
                }
                count += actual.len();
                tx.rollback();
            }
            assert_eq!(count, expected_rows);
            assert_eq!(fs::read(&source)?, original);
        }
        Ok(())
    }

    // A valid header declares one table, followed by a corrupt linked page and an
    // opaque unlinked page. Saving must not require successful row/page decoding.
    fn opaque_image() -> Vec<u8> {
        let mut bytes: Vec<u8> = (0..1536).map(|i| (i * 37 + 19) as u8).collect();
        bytes[4..8].copy_from_slice(&512u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
        bytes[28..32].copy_from_slice(&0u32.to_le_bytes());
        bytes[36..40].copy_from_slice(&1u32.to_le_bytes());
        bytes[40..44].copy_from_slice(&1u32.to_le_bytes());
        bytes
    }

    #[test]
    fn saves_preserve_corrupt_linked_pages_unlinked_pages_and_header_slack() -> Result<()> {
        let original = opaque_image();
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let mut connection =
                DeviceSqlConnection::from_reader(Cursor::new(original.clone()), kind)?;
            assert!(connection
                .iter_raw(kind.table_kind(0))?
                .next()
                .unwrap()
                .is_err());
            let mut output = Vec::new();
            connection.write_to(&mut output)?;
            assert_eq!(output, original);
        }
        Ok(())
    }

    #[test]
    fn save_refuses_existing_destination_source_and_hard_link() -> Result<()> {
        let temporary = create_temp_dir();
        let source = temporary.path().join("source.pdb");
        let existing = temporary.path().join("existing.pdb");
        let alias = temporary.path().join("alias.pdb");
        let original = opaque_image();
        fs::write(&source, &original)?;
        fs::write(&existing, b"keep me")?;
        fs::hard_link(&source, &alias)?;
        let mut connection = DeviceSqlConnection::open(&source, DatabaseKind::Standard)?;
        for path in [&source, &existing, &alias] {
            assert!(
                matches!(connection.save_as(path), Err(Error::Save { source, .. }) if source.kind() == io::ErrorKind::AlreadyExists)
            );
        }
        assert_eq!(fs::read(&source)?, original);
        assert_eq!(fs::read(&alias)?, original);
        assert_eq!(fs::read(&existing)?, b"keep me");
        assert_eq!(fs::read_dir(temporary.path())?.count(), 3);
        Ok(())
    }

    struct FailingSource {
        bytes: Cursor<Vec<u8>>,
        fail: Rc<Cell<bool>>,
    }
    impl Read for FailingSource {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.fail.get() && self.bytes.position() >= 512 {
                return Err(io::Error::other("injected source failure"));
            }
            self.bytes.read(output)
        }
    }
    impl Seek for FailingSource {
        fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
            self.bytes.seek(from)
        }
    }

    #[test]
    fn failed_source_copy_leaves_no_destination_or_temporary_file_and_can_retry() -> Result<()> {
        let temporary = create_temp_dir();
        let path = temporary.path().join("copy.pdb");
        let fail = Rc::new(Cell::new(false));
        let original = opaque_image();
        let source = FailingSource {
            bytes: Cursor::new(original.clone()),
            fail: fail.clone(),
        };
        let mut connection = DeviceSqlConnection::from_reader(source, DatabaseKind::Extended)?;
        let header = connection.header().clone();
        fail.set(true);
        assert!(matches!(connection.save_as(&path), Err(Error::Save { .. })));
        assert!(!path.exists());
        assert_eq!(fs::read_dir(temporary.path())?.count(), 0);
        assert_eq!(connection.header(), &header);
        assert_eq!(connection.modified_page_count(), 0);
        fail.set(false);
        connection.save_as(&path)?;
        assert_eq!(fs::read(&path)?, original);
        Ok(())
    }

    struct FailingOutput {
        remaining: usize,
        written: Vec<u8>,
    }
    impl Write for FailingOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.remaining == 0 {
                return Err(io::Error::other("injected output failure"));
            }
            let count = bytes.len().min(self.remaining);
            self.written.extend_from_slice(&bytes[..count]);
            self.remaining -= count;
            Ok(count)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn failed_output_stream_does_not_change_connection_and_can_retry() -> Result<()> {
        let original = opaque_image();
        let mut connection = DeviceSqlConnection::from_reader(
            Cursor::new(original.clone()),
            DatabaseKind::Standard,
        )?;
        let mut output = FailingOutput {
            remaining: 700,
            written: Vec::new(),
        };
        assert!(matches!(
            connection.write_to(&mut output),
            Err(Error::Io(_))
        ));
        assert_eq!(output.written, original[..700]);
        let mut retry = Vec::new();
        connection.write_to(&mut retry)?;
        assert_eq!(retry, original);
        assert_eq!(connection.modified_page_count(), 0);
        Ok(())
    }
}

mod save_back {
    //! Repeated save-back, conflict detection, and bounded retained page buffers.

    use super::support::{entry as row, entry_table as table};
    use super::*;

    #[test]
    fn repeated_save_rebases_after_growth_updates_and_deletions_in_both_kinds() -> Result<()> {
        let temporary = create_temp_dir();
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let path = temporary.path().join(format!("{kind:?}.pdb"));
            DeviceSqlConnection::create(kind)?.save_as(&path)?;
            let mut db = DeviceSqlConnection::open(&path, kind)?;
            let limits = ResourceLimits {
                cached_pages: 1,
                dirty_pages: Some(16),
            };
            db.set_resource_limits(limits)?;
            assert_eq!(db.save()?, SaveStats::default());
            let initial_pages = db.page_count();
            for pass in 0..3 {
                let mut tx = db.transaction();
                if pass == 0 {
                    for id in 0..700 {
                        tx.insert(table(kind), &row(kind, id))?;
                    }
                    assert!(tx.page_count() > initial_pages);
                } else {
                    let first = tx.iter_raw(table(kind))?.next().unwrap()?;
                    let handle = tx.locate(table(kind), first.page(), first.slot().slot)?;
                    if pass == 1 {
                        tx.update(&handle, &row(kind, 900))?;
                    } else {
                        tx.delete(&handle)?;
                    }
                }
                tx.commit()?;
                let mut expected = Vec::new();
                db.write_to(&mut expected)?;
                assert!(db.resource_usage().dirty_pages > 0);
                let stats = db.save()?;
                assert_eq!(stats.pages_written, db.page_count());
                assert_eq!(stats.bytes_written, expected.len() as u64);
                assert_eq!(fs::read(&path)?, expected);
                assert_eq!(db.resource_usage().dirty_pages, 0);
                assert_eq!(db.resource_usage().cached_pages, 0);
                assert_eq!(db.resource_limits(), limits);
                assert_eq!(db.save()?, SaveStats::default());
                let mut reopened = DeviceSqlConnection::open(&path, kind)?;
                let rows = decode_rows(reopened.iter_raw(table(kind))?)?;
                assert_eq!(rows.len(), if pass == 2 { 699 } else { 700 });
                if pass == 1 {
                    assert_eq!(rows[0], row(kind, 900));
                }
                assert_eq!(decode_rows(db.iter_raw(table(kind))?)?, rows);
            }
            // Dropping/rolling back a later transaction remains a no-op on disk.
            let before = fs::read(&path)?;
            db.transaction().insert(table(kind), &row(kind, 1234))?;
            assert_eq!(db.save()?, SaveStats::default());
            assert_eq!(fs::read(&path)?, before);
        }
        Ok(())
    }

    #[test]
    fn source_changes_are_rejected_without_publication_or_losing_pending_pages() -> Result<()> {
        let temporary = create_temp_dir();
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let path = temporary.path().join(format!("{kind:?}.pdb"));
            let copy = temporary.path().join(format!("copy-{kind:?}.pdb"));
            DeviceSqlConnection::create(kind)?.save_as(&path)?;
            let mut db = DeviceSqlConnection::open(&path, kind)?;
            let mut tx = db.transaction();
            tx.insert(table(kind), &row(kind, 7))?;
            tx.commit()?;
            let pending = db.modified_page_count();
            OpenOptions::new()
                .append(true)
                .open(&path)?
                .write_all(b"external edit")?;
            let external = fs::read(&path)?;
            assert!(matches!(db.save(), Err(Error::SourceChanged(_))));
            assert!(matches!(db.save_pages(), Err(Error::SourceChanged(_))));
            assert!(matches!(db.save_as(&copy), Err(Error::SourceChanged(_))));
            assert!(!copy.exists());
            assert_eq!(db.modified_page_count(), pending);
            assert_eq!(fs::read(&path)?, external);
            let mut output = Vec::new();
            assert!(matches!(
                db.write_to(&mut output),
                Err(Error::SourceChanged(_))
            ));
            assert!(output.is_empty());
            fs::remove_file(&path)?;
            assert!(matches!(db.save(), Err(Error::SourceChanged(_))));
        }
        assert_eq!(fs::read_dir(temporary.path())?.count(), 0);
        Ok(())
    }

    #[test]
    fn reader_connections_cannot_implicitly_choose_a_save_path() -> Result<()> {
        let temporary = create_temp_dir();
        let path = temporary.path().join("source.pdb");
        DeviceSqlConnection::create(DatabaseKind::Standard)?.save_as(&path)?;
        let mut db = DeviceSqlConnection::from_reader(File::open(&path)?, DatabaseKind::Standard)?;
        assert!(matches!(db.save(), Err(Error::NoSourcePath)));
        assert!(matches!(db.save_pages(), Err(Error::NoSourcePath)));
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn failed_replacement_retains_source_and_overlay_and_can_retry() -> Result<()> {
        use std::os::windows::fs::OpenOptionsExt;
        let temporary = create_temp_dir();
        let path = temporary.path().join("source.pdb");
        let kind = DatabaseKind::Standard;
        DeviceSqlConnection::create(kind)?.save_as(&path)?;
        let mut db = DeviceSqlConnection::open(&path, kind)?;
        let original = fs::read(&path)?;
        let mut tx = db.transaction();
        tx.insert(table(kind), &row(kind, 1))?;
        tx.commit()?;
        let pending = db.modified_page_count();
        // A different reader disallows deletion/renaming while its handle is open.
        let blocker = OpenOptions::new().read(true).share_mode(3).open(&path)?;
        assert!(matches!(db.save(), Err(Error::Save { .. })));
        assert_eq!(fs::read(&path)?, original);
        assert_eq!(db.modified_page_count(), pending);
        // Only the original database remains; temporary output is removed.
        assert_eq!(fs::read_dir(temporary.path())?.count(), 1);
        drop(blocker);
        db.save()?;
        assert_eq!(db.modified_page_count(), 0);
        Ok(())
    }

    #[test]
    fn dirty_budget_rejects_growth_atomically_and_saved_pages_release_the_budget() -> Result<()> {
        let temporary = create_temp_dir();
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let path = temporary.path().join(format!("{kind:?}.pdb"));
            DeviceSqlConnection::create(kind)?.save_as(&path)?;
            let mut db = DeviceSqlConnection::open(&path, kind)?;
            let limits = ResourceLimits {
                cached_pages: 1,
                dirty_pages: Some(3),
            };
            db.set_resource_limits(limits)?;
            let mut tx = db.transaction();
            let first = tx.insert(table(kind), &row(kind, 0))?;
            let mut inserted = 1;
            loop {
                let header = tx.header().clone();
                let count = tx.page_count();
                match tx.insert(table(kind), &row(kind, inserted)) {
                    Ok(_) => inserted += 1,
                    Err(Error::DirtyPageLimit { limit: 3, required }) => {
                        assert!(required > 3);
                        assert_eq!(tx.header(), &header);
                        assert_eq!(tx.page_count(), count);
                        assert_eq!(
                            tx.iter_raw(table(kind))?.collect::<Result<Vec<_>>>()?.len(),
                            inserted as usize
                        );
                        assert_eq!(tx.read(&first)?.decode()?, row(kind, 0));
                        break;
                    }
                    other => panic!("unexpected insertion result {other:?}"),
                }
                assert!(inserted < 1000, "budget was not enforced");
            }
            tx.commit()?;
            assert_eq!(db.resource_usage().dirty_pages, 3);
            assert!(matches!(
                db.set_resource_limits(ResourceLimits {
                    cached_pages: 0,
                    dirty_pages: Some(2)
                }),
                Err(Error::DirtyPageLimit { .. })
            ));
            assert_eq!(db.resource_limits(), limits);
            db.clear_read_caches();
            assert_eq!(db.resource_usage().cached_pages, 0);
            assert_eq!(db.resource_usage().dirty_pages, 3);
            db.save()?;
            // Growth needs header, previous data page, the EOF gap, and new page.
            // The formerly dirty index page must no longer consume this budget.
            db.set_resource_limits(ResourceLimits {
                cached_pages: 1,
                dirty_pages: Some(4),
            })?;
            let mut tx = db.transaction();
            tx.insert(table(kind), &row(kind, inserted))?;
            tx.commit()?;
            db.save()?;
            assert_eq!(
                db.iter_raw(table(kind))?.collect::<Result<Vec<_>>>()?.len(),
                inserted as usize + 1
            );
            db.set_resource_limits(ResourceLimits {
                cached_pages: 0,
                dirty_pages: Some(0),
            })?;
            let before_reads = db.read_stats().page_reads;
            let first = db.iter_raw(table(kind))?.next().unwrap()?;
            assert!(db.read_stats().page_reads > before_reads);
            assert_eq!(db.resource_usage().cached_pages, 0);
            let mut tx = db.transaction();
            let handle = tx.locate(table(kind), first.page(), first.slot().slot)?;
            assert!(matches!(
                tx.update(&handle, &row(kind, 9999)),
                Err(Error::DirtyPageLimit { .. })
            ));
            assert_eq!(tx.read(&handle)?.decode()?, first.decode()?);
            assert_eq!(tx.modified_page_count(), 0);
        }
        Ok(())
    }
}

mod compaction {
    //! Explicit compaction, canonical index reconstruction, and refusal of opaque rows.

    use super::support::association as entry;
    use super::*;

    fn assert_fresh_directories(bytes: Vec<u8>) -> Result<()> {
        let mut input = Cursor::new(bytes);
        let header = FileHeader::read_le(&mut input)?;
        for chain in read_table_chains(&mut input, &header)? {
            assert!(matches!(chain.pages[0].kind, PageHeaderKind::Index(_)));
            for page in chain.pages {
                if page.kind.is_index() {
                    assert!(
                        read_index_entries(&mut input, header.prefix.page_size, &page)?.is_empty()
                    );
                } else {
                    assert_eq!(page.common.counts.allocated(), page.common.counts.live());
                    read_row_directory(&mut input, header.prefix.page_size, &page)?;
                }
            }
        }
        Ok(())
    }

    #[test]
    fn compact_reclaims_pages_preserves_live_rows_and_allows_more_page_saves() -> Result<()> {
        for kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            let temp = create_temp_dir();
            let path = temp.path().join("compact.pdb");
            DeviceSqlConnection::create(kind)?.save_as(&path)?;
            let mut db = DeviceSqlConnection::open(&path, kind)?;
            let (table, _) = entry(kind, 0);
            let mut tx = db.transaction();
            for id in 0..1200 {
                tx.insert(table, &entry(kind, id).1)?;
            }
            tx.commit()?;
            db.save_pages()?;
            let large = fs::metadata(&path)?.len();
            let mut tx = db.transaction();
            let locations = tx.iter_raw(table)?.collect::<Result<Vec<_>>>()?;
            for row in locations.iter().skip(10) {
                let handle = tx.locate(table, row.page(), row.slot().slot)?;
                tx.delete(&handle)?;
            }
            tx.commit()?;
            let pending = db.modified_page_count();
            let (mut rebuilt, stats) = db.rebuild()?;
            assert_eq!(stats.live_rows, 10);
            assert!(stats.rebuilt_pages < stats.original_pages);
            assert_eq!(db.modified_page_count(), pending);
            assert_eq!(fs::metadata(&path)?.len(), large);
            let expected = (0..10).map(|id| entry(kind, id).1).collect::<Vec<_>>();
            assert_eq!(decode_rows(rebuilt.iter_raw(table)?)?, expected);
            let stats = db.compact()?;
            assert_eq!(stats.live_rows, 10);
            assert_eq!(db.resource_usage().dirty_pages, 0);
            assert!(fs::metadata(&path)?.len() < large);
            assert_fresh_directories(fs::read(&path)?)?;
            assert_eq!(decode_rows(db.iter_raw(table)?)?, expected);
            let mut tx = db.transaction();
            tx.insert(table, &entry(kind, 2000).1)?;
            tx.commit()?;
            db.save_pages()?;
            let mut reopened = DeviceSqlConnection::open(&path, kind)?;
            assert_eq!(
                reopened.iter_raw(table)?.collect::<Result<Vec<_>>>()?.len(),
                11
            );
        }
        Ok(())
    }

    #[test]
    fn demo_rebuilds_preserve_canonical_binary_content_in_both_namespaces() -> anyhow::Result<()> {
        let temp = create_temp_dir();
        for (kind, name) in [
            (DatabaseKind::Standard, "export.pdb"),
            (DatabaseKind::Extended, "exportExt.pdb"),
        ] {
            let path = temp.path().join(name);
            fs::copy(export_dir(7).join("rekordbox").join(name), &path)?;
            let original = fs::read(&path)?;
            let mut db = DeviceSqlConnection::open(&path, kind)?;
            let expected = contents::read_rows(&mut db)?;
            let (mut first, stats) = db.rebuild()?;
            let actual = contents::read_rows(&mut first)?;
            contents::verify(&expected, &actual, kind, "compaction")?;
            assert!(stats.live_rows > 0);
            let mut first_bytes = Vec::new();
            first.write_to(&mut first_bytes)?;
            assert_fresh_directories(first_bytes.clone())?;
            // Rebuilding the same original input must be deterministic; compaction
            // is the same result with publication/rebasing added.
            db.compact()?;
            assert_eq!(fs::read(&path)?, first_bytes);
            assert_eq!(
                fs::read(export_dir(7).join("rekordbox").join(name))?,
                original
            );
            assert_eq!(
                db.header().prefix.unknown_10,
                first.header().prefix.unknown_10
            );
        }
        Ok(())
    }

    #[test]
    fn rebuild_failures_never_publish_or_silently_drop_live_opaque_rows() -> Result<()> {
        let kind = DatabaseKind::Standard;
        let mut db = DeviceSqlConnection::create(kind)?;
        let (table, row) = entry(kind, 7);
        let mut tx = db.transaction();
        tx.insert(table, &row)?;
        tx.commit()?;
        let mut bytes = Vec::new();
        db.write_to(&mut bytes)?;
        // Switch the populated known table's directory/pages to an unknown table,
        // retaining a canonical directory by swapping its physical chains.
        let a = 28 + 8 * 16;
        let b = 28 + 9 * 16;
        for offset in 4..16 {
            bytes.swap(a + offset, b + offset);
        }
        for (page, code) in [(17usize, 9u32), (18, 9), (19, 8)] {
            bytes[page * 4096 + 8..page * 4096 + 12].copy_from_slice(&code.to_le_bytes());
        }
        let temp = create_temp_dir();
        let path = temp.path().join("opaque.pdb");
        fs::write(&path, &bytes)?;
        let mut db = DeviceSqlConnection::open(&path, kind)?;
        assert!(matches!(
            db.compact(),
            Err(Error::UnsupportedMutation { .. })
        ));
        assert_eq!(fs::read(&path)?, bytes);
        assert_eq!(db.modified_page_count(), 0);

        let mut db = DeviceSqlConnection::create(kind)?;
        let mut tx = db.transaction();
        tx.insert(table, &row)?;
        tx.commit()?;
        let mut bytes = Vec::new();
        db.write_to(&mut bytes)?;
        bytes[4000] = 7; // An opaque page-zero byte cannot safely be relocated.
        let mut db = DeviceSqlConnection::from_reader(Cursor::new(bytes), kind)?;
        assert!(matches!(db.rebuild(), Err(Error::InvalidCreation { .. })));
        Ok(())
    }

    #[test]
    fn rebuild_obeys_dirty_budget_without_consuming_original_edits() -> Result<()> {
        let temp = create_temp_dir();
        let path = temp.path().join("budget.pdb");
        let mut db = DeviceSqlConnection::create(DatabaseKind::Standard)?;
        let (table, row) = entry(DatabaseKind::Standard, 1);
        let mut tx = db.transaction();
        tx.insert(table, &row)?;
        tx.commit()?;
        db.save_as(&path)?;
        let mut db = DeviceSqlConnection::open(&path, DatabaseKind::Standard)?;
        db.set_resource_limits(ResourceLimits {
            cached_pages: 1,
            dirty_pages: Some(2),
        })?;
        assert!(matches!(db.rebuild(), Err(Error::DirtyPageLimit { .. })));
        assert_eq!(db.iter_raw(table)?.next().unwrap()?.decode()?, row);
        Ok(())
    }
}

mod contents {
    //! Test-only comparison of decoded contents, ignoring physical placement.

    use std::collections::BTreeMap;
    use std::io::{Read, Seek};

    use anyhow::{bail, ensure, Result};
    use rbox_pdb::binary::{DatabaseKind, ExtendedRow, Row, StandardRow, StringOffsets};
    use rbox_pdb::devicesql::DeviceSqlConnection;
    #[cfg(test)]
    use rbox_testing::data::export_dir;

    /// Decoded live rows keyed by their table code within one database namespace.
    pub type Tables = BTreeMap<u32, Vec<Row>>;

    /// Decode every live row, rejecting unsupported layouts in these demo fixtures.
    pub fn read_rows<R: Read + Seek>(db: &mut DeviceSqlConnection<R>) -> Result<Tables> {
        let mut tables = Tables::new();
        for pointer in db.header().tables.clone() {
            let table = db.kind().table_kind(pointer.table_type);
            let rows = db
                .iter_raw(table)?
                .map(|record| record?.decode())
                .collect::<rbox_pdb::Result<Vec<_>>>()?;
            ensure!(
                rows.iter().all(|row| !matches!(
                    row,
                    Row::Standard(StandardRow::Opaque) | Row::Extended(ExtendedRow::Opaque)
                )),
                "{table:?}: unexpected opaque live row in compaction fixture"
            );
            tables.insert(pointer.table_type, rows);
        }
        Ok(tables)
    }

    fn clear_offsets<const N: usize>(strings: &mut StringOffsets<N>) {
        match strings {
            StringOffsets::Near { offsets, .. } => *offsets = [0; N],
            StringOffsets::Far { offsets, .. } => *offsets = [0; N],
        }
    }

    // Ignore placement only. String frames, markers, unknown fields, IDs, and every
    // metadata value still participate in equality. Sorting preserves duplicates.
    fn comparable(rows: &[Row]) -> Vec<String> {
        let mut values = Vec::new();
        for mut row in rows.iter().cloned() {
            match &mut row {
                Row::Standard(StandardRow::Track(r)) => {
                    r.header.index_shift = 0;
                    clear_offsets(&mut r.strings);
                }
                Row::Standard(StandardRow::Artist(r)) => {
                    r.header.index_shift = 0;
                    clear_offsets(&mut r.strings);
                }
                Row::Standard(StandardRow::ExportProperty(r)) => r.index_shift = 0,
                Row::Extended(ExtendedRow::MyTag(r)) => {
                    r.header.index_shift = 0;
                    clear_offsets(&mut r.strings);
                }
                Row::Extended(ExtendedRow::DatabaseProperty(r)) => {
                    r.header.index_shift = 0;
                    clear_offsets(&mut r.strings);
                }
                _ => {}
            }
            values.push(format!("{row:?}"));
        }
        values.sort();
        values
    }

    /// Compare live contents and string frames while ignoring physical row placement.
    pub fn verify(
        expected: &Tables,
        actual: &Tables,
        kind: DatabaseKind,
        label: &str,
    ) -> Result<()> {
        ensure!(
            expected.keys().eq(actual.keys()),
            "{label}: table directory differs"
        );
        for (&code, expected) in expected {
            let actual = &actual[&code];
            let expected_values = comparable(expected);
            let actual_values = comparable(actual);
            if expected_values != actual_values {
                let missing = expected_values.iter().find(|r| !actual_values.contains(r));
                let extra = actual_values.iter().find(|r| !expected_values.contains(r));
                bail!("{label}: {:?}: expected {} rows, got {}\nFirst missing: {missing:?}\nFirst extra: {extra:?}",
                    kind.table_kind(code), expected.len(), actual.len());
            }
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use rbox_testing::data::create_temp_dir;

        #[test]
        fn comparison_ignores_placement_but_detects_unknown_changes_and_duplicates() -> Result<()> {
            let temporary = create_temp_dir();
            for (kind, name) in [
                (DatabaseKind::Standard, "export.pdb"),
                (DatabaseKind::Extended, "exportExt.pdb"),
            ] {
                let path = temporary.path().join(name);
                std::fs::copy(export_dir(7).join("rekordbox").join(name), &path)?;
                let mut db = DeviceSqlConnection::open(&path, kind)?;
                let rows = read_rows(&mut db)?;
                let code = if kind == DatabaseKind::Standard { 0 } else { 3 };
                let original = &rows[&code];
                let mut moved = original.clone();
                moved.reverse();
                for row in &mut moved {
                    match row {
                        Row::Standard(StandardRow::Track(r)) => {
                            r.header.index_shift = 0;
                            clear_offsets(&mut r.strings);
                        }
                        Row::Extended(ExtendedRow::MyTag(r)) => {
                            r.header.index_shift = 0;
                            clear_offsets(&mut r.strings);
                        }
                        _ => unreachable!(),
                    }
                }
                assert_eq!(comparable(original), comparable(&moved));
                let mut duplicate = moved.clone();
                duplicate.push(moved[0].clone());
                assert_ne!(comparable(original), comparable(&duplicate));
                match &mut moved[0] {
                    Row::Standard(StandardRow::Track(r)) => r.header.content_link ^= 1,
                    Row::Extended(ExtendedRow::MyTag(r)) => r.header.unknown_04 ^= 1,
                    _ => unreachable!(),
                }
                assert_ne!(comparable(original), comparable(&moved));
            }
            Ok(())
        }
    }
}
