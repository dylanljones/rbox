// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use binrw::{BinRead, BinWrite, NullWideString};
use rbox::anlz::anlz::{
    AnlzData, Beat, Content, Section, TinyWaveformColumn, WaveformColorDetailColumn,
};
use rbox::anlz::{Anlz, AnlzString, AnlzTag, ColorIndex};
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
                        cue.comment = AnlzString::from("Edited \u{1f3b5} cue");
                        let copy = cue.clone();
                        g.cues.push(copy);
                    }
                }
                Content::Path(p) => {
                    p.path = NullWideString::from("/Music/\u{1f3b5}.flac");
                }
                Content::VBR(v) => v.index[0] = v.index[0].wrapping_add(1),
                Content::VocalDetection(v) => v.confidence.push(1),
                Content::SeekIndex(s) => {
                    s.entries.pop();
                }
                Content::Waveform3BandCalibration(w) => w.low_gain = w.low_gain.wrapping_add(1),
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
        assert_eq!(column.height(), byte & 15);
        assert_eq!(column.unused(), byte >> 4);
    }
    for word in 0..=u16::MAX {
        let column =
            WaveformColorDetailColumn::read_be(&mut Cursor::new(word.to_be_bytes())).unwrap();
        assert_eq!(column.red(), ((word >> 13) & 7) as u8);
        assert_eq!(column.green(), ((word >> 10) & 7) as u8);
        assert_eq!(column.blue(), ((word >> 7) & 7) as u8);
        assert_eq!(column.height(), ((word >> 2) & 31) as u8);
        assert_eq!(column.low_bits(), (word & 3) as u8);
        let fine = (((word & 1) << 1) | ((word & 2) >> 1)) as u8;
        assert_eq!(column.fine_height_substep(), fine);
        assert_eq!(column.full_height(), (((word >> 2) & 31) as u8) * 4 + fine);
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
                list.cues[0].comment = AnlzString::from("New \u{1f3b5}");
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

fn raw_tag(magic: &[u8; 4], header_size: u32, body: &[u8]) -> Vec<u8> {
    let mut bytes = magic.to_vec();
    bytes.extend(header_size.to_be_bytes());
    bytes.extend((12 + body.len() as u32).to_be_bytes());
    bytes.extend(body);
    bytes
}

fn raw_file(tags: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = b"PMAI".to_vec();
    bytes.extend(28u32.to_be_bytes());
    bytes.extend((28 + tags.iter().map(Vec::len).sum::<usize>() as u32).to_be_bytes());
    bytes.extend([0; 16]);
    for tag in tags {
        bytes.extend(tag);
    }
    bytes
}

fn pvdi(confidence: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    for value in [0x400_u32, 0x56220001, confidence.len() as u32] {
        body.extend(value.to_be_bytes());
    }
    body.extend(confidence);
    raw_tag(b"PVDI", 24, &body)
}

fn pvb2(entries: &[(u64, u64, u32)]) -> Vec<u8> {
    let mut body = 7u32.to_be_bytes().to_vec();
    body.extend((u64::from(u32::MAX) + 1025).to_be_bytes());
    body.extend((entries.len() as u32).to_be_bytes());
    body.extend(20u32.to_be_bytes());
    for (sample, offset, samples) in entries {
        body.extend(sample.to_be_bytes());
        body.extend(offset.to_be_bytes());
        body.extend(samples.to_be_bytes());
    }
    raw_tag(b"PVB2", 32, &body)
}

#[test]
fn vocal_detection_and_seek_index_decode_and_resize() {
    for len in [0, 20, 3731] {
        let confidence: Vec<_> = (0..len).map(|i| (i % 256) as u8).collect();
        let raw = pvdi(&confidence);
        let mut section = Section::read_be(&mut Cursor::new(&raw)).unwrap();
        let mut encoded = Cursor::new(Vec::new());
        section.write_be(&mut encoded).unwrap();
        assert_eq!(encoded.into_inner(), raw);
        if let Content::VocalDetection(data) = &mut section.content {
            assert_eq!(data.confidence, confidence);
            assert_eq!((data.u1, data.u2), (0x400, 0x56220001));
            data.confidence.extend([1, 2, 255]);
        } else {
            panic!("Expected vocal detection");
        }
        let mut encoded = Cursor::new(Vec::new());
        section.write_be(&mut encoded).unwrap();
        let mut expected = confidence;
        expected.extend([1, 2, 255]);
        assert_eq!(encoded.into_inner(), pvdi(&expected));
    }

    let large = (u64::from(u32::MAX) + 2, (1u64 << 33) + 2, 1024);
    for entries in [vec![], vec![(0, 128, 4096), (4096, 8192, 4608), large]] {
        let raw = pvb2(&entries);
        let mut section = Section::read_be(&mut Cursor::new(&raw)).unwrap();
        let mut encoded = Cursor::new(Vec::new());
        section.write_be(&mut encoded).unwrap();
        assert_eq!(encoded.into_inner(), raw);
        if let Content::SeekIndex(data) = &mut section.content {
            assert_eq!(data.u1, 7);
            assert_eq!(data.total_samples, u64::from(u32::MAX) + 1025);
            let actual: Vec<_> = data
                .entries
                .iter()
                .map(|e| (e.sample, e.offset, e.frame_samples))
                .collect();
            assert_eq!(actual, entries);
            data.entries.push(rbox::anlz::anlz::SeekPoint {
                sample: large.0,
                offset: large.1,
                frame_samples: large.2,
            });
        } else {
            panic!("Expected seek index");
        }
        let mut encoded = Cursor::new(Vec::new());
        section.write_be(&mut encoded).unwrap();
        let mut expected = entries;
        expected.push(large);
        assert_eq!(encoded.into_inner(), pvb2(&expected));
    }
}

#[test]
fn new_tag_getters_and_structured_vbr_preserve_values() {
    let mut vbr_body = 0x12345678_u32.to_be_bytes().to_vec();
    let index: Vec<_> = (0..400u32).map(|i| 0x01020300 + i).collect();
    for value in &index {
        vbr_body.extend(value.to_be_bytes());
    }
    vbr_body.extend(0x87654321_u32.to_be_bytes());
    let vbr = raw_tag(b"PVBR", 16, &vbr_body);
    let calibration = raw_tag(b"PWVC", 14, &[0, 0, 0, 1, 1, 0, 0xff, 0xff]);
    let tags = vec![vbr.clone(), pvdi(&[0, 1, 255]), pvb2(&[]), calibration];
    let bytes = raw_file(&tags);
    let data = AnlzData::read(&mut Cursor::new(&bytes)).unwrap();
    assert_eq!(encode(&data), bytes);
    let tmp = common::create_temp_dir();
    let path = tmp.path().join("ANLZ0000.DAT");
    std::fs::write(&path, &bytes).unwrap();
    let mut file = Anlz::load(path).unwrap();
    let v = file.get_vbr().unwrap();
    assert_eq!(v.index.as_slice(), index);
    assert_eq!((v.u1, v.u2), (0x12345678, 0x87654321));
    v.index[0] = 42;
    let mut expected_raw = vbr[16..].to_vec();
    expected_raw[..4].copy_from_slice(&42u32.to_be_bytes());
    assert_eq!(file.get_vbr_data().unwrap(), expected_raw);
    file.get_vocal_detection().unwrap().confidence.push(42);
    file.get_seek_index()
        .unwrap()
        .entries
        .push(rbox::anlz::anlz::SeekPoint {
            sample: 1u64 << 40,
            offset: 1u64 << 48,
            frame_samples: 4096,
        });
    let c = file.get_waveform_3band_calibration().unwrap();
    // assert_eq!(c.reserved, 0x1234);
    assert_eq!([c.low_gain, c.mid_gain, c.high_gain], [1, 256, 65535]);
    c.mid_gain = 512;
    for tag in [
        AnlzTag::VocalDetection,
        AnlzTag::SeekIndex,
        AnlzTag::Waveform3BandCalibration,
    ] {
        assert!(file.contains(tag.clone()));
        assert_eq!(AnlzTag::from(tag.to_string()), tag);
    }
    let output = tmp.path().join("OUT.DAT");
    file.dump_copy(&output).unwrap();
    check_lengths(&std::fs::read(&output).unwrap());
    let mut copy = Anlz::load(output).unwrap();
    assert_eq!(copy.get_vbr().unwrap().index[0], 42);
    assert_eq!(
        copy.get_vocal_detection().unwrap().confidence,
        [0, 1, 255, 42]
    );
    assert_eq!(copy.get_seek_index().unwrap().entries[0].offset, 1u64 << 48);
    let c = copy.get_waveform_3band_calibration().unwrap();
    assert_eq!([c.low_gain, c.mid_gain, c.high_gain], [1, 512, 65535]);
}

#[test]
fn new_tags_reject_inconsistent_lengths() {
    let valid = [
        pvdi(&[1, 2, 3]),
        pvb2(&[(0, 128, 4096)]),
        raw_tag(b"PWVC", 14, &[0; 8]),
        raw_tag(b"PVBR", 16, &[0; 1608]),
    ];
    for bytes in valid {
        for field in [4, 8] {
            let mut bad = bytes.clone();
            bad[field..field + 4].copy_from_slice(&1u32.to_be_bytes());
            assert!(Section::read_be(&mut Cursor::new(bad)).is_err());
        }
        let mut truncated = bytes;
        truncated.pop();
        assert!(Section::read_be(&mut Cursor::new(truncated)).is_err());
    }
    let mut confidence = pvdi(&[1, 2, 3]);
    confidence[20..24].copy_from_slice(&4u32.to_be_bytes());
    assert!(Section::read_be(&mut Cursor::new(confidence)).is_err());
    for (offset, value) in [(24, u32::MAX), (28, 19)] {
        let mut seek = pvb2(&[(0, 128, 4096)]);
        seek[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
        assert!(Section::read_be(&mut Cursor::new(seek)).is_err());
    }
}

#[test]
fn anlz_strings_preserve_encoding_and_validate_input() {
    let text = AnlzString::from("A\u{1f3b5}");
    assert_eq!(text.as_str(), "A\u{1f3b5}");
    assert_eq!(text.payload_byte_len(), 8); // three UTF-16 units and one NUL
    let mut encoded = Cursor::new(Vec::new());
    text.write_be(&mut encoded).unwrap();
    assert_eq!(
        encoded.into_inner(),
        [0, 0, 0, 8, 0, 65, 0xd8, 0x3c, 0xdf, 0xb5, 0, 0]
    );

    for units in [
        vec![],
        vec![0],
        vec![0, 0],
        vec![65],
        vec![65, 0, 0],
        vec![0xd83c, 0xdfb5, 0],
        vec![65, 0, 66, 0],
    ] {
        let mut bytes = (units.len() as u32 * 2).to_be_bytes().to_vec();
        for unit in units {
            bytes.extend((unit as u16).to_be_bytes());
        }
        let parsed = AnlzString::read_be(&mut Cursor::new(&bytes)).unwrap();
        let mut output = Cursor::new(Vec::new());
        parsed.write_be(&mut output).unwrap();
        assert_eq!(output.into_inner(), bytes);
    }
    for bytes in [
        vec![0, 0, 0, 3, 0, 65, 0],      // odd byte count
        vec![0, 0, 0, 4, 0xd8, 0, 0, 0], // lone surrogate
        vec![0, 0, 0, 4, 0, 65],         // truncated
        vec![0xff, 0xff, 0xff, 0xfe],    // no huge up-front allocation
    ] {
        assert!(AnlzString::read_be(&mut Cursor::new(bytes)).is_err());
    }
    let mut empty = AnlzString::default();
    assert_eq!(empty.payload_byte_len(), 0);
    empty.set("Hi");
    assert_eq!(empty.payload_byte_len(), 6);

    // An overlong comment must not read into the RGB or following entry.
    let mut body = Vec::new();
    for value in [16u32, 48, 0, 0x010003e8, 0, u32::MAX, 0, 0, 0, 2] {
        body.extend(value.to_be_bytes());
    }
    body.extend([0, 0, 0, 0]);
    let mut cue = b"PCP2".to_vec();
    cue.extend(body);
    let mut reader = Cursor::new(cue);
    assert!(rbox::anlz::anlz::ExtendedCue::read_be(&mut reader).is_err());
}

#[test]
fn three_band_columns_use_low_mid_high_disk_order() {
    use rbox::anlz::anlz::Waveform3BandColumn;
    // PWV7 samples from rekordcrate's descending 20 kHz -> 20 Hz sweep fixture,
    // commit 14d54eded7b0d2f6fe67c7871533dca35a73e5fa, at 5%, 15%, and 25%.
    for (bytes, dominant) in [([0u8, 2, 118], 2), ([8, 78, 1], 1), ([81, 17, 0], 0)] {
        let column = Waveform3BandColumn::read_be(&mut Cursor::new(bytes)).unwrap();
        let bands = [column.low(), column.mid(), column.high()];
        assert_eq!(bands, bytes);
        assert!(bands[dominant] as u16 * 2 > bands.iter().map(|x| u16::from(*x)).sum::<u16>());
        let mut output = Cursor::new(Vec::new());
        column.write_be(&mut output).unwrap();
        assert_eq!(output.into_inner(), bytes);
    }
}

#[test]
fn named_cue_colors_retain_unknown_values() {
    for value in 0..=u8::MAX {
        let color = ColorIndex::read_be(&mut Cursor::new([value])).unwrap();
        assert_eq!(u8::from(color), value);
        assert_eq!(color, ColorIndex::from(value));
        let mut output = Cursor::new(Vec::new());
        color.write_be(&mut output).unwrap();
        assert_eq!(output.into_inner(), [value]);
    }
    assert_eq!(ColorIndex::from(0), ColorIndex::None);
    assert_eq!(ColorIndex::from(1), ColorIndex::Pink);
    assert_eq!(ColorIndex::from(8), ColorIndex::Purple);
    assert_eq!(ColorIndex::from(99), ColorIndex::Other(99));
}
