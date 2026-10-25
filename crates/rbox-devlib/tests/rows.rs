// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Bounded row decoding, all known layouts, string offsets, and opaque data preservation.

use binrw::{BinRead, BinWrite, Endian};
use rbox_devlib::binary::{
    read_row, read_row_directory, AlbumRow, ArtistRow, BrowseCategoryRow, BrowseColumnRow,
    BrowseSortRow, ColorRow, DatabaseKind, DatabasePropertyRow, ExportPropertyRow, ExtPageType,
    ExtendedRow, HistoryEntryRow, KeyRow, MyTagHeader, MyTagRow, NamedRow, PageHeader, PageIndex,
    PageType, PlaylistEntryRow, PlaylistTreeRow, Row, RowDirectory, RowRecord, StandardRow,
    StringOffsets, TableKind, TrackMyTagRow, TrackRow,
};
use rbox_devlib::{Error, Result};
use std::io::{self, Cursor, Read, Seek, SeekFrom};

mod synthetic_variants {
    //! Explicit wire examples for layouts absent from the approved native exports.

    use super::*;
    use rbox_devlib::binary::RecommendLikeRow;

    // Build an independent one-group data page without using the writer under test.
    fn page_file(table: u32, page_no: usize, rows: &[Vec<u8>]) -> Vec<u8> {
        assert!(!rows.is_empty() && rows.len() <= 16);
        let mut file = vec![0; (page_no + 1) * 4096];
        let page = &mut file[page_no * 4096..];
        page[4..8].copy_from_slice(&(page_no as u32).to_le_bytes());
        page[8..12].copy_from_slice(&table.to_le_bytes());
        let count = rows.len() as u32;
        page[24..27].copy_from_slice(&(count | (count << 13)).to_le_bytes()[..3]);
        page[27] = 0x24;
        let mut used = 0;
        for (slot, row) in rows.iter().enumerate() {
            page[40 + used..40 + used + row.len()].copy_from_slice(row);
            let offset = 4096 - 6 - 2 * slot;
            page[offset..offset + 2].copy_from_slice(&(used as u16).to_le_bytes());
            used += row.len();
        }
        page[30..32].copy_from_slice(&(used as u16).to_le_bytes());
        page[4092..4094].copy_from_slice(&((1u32 << count) - 1).to_le_bytes()[..2]);
        file
    }

    #[test]
    fn far_album_preserves_long_unicode_name_and_offset() -> Result<()> {
        let mut bytes = vec![0; 300];
        bytes[..2].copy_from_slice(&0x84u16.to_le_bytes());
        bytes[2..4].copy_from_slice(&(74u16 * 32).to_le_bytes());
        bytes[8..12].copy_from_slice(&245u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&166u32.to_le_bytes());
        bytes[20..24].copy_from_slice(&[3, 0, 0x2c, 1]);
        let expected = format!("\u{285}{}", "x".repeat(130));
        bytes.extend_from_slice(&[0x90, 0x0a, 1, 0]); // 266-byte UTF-16 frame.
        for ch in expected.encode_utf16() {
            bytes.extend_from_slice(&ch.to_le_bytes());
        }
        let row = AlbumRow::parse(&bytes)?;
        assert_eq!(
            (row.header.subtype, row.header.id, row.header.artist_id),
            (0x84, 166, 245)
        );
        assert_eq!(row.header.index_shift, 74 * 32);
        assert!(row.strings.is_far());
        assert_eq!(row.strings.offset(0), Some(300));
        assert_eq!(row.name.as_bytes(), &bytes[300..]);
        assert_eq!(row.name.text()?, expected);
        Ok(())
    }

    #[test]
    fn history_pages_decode_names_and_occurrences() -> Result<()> {
        let name = b"HISTORY 001";
        let mut playlist = 1u32.to_le_bytes().to_vec();
        playlist.push((name.len() as u8 + 1) * 2 + 1);
        playlist.extend_from_slice(name);
        let entries = (1u32..=13)
            .map(|id| [id, 1, id].into_iter().flat_map(u32::to_le_bytes).collect())
            .collect::<Vec<Vec<u8>>>();
        for (page_no, table, rows) in [(24, 11, vec![playlist]), (26, 12, entries)] {
            let mut input = Cursor::new(page_file(table, page_no, &rows));
            input.set_position((page_no * 4096) as u64);
            let header = PageHeader::read_le(&mut input)?;
            let directory = read_row_directory(&mut input, 4096, &header)?;
            assert_eq!(header.common.counts.live() as usize, rows.len());
            for slot in 0..rows.len() as u16 {
                let record =
                    read_row(&mut input, DatabaseKind::Standard, &directory, slot)?.unwrap();
                assert_eq!(record.as_bytes(), rows[slot as usize]);
                match record.decode()? {
                    Row::Standard(StandardRow::HistoryPlaylist(row)) => {
                        assert_eq!(page_no, 24);
                        assert_eq!(row.id, 1);
                        assert_eq!(row.name.text()?, "HISTORY 001");
                    }
                    Row::Standard(StandardRow::HistoryEntry(row)) => {
                        assert_eq!(page_no, 26);
                        assert_eq!(
                            (row.track_id, row.playlist_id, row.position),
                            (u32::from(slot + 1), 1, u32::from(slot + 1))
                        );
                    }
                    other => panic!("unexpected history row: {other:?}"),
                }
            }
        }
        Ok(())
    }

