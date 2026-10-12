// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Public section removal, duplicate handling, and persistence.

use rbox_anlz::{
    Anlz, AnlzError, AnlzTag, BeatGrid, Content, CueList, CueListType, ExtendedCueList, Section,
};
use rbox_testing::data::create_temp_dir;

#[test]
fn indexed_and_tag_removal_survive_copy_and_repeated_saves() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    let path = dir.path().join("ANLZ0000.DAT");
    let mut file = Anlz::new_dat(&path)?;
    file.set_path("/track.flac")?;
    for _ in 0..3 {
        file.push_section(Section {
            tag: AnlzTag::BeatGrid,
            content: Content::BeatGrid(BeatGrid::new(vec![])),
        })?;
    }
    file.dump()?;
    drop(file);
    let original = std::fs::read(&path)?;
    let mut file = Anlz::load(&path)?;
    assert!(matches!(
        file.remove_section(4),
        Err(AnlzError::SectionIndexOutOfBounds { .. })
    ));
    assert_eq!(file.section_count(), 4);
    assert!(file.remove_beat_grid());
    file.remove_section(1)?;
    assert_eq!(file.section_count(), 2);
    let copy = dir.path().join("copy.DAT");
    file.dump_copy(&copy)?;
    assert_eq!(Anlz::load(&copy)?.section_count(), 2);
    assert_eq!(std::fs::read(&path)?, original);
    file.dump()?;
    assert_eq!(file.remove_sections_by_tag(AnlzTag::BeatGrid), 1);
    assert_eq!(file.remove_sections_by_tag(AnlzTag::BeatGrid), 0);
    assert!(!file.remove_beat_grid());
    assert!(file.remove_path());
    file.dump()?;
    assert_eq!(std::fs::metadata(&path)?.len(), 28);
    assert_eq!(Anlz::load(&path)?.section_count(), 0);
    file.set_beat_grid(BeatGrid::new(vec![]))?;
    file.dump()?;
    assert_eq!(Anlz::load(&path)?.section_count(), 1);
    Ok(())
}

#[test]
fn cue_removal_selects_only_the_first_matching_subtype() -> anyhow::Result<()> {
    let dir = create_temp_dir();
    let path = dir.path().join("ANLZ0000.EXT");
    let mut file = Anlz::new_ext(&path)?;
    for kind in [
        CueListType::HotCues,
        CueListType::MemoryCues,
        CueListType::HotCues,
    ] {
        file.push_section(Section {
            tag: AnlzTag::CueList,
            content: Content::CueList(CueList::new(kind.clone(), vec![])),
        })?;
        file.push_section(Section {
            tag: AnlzTag::ExtendedCueList,
            content: Content::ExtendedCueList(ExtendedCueList::new(kind, vec![])),
        })?;
    }
    file.dump()?;
    drop(file);
    let mut file = Anlz::load(&path)?;
    assert!(file.remove_hot_cues()?);
    assert!(file.get_hot_cues()?.is_some()); // duplicate remains
    assert!(file.get_memory_cues()?.is_some());
    assert!(file.remove_extended_memory_cues()?);
    assert!(!file.remove_extended_memory_cues()?);
    assert!(file.remove_extended_hot_cues()?);
    assert!(file.get_extended_hot_cues()?.is_some());
    assert!(file.remove_memory_cues()?);
    assert!(!file.remove_memory_cues()?);
    file.dump()?;
    drop(file);
    let mut file = Anlz::load(&path)?;
    assert_eq!(file.section_count(), 2);
    assert!(file.remove_cue_list());
    assert!(file.remove_extended_cue_list());
    assert_eq!(file.section_count(), 0);
    Ok(())
}
