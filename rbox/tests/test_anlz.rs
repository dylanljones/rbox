// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use rbox::anlz::{Anlz, AnlzPaths, AnlzTag};
use std::path::PathBuf;

mod common;

fn get_anlz_paths(root: &PathBuf) -> AnlzPaths {
    AnlzPaths {
        dat: root.join("ANLZ0000.DAT"),
        ext: Some(root.join("ANLZ0000.EXT")),
        ex2: Some(root.join("ANLZ0000.2EX")),
    }
}

struct AnlzFiles {
    dat: Anlz,
    ext: Anlz,
    ex2: Anlz,
}

fn get_anlz_files(root: &PathBuf) -> AnlzFiles {
    let paths = get_anlz_paths(root);
    AnlzFiles {
        dat: Anlz::load(paths.dat.clone()).unwrap(),
        ext: Anlz::load(paths.ext.unwrap().clone()).unwrap(),
        ex2: Anlz::load(paths.ex2.unwrap().clone()).unwrap(),
    }
}

#[test]
fn test_read_anlz() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let paths = get_anlz_paths(&tmp_dir.path().to_path_buf());
    let _ = Anlz::load(paths.dat);
    let _ = Anlz::load(paths.ext.unwrap());
    let _ = Anlz::load(paths.ex2.unwrap());
    Ok(())
}

fn test_write(file: &PathBuf) -> anyhow::Result<()> {
    // Load file
    let mut anlz = Anlz::load(file.clone())?;
    // Write new file
    let out_file = file
        .with_file_name("ANLZ_OUT")
        .with_extension(file.extension().unwrap());
    anlz.dump_copy(out_file.clone())?;
    // Check contents
    let contents1 = std::fs::read(&file)?;
    let contents2 = std::fs::read(&out_file)?;
    assert_eq!(contents1, contents2);
    // Remove file
    std::fs::remove_file(out_file)?;
    Ok(())
}

#[test]
fn test_write_anlz() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let paths = get_anlz_paths(&tmp_dir.path().to_path_buf());
    test_write(&paths.dat)?;
    test_write(&paths.ext.unwrap())?;
    test_write(&paths.ex2.unwrap())?;
    Ok(())
}

#[test]
fn test_contains() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let files = get_anlz_files(&tmp_dir.path().to_path_buf());
    assert!(files.dat.contains(AnlzTag::Path));
    assert!(files.ext.contains(AnlzTag::Path));
    assert!(files.ex2.contains(AnlzTag::Path));

    assert!(files.dat.contains(AnlzTag::BeatGrid));
    assert!(!files.ext.contains(AnlzTag::BeatGrid));
    assert!(!files.ex2.contains(AnlzTag::BeatGrid));
    Ok(())
}

#[test]
fn test_get_tags() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let _ = files.dat.get_tags()?;
    let _ = files.ext.get_tags()?;
    let _ = files.ex2.get_tags()?;
    Ok(())
}

#[test]
fn test_get_beat_grid() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_beat_grid();
    let item2 = files.ext.get_beat_grid();
    let item3 = files.ex2.get_beat_grid();
    assert!(item1.is_some());
    assert!(item2.is_none());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_extended_beat_grid() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_extended_beat_grid();
    let item2 = files.ext.get_extended_beat_grid();
    let item3 = files.ex2.get_extended_beat_grid();
    assert!(item1.is_none());
    assert!(item2.is_some());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_hot_cues() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_hot_cues();
    let item2 = files.ext.get_hot_cues();
    let item3 = files.ex2.get_hot_cues();
    assert!(item1.is_some());
    assert!(item2.is_some());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_memory_cues() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_memory_cues();
    let item2 = files.ext.get_memory_cues();
    let item3 = files.ex2.get_memory_cues();
    assert!(item1.is_some());
    assert!(item2.is_some());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_extended_hot_cues() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_extended_hot_cues();
    let item2 = files.ext.get_extended_hot_cues();
    let item3 = files.ex2.get_extended_hot_cues();
    assert!(item1.is_none());
    assert!(item2.is_some());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_extended_memory_cues() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_extended_memory_cues();
    let item2 = files.ext.get_extended_memory_cues();
    let item3 = files.ex2.get_extended_memory_cues();
    assert!(item1.is_none());
    assert!(item2.is_some());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_path() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_path();
    let item2 = files.ext.get_path();
    let item3 = files.ex2.get_path();
    assert!(item1.is_some());
    assert!(item2.is_some());
    assert!(item3.is_some());
    Ok(())
}