    #[test]
    fn recommend_like_page_decodes_all_pairs_without_namespace_confusion() -> Result<()> {
        let rows = (1u32..=14)
            .map(|id| {
                let mut bytes = (0x1234_5678_0000u64 + u64::from(id)).to_le_bytes().to_vec();
                for value in [id, id + 1, 0x11223344, 3] {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
                bytes
            })
            .collect::<Vec<_>>();
        let mut input = Cursor::new(page_file(0, 2, &rows));
        input.set_position(8192);
        let page = PageHeader::read_le(&mut input)?;
        assert_eq!(page.common.table_type, 0);
        assert_eq!(page.common.counts.live(), 14);
        let directory = read_row_directory(&mut input, 4096, &page)?;
        for (slot, bytes) in rows.iter().enumerate() {
            let record =
                read_row(&mut input, DatabaseKind::Extended, &directory, slot as u16)?.unwrap();
            assert_eq!(record.as_bytes(), bytes);
            let Row::Extended(ExtendedRow::RecommendLike(row)) = record.decode()? else {
                panic!("expected track pair")
            };
            let id = slot as u32 + 1;
            assert_eq!((row.track_id_1, row.track_id_2), (id, id + 1));
            assert_eq!(row.unknown_00, 0x1234_5678_0000 + u64::from(id));
            assert_eq!((row.unknown_10, row.unknown_14), (0x11223344, 3));
            let mut output = Cursor::new(Vec::new());
            row.write_le(&mut output)?;
            assert_eq!(output.into_inner(), *bytes);
        }
        let record = read_row(&mut input, DatabaseKind::Standard, &directory, 0)?.unwrap();
        assert!(matches!(
            record.decode()?,
            Row::Standard(StandardRow::Opaque)
        ));
        Ok(())
    }

    #[test]
    fn recommend_like_retains_unknowns_and_rejects_every_truncation() -> Result<()> {
        let bytes: Vec<u8> = (0..24).collect();
        let row = RecommendLikeRow::read_le(&mut Cursor::new(&bytes))?;
        assert_eq!(row.unknown_00, 0x0706_0504_0302_0100);
        assert_eq!((row.unknown_10, row.unknown_14), (0x1312_1110, 0x1716_1514));
        let mut output = Cursor::new(Vec::new());
        row.write_le(&mut output)?;
        assert_eq!(output.into_inner(), bytes);
        for end in 0..24 {
            assert!(RecommendLikeRow::read_le(&mut Cursor::new(&bytes[..end])).is_err());
        }
        let mut with_tail = bytes;
        with_tail.extend_from_slice(&[0xaa; 8]);
        let mut input = Cursor::new(with_tail);
        RecommendLikeRow::read_le(&mut input)?;
        assert_eq!(input.position(), 24);
        Ok(())
    }
}

// Independent wire fixture: the payload is the first of two live row windows.
// Extra heap bytes must never be used to finish decoding the selected row.
fn bounded_record(
    payload: &[u8],
    kind: DatabaseKind,
    table: u32,
    page_size: usize,
    following_bytes: usize,
) -> Result<RowRecord> {
    let mut file = vec![0; page_size * 2];
    let page = &mut file[page_size..];
    page[4..8].copy_from_slice(&1u32.to_le_bytes());
    page[8..12].copy_from_slice(&table.to_le_bytes());
    page[24..28].copy_from_slice(&[2, 0x40, 0, 0x24]);
    page[30..32].copy_from_slice(&((payload.len() + following_bytes) as u16).to_le_bytes());
    page[40..40 + payload.len()].copy_from_slice(payload);
    page[page_size - 8..page_size - 6].copy_from_slice(&(payload.len() as u16).to_le_bytes());
    page[page_size - 6..].copy_from_slice(&[0, 0, 3, 0, 1, 0]);
    let mut input = Cursor::new(file);
    input.set_position(page_size as u64);
    let header = PageHeader::read_le(&mut input)?;
    let directory = read_row_directory(&mut input, page_size as u32, &header)?;
    Ok(read_row(&mut input, kind, &directory, 0)?.unwrap())
}

mod windows {
    //! Bounded row reads, namespace dispatch, string preservation, and malformed input.

    use super::*;

    // Two live windows in nonmonotonic slot order; slot 1 is inactive with a stale offset.
    // First window is a label followed by opaque bytes; second contains an empty name.
    fn fixture(table: u32) -> Result<(Vec<u8>, RowDirectory)> {
        let mut bytes = vec![0; 1024];
        bytes[516..520].copy_from_slice(&1u32.to_le_bytes());
        bytes[520..524].copy_from_slice(&table.to_le_bytes());
        bytes[536..539].copy_from_slice(&[3, 0x40, 0]); // 3 allocated, 2 live.
        bytes[539] = 0x24;
        bytes[542..544].copy_from_slice(&24u16.to_le_bytes());
        bytes[552..564]
            .copy_from_slice(&[42, 0, 0, 0, 9, b'A', b'B', b'C', 0xde, 0xad, 0xbe, 0xef]);
        bytes[564..569].copy_from_slice(&[7, 0, 0, 0, 3]);
        bytes[1014..1024].copy_from_slice(&[0, 0, 0xff, 0xff, 12, 0, 5, 0, 6, 0]);
        let mut input = Cursor::new(&bytes);
        input.set_position(512);
        let header = PageHeader::read_le(&mut input)?;
        let directory = read_row_directory(&mut input, 512, &header)?;
        Ok((bytes, directory))
    }

    struct WindowOnlyReader {
        input: Cursor<Vec<u8>>,
        allowed: std::ops::Range<u64>,
        bytes_read: usize,
        seeks: usize,
    }

