// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Table namespaces, linked-page traversal, index entries, and row directories.

use binrw::{BinRead, BinWrite, Endian};
use rbox_devlib::binary::{
    read_index_entries, read_row_directory, read_table_chains, DatabaseKind, ExtPageType,
    FileHeader, IndexEntry, PackedRowCounts, PageHeader, PageHeaderKind, PageIndex, PageType,
    RowGroup, TableKind, TablePointer,
};
use rbox_devlib::{Error, Result};
use std::io::{self, Cursor, Read, Seek, SeekFrom};

mod table_names {
    //! Public table-kind conversions, namespace collisions, and unresolved codes.

    use super::*;

    #[test]
    fn overlapping_codes_require_explicit_database_context() {
        let standard = DatabaseKind::Standard;
        let extended = DatabaseKind::Extended;
        assert_eq!(
            standard.table_kind(3),
            TableKind::Standard(PageType::Albums)
        );
        assert_eq!(
            extended.table_kind(3),
            TableKind::Extended(ExtPageType::MyTags)
        );
        assert_ne!(standard.table_kind(3), extended.table_kind(3));
        assert_eq!(standard.table_name(3), Some("Albums"));
        assert_eq!(extended.table_name(3), Some("My Tags"));
        assert_eq!(standard.table_name(4), Some("Labels"));
        assert_eq!(extended.table_name(4), Some("Track My Tags"));
        assert_eq!(standard.table_name(7), Some("Playlist tree"));
        assert_eq!(extended.table_name(7), Some("Database property options"));
    }

    #[test]
    fn decimal_history_codes_are_distinct_from_hexadecimal_browse_codes() {
        let kind = DatabaseKind::Standard;
        assert_eq!(kind.table_name(11), Some("History playlists"));
        assert_eq!(kind.table_name(12), Some("History entries"));
        assert_eq!(kind.table_name(0x11), Some("Browse category settings"));
        assert_eq!(kind.table_name(0x12), Some("Browse sort settings"));
        assert_eq!(kind.table_name(19), Some("Export properties"));
    }

    #[test]
    fn unknown_codes_are_not_given_invented_names() {
        for code in [9, 10, 14, 15, 20, 0x100, u32::MAX] {
            assert_eq!(DatabaseKind::Standard.table_name(code), None);
        }
        for code in [9, 19, 0x100, u32::MAX] {
            assert_eq!(DatabaseKind::Extended.table_name(code), None);
        }
    }

    #[test]
    fn table_kinds_roundtrip_codes_and_namespaces() {
        for database in [DatabaseKind::Standard, DatabaseKind::Extended] {
            for code in (0..=256).chain([0x8000_0000, u32::MAX]) {
                let table = database.table_kind(code);
                assert_eq!(table.code(), code);
                assert_eq!(table.database_kind(), database);
                assert_eq!(table.database_kind().table_kind(table.code()), table);
                assert_eq!(table.name(), database.table_name(code));
            }
        }
    }

    #[test]
    fn unknown_kinds_keep_full_width_codes_and_database_identity() {
        for database_kind in [DatabaseKind::Standard, DatabaseKind::Extended] {
            for code in [9, 10, 14, 15, 20, 0x8000_0000, u32::MAX] {
                assert_eq!(
                    database_kind.table_kind(code),
                    match database_kind {
                        DatabaseKind::Standard => TableKind::Standard(PageType::Unknown(code)),
                        DatabaseKind::Extended => TableKind::Extended(ExtPageType::Unknown(code)),
                    },
                );
            }
        }
        assert_ne!(
            DatabaseKind::Standard.table_kind(u32::MAX),
            DatabaseKind::Extended.table_kind(u32::MAX),
        );
    }

