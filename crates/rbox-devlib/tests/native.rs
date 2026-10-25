// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Native baselines and exact reconstruction using only export/empty and the RBv7 demo.

use binrw::{BinRead, BinWrite};
use rbox_devlib::binary::{
    read_index_entries, read_row, read_row_directory, read_table_chains, DatabaseKind, ExtPageType,
    ExtendedRow, FileHeader, FileHeaderPrefix, IndexEntry, PageHeader, PageHeaderKind, PageIndex,
    PageType, Row, RowRecord, StandardRow, StringOffsets, TableKind, TrackRow,
};
use rbox_devlib::{DeviceSqlConnection, Result};
use rbox_testing::data::{create_temp_dir, export_dir, testdata_dir};
use std::collections::HashSet;
use std::fs;
use std::io::Cursor;

#[path = "support/empty_export.rs"]
mod empty_export;

mod demo {
    //! Native RB7 standard/extended header baselines, boundaries, and byte preservation.

    use super::*;

    fn assert_wire_fields<T>(fields: &T, bytes: &[u8]) -> Result<()>
    where
        T: for<'a> BinWrite<Args<'a> = ()>,
    {
        let mut output = Cursor::new(Vec::new());
        fields.write_le(&mut output)?;
        let output = output.into_inner();
        assert_eq!(output, &bytes[..output.len()]);
        Ok(())
    }

    fn check_remaining_native_row(row: &Row, record: &RowRecord) -> Result<()> {
        let slot = record.slot().slot;
        match row {
            Row::Standard(StandardRow::Track(row)) => {
                assert_eq!(record.page(), PageIndex(2));
                assert_native_track(row, record)?;
            }
            Row::Standard(StandardRow::Artist(row)) => {
                assert_eq!(record.page(), PageIndex(6));
                assert_eq!(
                    (row.header.subtype, row.header.index_shift, row.header.id),
                    (0x60, 0, 1)
                );
                assert_eq!(row.name.text()?, "Loopmasters");
                assert_eq!(
                    row.strings,
                    StringOffsets::Near {
                        marker: 3,
                        offsets: [10]
                    }
                );
                assert_wire_fields(&row.header, record.as_bytes())?;
                assert_wire_fields(&row.strings, &record.as_bytes()[8..])?;
                assert_wire_fields(&row.name, &record.as_bytes()[10..])?;
            }
            Row::Standard(StandardRow::Color(row)) => {
                assert_eq!(record.page(), PageIndex(14));
                assert_eq!((row.unknown_04, row.id), (slot as u8 + 1, slot + 1));
                assert_eq!((row.unknown_00, row.unknown_07), (0, 0));
                assert_eq!(
                    row.name.text()?,
                    ["Pink", "Red", "Orange", "Yellow", "Green", "Aqua", "Blue", "Purple"]
                        [slot as usize]
                );
                assert_wire_fields(row, record.as_bytes())?;
            }
            Row::Standard(StandardRow::PlaylistTree(row)) => {
                assert_eq!(record.page(), PageIndex(16));
                assert_eq!(
                    (
                        row.parent_id,
                        row.unknown_04,
                        row.position,
                        row.id,
                        row.folder_flag
                    ),
                    (0, 0, 0, 1, 0)
                );
                assert_eq!(row.name.text()?, "Demo Tracks");
                assert_wire_fields(row, record.as_bytes())?;
            }
            Row::Standard(StandardRow::PlaylistEntry(row)) => {
                assert_eq!(record.page(), PageIndex(18));
                assert_eq!(
                    (row.position, row.track_id, row.playlist_id),
                    (u32::from(slot) - 3, u32::from(slot) - 3, 1)
                );
                assert_wire_fields(row, record.as_bytes())?;
            }
            Row::Standard(StandardRow::BrowseColumn(row)) => {
                assert_eq!(record.page(), PageIndex(34));
                let i = slot as usize;
                assert_eq!(row.id, slot + 1);
                assert_eq!(
                    row.code,
                    [
                        0x80, 0x81, 0x82, 0x83, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8d,
                        0x8e, 0x92, 0x93, 0x94, 0x84, 0x98, 0x95, 0x91, 0x96, 0x8c, 0x97, 0x90,
                        0xa1, 0xa2, 0xaa
                    ][i]
                );
                let name = [
                    "GENRE",
                    "ARTIST",
                    "ALBUM",
                    "TRACK",
                    "BPM",
                    "RATING",
                    "YEAR",
                    "REMIXER",
                    "LABEL",
                    "ORIGINAL ARTIST",
                    "KEY",
                    "CUE",
                    "COLOR",
                    "TIME",
                    "BITRATE",
                    "FILE NAME",
                    "PLAYLIST",
                    "HOT CUE BANK",
                    "HISTORY",
                    "SEARCH",
                    "COMMENTS",
                    "DATE ADDED",
                    "DJ PLAY COUNT",
                    "FOLDER",
                    "DEFAULT",
                    "ALPHABET",
                    "MATCHING",
                ][i];
                assert_eq!(row.name.text()?, format!("\u{fffa}{name}\u{fffb}"));
                assert_wire_fields(row, record.as_bytes())?;
            }
            Row::Standard(StandardRow::BrowseCategory(row)) => {
                assert!(matches!(record.page().0, 36 | 44));
                if record.page().0 == 44 {
                    assert_eq!(
                        (
                            row.column_id,
                            row.selector,
                            row.unknown_04,
                            row.state,
                            row.position
                        ),
                        (18, 23, 0x63, 0, 11)
                    );
                }
                assert_wire_fields(row, record.as_bytes())?;
            }
            Row::Standard(StandardRow::BrowseSort(row)) => {
                assert_eq!(record.page(), PageIndex(38));
                assert_eq!(
                    row.raw_tail,
                    if slot < 10 {
                        1
                    } else {
                        u32::from(slot - 9) << 8
                    }
                );
                assert_wire_fields(row, record.as_bytes())?;
            }
            Row::Standard(StandardRow::ExportProperty(row)) => {
                assert_eq!(record.page(), PageIndex(40));
                assert_eq!(
                    (
                        row.subtype,
                        row.index_shift,
                        row.track_count,
                        row.unknown_08,
                        row.background_color,
                        row.unknown_0a,
                        row.separator
                    ),
                    (0x280, 0x40, 2, 0, 0, 0, 0x1e19)
                );
                assert_eq!(row.date.text()?, "2025-11-23");
                assert_eq!(row.version.text()?, "1000");
                assert_eq!(row.name.text()?, "");
                assert_wire_fields(row, record.as_bytes())?;
                assert_eq!(&record.as_bytes()[31..], &[0; 9]);
            }
            Row::Extended(ExtendedRow::DatabaseProperty(row)) => {
                assert_eq!(record.page(), PageIndex(16));
                assert_eq!(
                    (
                        row.header.subtype,
                        row.header.index_shift,
                        row.header.my_tag_master_db_id
                    ),
                    (0x700, 0, 3499321918)
                );
                assert_eq!(row.header.unknown_words, [0; 5]);
                assert_eq!(
                    row.strings,
                    StringOffsets::Near {
                        marker: 3,
                        offsets: [34, 35, 36, 37, 38]
                    }
                );
                for frame in &row.unknown_strings {
                    assert_eq!(frame.as_bytes(), &[3]);
                }
                assert_wire_fields(&row.header, record.as_bytes())?;
                assert_wire_fields(&row.strings, &record.as_bytes()[28..])?;
                assert_eq!(&record.as_bytes()[39..], &[0; 21]);
            }
            other => panic!("unexpected native row: {other:?}"),
        }
        Ok(())
    }

