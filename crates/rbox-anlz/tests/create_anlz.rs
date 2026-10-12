// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Creating files and replacing sections through the public API.

use binrw::BinRead;
use rbox_anlz::{
    Anlz, AnlzData, AnlzError, AnlzTag, Beat, BeatGrid, Content, CueList, CueListType,
    ExtendedCueList, Section, Waveform3BandColumn, Waveform3BandPreview, WaveformColorDetail,
    WaveformColorDetailColumn,
};
use rbox_testing::data::create_temp_dir;
use std::io::Cursor;

#[test]
fn empty_file_can_be_saved_and_extended_later() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    let output = dir.path().join("ANLZ0000.2EX");
    let mut file = Anlz::new_2ex(&output)?;
    file.dump()?;
    assert_eq!(std::fs::metadata(&output)?.len(), 28);
    file.set_waveform_3band_preview(Waveform3BandPreview::new(vec![]))?;
    file.dump()?;
    assert_eq!(Anlz::load(&output)?.section_count(), 1);
    Ok(())
}

#[test]
fn new_files_save_copy_and_then_adopt_their_output() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    let base = dir.path().join("ANLZ0000");
    for (extension, mut file) in [
        ("DAT", Anlz::new_dat(&base)?),
        ("EXT", Anlz::new_ext(&base)?),
        ("2EX", Anlz::new_2ex(&base)?),
    ] {
        let output = base.with_extension(extension);
        assert!(!output.exists());
        assert_eq!(file.section_count(), 0);
        file.set_path(r"C:\Music\track.flac")?;
        let copy = dir.path().join(format!("copy.{extension}"));
        file.dump_copy(&copy)?;
        assert!(!output.exists());
        file.dump()?;
        assert_eq!(std::fs::read(&copy)?, std::fs::read(&output)?);
        file.set_path("/Music/renamed.flac")?;
        file.dump()?;
        let mut reloaded = Anlz::load(&output)?;
        assert_eq!(reloaded.get_path()?.as_deref(), Some("/Music/renamed.flac"));
        let bytes = std::fs::read(&output)?;
        let data = AnlzData::read_be(&mut Cursor::new(&bytes))?;
        assert_eq!(data.header.total_size as usize, bytes.len());
        assert_eq!(data.header.size, 28);
        assert_eq!(
            data.header_data,
            [0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
        );
    }
    Ok(())
}

#[test]
fn section_setters_preserve_order_duplicates_and_cue_subtypes() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    let output = dir.path().join("ANLZ0000.DAT");
    let mut file = Anlz::new_dat(&output)?;
    file.set_memory_cues(CueList::new(CueListType::MemoryCues, vec![]))?;
    file.set_hot_cues(CueList::new(CueListType::HotCues, vec![]))?;
    file.push_section(Section {
        tag: AnlzTag::CueList,
        content: Content::CueList(CueList::new(CueListType::HotCues, vec![])),
    })?;
    // Preserve an unknown section's complete wire representation.
    let raw = [
        b"ZZZZ".as_slice(),
        &12u32.to_be_bytes(),
        &15u32.to_be_bytes(),
        &[9, 8, 7],
    ]
    .concat();
    file.push_section(Section::read_be(&mut Cursor::new(&raw))?)?;
    file.set_path("/track.flac")?; // inserts PPTH first
    file.dump()?;
    drop(file);
    let mut file = Anlz::load(&output)?;
    let mut hot = CueList::new(CueListType::HotCues, vec![]);
    hot.memory_count = 17; // distinguish which duplicate was replaced
    file.set_cue_list(hot)?;
    assert_eq!(file.section_count(), 5);
    assert_eq!(file.get_memory_cues()?.unwrap().memory_count, -1);
    assert_eq!(file.get_hot_cues()?.unwrap().memory_count, 17);
    match &file.get_section(2)?.unwrap().content {
        Content::CueList(value) => {
            assert_eq!(value.list_type, CueListType::HotCues);
            assert_eq!(value.memory_count, -1);
        }
        _ => panic!("expected second hot-cue list"),
    }
    file.dump()?;
    assert!(std::fs::read(&output)?.ends_with(&raw));
    Ok(())
}

#[test]
fn authored_data_and_waveforms_roundtrip() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    let output = dir.path().join("ANLZ0000.EXT");
    let mut file = Anlz::new_ext(&output)?;
    file.set_beat_grid(BeatGrid::new(vec![Beat {
        beat_number: 1,
        tempo: 12800,
        time: 500,
    }]))?;
    file.set_extended_memory_cues(ExtendedCueList::new(CueListType::MemoryCues, vec![]))?;
    file.set_extended_hot_cues(ExtendedCueList::new(CueListType::HotCues, vec![]))?;
    file.set_waveform_color_detail(WaveformColorDetail::new(vec![
        WaveformColorDetailColumn::new(),
    ]))?;
    file.set_waveform_3band_preview(Waveform3BandPreview::new(vec![Waveform3BandColumn::new()]))?;
    file.dump()?;
    let mut reloaded = Anlz::load(&output)?;
    assert_eq!(reloaded.get_beat_grid()?.unwrap().beats[0].tempo, 12800);
    assert_eq!(reloaded.get_beat_grid()?.unwrap().header_flags, 0x0008_0000);
    assert!(reloaded.get_extended_hot_cues()?.is_some());
    assert!(reloaded.get_extended_memory_cues()?.is_some());
    assert_eq!(reloaded.get_waveform_color_detail()?.unwrap().data.len(), 1);
    assert_eq!(
        reloaded.get_waveform_3band_preview()?.unwrap().data.len(),
        1
    );
    Ok(())
}

#[test]
fn invalid_edits_leave_existing_sections_and_destination_intact() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    let output = dir.path().join("ANLZ0000.DAT");
    std::fs::write(&output, b"existing destination")?;
    let mut file = Anlz::new_dat(&output)?;
    file.set_path("/track.flac")?;
    let bad = Section {
        tag: AnlzTag::Path,
        content: Content::BeatGrid(BeatGrid::new(vec![])),
    };
    assert!(matches!(
        file.set_section(bad),
        Err(AnlzError::TagContentMismatch { .. })
    ));
    let valid = Section {
        tag: AnlzTag::BeatGrid,
        content: Content::BeatGrid(BeatGrid::new(vec![])),
    };
    assert!(matches!(
        file.insert_section(2, valid.clone()),
        Err(AnlzError::SectionIndexOutOfBounds { .. })
    ));
    assert!(matches!(
        file.set_section_at(1, valid),
        Err(AnlzError::SectionIndexOutOfBounds { .. })
    ));
    assert_eq!(file.section_count(), 1);
    assert_eq!(file.get_path()?.as_deref(), Some("/track.flac"));
    assert_eq!(std::fs::read(output)?, b"existing destination");
    Ok(())
}
