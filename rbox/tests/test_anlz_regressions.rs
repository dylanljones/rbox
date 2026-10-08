// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use binrw::{BinRead, BinWrite, NullWideString};
use rbox::anlz::anlz::{
    AnlzData, Beat, Content, Section, TinyWaveformColumn, WaveformColorDetailColumn,
};
use rbox::anlz::{Anlz, AnlzTag};
use std::io::Cursor;
use std::path::{Path, PathBuf};

mod common;

fn fixtures(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            fixtures(&path, files);
        } else if path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("ANLZ")
            && matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("DAT" | "EXT" | "2EX")
            )
        {
            files.push(path);
        }
    }
}

fn all_fixtures() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for version in [6, 7] {
        fixtures(&common::testdata_demo_dir(version), &mut files);
    }
    assert!(!files.is_empty());
    files
}

fn encode(data: &AnlzData) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    data.write(&mut out).unwrap();
    out.into_inner()
}

fn check_lengths(bytes: &[u8]) {
    let read_u32 =
        |offset| u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
    assert_eq!(read_u32(8), bytes.len());
    let mut offset = read_u32(4);
    while offset < bytes.len() {
        let size = read_u32(offset + 8);
        assert!(size >= read_u32(offset + 4));
        let mut reader = Cursor::new(&bytes[offset..offset + size]);
        Section::read_be(&mut reader).unwrap();
        assert_eq!(reader.position() as usize, size);
        offset += size;
    }
    assert_eq!(offset, bytes.len());
}

#[test]
fn all_rb6_and_rb7_fixtures_round_trip_exactly() {
    for path in all_fixtures() {
        let bytes = std::fs::read(&path).unwrap();
        let data = AnlzData::read(&mut Cursor::new(&bytes))
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        assert_eq!(encode(&data), bytes, "{}", path.display());
    }
}

#[test]
fn resized_sections_and_comments_have_consistent_lengths() {
    for path in all_fixtures() {
        let bytes = std::fs::read(path).unwrap();
        let mut data = AnlzData::read(&mut Cursor::new(bytes)).unwrap();
        for section in &mut data.sections {
            match &mut section.content {
                Content::BeatGrid(g) => g.beats.push(Beat {
                    beat_number: 1,
                    tempo: 12000,
                    time: 0,
                }),
                Content::ExtendedBeatGrid(g) => {
                    g.beats.pop();
                }
                Content::CueList(g) => {
                    g.cues.pop();
                }
                Content::ExtendedCueList(g) => {
                    if let Some(cue) = g.cues.first_mut() {
                        cue.comment = NullWideString::from("Edited \u{1f3b5} cue");
                        let copy = cue.clone();
                        g.cues.push(copy);
                    }
                }
                Content::Path(p) => {
                    p.path = NullWideString::from("/Music/\u{1f3b5}.flac");
                }
                Content::VBR(v) => v.data.push(0),
                Content::TinyWaveformPreview(w) => {
                    w.data.pop();
                }
                Content::WaveformPreview(w) => {
                    w.data.pop();
                }
                Content::WaveformDetail(w) => {
                    w.data.pop();
                }
                Content::WaveformColorPreview(w) => {
                    w.data.pop();
                }
                Content::WaveformColorDetail(w) => {
                    w.data.pop();
                }
                Content::Waveform3BandPreview(w) => {
                    w.data.pop();
                }
                Content::Waveform3BandDetail(w) => {
                    w.data.pop();
                }
                Content::SongStructure(s) => {
                    s.data.phrases.pop();
                }
                Content::Unknown(_) => {}
            }
        }
        // Direct BinWrite must work without a preceding update_total_size().
        let bytes = encode(&data);
        check_lengths(&bytes);
        let parsed = AnlzData::read(&mut Cursor::new(&bytes)).unwrap();
        assert_eq!(encode(&parsed), bytes);
        data.update_total_size();
        assert_eq!(data.header.total_size as usize, bytes.len());
    }
}

fn pssi(version: u16, encrypted: bool, entries: u16) -> Vec<u8> {
    let size = 32 + u32::from(entries) * 24;
    let mut bytes = b"PSSI".to_vec();
    bytes.extend(32u32.to_be_bytes());
    bytes.extend(size.to_be_bytes());
    bytes.extend(version.to_be_bytes());
    bytes.extend(24u16.to_be_bytes());
    bytes.extend(entries.to_be_bytes());
    let mut payload = vec![0; 14 + usize::from(entries) * 24];
    payload[1] = 1; // mood: High; bank: Default
    if encrypted {
        let mask = [
            0xcb_u8, 0xe1, 0xee, 0xfa, 0xe5, 0xee, 0xad, 0xee, 0xe9, 0xd2, 0xe9, 0xeb, 0xe1, 0xe9,
            0xf3, 0xe8, 0xe9, 0xf4, 0xe1,
        ];
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[(i + usize::from(version)) % 19].wrapping_add(entries as u8);
        }
    }
    bytes.extend(payload);
    bytes
}