    fn assert_native_track(row: &TrackRow, record: &RowRecord) -> Result<()> {
        let h = &row.header;
        let second = record.slot().slot == 9;
        assert_eq!(record.slot().slot, if second { 9 } else { 3 });
        assert_eq!((h.subtype, h.index_shift), (0x24, record.slot().slot * 32));
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
            [
                788224,
                44100,
                0,
                if second { 5124342 } else { 6899624 },
                if second { 73030455 } else { 229237539 },
                3499321917,
                0,
                0,
                0,
                1,
                0,
                320,
                0,
                if second { 12000 } else { 12800 },
                0,
                0,
                1,
                if second { 2 } else { 1 }
            ]
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
            [0, 0, 0, 16, if second { 128 } else { 172 }, 41]
        );
        assert_eq!((h.color_id, h.rating, h.audio_format), (0, 0, 1));
        let t = &row.text;
        let frames = [
            &t.isrc,
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
        let expected = [
            "",
            "",
            "1",
            "1",
            if second { "9" } else { "3" },
            "",
            "ON",
            "ON",
            "",
            "",
            "2018-05-25",
            "",
            "",
            "",
            if second {
                "/PIONEER/USBANLZ/P053/0001D21F/ANLZ0000.DAT"
            } else {
                "/PIONEER/USBANLZ/P016/0000875E/ANLZ0000.DAT"
            },
            "2025-11-23",
            "Tracks by www.loopmasters.com",
            if second {
                "Demo Track 2"
            } else {
                "Demo Track 1"
            },
            "",
            if second {
                "Demo Track 2.mp3"
            } else {
                "Demo Track 1.mp3"
            },
            if second {
                "/Contents/Loopmasters/UnknownAlbum/Demo Track 2.mp3"
            } else {
                "/Contents/Loopmasters/UnknownAlbum/Demo Track 1.mp3"
            },
        ];
        for (i, (frame, expected)) in frames.into_iter().zip(expected).enumerate() {
            assert_eq!(frame.text()?, expected);
            let offset = row.strings.offset(i).unwrap() as usize;
            assert_eq!(
                frame.as_bytes(),
                &record.as_bytes()[offset..offset + frame.as_bytes().len()]
            );
        }
        assert_wire_fields(&row.header, record.as_bytes())?;
        assert_wire_fields(&row.strings, &record.as_bytes()[92..])?;
        Ok(())
    }

    fn fixture_bytes(name: &str) -> Result<Vec<u8>> {
        let temporary = create_temp_dir();
        let path = temporary.path().join(name);
        fs::copy(export_dir(7).join("rekordbox").join(name), &path)?;
        Ok(fs::read(path)?)
    }

    #[test]
    fn standard_live_rows_match_native_fields_and_bytes() -> Result<()> {
        check_native_rows("export.pdb", DatabaseKind::Standard, 82, 82, 21)
    }

    #[test]
    fn extended_live_rows_decode_tags_properties_and_preserve_native_bytes() -> Result<()> {
        check_native_rows("exportExt.pdb", DatabaseKind::Extended, 29, 29, 0)
    }