    #[test]
    fn page_type_codecs_match_namespace_conversions_and_preserve_wire_bytes(
    ) -> rbox_devlib::Result<()> {
        for code in (0u32..=256).chain([0x8000_0000, u32::MAX]) {
            let mut bytes = vec![0xaa];
            bytes.extend_from_slice(&code.to_le_bytes());
            bytes.push(0xbb);
            let mut input = Cursor::new(&bytes);
            input.set_position(1);
            // The on-disk byte order remains little endian even with a big-endian caller.
            let standard = PageType::read_options(&mut input, Endian::Big, ())?;
            assert_eq!(input.position(), 5);
            assert_eq!(standard, PageType::from_code(code));
            assert_eq!(standard.code(), code);
            input.set_position(1);
            let extended = ExtPageType::read_options(&mut input, Endian::Big, ())?;
            assert_eq!(input.position(), 5);
            assert_eq!(extended, ExtPageType::from_code(code));
            assert_eq!(extended.code(), code);

            let mut output = Cursor::new(bytes.clone());
            output.set_position(1);
            standard.write_options(&mut output, Endian::Big, ())?;
            assert_eq!(output.position(), 5);
            assert_eq!(output.get_ref(), &bytes);
            output.set_position(1);
            extended.write_options(&mut output, Endian::Big, ())?;
            assert_eq!(output.position(), 5);
            assert_eq!(output.into_inner(), bytes);
        }
        Ok(())
    }

    #[test]
    fn truncated_page_type_codes_fail_in_both_namespaces() {
        for code in [3u32, u32::MAX] {
            for length in 0..4 {
                let bytes = code.to_le_bytes();
                assert!(PageType::read_le(&mut Cursor::new(&bytes[..length])).is_err());
                assert!(ExtPageType::read_le(&mut Cursor::new(&bytes[..length])).is_err());
            }
        }
    }
}

mod table_chains {
    //! Chain traversal on independent wire bytes, including malformed links and lazy I/O.

    use super::*;

    const PAGE_SIZE: usize = 512;