fn update_anlz_path(file: &PathBuf, new_path: &str) -> anyhow::Result<()> {
    // Load file
    let mut anlz = Anlz::load(file.clone())?;
    // Update path
    anlz.set_path(new_path)?;
    // Write new file
    let out_file = file
        .with_file_name("ANLZ_OUT")
        .with_extension(file.extension().unwrap());
    anlz.dump_copy(out_file.clone())?;

    // Try to read the file
    let _ = Anlz::load(out_file.clone())?;
    // Remove file
    std::fs::remove_file(out_file)?;
    Ok(())
}

#[test]
fn test_set_path() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let paths = get_anlz_paths(&tmp_dir.path().to_path_buf());

    let new_path = "";
    update_anlz_path(&paths.dat, new_path)?;
    update_anlz_path(&paths.ext.as_ref().unwrap(), new_path)?;
    update_anlz_path(&paths.ex2.as_ref().unwrap(), new_path)?;

    let new_path = "/New/Path";
    update_anlz_path(&paths.dat, new_path)?;
    update_anlz_path(&paths.ext.as_ref().unwrap(), new_path)?;
    update_anlz_path(&paths.ex2.as_ref().unwrap(), new_path)?;
    Ok(())
}

#[test]
fn test_get_vbr() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_vbr_data();
    let item2 = files.ext.get_vbr_data();
    let item3 = files.ex2.get_vbr_data();
    assert!(item1.is_some());
    assert!(item2.is_none());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_tiny_waveform_preview() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_tiny_waveform_preview();
    let item2 = files.ext.get_tiny_waveform_preview();
    let item3 = files.ex2.get_tiny_waveform_preview();
    assert!(item1.is_some());
    assert!(item2.is_none());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_waveform_preview() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_waveform_preview();
    let item2 = files.ext.get_waveform_preview();
    let item3 = files.ex2.get_waveform_preview();
    assert!(item1.is_some());
    assert!(item2.is_none());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_waveform_detail() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_waveform_detail();
    let item2 = files.ext.get_waveform_detail();
    let item3 = files.ex2.get_waveform_detail();
    assert!(item1.is_none());
    assert!(item2.is_some());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_waveform_color_preview() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_waveform_color_preview();
    let item2 = files.ext.get_waveform_color_preview();
    let item3 = files.ex2.get_waveform_color_preview();
    assert!(item1.is_none());
    assert!(item2.is_some());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_waveform_color_detail() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_waveform_color_detail();
    let item2 = files.ext.get_waveform_color_detail();
    let item3 = files.ex2.get_waveform_color_detail();
    assert!(item1.is_none());
    assert!(item2.is_some());
    assert!(item3.is_none());
    Ok(())
}

#[test]
fn test_get_waveform_3band_preview() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_waveform_3band_preview();
    let item2 = files.ext.get_waveform_3band_preview();
    let item3 = files.ex2.get_waveform_3band_preview();
    assert!(item1.is_none());
    assert!(item2.is_none());
    assert!(item3.is_some());
    Ok(())
}

#[test]
fn test_get_waveform_3band_detail() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_waveform_3band_detail();
    let item2 = files.ext.get_waveform_3band_detail();
    let item3 = files.ex2.get_waveform_3band_detail();
    assert!(item1.is_none());
    assert!(item2.is_none());
    assert!(item3.is_some());
    Ok(())
}

#[test]
fn test_get_song_structure() -> anyhow::Result<()> {
    let tmp_dir = common::setup_anlz_files();
    let mut files = get_anlz_files(&tmp_dir.path().to_path_buf());
    let item1 = files.dat.get_song_structure();
    let item2 = files.ext.get_song_structure();
    let item3 = files.ex2.get_song_structure();
    assert!(item1.is_none());
    assert!(item2.is_some());
    assert!(item3.is_none());
    Ok(())
}
