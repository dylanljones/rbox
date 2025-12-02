// Author: Dylan Jones
// Date:   2025-05-06

#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir_in, NamedTempFile, TempDir};

/// Copy a directory tree from src to dst.
fn copy_tree(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_tree(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Get the root directory of the testdata.
pub fn testdata_dir() -> PathBuf {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let root = current_dir.parent().unwrap().join(".testdata");
    root
}

/// Get the root directory of the demo rekordbox library testdata.
pub fn testdata_demo_dir(v: usize) -> PathBuf {
    let root = testdata_dir();
    root.join(format!("RBv{}", v)).join("demo")
}

/// Get the options.json file path of the demo rekordbox library testdata.
pub fn options_path(v: usize) -> PathBuf {
    let root = testdata_demo_dir(v);
    root.join("rekordboxAgent")
        .join("storage")
        .join("options.json")
}

/// Get the root directory of the MySettings testdata.
pub fn settings_dir() -> PathBuf {
    let root = testdata_dir();
    root.join("mysettings")
}

pub fn anlz_dir(v: usize) -> PathBuf {
    let root = testdata_demo_dir(v);
    root.join("rekordbox")
        .join("share")
        .join("PIONEER")
        .join("USBANLZ")
}

pub fn export_dir(v: usize) -> PathBuf {
    let root = testdata_demo_dir(v);
    root.join("export").join("PIONEER")
}

/// Get the root directory of the temp files and create it if it doesn't exist.
pub fn get_temp_root_dir() -> PathBuf {
    let tmp_root = testdata_dir();
    if !tmp_root.exists() {
        fs::create_dir_all(&tmp_root).expect("Failed to create temp root directory");
    }
    tmp_root
}

/// Create a copy of a fixture path as a temp file.
fn create_temp_file(fixture_path: &PathBuf) -> NamedTempFile {
    let tmp_root = get_temp_root_dir();
    let tmp = NamedTempFile::new_in(tmp_root).expect("Failed to create temp file");
    fs::copy(&fixture_path, tmp.path()).expect("Failed to copy file");
    tmp
}

/// Create a new temp directory.
pub fn create_temp_dir() -> TempDir {
    let tmp_root = get_temp_root_dir();
    fs::create_dir_all(&tmp_root).expect("Failed to create temp root directory");
    tempdir_in(tmp_root).expect("Failed to create temp directory")
}

/// Setup code for the master.db tests
pub fn setup_master_db_path(v: usize) -> NamedTempFile {
    let fixture_path = testdata_demo_dir(v).join("rekordbox").join("master.db");
    let tmp = create_temp_file(&fixture_path);
    tmp
}

/// Setup code for the master.db tests that need all additional files.
pub fn setup_master_db_dir_path(v: usize) -> TempDir {
    let rb_dir = testdata_demo_dir(v).join("rekordbox");

    // Recreate the temp directory with all files
    let tmp_dir = create_temp_dir();
    let dir = tmp_dir.path().join("rekordbox");
    fs::create_dir_all(&dir).expect("Failed to create temp directory");
    // Copy the database file
    let src = rb_dir.join("master.db");
    fs::copy(&src, dir.join("master.db")).expect("Failed to copy database file");
    // Copy the playlist xml file
    let src = rb_dir.join("masterPlaylists6.xml");
    fs::copy(&src, dir.join("masterPlaylists6.xml")).expect("Failed to copy xml file");

    tmp_dir
}

/// Setup code for the one-library tests
pub fn setup_one_library_path() -> NamedTempFile {
    let fixture_path = export_dir(7).join("rekordbox").join("exportLibrary.db");
    let tmp = create_temp_file(&fixture_path);
    tmp
}

/// Setup code for the anlz files
pub fn setup_anlz_files() -> TempDir {
    let root = anlz_dir(6);
    let anlz_dir = root.join("e35").join("fa187-3f34-47e2-9880-2b33cb8d1304");
    // Recreate the temp directory with all files
    let tmp_dir = create_temp_dir();
    copy_tree(&anlz_dir, tmp_dir.path()).expect("Failed to copy files");

    tmp_dir
}

/// Setup code for the database tests.
pub fn setup_rekordbox_xml_path() -> NamedTempFile {
    let fixture_path = testdata_demo_dir(6).join("database.xml");
    create_temp_file(&fixture_path)
}