    impl Read for WindowOnlyReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let start = self.input.position();
            if start < self.allowed.start || start + buf.len() as u64 > self.allowed.end {
                return Err(io::Error::other("read outside selected row window"));
            }
            let count = self.input.read(buf)?;
            self.bytes_read += count;
            Ok(count)
        }
    }

    impl Seek for WindowOnlyReader {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.seeks += 1;
            self.input.seek(position)
        }
    }

    #[test]
    fn only_selected_live_window_is_read_and_decoding_preserves_extra_bytes() -> Result<()> {
        let (bytes, directory) = fixture(4)?;
        let mut input = WindowOnlyReader {
            input: Cursor::new(bytes),
            allowed: 552..564,
            bytes_read: 0,
            seeks: 0,
        };
        let record = read_row(&mut input, DatabaseKind::Standard, &directory, 2)?.unwrap();
        assert_eq!(input.bytes_read, 12);
        assert_eq!(input.input.position(), 564);
        assert_eq!(record.page(), PageIndex(1));
        assert_eq!(record.table(), TableKind::Standard(PageType::Labels));
        assert_eq!(record.slot().slot, 2);
        assert_eq!(record.slot().heap_offset, 0);
        assert!(record.slot().live && record.slot().touched);
        let Row::Standard(StandardRow::Label(row)) = record.decode()? else {
            panic!("expected label")
        };
        assert_eq!(row.id, 42);
        assert_eq!(row.name.text()?, "ABC");
        let mut output = Cursor::new(Vec::new());
        row.write_le(&mut output)?;
        assert_eq!(output.into_inner(), record.as_bytes()[..8]);
        assert_eq!(&record.as_bytes()[8..], &[0xde, 0xad, 0xbe, 0xef]);
        for slot in [1, 3, u16::MAX] {
            assert!(read_row(&mut input, DatabaseKind::Standard, &directory, slot)?.is_none());
        }
        assert_eq!(input.bytes_read, 12);
        assert_eq!(input.seeks, 1);
        input.allowed = 564..576;
        let record = read_row(&mut input, DatabaseKind::Standard, &directory, 0)?.unwrap();
        let Row::Standard(StandardRow::Label(row)) = record.decode()? else {
            panic!("expected label")
        };
        assert_eq!(row.id, 7);
        assert_eq!(row.name.text()?, "");
        Ok(())
    }

    #[test]
    fn dispatch_distinguishes_shared_layouts_and_overlapping_namespaces() -> Result<()> {
        for table in [1, 4, 11, 13, 0, u32::MAX] {
            let (bytes, directory) = fixture(table)?;
            let mut input = Cursor::new(bytes);
            let standard = read_row(&mut input, DatabaseKind::Standard, &directory, 2)?.unwrap();
            match (table, standard.decode()?) {
                (1, Row::Standard(StandardRow::Genre(_)))
                | (4, Row::Standard(StandardRow::Label(_)))
                | (11, Row::Standard(StandardRow::HistoryPlaylist(_)))
                | (13, Row::Standard(StandardRow::Artwork(_)))
                | (0 | u32::MAX, Row::Standard(StandardRow::Opaque)) => {}
                other => panic!("incorrect dispatch: {other:?}"),
            }
            let extended = read_row(&mut input, DatabaseKind::Extended, &directory, 2)?.unwrap();
            if table == 0 || table == 4 {
                // Extended pairs/associations need 24/16 bytes respectively; these
                // 12-byte standard-label windows are truncated in that namespace.
                assert!(matches!(extended.decode(), Err(Error::RowDecode { .. })));
            } else {
                assert_eq!(extended.decode()?, Row::Extended(ExtendedRow::Opaque));
            }
            assert_eq!(
                extended.table(),
                TableKind::Extended(ExtPageType::from_code(table))
            );
            assert_eq!(extended.as_bytes(), standard.as_bytes());
        }
        Ok(())
    }

    #[test]
    fn malformed_named_strings_cannot_read_into_the_next_live_window() -> Result<()> {
        let (mut bytes, directory) = fixture(4)?;
        // The frame claims 20 bytes of payload: more than this window, but within the file.
        bytes[556] = 43;
        let record = read_row(
            &mut Cursor::new(bytes),
            DatabaseKind::Standard,
            &directory,
            2,
        )?
        .unwrap();
        assert!(matches!(
            record.decode(),
            Err(Error::RowDecode {
                page: PageIndex(1),
                table: TableKind::Standard(PageType::Labels),
                slot: 2,
                ..
            })
        ));
        assert_eq!(record.as_bytes()[4], 43); // Still available after decoding fails.
        Ok(())
    }

    #[test]
    fn truncated_window_returns_contextual_io_error() -> Result<()> {
        let (mut bytes, directory) = fixture(4)?;
        bytes.truncate(560);
        assert!(matches!(
            read_row(
                &mut Cursor::new(bytes),
                DatabaseKind::Standard,
                &directory,
                2
            ),
            Err(Error::RowRead {
                page: PageIndex(1),
                slot: 2,
                ..
            })
        ));
        Ok(())
    }

    #[test]
    fn named_codec_preserves_frames_is_little_endian_and_leaves_tail_unread() -> Result<()> {
        for frame in [
            vec![3],
            vec![7, 0xff, 0], // Invalid ASCII remains intact.
            vec![0x40, 6, 0, 0xab, b'A', b'B'],
            vec![0x90, 8, 0, 0xcd, 0x3d, 0xd8, 0, 0xde], // Surrogate pair.
            vec![0x90, 6, 0, 0, 0, 0xd8],                // Unpaired surrogate.
            vec![0x42, 5, 0, 0xef, 0xaa],                // Unknown long marker.
        ] {
            let mut bytes = vec![0x44, 0x33, 0x22, 0x11];
            bytes.extend_from_slice(&frame);
            let encoded = bytes.clone();
            bytes.extend_from_slice(&[0xde, 0xad]);
            let mut input = Cursor::new(bytes);
            let row = NamedRow::read_options(&mut input, Endian::Big, ())?;
            assert_eq!(row.id, 0x11223344);
            assert_eq!(row.name.as_bytes(), frame);
            assert_eq!(input.position() as usize, encoded.len());
            let mut output = Cursor::new(Vec::new());
            row.write_options(&mut output, Endian::Big, ())?;
            assert_eq!(output.into_inner(), encoded);
            for end in 0..encoded.len() {
                assert!(NamedRow::read_le(&mut Cursor::new(&encoded[..end])).is_err());
            }
        }
        Ok(())
    }
}

mod layouts {
    //! Documented row families: exact wire layouts, namespace dispatch, and bounds.

    use super::*;
    fn record(bytes: &[u8], kind: DatabaseKind, table: u32) -> Result<RowRecord> {
        super::bounded_record(bytes, kind, table, 4096, 64)
    }