    fn word(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    // Unknown table code; deliberately descending, then ascending: 3 -> 1 -> 2.
    // Page 4 is unlinked garbage and must never be decoded. Allocator refs exceed EOF.
    fn fixture() -> (Vec<u8>, FileHeader) {
        let mut bytes = vec![0; PAGE_SIZE * 5];
        word(&mut bytes, 4, PAGE_SIZE as u32);
        word(&mut bytes, 8, 1);
        word(&mut bytes, 12, 101);
        word(&mut bytes, 28, 0xfeed);
        word(&mut bytes, 32, 100);
        word(&mut bytes, 36, 3);
        word(&mut bytes, 40, 2);
        for (page, next, flags) in [(3, 1, 0x64), (1, 2, 0x24), (2, 100, 0x24)] {
            let offset = page * PAGE_SIZE;
            word(&mut bytes, offset + 4, page as u32);
            word(&mut bytes, offset + 8, 0xfeed);
            word(&mut bytes, offset + 12, next);
            bytes[offset + 27] = flags;
        }
        bytes[4 * PAGE_SIZE..].fill(0xff);
        let header = FileHeader::read_le(&mut Cursor::new(&bytes)).unwrap();
        (bytes, header)
    }

    #[test]
    fn follows_noncontiguous_links_and_stops_before_allocation_candidate() -> Result<()> {
        let (bytes, header) = fixture();
        let chains = read_table_chains(&mut Cursor::new(bytes), &header)?;
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].table, header.tables[0]);
        let indices: Vec<_> = chains[0]
            .pages
            .iter()
            .map(|page| page.common.page_index.0)
            .collect();
        assert_eq!(indices, [3, 1, 2]);
        Ok(())
    }

    #[test]
    fn single_index_page_is_a_valid_empty_table() -> Result<()> {
        let (bytes, mut header) = fixture();
        header.tables[0].last_page = PageIndex(3);
        let chains = read_table_chains(&mut Cursor::new(bytes), &header)?;
        assert_eq!(chains[0].pages.len(), 1);
        assert!(chains[0].pages[0].kind.is_index());
        Ok(())
    }

    #[test]
    fn rejects_zero_sentinel_and_out_of_file_endpoints() {
        for page in [0, PageIndex::NONE.0, 5, u32::MAX] {
            for first in [true, false] {
                let (bytes, mut header) = fixture();
                if first {
                    header.tables[0].first_page = PageIndex(page);
                } else {
                    header.tables[0].last_page = PageIndex(page);
                }
                assert!(
                    matches!(read_table_chains(&mut Cursor::new(bytes), &header),
                    Err(Error::InvalidPageReference { table_index: 0, page: actual }) if actual == PageIndex(page))
                );
            }
        }
    }

    #[test]
    fn rejects_invalid_successors_before_declared_last_page() {
        for next in [0, PageIndex::NONE.0, 5, u32::MAX] {
            let (mut bytes, header) = fixture();
            word(&mut bytes, PAGE_SIZE + 12, next);
            assert!(
                matches!(read_table_chains(&mut Cursor::new(bytes), &header),
                Err(Error::InvalidPageReference { table_index: 0, page }) if page == PageIndex(next))
            );
        }
    }

    #[test]
    fn rejects_self_links_and_longer_cycles_before_declared_last_page() {
        for next in [1, 3] {
            let (mut bytes, header) = fixture();
            word(&mut bytes, PAGE_SIZE + 12, next);
            assert!(
                matches!(read_table_chains(&mut Cursor::new(bytes), &header),
                Err(Error::PageCycle { table_index: 0, page }) if page == PageIndex(next))
            );
        }
    }

    #[test]
    fn last_page_successor_is_never_followed_or_validated() -> Result<()> {
        for next in [0, 2, 3, 4, PageIndex::NONE.0, u32::MAX] {
            let (mut bytes, header) = fixture();
            word(&mut bytes, 2 * PAGE_SIZE + 12, next);
            assert_eq!(
                read_table_chains(&mut Cursor::new(bytes), &header)?[0]
                    .pages
                    .len(),
                3
            );
        }
        Ok(())
    }

    #[test]
    fn rejects_shared_pages_even_when_table_codes_match() {
        let (bytes, mut header) = fixture();
        header.tables.push(TablePointer {
            first_page: PageIndex(1),
            ..header.tables[0]
        });
        header.prefix.table_count = 2;
        assert!(matches!(
            read_table_chains(&mut Cursor::new(bytes), &header),
            Err(Error::SharedPage {
                page: PageIndex(1),
                first_table: 0,
                second_table: 1
            })
        ));
    }

    #[test]
    fn rejects_self_index_and_table_code_mismatches() {
        let (mut bytes, header) = fixture();
        word(&mut bytes, PAGE_SIZE + 4, 4);
        assert!(matches!(
            read_table_chains(&mut Cursor::new(bytes), &header),
            Err(Error::PageIndexMismatch {
                table_index: 0,
                expected: PageIndex(1),
                actual: PageIndex(4)
            })
        ));
        let (mut bytes, header) = fixture();
        word(&mut bytes, PAGE_SIZE + 8, 7);
        assert!(matches!(
            read_table_chains(&mut Cursor::new(bytes), &header),
            Err(Error::PageTableMismatch {
                table_index: 0,
                page: PageIndex(1),
                expected: 0xfeed,
                actual: 7
            })
        ));
    }

    #[test]
    fn rejects_invalid_layouts_before_traversal() {
        for size in [0, 40, 511, 131072] {
            let (bytes, mut header) = fixture();
            header.prefix.page_size = size;
            assert!(matches!(
                read_table_chains(&mut Cursor::new(bytes), &header),
                Err(Error::InvalidFileLayout { .. })
            ));
        }
        for length in [0, PAGE_SIZE - 1, PAGE_SIZE * 5 - 1] {
            let (mut bytes, header) = fixture();
            bytes.truncate(length);
            assert!(matches!(
                read_table_chains(&mut Cursor::new(bytes), &header),
                Err(Error::InvalidFileLayout { .. })
            ));
        }
    }

    struct HeaderOnlyReader {
        inner: Cursor<Vec<u8>>,
        bytes_read: usize,
        fail_page: Option<usize>,
    }

    impl Read for HeaderOnlyReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let start = self.inner.position() as usize;
            let page = start / PAGE_SIZE;
            if self.fail_page == Some(page) {
                return Err(io::Error::other("injected read failure"));
            }
            let header_end = match page {
                1 | 2 => 40,
                3 => 60,
                _ => 0,
            };
            if start % PAGE_SIZE + buffer.len() > header_end {
                return Err(io::Error::other(
                    "attempt to read a payload or unlinked page",
                ));
            }
            let count = self.inner.read(buffer)?;
            self.bytes_read += count;
            Ok(count)
        }
    }

    impl Seek for HeaderOnlyReader {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.inner.seek(position)
        }
    }

    #[test]
    fn reads_only_linked_headers_and_leaves_payloads_and_unlinked_pages_unread() -> Result<()> {
        let (bytes, header) = fixture();
        let mut input = HeaderOnlyReader {
            inner: Cursor::new(bytes),
            bytes_read: 0,
            fail_page: None,
        };
        let chains = read_table_chains(&mut input, &header)?;
        assert_eq!(chains[0].pages.len(), 3);
        assert_eq!(input.bytes_read, 60 + 40 + 40);
        Ok(())
    }

    #[test]
    fn page_read_errors_include_table_and_page_context() {
        let (bytes, header) = fixture();
        let mut input = HeaderOnlyReader {
            inner: Cursor::new(bytes),
            bytes_read: 0,
            fail_page: Some(1),
        };
        assert!(matches!(
            read_table_chains(&mut input, &header),
            Err(Error::PageRead {
                table_index: 0,
                page: PageIndex(1),
                ..
            })
        ));
    }

    #[test]
    fn empty_directory_reads_no_pages() -> Result<()> {
        let (bytes, mut header) = fixture();
        header.tables.clear();
        header.prefix.table_count = 0;
        let mut input = HeaderOnlyReader {
            inner: Cursor::new(bytes),
            bytes_read: 0,
            fail_page: None,
        };
        assert!(read_table_chains(&mut input, &header)?.is_empty());
        assert_eq!(input.bytes_read, 0);
        Ok(())
    }
}