#[test]
fn pssi_versions_plain_and_masked_round_trip() {
    for version in [0, 1] {
        for encrypted in [false, true] {
            // Includes counts that wrap the mask addition.
            for entries in [0, 1, 300] {
                let bytes = pssi(version, encrypted, entries);
                let mut section = Section::read_be(&mut Cursor::new(&bytes)).unwrap();
                if let Content::SongStructure(s) = &mut section.content {
                    assert_eq!(s.version, version);
                    assert_eq!(s.is_encrypted, encrypted);
                    assert_eq!(s.data.phrases.len(), usize::from(entries));
                } else {
                    panic!("Expected PSSI");
                }
                let mut out = Cursor::new(Vec::new());
                section.write_be(&mut out).unwrap();
                assert_eq!(out.into_inner(), bytes);
                if entries > 0 {
                    if let Content::SongStructure(s) = &mut section.content {
                        s.data.phrases.pop();
                    }
                    let mut out = Cursor::new(Vec::new());
                    section.write_be(&mut out).unwrap();
                    let changed = out.into_inner();
                    let parsed = Section::read_be(&mut Cursor::new(&changed)).unwrap();
                    if let Content::SongStructure(s) = parsed.content {
                        assert_eq!(s.data.phrases.len(), usize::from(entries - 1));
                        assert_eq!(s.is_encrypted, encrypted);
                    } else {
                        panic!("Expected PSSI");
                    }
                }
            }
        }
    }
}

#[test]
fn pqt2_preserves_both_observed_versions() {
    for version in [0x01000002_u32, 0x02000002] {
        let mut bytes = b"PQT2".to_vec();
        for n in [56, 56, 0, version, 0, 0, 0, 0, 0, 0, 0, 0, 0] {
            bytes.extend(n.to_be_bytes());
        }
        let section = Section::read_be(&mut Cursor::new(&bytes)).unwrap();
        let mut out = Cursor::new(Vec::new());
        section.write_be(&mut out).unwrap();
        assert_eq!(out.into_inner(), bytes);
    }
}

#[test]
fn waveform_values_match_disk_bit_layout() {
    for byte in 0..=u8::MAX {
        let column = TinyWaveformColumn::read_be(&mut Cursor::new([byte])).unwrap();
        assert_eq!(column.height(), byte & 31);
        assert_eq!(column.whiteness(), byte >> 5);
    }
    for word in 0..=u16::MAX {
        let column =
            WaveformColorDetailColumn::read_be(&mut Cursor::new(word.to_be_bytes())).unwrap();
        assert_eq!(column.red(), ((word >> 13) & 7) as u8);
        assert_eq!(column.green(), ((word >> 10) & 7) as u8);
        assert_eq!(column.blue(), ((word >> 7) & 7) as u8);
        assert_eq!(column.height(), ((word >> 2) & 31) as u8);
        let mut out = Cursor::new(Vec::new());
        column.write_be(&mut out).unwrap();
        assert_eq!(out.into_inner(), word.to_be_bytes());
    }
}

#[test]
fn path_normalization_and_io_errors() {
    let temp = common::setup_anlz_files();
    let mut file = Anlz::load(temp.path().join("ANLZ0000.DAT")).unwrap();
    file.set_path(r"C:\Music\track.flac").unwrap();
    assert_eq!(file.get_path().unwrap(), "C:/Music/track.flac");
    assert!(Anlz::load(temp.path().join("missing.DAT")).is_err());
    let invalid = temp.path().join("invalid.DAT");
    std::fs::write(&invalid, b"not an analysis file").unwrap();
    assert!(Anlz::load(invalid).is_err());
    assert!(file
        .dump_copy(temp.path().join("missing").join("out.DAT"))
        .is_err());
    assert_eq!(
        AnlzTag::from("SongStructure".to_owned()),
        AnlzTag::SongStructure
    );
}

#[test]
fn extended_cues_preserve_comment_padding_and_variable_trailers() {
    for units in [vec![], vec![0], vec![65], vec![65, 0], vec![65, 0, 0]] {
        for padding in [0, 20, 40] {
            let comment_len = units.len() as u32 * 2;
            let cue_len = 48 + comment_len + padding;
            let mut bytes = b"PCO2".to_vec();
            for value in [20_u32, 20 + cue_len, 0] {
                bytes.extend(value.to_be_bytes());
            }
            bytes.extend([0, 1, 0, 0]); // count, reserved
            bytes.extend(b"PCP2");
            for value in [
                16_u32,
                cue_len,
                0,
                0x010003e8,
                0,
                u32::MAX,
                0,
                0,
                0,
                comment_len,
            ] {
                bytes.extend(value.to_be_bytes());
            }
            for unit in &units {
                bytes.extend((*unit as u16).to_be_bytes());
            }
            bytes.extend([1, 2, 3, 4]); // color index and RGB
            bytes.extend(vec![0xab; padding as usize]);
            let mut section = Section::read_be(&mut Cursor::new(&bytes)).unwrap();
            let mut out = Cursor::new(Vec::new());
            section.write_be(&mut out).unwrap();
            assert_eq!(out.into_inner(), bytes);
            if let Content::ExtendedCueList(list) = &mut section.content {
                assert_eq!(list.cues[0].hot_cue_color_rgb, (2, 3, 4));
                list.cues[0].comment = NullWideString::from("New \u{1f3b5}");
            }
            let mut out = Cursor::new(Vec::new());
            section.write_be(&mut out).unwrap();
            let changed = out.into_inner();
            assert_eq!(
                u32::from_be_bytes(changed[8..12].try_into().unwrap()) as usize,
                changed.len()
            );
            let reparsed = Section::read_be(&mut Cursor::new(&changed)).unwrap();
            if let Content::ExtendedCueList(list) = reparsed.content {
                assert_eq!(list.cues[0].comment.to_string(), "New \u{1f3b5}");
                assert_eq!(list.cues[0].hot_cue_color_rgb, (2, 3, 4));
            } else {
                panic!("Expected extended cues");
            }
        }
    }
}