    #[test]
    fn extended_tags_match_independently_inspected_native_fields_and_frames() -> Result<()> {
        // Slot order, IDs, parents, positions, attributes and names read directly from
        // page 8's footer and byte offsets, independently of the row codec.
        let expected = [
            (1, 0, 0, 1, "Genre"),
            (1863611723, 1, 0, 0, "Acid House"),
            (4129554587, 1, 1, 0, "Deep House"),
            (3061988955, 1, 2, 0, "Techno"),
            (642138584, 1, 3, 0, "Nu Disco"),
            (1626774977, 1, 4, 0, "Electro House"),
            (4070996164, 1, 5, 0, "Bass Music"),
            (2621260931, 1, 6, 0, "Trap"),
            (2, 0, 1, 1, "Components"),
            (534796494, 2, 0, 0, "Synth"),
            (792608194, 2, 1, 0, "Vocal"),
            (3435271903, 2, 2, 0, "Beat"),
            (3283114510, 2, 3, 0, "Sub Bass"),
            (479189410, 2, 4, 0, "Percussion"),
            (3114346071, 2, 5, 0, "Piano"),
            (3924118238, 2, 6, 0, "Dark"),
            (2261816450, 2, 7, 0, "Upper"),
            (3, 0, 2, 1, "Situation"),
            (3853154444, 3, 0, 0, "Main Floor"),
            (1282686349, 3, 1, 0, "Second Floor"),
            (1584558059, 3, 2, 0, "Lounge"),
            (4111223922, 3, 3, 0, "Mid Night"),
            (3328208362, 3, 4, 0, "Morning"),
            (906384457, 3, 5, 0, "Build up"),
            (205699244, 3, 6, 0, "Peak Time"),
            (523363180, 3, 7, 0, "Build down"),
            (4, 0, 3, 1, "Untitled Column"),
            (486635106, 4, 0, 0, "My Comment"),
        ];
        let bytes = fixture_bytes("exportExt.pdb")?;
        let mut input = Cursor::new(&bytes);
        input.set_position(8 * 4096);
        let page = PageHeader::read_le(&mut input)?;
        let directory = read_row_directory(&mut input, 4096, &page)?;
        assert_eq!(directory.slots().count(), expected.len());
        for (slot, &(id, parent, position, attribute, name)) in expected.iter().enumerate() {
            let record =
                read_row(&mut input, DatabaseKind::Extended, &directory, slot as u16)?.unwrap();
            let Row::Extended(ExtendedRow::MyTag(row)) = record.decode()? else {
                panic!("expected tag")
            };
            assert_eq!(
                (
                    row.header.id,
                    row.header.parent_id,
                    row.header.position,
                    row.header.attribute
                ),
                (id, parent, position, attribute)
            );
            assert_eq!(row.header.subtype, 0x0680);
            assert_eq!(row.header.index_shift, slot as u16 * 32);
            assert_eq!(
                (
                    row.header.unknown_04,
                    row.header.unknown_08,
                    row.header.unknown_18,
                    row.header.unknown_1a
                ),
                (0, 0, 0, 0)
            );
            assert_eq!(row.header.is_category(), Some(attribute == 1));
            assert_eq!(row.name.text()?, name);
            assert_eq!(row.unknown_string.as_bytes(), &[3]);
            let reserved_offset = 32 + name.len();
            assert_eq!(
                row.strings,
                StringOffsets::Near {
                    marker: 3,
                    offsets: [31, reserved_offset as u8]
                }
            );
            assert_eq!(row.name.as_bytes(), &record.as_bytes()[31..reserved_offset]);
            assert_eq!(record.as_bytes()[reserved_offset], 3);
            let mut output = Cursor::new(Vec::new());
            row.header.write_le(&mut output)?;
            row.strings.write_le(&mut output)?;
            assert_eq!(output.into_inner(), record.as_bytes()[..31]);
        }
        Ok(())
    }