mod index_entries {
    //! Packed index words, exact array boundaries, and malformed index-page inputs.

    use super::*;

    fn fixture() -> (Vec<u8>, PageHeader) {
        let mut bytes = vec![0; 1536];
        bytes[516..520].copy_from_slice(&1u32.to_le_bytes());
        bytes[539] = 0x64;
        bytes[548..550].copy_from_slice(&2u16.to_le_bytes()); // Apparent capacity isn't a read length.
        bytes[550..552].copy_from_slice(&u16::MAX.to_le_bytes()); // Uninterpreted high-water value.
        bytes[568..570].copy_from_slice(&5u16.to_le_bytes());
        bytes[570..572].copy_from_slice(&u16::MAX.to_le_bytes());
        bytes[572..592].copy_from_slice(&[
            0x17, 0, 0, 0, // Page 2, all flag bits set.
            0xf8, 0xff, 0xff, 0x1f, // Exact empty marker.
            0xf9, 0xff, 0xff, 0x1f, // Same reference, different flags: not the empty marker.
            0xff, 0xff, 0xff, 0xff, // Full 29-bit reference, flags 7.
            0, 0, 0, 0, // Page zero, not silently interpreted as empty.
        ]);
        bytes[592..1024].fill(0xab); // Unused entry area and tail must remain unread.
        bytes[1024..].fill(0xcd); // Referenced pages must not be loaded.
        let mut input = Cursor::new(&bytes);
        input.set_position(512);
        let header = PageHeader::read_le(&mut input).unwrap();
        (bytes, header)
    }

    struct EntryOnlyReader {
        inner: Cursor<Vec<u8>>,
        read_end: u64,
        bytes_read: usize,
    }