    // Independent field bytes, with unequal IDs/reserved fields and non-default states.
    fn inline_cases() -> Vec<(DatabaseKind, u32, Vec<u8>)> {
        use DatabaseKind::{Extended, Standard};
        vec![
            (Standard, 5, vec![1, 0, 0, 0, 9, 0, 0, 0, 5, b'K']),
            (
                Standard,
                6,
                vec![0x44, 0x33, 0x22, 0x11, 7, 0x34, 0x12, 0xaa, 5, b'C'],
            ),
            (
                Standard,
                7,
                vec![
                    2, 0, 0, 0, 0xaa, 0xbb, 0xcc, 0xdd, 9, 0, 0, 0, 3, 0, 0, 0, 0xff, 0, 0, 0, 5,
                    b'P',
                ],
            ),
            (Standard, 8, vec![8, 0, 0, 0, 7, 0, 0, 0, 6, 0, 0, 0]),
            (Standard, 12, vec![7, 0, 0, 0, 6, 0, 0, 0, 8, 0, 0, 0]),
            (
                Standard,
                16,
                vec![
                    0x34, 0x12, 0x78, 0x56, 0x90, 8, 0, 0xab, 0xfa, 0xff, 0xfb, 0xff,
                ],
            ),
            (
                Standard,
                17,
                vec![0x34, 0x12, 0x78, 0x56, 0xab, 3, 0xde, 0xbc],
            ),
            (
                Standard,
                18,
                vec![0x34, 0x12, 0x78, 0x56, 0xef, 0xcd, 0xab, 0x89],
            ),
            (
                Extended,
                4,
                vec![
                    0x44, 0x33, 0x22, 0x11, 7, 0, 0, 0, 6, 0, 0, 0, 0x90, 0xaa, 0xbb, 0xcc,
                ],
            ),
        ]
    }

    fn check_codec<T>(bytes: &[u8]) -> Result<T>
    where
        T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()>,
    {
        let mut framed = vec![0xfa; 3];
        framed.extend_from_slice(bytes);
        framed.extend_from_slice(&[0xfb; 5]);
        let mut input = Cursor::new(framed);
        input.set_position(3);
        let row = T::read_options(&mut input, Endian::Big, ())?;
        assert_eq!(input.position() as usize, 3 + bytes.len());
        let mut output = Cursor::new(vec![0xfa; 3]);
        output.set_position(3);
        row.write_options(&mut output, Endian::Big, ())?;
        assert_eq!(&output.get_ref()[3..], bytes);
        assert_eq!(&output.get_ref()[..3], &[0xfa; 3]);
        for end in 0..bytes.len() {
            assert!(T::read_le(&mut Cursor::new(&bytes[..end])).is_err());
        }
        Ok(row)
    }

    #[test]
    fn inline_layouts_preserve_every_field_and_consume_only_encoded_bytes() -> Result<()> {
        for (kind, table, bytes) in inline_cases() {
            let decoded = record(&bytes, kind, table)?.decode()?;
            match (kind, table) {
                (DatabaseKind::Standard, 5) => {
                    let row = check_codec::<KeyRow>(&bytes)?;
                    assert_eq!((row.id, row.secondary_id), (1, 9));
                    assert_eq!(row.name.text()?, "K");
                    assert_eq!(decoded, Row::Standard(StandardRow::Key(row)));
                }
                (DatabaseKind::Standard, 6) => {
                    let row = check_codec::<ColorRow>(&bytes)?;
                    assert_eq!(
                        (row.unknown_00, row.unknown_04, row.id, row.unknown_07),
                        (0x11223344, 7, 0x1234, 0xaa)
                    );
                    assert_eq!(row.name.text()?, "C");
                    assert_eq!(decoded, Row::Standard(StandardRow::Color(row)));
                }
                (DatabaseKind::Standard, 7) => {
                    let row = check_codec::<PlaylistTreeRow>(&bytes)?;
                    assert_eq!(
                        (
                            row.parent_id,
                            row.unknown_04,
                            row.position,
                            row.id,
                            row.folder_flag
                        ),
                        (2, 0xddccbbaa, 9, 3, 255)
                    );
                    assert_eq!(row.name.text()?, "P");
                    assert_eq!(decoded, Row::Standard(StandardRow::PlaylistTree(row)));
                }
                (DatabaseKind::Standard, 8) => {
                    let row = check_codec::<PlaylistEntryRow>(&bytes)?;
                    assert_eq!((row.position, row.track_id, row.playlist_id), (8, 7, 6));
                    assert_eq!(decoded, Row::Standard(StandardRow::PlaylistEntry(row)));
                }
                (DatabaseKind::Standard, 12) => {
                    let row = check_codec::<HistoryEntryRow>(&bytes)?;
                    assert_eq!((row.track_id, row.playlist_id, row.position), (7, 6, 8));
                    assert_eq!(decoded, Row::Standard(StandardRow::HistoryEntry(row)));
                }
                (DatabaseKind::Standard, 16) => {
                    let row = check_codec::<BrowseColumnRow>(&bytes)?;
                    assert_eq!((row.id, row.code), (0x1234, 0x5678));
                    assert_eq!(row.name.text()?, "\u{fffa}\u{fffb}");
                    assert_eq!(decoded, Row::Standard(StandardRow::BrowseColumn(row)));
                }
                (DatabaseKind::Standard, 17) => {
                    let row = check_codec::<BrowseCategoryRow>(&bytes)?;
                    assert_eq!(
                        (
                            row.column_id,
                            row.selector,
                            row.unknown_04,
                            row.state,
                            row.position
                        ),
                        (0x1234, 0x5678, 0xab, 3, 0xbcde)
                    );
                    assert_eq!(decoded, Row::Standard(StandardRow::BrowseCategory(row)));
                }
                (DatabaseKind::Standard, 18) => {
                    let row = check_codec::<BrowseSortRow>(&bytes)?;
                    assert_eq!(
                        (row.column_id, row.selector, row.raw_tail),
                        (0x1234, 0x5678, 0x89abcdef)
                    );
                    assert_eq!(decoded, Row::Standard(StandardRow::BrowseSort(row)));
                }
                (DatabaseKind::Extended, 4) => {
                    let row = check_codec::<TrackMyTagRow>(&bytes)?;
                    assert_eq!(
                        (
                            row.unknown_00,
                            row.track_id,
                            row.tag_id,
                            row.reserved_string_marker,
                            row.reserved_tail
                        ),
                        (0x11223344, 7, 6, 0x90, [0xaa, 0xbb, 0xcc])
                    );
                    assert_eq!(decoded, Row::Extended(ExtendedRow::TrackMyTag(row)));
                }
                _ => unreachable!(),
            }
            for end in 1..bytes.len() {
                assert!(matches!(
                    record(&bytes[..end], kind, table)?.decode(),
                    Err(Error::RowDecode { .. })
                ));
            }
        }
        Ok(())
    }