    fn check_native_rows(
        name: &str,
        kind: DatabaseKind,
        expected_live: usize,
        expected_decoded: usize,
        expected_inactive: usize,
    ) -> Result<()> {
        let bytes = fixture_bytes(name)?;
        let mut input = Cursor::new(&bytes);
        let header = FileHeader::read_le(&mut input)?;
        let mut live = 0;
        let mut decoded = 0;
        let mut inactive = 0;
        for chain in read_table_chains(&mut input, &header)? {
            for page in chain.pages.iter().filter(|page| !page.kind.is_index()) {
                let directory = read_row_directory(&mut input, header.prefix.page_size, page)?;
                for slot in directory.slots() {
                    assert_eq!(directory.slot(slot.slot), Some(slot));
                    let before = input.position();
                    let record = read_row(&mut input, kind, &directory, slot.slot)?;
                    if !slot.live {
                        inactive += 1;
                        assert!(record.is_none());
                        assert_eq!(input.position(), before);
                        continue;
                    }
                    live += 1;
                    let record = record.unwrap();
                    assert_eq!(record.page(), page.common.page_index);
                    assert_eq!(record.table(), kind.table_kind(chain.table.table_type));
                    assert_eq!(record.slot(), slot);
                    let start = page.common.page_index.0 as usize
                        * header.prefix.page_size as usize
                        + 40
                        + usize::from(slot.heap_offset);
                    let end = start + record.as_bytes().len();
                    assert_eq!(record.as_bytes(), &bytes[start..end]);
                    assert_eq!(input.position() as usize, end);
                    match record.decode()? {
                        Row::Standard(StandardRow::Label(row)) => {
                            decoded += 1;
                            assert_eq!(record.page(), PageIndex(10));
                            assert_eq!(slot.slot, 0);
                            // Independent byte baseline: u32 ID, short marker, eleven ASCII bytes.
                            assert_eq!(record.as_bytes(), b"\x01\x00\x00\x00\x19Loopmasters");
                            assert_eq!(row.id, 1);
                            assert_eq!(row.name.text()?, "Loopmasters");
                            let mut output = Cursor::new(Vec::new());
                            row.write_le(&mut output)?;
                            assert_eq!(output.into_inner(), record.as_bytes());
                        }
                        Row::Extended(ExtendedRow::MyTag(row)) => {
                            decoded += 1;
                            assert_eq!(kind, DatabaseKind::Extended);
                            assert_eq!(record.page(), PageIndex(8));
                            assert_eq!(row.header.index_shift, slot.slot * 32);
                        }
                        Row::Standard(StandardRow::Opaque) => {
                            assert_eq!(kind, DatabaseKind::Standard)
                        }
                        Row::Extended(ExtendedRow::Opaque) => {
                            assert_eq!(kind, DatabaseKind::Extended)
                        }
                        other => {
                            check_remaining_native_row(&other, &record)?;
                            decoded += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(
            (live, decoded, inactive),
            (expected_live, expected_decoded, expected_inactive)
        );
        Ok(())
    }

    #[test]
    fn standard_index_entries_match_native_raw_words_and_leave_filler_unread() -> Result<()> {
        check_native_indexes(
            "export.pdb",
            20,
            &[
                (1, &[0x10, 0x188]),
                (15, &[0x80]),
                (17, &[0x90]),
                (19, &[0xa3]),
                (35, &[0x123]),
                (39, &[0x140]),
            ],
        )
    }

    #[test]
    fn extended_indexes_have_no_declared_entries_despite_linked_data_pages() -> Result<()> {
        check_native_indexes("exportExt.pdb", 9, &[])
    }

    fn check_native_indexes(
        name: &str,
        expected_pages: usize,
        populated: &[(u32, &[u32])],
    ) -> Result<()> {
        let bytes = fixture_bytes(name)?;
        let mut input = Cursor::new(&bytes);
        let header = FileHeader::read_le(&mut input)?;
        let chains = read_table_chains(&mut input, &header)?;
        let mut index_pages = 0;
        let mut total = 0;
        let mut data_without_entries = Vec::new();
        for chain in chains {
            let mut references = Vec::new();
            for page in &chain.pages {
                if !page.kind.is_index() {
                    continue;
                }
                index_pages += 1;
                let entries = read_index_entries(&mut input, header.prefix.page_size, page)?;
                let raw: Vec<_> = entries.iter().map(|entry| entry.0).collect();
                let expected = populated
                    .iter()
                    .find(|(index, _)| *index == page.common.page_index.0)
                    .map(|(_, words)| *words)
                    .unwrap_or(&[]);
                assert_eq!(raw, expected, "{name}: page {}", page.common.page_index.0);
                let start =
                    page.common.page_index.0 as usize * header.prefix.page_size as usize + 60;
                let end = start + entries.len() * 4;
                assert_eq!(input.position() as usize, end);
                assert_eq!(&bytes[end..end + 4], &[0xf8, 0xff, 0xff, 0x1f]); // First unused entry.
                let mut output = Cursor::new(Vec::new());
                for entry in &entries {
                    entry.write_le(&mut output)?;
                    if !entry.is_empty() {
                        references.push(entry.page_index());
                    }
                }
                assert_eq!(output.into_inner(), bytes[start..end]);
                total += entries.len();
            }
            for page in chain.pages {
                if !page.kind.is_index() && !references.contains(&page.common.page_index) {
                    data_without_entries.push(page.common.page_index.0);
                }
            }
        }
        assert_eq!(index_pages, expected_pages);
        assert_eq!(
            total,
            populated
                .iter()
                .map(|(_, words)| words.len())
                .sum::<usize>()
        );
        if name == "exportExt.pdb" {
            assert_eq!(data_without_entries, [8, 16]);
        } else {
            assert_eq!(data_without_entries, [6, 10, 14, 34, 44, 38]);
        }
        Ok(())
    }

    #[test]
    fn standard_row_directories_match_native_bytes_and_live_counts() -> Result<()> {
        check_native_directories(
            "export.pdb",
            103,
            &[
                2, 0, 1, 0, 1, 0, 8, 1, 2, 0, 0, 0, 0, 0, 0, 0, 27, 22, 17, 1,
            ],
        )
    }

    #[test]
    fn extended_row_directories_match_native_bytes_and_live_counts() -> Result<()> {
        check_native_directories("exportExt.pdb", 29, &[0, 0, 0, 28, 0, 0, 0, 1, 0])
    }

    fn check_native_directories(
        name: &str,
        expected_allocated: usize,
        expected_live: &[usize],
    ) -> Result<()> {
        let bytes = fixture_bytes(name)?;
        let mut input = Cursor::new(&bytes);
        let header = FileHeader::read_le(&mut input)?;
        let chains = read_table_chains(&mut input, &header)?;
        let mut allocated = 0;
        let mut live_counts = Vec::new();
        for chain in chains {
            let mut live = 0;
            for page in chain.pages {
                if page.kind.is_index() {
                    continue;
                }
                let directory = read_row_directory(&mut input, header.prefix.page_size, &page)?;
                let slots: Vec<_> = directory.slots().collect();
                allocated += slots.len();
                live += slots.iter().filter(|slot| slot.live).count();
                let mut footer = Cursor::new(Vec::new());
                for group in directory.groups().iter().rev() {
                    group.write_le(&mut footer)?;
                }
                let footer = footer.into_inner();
                let end =
                    (page.common.page_index.0 as usize + 1) * header.prefix.page_size as usize;
                assert_eq!(input.position() as usize, end);
                assert_eq!(footer, bytes[end - footer.len()..end]);
                if name == "export.pdb" && page.common.page_index == PageIndex(40) {
                    assert_eq!(directory.groups()[0].offsets, [80, 40, 0]);
                    assert_eq!(directory.groups()[0].presence_flags, 4);
                    assert_eq!(directory.groups()[0].transaction_flags, 6);
                    assert!(!slots[1].live && slots[1].touched);
                    assert_eq!(directory.live_row_window(1), None);
                    assert_eq!(directory.live_row_window(2), Some(80..120));
                }
                if name == "export.pdb" && page.common.page_index == PageIndex(38) {
                    assert_eq!(footer.len(), 42); // 16 offsets plus a one-slot partial group.
                }
                if name == "exportExt.pdb" && page.common.page_index == PageIndex(8) {
                    assert_eq!(directory.groups().len(), 2);
                    assert_eq!(directory.groups()[1].offsets.len(), 12);
                    assert_eq!(directory.groups()[1].presence_flags, 0x0fff);
                }
            }
            live_counts.push(live);
        }
        assert_eq!(allocated, expected_allocated);
        assert_eq!(live_counts, expected_live);
        Ok(())
    }

    #[test]
    fn standard_table_chains_match_native_baseline() -> Result<()> {
        check_native_chains(
            "export.pdb",
            &[
                &[1, 2, 49],
                &[3],
                &[5, 6],
                &[7],
                &[9, 10],
                &[11],
                &[13, 14],
                &[15, 16],
                &[17, 18],
                &[19, 20],
                &[21],
                &[23],
                &[25],
                &[27],
                &[29],
                &[31],
                &[33, 34],
                &[35, 36, 44],
                &[37, 38],
                &[39, 40],
            ],
            33,
            16,
        )
    }

    #[test]
    fn extended_table_chains_match_native_baseline() -> Result<()> {
        check_native_chains(
            "exportExt.pdb",
            &[
                &[1],
                &[3],
                &[5],
                &[7, 8],
                &[9],
                &[11],
                &[13],
                &[15, 16],
                &[17],
            ],
            11,
            6,
        )
    }

    fn check_native_chains(
        name: &str,
        expected: &[&[u32]],
        linked: usize,
        unlinked: usize,
    ) -> Result<()> {
        let bytes = fixture_bytes(name)?;
        let mut input = Cursor::new(&bytes);
        let header = FileHeader::read_le(&mut input)?;
        let chains = read_table_chains(&mut input, &header)?;
        assert_eq!(chains.len(), expected.len());
        for (index, (chain, expected_pages)) in chains.iter().zip(expected).enumerate() {
            assert_eq!(chain.table, header.tables[index]);
            let actual: Vec<_> = chain
                .pages
                .iter()
                .map(|page| page.common.page_index.0)
                .collect();
            assert_eq!(actual, *expected_pages, "{name}: directory entry {index}");
        }
        let actual_linked: usize = chains.iter().map(|chain| chain.pages.len()).sum();
        assert_eq!(actual_linked, linked);
        assert_eq!(
            bytes.len() / header.prefix.page_size as usize - 1 - actual_linked,
            unlinked
        );
        Ok(())
    }

    #[test]
    fn native_table_names_use_explicit_namespaces_without_changing_bytes() -> Result<()> {
        for (name, kind, expected_kind, named_tables) in [
            (
                "export.pdb",
                DatabaseKind::Standard,
                TableKind::Standard(PageType::Albums),
                16,
            ),
            (
                "exportExt.pdb",
                DatabaseKind::Extended,
                TableKind::Extended(ExtPageType::MyTags),
                9,
            ),
        ] {
            let bytes = fixture_bytes(name)?;
            let header = FileHeader::read_le(&mut Cursor::new(&bytes))?;
            let table = header
                .tables
                .iter()
                .find(|table| table.table_type == 3)
                .unwrap();
            assert_eq!(kind.table_kind(table.table_type), expected_kind);
            for table in &header.tables {
                let table_kind = kind.table_kind(table.table_type);
                assert_eq!(table_kind.database_kind(), kind);
                assert_eq!(table_kind.code(), table.table_type);
            }
            assert_eq!(
                header
                    .tables
                    .iter()
                    .filter(|table| kind.table_name(table.table_type).is_some())
                    .count(),
                named_tables,
            );
            let mut output = Cursor::new(Vec::new());
            header.write_le(&mut output)?;
            assert_eq!(output.into_inner(), bytes[..4096]);
        }
        Ok(())
    }

    fn check_file_header(
        name: &str,
        physical_pages: usize,
        next_unused_page: u32,
        sequence: u32,
        tables: &[[u32; 4]],
    ) -> Result<()> {
        let bytes = fixture_bytes(name)?;
        assert_eq!(bytes.len(), physical_pages * 4096);
        let mut input = Cursor::new(&bytes);
        let header = FileHeader::read_le(&mut input)?;
        assert_eq!(input.position(), 4096);
        assert_eq!(
            header.prefix,
            FileHeaderPrefix {
                signature: 0,
                page_size: 4096,
                table_count: tables.len() as u32,
                next_unused_page: PageIndex(next_unused_page),
                unknown_10: 5,
                sequence,
                unknown_18: 0,
            }
        );
        let actual_tables: Vec<_> = header
            .tables
            .iter()
            .map(|table| {
                [
                    table.table_type,
                    table.empty_candidate.0,
                    table.first_page.0,
                    table.last_page.0,
                ]
            })
            .collect();
        assert_eq!(actual_tables, tables);
        // These native allocator references legitimately exceed physical EOF.
        assert!(header.prefix.next_unused_page.file_offset(4096).unwrap() > bytes.len() as u64);
        let mut output = Cursor::new(Vec::new());
        header.write_le(&mut output)?;
        assert_eq!(output.into_inner(), bytes[..4096]);
        Ok(())
    }

    #[test]
    fn standard_file_header_matches_native_baseline() -> Result<()> {
        check_file_header(
            "export.pdb",
            50,
            54,
            47,
            &[
                [0, 51, 1, 49],
                [1, 4, 3, 3],
                [2, 47, 5, 6],
                [3, 8, 7, 7],
                [4, 48, 9, 10],
                [5, 12, 11, 11],
                [6, 42, 13, 14],
                [7, 46, 15, 16],
                [8, 50, 17, 18],
                [9, 52, 19, 20],
                [10, 22, 21, 21],
                [11, 24, 23, 23],
                [12, 26, 25, 25],
                [13, 28, 27, 27],
                [14, 30, 29, 29],
                [15, 32, 31, 31],
                [16, 43, 33, 34],
                [17, 53, 35, 44],
                [18, 45, 37, 38],
                [19, 41, 39, 40],
            ],
        )
    }

    #[test]
    fn extended_file_header_matches_native_baseline() -> Result<()> {
        check_file_header(
            "exportExt.pdb",
            18,
            21,
            14,
            &[
                [0, 2, 1, 1],
                [1, 4, 3, 3],
                [2, 6, 5, 5],
                [3, 20, 7, 8],
                [4, 10, 9, 9],
                [5, 12, 11, 11],
                [6, 14, 13, 13],
                [7, 19, 15, 16],
                [8, 18, 17, 17],
            ],
        )
    }

    fn check_page_headers(
        name: &str,
        physical_pages: usize,
        index_headers: usize,
    ) -> Result<Vec<PageHeader>> {
        let bytes = fixture_bytes(name)?;
        assert_eq!(bytes.len(), physical_pages * 4096);
        let mut input = Cursor::new(&bytes);
        let mut pages = Vec::new();
        let mut indices = 0;
        for physical_page in 1..physical_pages {
            let offset = physical_page * 4096;
            input.set_position(offset as u64);
            let page = PageHeader::read_le(&mut input)?;
            let size = match page.kind {
                PageHeaderKind::Data(_) => 40,
                PageHeaderKind::Index(_) => {
                    indices += 1;
                    60
                }
            };
            assert_eq!(page.binary_size() as usize, size);
            assert_eq!(input.position() as usize, offset + size);
            let mut output = Cursor::new(Vec::new());
            page.write_le(&mut output)?;
            assert_eq!(
                output.into_inner(),
                bytes[offset..offset + size],
                "page {physical_page}"
            );
            pages.push(page);
        }
        assert_eq!(indices, index_headers);
        Ok(pages)
    }

    #[test]
    fn standard_page_headers_preserve_bytes_and_match_native_fields() -> Result<()> {
        let pages = check_page_headers("export.pdb", 50, 20)?;
        let page = &pages[1]; // Physical page 2: tracks, including inactive slots.
        assert!(matches!(page.kind, PageHeaderKind::Data(_)));
        assert_eq!(page.common.page_index, PageIndex(2));
        assert_eq!(page.common.table_type, 0);
        assert_eq!(page.common.next_page, PageIndex(49));
        assert_eq!(page.common.sequence, 38);
        assert_eq!(page.common.flags.0, 0x34);
        assert_eq!(page.common.counts.allocated(), 10);
        assert_eq!(page.common.counts.live(), 2);
        assert_eq!(page.common.free_size, 192);
        assert_eq!(page.common.used_size, 3840);
        Ok(())
    }

    #[test]
    fn extended_page_headers_preserve_bytes_and_match_native_fields() -> Result<()> {
        let pages = check_page_headers("exportExt.pdb", 18, 9)?;
        let page = &pages[7]; // Physical page 8: extended tags/categories, not albums.
        assert!(matches!(page.kind, PageHeaderKind::Data(_)));
        assert_eq!(page.common.page_index, PageIndex(8));
        assert_eq!(page.common.table_type, 3);
        assert_eq!(page.common.next_page, PageIndex(20));
        assert_eq!(page.common.sequence, 2);
        assert_eq!(page.common.flags.0, 0x24);
        assert_eq!(page.common.counts.allocated(), 28);
        assert_eq!(page.common.counts.live(), 28);
        assert_eq!(page.common.free_size, 2568);
        assert_eq!(page.common.used_size, 1424);

        // The empty table 0 index has a candidate successor, but no data-page reference.
        assert_eq!(pages[0].common.next_page, PageIndex(2));
        let PageHeaderKind::Index(index) = pages[0].kind else {
            panic!("expected the native extended table 0 index header");
        };
        assert_eq!(index.data_page, PageIndex::NONE);
        assert_eq!(index.entry_count, 0);
        Ok(())
    }
}

mod exports {
    //! Complete live-row scans of the two approved native fixture pairs.

    use super::*;

    #[test]
    fn standard_exports_decode_every_live_window() -> Result<()> {
        for (directory, expected, inactive) in [
            (
                export_dir(7).join("rekordbox"),
                [
                    2, 0, 1, 0, 1, 0, 8, 1, 2, 0, 0, 0, 0, 0, 0, 0, 27, 22, 17, 1,
                ],
                21,
            ),
            (
                testdata_dir().join("export/empty/PIONEER/rekordbox"),
                [
                    0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 27, 22, 17, 1,
                ],
                0,
            ),
        ] {
            check_export(
                directory.join("export.pdb"),
                DatabaseKind::Standard,
                &expected,
                inactive,
            )?;
        }
        Ok(())
    }

    #[test]
    fn extended_exports_decode_every_live_window() -> Result<()> {
        for (directory, expected) in [
            (
                export_dir(7).join("rekordbox"),
                [0, 0, 0, 28, 0, 0, 0, 1, 0],
            ),
            (
                testdata_dir().join("export/empty/PIONEER/rekordbox"),
                [0, 0, 0, 29, 0, 0, 0, 1, 0],
            ),
        ] {
            check_export(
                directory.join("exportExt.pdb"),
                DatabaseKind::Extended,
                &expected,
                0,
            )?;
        }
        Ok(())
    }

    fn check_export(
        source: std::path::PathBuf,
        kind: DatabaseKind,
        expected: &[usize],
        expected_inactive: usize,
    ) -> Result<()> {
        let temporary = create_temp_dir();
        let path = temporary.path().join("export.pdb");
        fs::copy(&source, &path)?;
        let bytes = fs::read(path)?;
        let mut input = Cursor::new(&bytes);
        let header = FileHeader::read_le(&mut input)?;
        assert_eq!(header.prefix.page_size, 4096);
        let mut counts = vec![0; expected.len()];
        let mut inactive = 0;
        for chain in read_table_chains(&mut input, &header)? {
            let table = chain.table.table_type as usize;
            for page in chain.pages.iter().filter(|page| !page.kind.is_index()) {
                let directory = read_row_directory(&mut input, header.prefix.page_size, page)?;
                for slot in directory.slots() {
                    if !slot.live {
                        inactive += 1;
                        continue;
                    }
                    let record =
                        read_row(&mut input, kind, &directory, slot.slot)?.expect("live row");
                    let start =
                        page.common.page_index.0 as usize * 4096 + 40 + slot.heap_offset as usize;
                    let end = start + record.as_bytes().len();
                    assert_eq!(record.as_bytes(), &bytes[start..end]);
                    assert_eq!(input.position() as usize, end);
                    assert!(
                        !matches!(
                            record.decode()?,
                            Row::Standard(StandardRow::Opaque) | Row::Extended(ExtendedRow::Opaque)
                        ),
                        "unexpected opaque row in {source:?}, table {table}, page {:?}, slot {}",
                        record.page(),
                        slot.slot
                    );
                    counts[table] += 1;
                }
            }
        }
        assert_eq!(counts, expected, "{source:?}");
        assert_eq!(inactive, expected_inactive, "{source:?}");
        Ok(())
    }
}
mod empty_usb {
    //! Fresh empty USB profiles, native index/footer evidence, and exact reconstruction.

    use super::*;

    #[test]
    fn native_demo_exports_every_master_tag_with_its_original_id() -> anyhow::Result<()> {
        use std::collections::BTreeMap;

        use rbox_db::{
            connection::{ConnectionOptions, OpenMode},
            Model,
        };
        use rbox_devlib::binary::{ExtPageType, ExtendedRow, Row, TableKind};
        use rbox_masterdb::{enums::MyTagType, establish_connection, DjmdMyTag};
        use rbox_testing::data::{export_dir, setup_master_db_path};

        // The fresh empty USB has no matching master.db snapshot. Check ID provenance
        // against the paired RB7 demo instead, using only a temporary SQL database.
        let copy = setup_master_db_path(7);
        let mut master = establish_connection(
            copy.path(),
            &ConnectionOptions {
                open_mode: OpenMode::ReadOnly,
                ..ConnectionOptions::default()
            },
        )?;
        let source = DjmdMyTag::all(&mut master)?;
        let mut expected = BTreeMap::new();
        for tag in source {
            let parent = if tag.parent_id == "root" {
                0
            } else {
                tag.parent_id.parse::<u32>()?
            };
            expected.insert(
                tag.id.parse::<u32>()?,
                (
                    tag.name,
                    parent,
                    u32::try_from(tag.seq - 1)?,
                    u8::from(tag.attribute == MyTagType::Folder),
                ),
            );
        }
        let mut export = DeviceSqlConnection::open(
            export_dir(7).join("rekordbox/exportExt.pdb"),
            DatabaseKind::Extended,
        )?;
        let mut actual = BTreeMap::new();
        for record in export.iter_raw(TableKind::Extended(ExtPageType::MyTags))? {
            let Row::Extended(ExtendedRow::MyTag(tag)) = record?.decode()? else {
                anyhow::bail!("unexpected My Tag row");
            };
            assert!(
                actual
                    .insert(
                        tag.header.id,
                        (
                            tag.name.text()?,
                            tag.header.parent_id,
                            tag.header.position,
                            tag.header.attribute
                        )
                    )
                    .is_none(),
                "duplicate tag ID"
            );
        }
        assert!(!expected.is_empty());
        assert_eq!(actual, expected);
        println!(
            "All {} demo categories/tags retain their master IDs and metadata",
            actual.len()
        );
        Ok(())
    }

    #[test]
    fn both_files_recreate_exactly_from_explicit_inputs() -> Result<()> {
        let temporary = create_temp_dir();
        for (kind, name, options, expected_rows) in [
            (
                DatabaseKind::Standard,
                "export.pdb",
                empty_export::standard()?,
                75,
            ),
            (
                DatabaseKind::Extended,
                "exportExt.pdb",
                empty_export::extended()?,
                30,
            ),
        ] {
            // Author and save before the first access to the reference PDB.
            let mut db = DeviceSqlConnection::create_with_options(kind, &options)?;
            let output = temporary.path().join(name);
            db.save_as(&output)?;
            let reference = testdata_dir()
                .join("export/empty/PIONEER/rekordbox")
                .join(name);
            let original = fs::read(&reference)?;
            assert_eq!(fs::read(&output)?, original, "{name}");
            let mut saved = DeviceSqlConnection::open(output, kind)?;
            let mut count = 0;
            for pointer in saved.header().tables.clone() {
                for record in saved.iter_raw(kind.table_kind(pointer.table_type))? {
                    record?.decode()?;
                    count += 1;
                }
            }
            assert_eq!(count, expected_rows);
            assert_eq!(fs::read(reference)?, original);
        }
        Ok(())
    }

    #[test]
    fn native_empty_indexes_and_data_pages_match_observed_layout() -> Result<()> {
        for (kind, name, table_count, pages, next_unused, sequence) in [
            (DatabaseKind::Standard, "export.pdb", 20, 41, 46, 7),
            (DatabaseKind::Extended, "exportExt.pdb", 9, 18, 21, 4),
        ] {
            let bytes = fs::read(
                testdata_dir()
                    .join("export/empty/PIONEER/rekordbox")
                    .join(name),
            )?;
            assert_eq!(bytes.len(), pages * 4096);
            let mut input = Cursor::new(&bytes);
            let header = FileHeader::read_le(&mut input)?;
            assert_eq!(header.prefix.page_size, 4096);
            assert_eq!(header.prefix.table_count, table_count);
            assert_eq!(header.prefix.next_unused_page, PageIndex(next_unused));
            assert_eq!(header.prefix.unknown_10, 5);
            assert_eq!(header.prefix.sequence, sequence);
            let chains = read_table_chains(&mut input, &header)?;
            assert_eq!(chains.len(), table_count as usize);
            let mut allocated = HashSet::from([0]);
            for chain in chains {
                let code = chain.table.table_type;
                // (rows, used bytes, free bytes, sequence, next allocation candidate).
                let expected = match (kind, code) {
                    (DatabaseKind::Standard, 6) => Some((8, 124, 3912, 2, 42)),
                    (DatabaseKind::Standard, 16) => Some((27, 724, 3270, 3, 43)),
                    (DatabaseKind::Standard, 17) => Some((22, 176, 3828, 4, 44)),
                    (DatabaseKind::Standard, 18) => Some((17, 136, 3878, 5, 45)),
                    (DatabaseKind::Standard, 19) => Some((1, 40, 4010, 1, 41)),
                    (DatabaseKind::Extended, 3) => Some((29, 1444, 2546, 2, 20)),
                    (DatabaseKind::Extended, 7) => Some((1, 60, 3990, 1, 19)),
                    _ => None,
                };
                assert_eq!(chain.pages.len(), if expected.is_some() { 2 } else { 1 });
                let first = &chain.pages[0];
                let PageHeaderKind::Index(index) = first.kind else {
                    panic!("missing index page")
                };
                assert_eq!(first.common.page_index, PageIndex(code * 2 + 1));
                assert_eq!(first.common.flags.0, 0x64);
                assert_eq!(first.common.sequence, 1);
                assert_eq!(index.entry_capacity, 1004);
                assert_eq!(index.entry_count, 0);
                assert_eq!(index.next_entry, 0);
                assert_eq!(index.first_empty, 0x1fff);
                assert_eq!((index.unknown_20, index.unknown_22), (0x1fff, 0x1fff));
                assert_eq!(
                    index.data_page,
                    if expected.is_some() {
                        PageIndex(code * 2 + 2)
                    } else {
                        PageIndex::NONE
                    }
                );
                assert!(read_index_entries(&mut input, 4096, first)?.is_empty());
                let start = first.common.page_index.0 as usize * 4096;
                // Inspect reserved bytes independently of the zero declared entry count.
                let mut filler = Cursor::new(&bytes[start + 60..start + 4076]);
                for _ in 0..1004 {
                    assert_eq!(IndexEntry::read_le(&mut filler)?, IndexEntry::EMPTY);
                }
                assert_eq!(&bytes[start + 4076..start + 4096], &[0; 20]);
                for page in &chain.pages {
                    allocated.insert(page.common.page_index.0 as usize);
                }
                if let Some((rows, used, free, page_sequence, candidate)) = expected {
                    let page = &chain.pages[1];
                    assert_eq!(page.common.counts.allocated(), rows);
                    assert_eq!(page.common.counts.live(), rows);
                    assert_eq!((page.common.used_size, page.common.free_size), (used, free));
                    assert_eq!(page.common.flags.0, 0x24);
                    assert_eq!(page.common.sequence, page_sequence);
                    assert_eq!(page.common.next_page, PageIndex(candidate));
                    assert_eq!(chain.table.empty_candidate, PageIndex(candidate));
                    let PageHeaderKind::Data(data) = page.kind else {
                        panic!("expected data page")
                    };
                    assert_eq!(data.transaction_row_count, rows);
                    assert_eq!(data.transaction_row_index, 0);
                    let directory = read_row_directory(&mut input, 4096, page)?;
                    assert_eq!(directory.slots().count(), usize::from(rows));
                    // Initialization marks every slot touched even without page flag 0x10.
                    assert!(directory.slots().all(|slot| slot.live && slot.touched));
                }
            }
            for (index, page) in bytes.chunks_exact(4096).enumerate() {
                if !allocated.contains(&index) {
                    assert!(page.iter().all(|&byte| byte == 0));
                }
            }
        }
        Ok(())
    }
}