    impl Read for EntryOnlyReader {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let position = self.inner.position();
            if position < 572 || position + output.len() as u64 > self.read_end {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "outside declared entry window",
                ));
            }
            let count = self.inner.read(output)?;
            self.bytes_read += count;
            Ok(count)
        }
    }

    impl Seek for EntryOnlyReader {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.inner.seek(position)
        }
    }

    #[test]
    fn reads_only_declared_entries_and_keeps_unknown_references() -> Result<()> {
        let (bytes, header) = fixture();
        let mut input = EntryOnlyReader {
            inner: Cursor::new(bytes),
            read_end: 592,
            bytes_read: 0,
        };
        let entries = read_index_entries(&mut input, 512, &header)?;
        assert_eq!(
            entries.iter().map(|entry| entry.0).collect::<Vec<_>>(),
            [0x17, 0x1fff_fff8, 0x1fff_fff9, u32::MAX, 0]
        );
        assert_eq!(entries[0].page_index(), PageIndex(2));
        assert_eq!(entries[0].flags(), 7);
        assert!(entries[1].is_empty());
        assert_eq!(entries[1].page_index(), PageIndex::NONE);
        assert!(!entries[2].is_empty());
        assert_eq!(entries[2].page_index(), PageIndex::NONE);
        assert_eq!(entries[2].flags(), 1);
        assert_eq!(entries[3].page_index(), PageIndex(0x1fff_ffff));
        assert!(!entries[4].is_empty());
        assert_eq!(input.bytes_read, 20);
        assert_eq!(input.inner.position(), 592);
        Ok(())
    }

    #[test]
    fn packed_word_codec_is_little_endian_and_preserves_all_flags() -> Result<()> {
        for reference in [0u32, 2, PageIndex::NONE.0, 0x1fff_ffff] {
            for flags in 0..8 {
                let raw = (reference << 3) | flags;
                let mut bytes = vec![0xaa];
                bytes.extend_from_slice(&raw.to_le_bytes());
                bytes.push(0xbb);
                let mut input = Cursor::new(&bytes);
                input.set_position(1);
                let entry = IndexEntry::read_options(&mut input, Endian::Big, ())?;
                assert_eq!(entry.0, raw);
                assert_eq!(entry.page_index(), PageIndex(reference));
                assert_eq!(entry.flags(), flags as u8);
                assert_eq!(entry.is_empty(), raw == 0x1fff_fff8);
                assert_eq!(input.position(), 5);
                let mut output = Cursor::new(bytes.clone());
                output.set_position(1);
                entry.write_options(&mut output, Endian::Big, ())?;
                assert_eq!(output.position(), 5);
                assert_eq!(output.into_inner(), bytes);
            }
        }
        Ok(())
    }

    #[test]
    fn zero_entries_read_nothing_and_physical_boundary_is_inclusive() -> Result<()> {
        for count in [0, 113] {
            // (512 - 60) / 4: physical bound, no assumed tail profile.
            let (bytes, mut header) = fixture();
            let PageHeaderKind::Index(ref mut index) = header.kind else {
                unreachable!()
            };
            index.entry_count = count;
            let mut input = EntryOnlyReader {
                inner: Cursor::new(bytes),
                read_end: 572 + u64::from(count) * 4,
                bytes_read: 0,
            };
            assert_eq!(
                read_index_entries(&mut input, 512, &header)?.len(),
                usize::from(count)
            );
            assert_eq!(input.bytes_read, usize::from(count) * 4);
        }
        Ok(())
    }

    #[test]
    fn oversized_counts_fail_before_any_stream_access() {
        for count in [114, u16::MAX] {
            let (bytes, mut header) = fixture();
            let PageHeaderKind::Index(ref mut index) = header.kind else {
                unreachable!()
            };
            index.entry_count = count;
            let mut input = Cursor::new(bytes);
            input.set_position(17);
            assert!(matches!(
                read_index_entries(&mut input, 512, &header),
                Err(Error::InvalidIndexEntries {
                    page: PageIndex(1),
                    ..
                })
            ));
            assert_eq!(input.position(), 17);
        }
    }

    #[test]
    fn rejects_wrong_page_kind_invalid_sizes_and_missing_physical_pages() -> Result<()> {
        let (bytes, header) = fixture();
        let mut invalid_header = header;
        invalid_header.common.flags.0 = 0x24;
        assert!(matches!(
            read_index_entries(&mut Cursor::new(&bytes), 512, &invalid_header),
            Err(Error::InvalidIndexEntries { .. })
        ));
        invalid_header = header;
        invalid_header.kind = PageHeader::read_le(&mut Cursor::new([0; 40]))?.kind;
        assert!(matches!(
            read_index_entries(&mut Cursor::new(&bytes), 512, &invalid_header),
            Err(Error::InvalidIndexEntries { .. })
        ));
        for size in [0, 40, 511, 513, u32::MAX] {
            assert!(matches!(
                read_index_entries(&mut Cursor::new(&bytes), size, &header),
                Err(Error::InvalidIndexEntries { .. })
            ));
        }
        for index in [0, 3, PageIndex::NONE.0, u32::MAX] {
            let mut invalid_header = header;
            invalid_header.common.page_index = PageIndex(index);
            assert!(matches!(
                read_index_entries(&mut Cursor::new(&bytes), 512, &invalid_header),
                Err(Error::InvalidIndexEntries { .. })
            ));
        }
        assert!(matches!(
            read_index_entries(&mut Cursor::new(&bytes[..1023]), 512, &header),
            Err(Error::InvalidIndexEntries { .. })
        ));
        Ok(())
    }

    #[test]
    fn truncated_entries_return_errors_with_page_context() {
        for length in 0..4 {
            assert!(IndexEntry::read_le(&mut Cursor::new(vec![0; length])).is_err());
        }
        let (bytes, header) = fixture();
        // Stream length still reports the complete page, but the last entry cannot be read.
        let mut input = EntryOnlyReader {
            inner: Cursor::new(bytes),
            read_end: 591,
            bytes_read: 0,
        };
        assert!(matches!(
            read_index_entries(&mut input, 512, &header),
            Err(Error::IndexEntriesRead {
                page: PageIndex(1),
                ..
            })
        ));
    }
}

