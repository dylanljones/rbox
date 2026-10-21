// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Verbatim section transfer, lazy decoding, and validation through the public API.

use rbox_anlz::{Anlz, AnlzTag};
use rbox_testing::data::create_temp_dir;

fn section(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = tag.to_vec();
    bytes.extend(12u32.to_be_bytes());
    bytes.extend((12 + payload.len() as u32).to_be_bytes());
    bytes.extend(payload);
    bytes
}

#[test]
fn raw_setter_uses_destination_file_order() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    for (extension, tags) in [
        (
            "DAT",
            vec![*b"PPTH", *b"PVBR", *b"PQTZ", *b"PWAV", *b"PWV2", *b"PCOB"],
        ),
        (
            "EXT",
            vec![
                *b"PPTH", *b"PWV3", *b"PCOB", *b"PCO2", *b"PQT2", *b"PWV5", *b"PWV4", *b"PWV7",
                *b"PWV6", *b"PVB2",
            ],
        ),
        ("2EX", vec![*b"PPTH", *b"PWV7", *b"PWV6", *b"PWVC"]),
    ] {
        let path = dir.path().join(format!("ordered.{extension}"));
        let mut file = match extension {
            "DAT" => Anlz::new_dat(&path)?,
            "EXT" => Anlz::new_ext(&path)?,
            _ => Anlz::new_2ex(&path)?,
        };
        // Deliberately invalid typed payloads ensure ordering does not decode them.
        for tag in tags.iter().rev() {
            file.set_section_bytes(section(tag, &[]))?;
        }
        for (index, tag) in tags.iter().enumerate() {
            assert_eq!(file.get_section_bytes(index)?, Some(section(tag, &[])));
        }
        let replacement = section(&tags[1], &[42]);
        file.set_section_bytes(replacement.clone())?;
        assert_eq!(file.section_count(), tags.len());
        assert_eq!(file.get_section_bytes(1)?, Some(replacement));
        let unknown = section(b"ZZZZ", &[1]);
        file.set_section_bytes(unknown.clone())?;
        assert_eq!(file.get_section_bytes(tags.len())?, Some(unknown));
        file.dump()?;
    }
    Ok(())
}

#[test]
fn copies_unknown_and_unparseable_sections_verbatim() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    let mut source = Anlz::new_ext(dir.path().join("source"))?;
    let unknown = section(b"ZZZZ", &[0, 1, 255, 3]);
    let malformed = section(b"PQTZ", &[]);
    source.insert_section_bytes(0, unknown.clone())?;
    source.insert_section_bytes(1, malformed.clone())?;
    source.insert_section_bytes(2, unknown.clone())?;
    source.dump()?;
    let mut source = Anlz::load(dir.path().join("source.EXT"))?;
    let mut destination = Anlz::new_ext(dir.path().join("destination"))?;
    for i in 0..source.section_count() {
        destination.insert_section_bytes(i, source.get_section_bytes(i)?.unwrap())?;
    }
    assert!(destination.get_section(0)?.is_some());
    assert!(destination.get_section(1).is_err());
    assert_eq!(destination.get_section_bytes(1)?, Some(malformed.clone()));
    let copy = dir.path().join("copy.EXT");
    destination.dump_copy(&copy)?;
    destination.dump()?;
    destination.dump()?;
    assert_eq!(
        std::fs::read(&copy)?,
        std::fs::read(dir.path().join("destination.EXT"))?
    );
    let saved = std::fs::read(&copy)?;
    assert_eq!(
        &saved[28..],
        [unknown.clone(), malformed, unknown.clone()].concat()
    );
    destination.set_section_bytes(unknown.clone())?;
    assert_eq!(destination.get_section_bytes(0)?, Some(unknown));
    assert!(destination.get_section_bytes(99)?.is_none());
    assert!(destination
        .get_section_bytes_by_tag(AnlzTag::Path)?
        .is_none());
    Ok(())
}

#[test]
fn typed_edits_override_raw_bytes_and_survive_saves() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    let mut source = Anlz::new_dat(dir.path().join("source"))?;
    source.set_path("C:/old.mp3")?;
    let original = source.get_section_bytes_by_tag(AnlzTag::Path)?.unwrap();
    let mut destination = Anlz::new_dat(dir.path().join("destination"))?;
    destination.insert_section_bytes(0, original.clone())?;
    assert!(destination.get_path()?.is_some());
    assert_eq!(destination.get_section_bytes(0)?, Some(original.clone()));
    destination.set_path("C:/much-longer-new-name.mp3")?;
    let edited = destination.get_section_bytes(0)?.unwrap();
    assert_ne!(edited, original);
    assert_eq!(
        u32::from_be_bytes(edited[8..12].try_into()?) as usize,
        edited.len()
    );
    destination.dump()?;
    assert_eq!(destination.get_section_bytes(0)?, Some(edited.clone()));
    destination.dump()?;
    assert_eq!(
        &std::fs::read(dir.path().join("destination.DAT"))?[28..],
        edited
    );
    destination.set_section_bytes(original.clone())?;
    assert_eq!(destination.get_section_bytes(0)?, Some(original));
    assert!(destination.get_path()?.is_some());
    Ok(())
}

#[test]
fn rejects_invalid_outer_headers_without_changing_sections() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    let mut file = Anlz::new_dat(dir.path().join("destination"))?;
    let valid = section(b"ZZZZ", &[1, 2, 3]);
    file.insert_section_bytes(0, valid.clone())?;
    let mut short_header = valid.clone();
    short_header[4..8].copy_from_slice(&11u32.to_be_bytes());
    let mut oversized_header = valid.clone();
    oversized_header[4..8].copy_from_slice(&16u32.to_be_bytes());
    let mut trailing = valid.clone();
    trailing.push(0);
    for invalid in [
        vec![],
        valid[..11].to_vec(),
        valid[..14].to_vec(),
        short_header,
        oversized_header,
        trailing,
        section(b"PMAI", &[]),
    ] {
        assert!(file.set_section_bytes(invalid.clone()).is_err());
        assert!(file.insert_section_bytes(0, invalid).is_err());
        assert_eq!(file.section_count(), 1);
        assert_eq!(file.get_section_bytes(0)?, Some(valid.clone()));
    }

    assert!(file.insert_section_bytes(2, valid).is_err());
    Ok(())
}