    fn catalog_bytes(album: bool, far: bool) -> Vec<u8> {
        let subtype = match (album, far) {
            (false, false) => 0x0060u16,
            (false, true) => 0x0064,
            (true, false) => 0x0080,
            (true, true) => 0x0084,
        };
        let mut bytes = subtype.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0x60, 0]);
        if album {
            bytes.extend_from_slice(&[
                0x44, 0x33, 0x22, 0x11, 7, 0, 0, 0, 9, 0, 0, 0, 0x88, 0x77, 0x66, 0x55,
            ]);
        } else {
            bytes.extend_from_slice(&[9, 0, 0, 0]);
        }
        if far {
            bytes.extend_from_slice(&[7, 0xab, 0x2c, 1]);
        } else {
            bytes.extend_from_slice(&[7, 32]);
        }
        bytes.resize(if far { 300 } else { 32 }, 0xdd);
        bytes.extend_from_slice(&[0x90, 8, 0, 0xee, 0x3d, 0xd8, 0, 0xde]);
        bytes
    }

    #[test]
    fn artist_and_album_names_use_their_own_near_far_arrays() -> Result<()> {
        for album in [false, true] {
            for far in [false, true] {
                let bytes = catalog_bytes(album, far);
                let table = if album { 3 } else { 2 };
                let decoded = record(&bytes, DatabaseKind::Standard, table)?.decode()?;
                let (strings, name) = if album {
                    let row = AlbumRow::parse(&bytes)?;
                    assert_eq!(
                        (
                            row.header.index_shift,
                            row.header.unknown_04,
                            row.header.artist_id,
                            row.header.id,
                            row.header.unknown_10
                        ),
                        (0x60, 0x11223344, 7, 9, 0x55667788)
                    );
                    check_codec::<rbox_devlib::binary::AlbumHeader>(&bytes[..20])?;
                    assert_eq!(decoded, Row::Standard(StandardRow::Album(row.clone())));
                    (row.strings, row.name)
                } else {
                    let row = ArtistRow::parse(&bytes)?;
                    assert_eq!((row.header.index_shift, row.header.id), (0x60, 9));
                    check_codec::<rbox_devlib::binary::ArtistHeader>(&bytes[..8])?;
                    assert_eq!(decoded, Row::Standard(StandardRow::Artist(row.clone())));
                    (row.strings, row.name)
                };
                assert_eq!(strings.is_far(), far);
                assert_eq!(strings.offset(0), Some(if far { 300 } else { 32 }));
                assert_eq!(name.text()?, "\u{1f600}");
                for end in 1..bytes.len() {
                    assert!(matches!(
                        record(&bytes[..end], DatabaseKind::Standard, table)?.decode(),
                        Err(Error::RowDecode { .. })
                    ));
                }
            }
        }
        Ok(())
    }

    fn track_bytes() -> Vec<u8> {
        let mut bytes = vec![0; 136];
        bytes[..4].copy_from_slice(&[0x24, 0, 0x60, 0]);
        for index in 0..18 {
            bytes[4 + 4 * index..8 + 4 * index]
                .copy_from_slice(&(1001 + index as u32).to_le_bytes());
        }
        for index in 0..6 {
            bytes[76 + 2 * index..78 + 2 * index]
                .copy_from_slice(&(201 + index as u16).to_le_bytes());
        }
        bytes[88..94].copy_from_slice(&[0xaa, 0xff, 0xee, 0xff, 2, 0]); // Raw color/rating/format/marker.
                                                                        // Reverse physical string order, with an unreferenced gap after every frame.
        for index in (0..21).rev() {
            let offset = bytes.len() as u16;
            bytes[94 + 2 * index..96 + 2 * index].copy_from_slice(&offset.to_le_bytes());
            if index == 0 {
                bytes.extend_from_slice(&[0x90, 18, 0, 0, 3]);
                bytes.extend_from_slice(b"USABC1234567");
                bytes.push(0);
            } else {
                let text = format!("slot{index:02}");
                bytes.push((text.len() * 2 + 3) as u8);
                bytes.extend_from_slice(text.as_bytes());
            }
            bytes.push(0xdd);
        }
        bytes
    }

    #[test]
    fn tracks_keep_all_scalar_fields_and_all_21_string_slots() -> Result<()> {
        let bytes = track_bytes();
        let row = TrackRow::parse(&bytes)?;
        let h = &row.header;
        assert_eq!((h.subtype, h.index_shift), (0x24, 0x60));
        assert_eq!(
            [
                h.content_link,
                h.sample_rate,
                h.composer_id,
                h.file_size,
                h.master_content_id,
                h.master_db_id,
                h.artwork_id,
                h.key_id,
                h.original_artist_id,
                h.label_id,
                h.remixer_id,
                h.bitrate,
                h.track_number,
                h.tempo,
                h.genre_id,
                h.album_id,
                h.artist_id,
                h.id
            ],
            std::array::from_fn(|i| 1001 + i as u32)
        );
        assert_eq!(
            [
                h.disc_number,
                h.play_count,
                h.year,
                h.sample_depth,
                h.duration,
                h.raw_analysis
            ],
            [201, 202, 203, 204, 205, 206]
        );
        assert_eq!((h.color_id, h.rating, h.audio_format), (0xaa, 0xff, 0xffee));
        check_codec::<rbox_devlib::binary::TrackHeader>(&bytes[..92])?;
        let t = &row.text;
        assert_eq!(t.isrc.isrc_text()?, "USABC1234567");
        let frames = [
            &t.lyricist,
            &t.unknown_02,
            &t.unknown_03,
            &t.unknown_04,
            &t.message,
            &t.publish_track_info,
            &t.hot_cue_auto_load,
            &t.unknown_08,
            &t.unknown_09,
            &t.date_added,
            &t.release_date,
            &t.mix_name,
            &t.unknown_13,
            &t.analysis_path,
            &t.analysis_date,
            &t.comment,
            &t.title,
            &t.unknown_18,
            &t.filename,
            &t.file_path,
        ];
        for (index, frame) in frames.into_iter().enumerate() {
            assert_eq!(frame.text()?, format!("slot{:02}", index + 1));
        }
        assert!(matches!(row.strings, StringOffsets::Far { marker: 2, .. }));
        let record = record(&bytes, DatabaseKind::Standard, 0)?;
        assert_eq!(record.decode()?, Row::Standard(StandardRow::Track(row)));
        assert_eq!(record.as_bytes(), bytes);
        Ok(())
    }

    fn database_property_bytes() -> Vec<u8> {
        let mut bytes = vec![0; 56];
        bytes[..4].copy_from_slice(&[0, 7, 0x20, 0]);
        for index in 0..5 {
            bytes[4 + index * 4..8 + index * 4].copy_from_slice(&(10 + index as u32).to_le_bytes());
        }
        bytes[24..34].copy_from_slice(&[0x44, 0x33, 0x22, 0x11, 0xa5, 45, 34, 50, 34, 40]);
        bytes[34] = 3;
        bytes[40..43].copy_from_slice(&[7, b'x', b'y']);
        bytes[45..50].copy_from_slice(&[0x42, 5, 0, 0xab, 0xdd]); // Unknown long encoding.
        bytes[50..56].copy_from_slice(&[0x90, 6, 0, 0xcd, 0, 0xd8]); // Unpaired UTF-16 surrogate.
        bytes
    }

    #[test]
    fn database_properties_follow_all_five_offsets_without_guessing_their_meaning() -> Result<()> {
        let bytes = database_property_bytes();
        let row = DatabasePropertyRow::parse(&bytes)?;
        assert_eq!(row.header.index_shift, 0x20);
        assert_eq!(row.header.unknown_words, [10, 11, 12, 13, 14]);
        assert_eq!(row.header.my_tag_master_db_id, 0x11223344);
        check_codec::<rbox_devlib::binary::DatabasePropertyHeader>(&bytes[..28])?;
        assert_eq!(
            row.strings,
            StringOffsets::Near {
                marker: 0xa5,
                offsets: [45, 34, 50, 34, 40]
            }
        );
        assert_eq!(row.unknown_strings[0].as_bytes(), &bytes[45..50]);
        assert_eq!(row.unknown_strings[1], row.unknown_strings[3]);
        assert_eq!(row.unknown_strings[1].text()?, "");
        assert_eq!(row.unknown_strings[4].text()?, "xy");
        assert!(row.unknown_strings[2].text().is_err());
        assert_eq!(
            record(&bytes, DatabaseKind::Extended, 7)?.decode()?,
            Row::Extended(ExtendedRow::DatabaseProperty(row))
        );
        Ok(())
    }

    fn export_property_bytes() -> Vec<u8> {
        let mut bytes = vec![0x80, 2, 0x40, 0, 7, 0, 0, 0, 0xaa, 7, 0xcc, 0xbb];
        bytes.extend_from_slice(&[0x90, 6, 0, 0xdd, b'D', 0]); // UTF-16 date changes following positions.
        bytes.extend_from_slice(&[0x34, 0x12, 5, b'V']); // Separator stays raw, independent of positions.
        bytes.extend_from_slice(&[0x40, 8, 0, 0xee, b'N', b'a', b'm', b'e']);
        bytes
    }

    #[test]
    fn export_properties_have_sequential_strings_and_an_opaque_separator() -> Result<()> {
        let bytes = export_property_bytes();
        let row = check_codec::<ExportPropertyRow>(&bytes)?;
        assert_eq!(
            (
                row.subtype,
                row.index_shift,
                row.track_count,
                row.unknown_08,
                row.background_color,
                row.unknown_0a
            ),
            (0x280, 0x40, 7, 0xaa, 7, 0xbbcc)
        );
        assert_eq!(row.date.text()?, "D");
        assert_eq!(row.separator, 0x1234);
        assert_eq!(row.version.text()?, "V");
        assert_eq!(row.name.text()?, "Name");
        assert_eq!(
            record(&bytes, DatabaseKind::Standard, 19)?.decode()?,
            Row::Standard(StandardRow::ExportProperty(row))
        );
        Ok(())
    }

    #[test]
    fn every_new_offset_array_checks_each_reference_and_string_length() -> Result<()> {
        let cases = [
            (DatabaseKind::Standard, 0, track_bytes(), 94, 21, true, 136),
            (
                DatabaseKind::Standard,
                2,
                catalog_bytes(false, false),
                9,
                1,
                false,
                10,
            ),
            (
                DatabaseKind::Standard,
                2,
                catalog_bytes(false, true),
                10,
                1,
                true,
                12,
            ),
            (
                DatabaseKind::Standard,
                3,
                catalog_bytes(true, false),
                21,
                1,
                false,
                22,
            ),
            (
                DatabaseKind::Standard,
                3,
                catalog_bytes(true, true),
                22,
                1,
                true,
                24,
            ),
            (
                DatabaseKind::Extended,
                7,
                database_property_bytes(),
                29,
                5,
                false,
                34,
            ),
        ];
        for (kind, table, bytes, array_start, count, far, data_start) in cases {
            for field in 0..count {
                for invalid in [
                    0u16,
                    data_start - 1,
                    bytes.len() as u16,
                    if far { 65535 } else { 255 },
                ] {
                    let mut bad = bytes.clone();
                    let at = array_start + field * if far { 2 } else { 1 };
                    if far {
                        bad[at..at + 2].copy_from_slice(&invalid.to_le_bytes());
                    } else {
                        bad[at] = invalid as u8;
                    }
                    assert!(
                        matches!(
                            record(&bad, kind, table)?.decode(),
                            Err(Error::RowDecode { .. })
                        ),
                        "table {table} field {field} offset {invalid}"
                    );
                }
            }
            // Sequential cuts ensure no decoder can complete a frame using the next row.
            let last_required = if table == 0 {
                bytes.len() - 1
            } else {
                bytes.len()
            };
            for end in 1..last_required {
                assert!(
                    matches!(
                        record(&bytes[..end], kind, table)?.decode(),
                        Err(Error::RowDecode { .. })
                    ),
                    "table {table} cut {end}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn known_tables_keep_unknown_subtypes_opaque_and_do_not_conflate_namespaces() -> Result<()> {
        for (kind, table) in [
            (DatabaseKind::Standard, 0),
            (DatabaseKind::Standard, 2),
            (DatabaseKind::Standard, 3),
            (DatabaseKind::Standard, 19),
            (DatabaseKind::Extended, 3),
            (DatabaseKind::Extended, 7),
        ] {
            let raw = [0xff, 0xff, 0xaa];
            let record = record(&raw, kind, table)?;
            assert!(matches!(
                record.decode()?,
                Row::Standard(StandardRow::Opaque) | Row::Extended(ExtendedRow::Opaque)
            ));
            assert_eq!(record.as_bytes(), raw);
        }
        let raw = database_property_bytes();
        assert!(matches!(
            record(&raw, DatabaseKind::Extended, 7)?.decode()?,
            Row::Extended(ExtendedRow::DatabaseProperty(_))
        ));
        let standard = record(&raw, DatabaseKind::Standard, 7)?;
        assert_eq!(
            standard.table(),
            TableKind::Standard(PageType::PlaylistTree)
        );
        assert!(matches!(standard.decode(), Err(Error::RowDecode { .. }))); // Its inline-name frame is invalid here.
        for table in [1, 2, 5, 6, 8, u32::MAX] {
            assert_eq!(
                record(&[0xaa], DatabaseKind::Extended, table)?.decode()?,
                Row::Extended(ExtendedRow::Opaque)
            );
        }
        for table in [9, 10, 14, 15, u32::MAX] {
            assert_eq!(
                record(&[0xaa], DatabaseKind::Standard, table)?.decode()?,
                Row::Standard(StandardRow::Opaque)
            );
        }
        Ok(())
    }

    #[test]
    fn export_property_name_alignment_is_row_relative_and_lossless() -> Result<()> {
        // Baken's published layout for date 2024-11-16, version 1000, device name é.
        // This is a writer-compatibility vector, not a native Rekordbox fixture.
        let mut bytes = vec![0x80, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 23];
        bytes.extend_from_slice(b"2024-11-16");
        bytes.extend_from_slice(&[0x19, 0x1e, 11]);
        bytes.extend_from_slice(b"1000");
        assert_eq!(bytes.len(), 30);
        bytes.extend_from_slice(&[0, 0, 0x90, 6, 0, 0, 0xe9, 0]);
        // check_codec starts at file offset 3: alignment must still be relative to row.
        let row = check_codec::<ExportPropertyRow>(&bytes)?;
        assert_eq!(row.name_padding, [0, 0]);
        assert_eq!(row.name.text()?, "é");
        assert_eq!(
            record(&bytes, DatabaseKind::Standard, 19)?.decode()?,
            Row::Standard(StandardRow::ExportProperty(row))
        );
        // Continue accepting existing contiguous frames, including long ASCII.
        let contiguous = check_codec::<ExportPropertyRow>(&export_property_bytes())?;
        assert!(contiguous.name_padding.is_empty());
        Ok(())
    }

    #[test]
    fn truncated_recommend_like_is_an_error_in_its_own_namespace() -> Result<()> {
        let record = record(&[0; 23], DatabaseKind::Extended, 0)?;
        assert!(matches!(
            record.decode(),
            Err(Error::RowDecode { slot: 0, .. })
        ));
        assert_eq!(record.as_bytes(), &[0; 23]);
        Ok(())
    }
}

mod tags {
    //! Extended tag near/far layouts, preserved fields, and bounded string resolution.

    use super::*;
    fn record(bytes: &[u8], kind: DatabaseKind) -> Result<RowRecord> {
        super::bounded_record(bytes, kind, 3, 512, 16)
    }

    fn near_bytes() -> Vec<u8> {
        vec![
            0x80, 0x06, 0x60, 0x00, // Near subtype, slot information.
            0x44, 0x33, 0x22, 0x11, // Unknown 0x04.
            0x88, 0x77, 0x66, 0x55, // Unknown 0x08.
            2, 0, 0, 0, // Parent ID.
            5, 0, 0, 0, // Position.
            0x0d, 0x0c, 0x0b, 0x0a, // ID.
            0xaa, 0xbb, 0xcc, 0, // Reserved fields and tag attribute.
            0xa5, 34, 31, // Opaque marker, name after reserved string.
            5, b'X', 0xde, // Reserved string plus a gap.
            9, b'A', b'B', b'C', // Name.
            0xfa, 0xfb, // Trailing bytes outside encoded fields.
        ]
    }

    fn far_bytes() -> Vec<u8> {
        let mut bytes = near_bytes()[..28].to_vec();
        bytes[..2].copy_from_slice(&[0x84, 0x06]);
        bytes.extend_from_slice(&[0x03, 0xab, 0x2c, 0x01, 0x22, 0]); // marker, name=300, reserved=34.
        bytes.push(3); // Empty reserved frame before name.
        bytes.resize(300, 0xde);
        bytes.extend_from_slice(&[0x90, 8, 0, 0xef, 0x3d, 0xd8, 0, 0xde]); // UTF-16 emoji.
        bytes.extend_from_slice(&[0xfa, 0xfb]);
        bytes
    }

    #[test]
    fn near_fields_preserve_unknowns_marker_gaps_and_reserved_string() -> Result<()> {
        let bytes = near_bytes();
        let row = MyTagRow::parse(&bytes)?;
        assert_eq!(row.header.subtype, 0x0680);
        assert_eq!(row.header.index_shift, 0x60);
        assert_eq!(row.header.unknown_04, 0x11223344);
        assert_eq!(row.header.unknown_08, 0x55667788);
        assert_eq!(row.header.parent_id, 2);
        assert_eq!(row.header.position, 5);
        assert_eq!(row.header.id, 0x0a0b0c0d);
        assert_eq!(row.header.unknown_18, 0xbbaa);
        assert_eq!(row.header.unknown_1a, 0xcc);
        assert_eq!(row.header.is_category(), Some(false));
        assert_eq!(
            row.strings,
            StringOffsets::Near {
                marker: 0xa5,
                offsets: [34, 31]
            }
        );
        assert_eq!(row.name.text()?, "ABC");
        assert_eq!(row.unknown_string.text()?, "X");
        let record = record(&bytes, DatabaseKind::Extended)?;
        assert_eq!(record.decode()?, Row::Extended(ExtendedRow::MyTag(row)));
        assert_eq!(record.as_bytes(), bytes);
        Ok(())
    }

    #[test]
    fn far_offsets_above_255_are_row_relative_and_can_be_nonmonotonic() -> Result<()> {
        let bytes = far_bytes();
        let row = MyTagRow::parse(&bytes)?;
        assert!(row.strings.is_far());
        assert_eq!(
            row.strings,
            StringOffsets::Far {
                marker: 0xab03,
                offsets: [300, 34]
            }
        );
        assert_eq!(row.name.text()?, "\u{1f600}");
        assert_eq!(row.name.as_bytes(), &bytes[300..308]);
        assert_eq!(row.unknown_string.as_bytes(), &[3]);
        let record = record(&bytes, DatabaseKind::Extended)?;
        assert_eq!(record.decode()?, Row::Extended(ExtendedRow::MyTag(row)));
        assert_eq!(record.as_bytes(), bytes);

        // The second offset alone can require far storage, even with a nearby name.
        let mut reversed = bytes.clone();
        reversed[30..34].copy_from_slice(&[34, 0, 0x2c, 1]);
        let row = MyTagRow::parse(&reversed)?;
        assert_eq!(row.name.text()?, "");
        assert_eq!(row.unknown_string.text()?, "\u{1f600}");
        Ok(())
    }

    #[test]
    fn aliases_unknown_attributes_and_invalid_text_remain_lossless() -> Result<()> {
        let mut bytes = near_bytes();
        bytes[27] = 7;
        bytes[30] = 34; // Both fields reference the same frame.
        bytes[35] = 0xff;
        let row = MyTagRow::parse(&bytes)?;
        assert_eq!(row.header.is_category(), None);
        assert_eq!(row.name, row.unknown_string);
        assert_eq!(row.name.as_bytes(), &[9, 0xff, b'B', b'C']);
        assert!(row.name.text().is_err());
        bytes[27] = 1;
        assert_eq!(MyTagRow::parse(&bytes)?.header.is_category(), Some(true));
        // Empty frames are actual frames, not zero pointers.
        bytes[34] = 3;
        assert_eq!(MyTagRow::parse(&bytes)?.name.text()?, "");
        Ok(())
    }

    #[test]
    fn prefix_and_offset_codecs_preserve_wire_bytes_at_nonzero_positions() -> Result<()> {
        for bytes in [near_bytes(), far_bytes()] {
            let far = bytes[0] == 0x84;
            let array_end = if far { 34 } else { 31 };
            let mut input_bytes = vec![0xaa; 7];
            input_bytes.extend_from_slice(&bytes);
            let mut input = Cursor::new(input_bytes);
            input.set_position(7);
            let header = MyTagHeader::read_options(&mut input, Endian::Big, ())?;
            assert_eq!(input.position(), 35);
            let offsets = StringOffsets::<2>::read_options(&mut input, Endian::Big, (far,))?;
            assert_eq!(input.position() as usize, 7 + array_end);
            assert_eq!(offsets.offset(2), None);
            assert_eq!(offsets.offset(0), Some(if far { 300 } else { 34 }));
            let mut output = Cursor::new(vec![0xaa; 7]);
            output.set_position(7);
            header.write_options(&mut output, Endian::Big, ())?;
            offsets.write_options(&mut output, Endian::Big, ())?;
            assert_eq!(&output.get_ref()[7..], &bytes[..array_end]);
            assert_eq!(&output.get_ref()[..7], &[0xaa; 7]);
        }
        Ok(())
    }

    #[test]
    fn truncated_prefix_arrays_and_string_frames_fail() {
        for bytes in [near_bytes(), far_bytes()] {
            // The final two bytes are outside the encoded fields; all shorter prefixes
            // truncate either the header, array, target address, or a string frame.
            for end in 0..bytes.len() - 2 {
                assert!(
                    MyTagRow::parse(&bytes[..end]).is_err(),
                    "accepted truncation at {end}"
                );
            }
            assert!(MyTagRow::parse(&bytes[..bytes.len() - 2]).is_ok());
        }
    }

    #[test]
    fn invalid_offsets_and_lengths_fail_inside_the_selected_window() -> Result<()> {
        for far in [false, true] {
            let original = if far { far_bytes() } else { near_bytes() };
            let array_end = if far { 34 } else { 31 };
            let maximum = if far { u16::MAX } else { 255 };
            for offset in [0, 1, 28, array_end - 1, original.len() as u16, maximum] {
                for field in 0..2 {
                    let mut bytes = original.clone();
                    if far {
                        let start = 30 + field * 2;
                        bytes[start..start + 2].copy_from_slice(&offset.to_le_bytes());
                    } else {
                        bytes[29 + field] = offset as u8;
                    }
                    assert!(MyTagRow::parse(&bytes).is_err());
                    assert!(matches!(
                        record(&bytes, DatabaseKind::Extended)?.decode(),
                        Err(Error::RowDecode {
                            page: PageIndex(1),
                            table: TableKind::Extended(ExtPageType::MyTags),
                            slot: 0,
                            ..
                        })
                    ));
                }
            }
        }
        let mut bytes = near_bytes();
        bytes[34] = 127; // Claimed string crosses into the next physical live window.
        let record = record(&bytes, DatabaseKind::Extended)?;
        assert!(matches!(record.decode(), Err(Error::RowDecode { .. })));
        assert_eq!(record.as_bytes(), bytes);
        Ok(())
    }

    #[test]
    fn dispatch_keeps_standard_table_three_and_unknown_extended_subtypes_opaque() -> Result<()> {
        let mut bytes = near_bytes();
        assert_eq!(
            record(&bytes, DatabaseKind::Standard)?.decode()?,
            Row::Standard(StandardRow::Opaque)
        );
        bytes[..2].copy_from_slice(&[0x85, 0x06]);
        assert!(MyTagRow::parse(&bytes).is_err());
        let unknown = record(&bytes, DatabaseKind::Extended)?;
        assert_eq!(unknown.decode()?, Row::Extended(ExtendedRow::Opaque));
        assert_eq!(unknown.as_bytes(), bytes);
        assert!(matches!(
            record(&[0x80], DatabaseKind::Extended)?.decode(),
            Err(Error::RowDecode { .. })
        ));
        Ok(())
    }
}