mod row_directories {
    //! Reverse-footer layout, malformed directories, and explicit heap-read boundaries.

    use super::*;

    fn fixture() -> (Vec<u8>, PageHeader) {
        let mut bytes = vec![0; 1024];
        bytes[516..520].copy_from_slice(&1u32.to_le_bytes()); // Self-index.
        bytes[536..539].copy_from_slice(&[0x11, 0x60, 0]); // 17 allocated, 3 live.
        bytes[539] = 0x24;
        bytes[540..542].copy_from_slice(&366u16.to_le_bytes());
        bytes[542..544].copy_from_slice(&64u16.to_le_bytes());

        // Partial group first on disk: slot 16 offset=48, live bit 0, unknown bit 15.
        let mut footer = vec![48, 0, 1, 0x80, 0, 0x80];
        footer.extend_from_slice(&[0xff; 26]); // Inactive slots 15..3 have stale offsets.
        footer.extend_from_slice(&[0, 0, 0xff, 0xff, 24, 0]); // Slots 2, 1, 0.
        footer.extend_from_slice(&[5, 0, 6, 0]); // Live 0/2; touched 1/2.
        assert_eq!(footer.len(), 42);
        bytes[982..].copy_from_slice(&footer);
        let mut input = Cursor::new(&bytes);
        input.set_position(512);
        let header = PageHeader::read_le(&mut input).unwrap();
        (bytes, header)
    }

    #[test]
    fn partial_groups_masks_and_unsorted_live_windows_are_preserved() -> Result<()> {
        let (bytes, header) = fixture();
        let directory = read_row_directory(&mut Cursor::new(&bytes), 512, &header)?;
        assert_eq!(directory.groups().len(), 2);
        assert_eq!(directory.groups()[0].offsets.len(), 16);
        assert_eq!(directory.groups()[1].offsets, [48]);
        assert_eq!(directory.groups()[1].presence_flags, 0x8001);
        assert_eq!(directory.groups()[1].transaction_flags, 0x8000);
        let slots: Vec<_> = directory.slots().collect();
        assert_eq!(slots.len(), 17);
        assert_eq!(slots[0].heap_offset, 24);
        assert!(slots[0].live && !slots[0].touched);
        assert_eq!(slots[1].heap_offset, u16::MAX);
        assert!(!slots[1].live && slots[1].touched);
        assert!(slots[2].live && slots[2].touched);
        assert!(slots[16].live && !slots[16].touched);
        assert_eq!(directory.live_row_window(0), Some(24..48));
        assert_eq!(directory.live_row_window(2), Some(0..24));
        assert_eq!(directory.live_row_window(16), Some(48..64));
        for slot in [1, 15, 17, u16::MAX] {
            assert_eq!(directory.live_row_window(slot), None);
        }
        let mut output = Cursor::new(Vec::new());
        for group in directory.groups().iter().rev() {
            group.write_le(&mut output)?;
        }
        assert_eq!(output.into_inner(), bytes[982..]);
        Ok(())
    }

    struct FooterOnlyReader {
        inner: Cursor<Vec<u8>>,
        footer_start: u64,
        bytes_read: usize,
        fail_read: bool,
    }

    impl Read for FooterOnlyReader {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.fail_read || self.inner.position() < self.footer_start {
                return Err(io::Error::other("read outside footer or injected failure"));
            }
            let count = self.inner.read(output)?;
            self.bytes_read += count;
            Ok(count)
        }
    }

    impl Seek for FooterOnlyReader {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.inner.seek(position)
        }
    }

    #[test]
    fn directory_reads_never_touch_headers_heap_or_free_space() -> Result<()> {
        let (bytes, header) = fixture();
        let mut input = FooterOnlyReader {
            inner: Cursor::new(bytes),
            footer_start: 982,
            bytes_read: 0,
            fail_read: false,
        };
        read_row_directory(&mut input, 512, &header)?;
        assert_eq!(input.bytes_read, 42);
        assert_eq!(input.inner.position(), 1024);
        Ok(())
    }

    #[test]
    fn empty_full_partial_and_more_than_255_slot_directories_have_exact_sizes() -> Result<()> {
        for allocated in [0u16, 1, 16, 17, 32, 284] {
            let (_, mut header) = fixture();
            header.common.counts = PackedRowCounts::new(allocated, 0)?;
            header.common.used_size = 0;
            let size = 2 * usize::from(allocated) + 4 * usize::from(allocated).div_ceil(16);
            let mut input = FooterOnlyReader {
                inner: Cursor::new(vec![0; 8192]),
                footer_start: (8192 - size) as u64,
                bytes_read: 0,
                fail_read: false,
            };
            let directory = read_row_directory(&mut input, 4096, &header)?;
            let slots: Vec<_> = directory.slots().map(|slot| slot.slot).collect();
            assert_eq!(slots, (0..allocated).collect::<Vec<_>>());
            assert_eq!(input.bytes_read, size);
            assert_eq!(
                directory.groups().len(),
                usize::from(allocated).div_ceil(16)
            );
            assert!(directory.slots().all(|slot| !slot.live));
        }
        Ok(())
    }

    #[test]
    fn rejects_count_disagreements_and_heap_directory_overlap() -> Result<()> {
        for (allocated, live, used) in [(17, 18, 64), (17, 2, 64), (8191, 0, 0), (17, 3, 431)] {
            let (bytes, mut header) = fixture();
            header.common.counts = PackedRowCounts::new(allocated, live)?;
            header.common.used_size = used;
            assert!(matches!(
                read_row_directory(&mut Cursor::new(bytes), 512, &header),
                Err(Error::InvalidRowDirectory {
                    page: PageIndex(1),
                    ..
                })
            ));
        }
        Ok(())
    }

    #[test]
    fn rejects_live_offsets_outside_heap_and_duplicate_live_starts() {
        for offset in [0u16, 64, u16::MAX] {
            let (mut bytes, header) = fixture();
            // Slot 0 duplicates live slot 2 at zero, or points beyond the heap.
            bytes[1018..1020].copy_from_slice(&offset.to_le_bytes());
            assert!(matches!(
                read_row_directory(&mut Cursor::new(bytes), 512, &header),
                Err(Error::InvalidRowDirectory {
                    page: PageIndex(1),
                    ..
                })
            ));
        }
    }

    #[test]
    fn rejects_index_pages_invalid_sizes_indices_and_truncated_pages() {
        let (bytes, mut header) = fixture();
        header.common.flags.0 = 0x64;
        assert!(matches!(
            read_row_directory(&mut Cursor::new(&bytes), 512, &header),
            Err(Error::InvalidRowDirectory { .. })
        ));
        let (_, header) = fixture();
        for size in [0, 40, 511, 513, u32::MAX] {
            assert!(matches!(
                read_row_directory(&mut Cursor::new(&bytes), size, &header),
                Err(Error::InvalidRowDirectory { .. })
            ));
        }
        for page in [0, 2, PageIndex::NONE.0, u32::MAX] {
            let mut header = header;
            header.common.page_index = PageIndex(page);
            assert!(matches!(
                read_row_directory(&mut Cursor::new(&bytes), 512, &header),
                Err(Error::InvalidRowDirectory { .. })
            ));
        }
        assert!(matches!(
            read_row_directory(&mut Cursor::new(&bytes[..1023]), 512, &header),
            Err(Error::InvalidRowDirectory { .. })
        ));
    }

    #[test]
    fn read_failures_retain_page_context() {
        let (bytes, header) = fixture();
        let mut input = FooterOnlyReader {
            inner: Cursor::new(bytes),
            footer_start: 982,
            bytes_read: 0,
            fail_read: true,
        };
        assert!(matches!(
            read_row_directory(&mut input, 512, &header),
            Err(Error::RowDirectoryRead {
                page: PageIndex(1),
                ..
            })
        ));
    }

    #[test]
    fn group_codec_rejects_invalid_counts_before_reading_or_writing() {
        for count in [0, 17, u16::MAX] {
            let mut input = Cursor::new(vec![0; 40]);
            assert!(RowGroup::read_le_args(&mut input, (count,)).is_err());
            assert_eq!(input.position(), 0);
        }
        for count in [0, 17] {
            let group = RowGroup {
                offsets: vec![0; count],
                presence_flags: 0,
                transaction_flags: 0,
            };
            let mut output = Cursor::new(Vec::new());
            assert!(group.write_le(&mut output).is_err());
            assert!(output.into_inner().is_empty());
        }
        for length in 0..6 {
            assert!(RowGroup::read_le_args(&mut Cursor::new(vec![0; length]), (1,)).is_err());
        }
    }
}
