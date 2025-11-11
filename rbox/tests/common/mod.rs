// Author: Dylan Jones
// Date:   2025-05-06

#![allow(dead_code)]

use rbox::{Anlz, MasterDb};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir_in, NamedTempFile, TempDir};

/// Get the root directory of the testdata.
pub fn testdata_dir() -> PathBuf {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let root = current_dir.parent().unwrap().join(".testdata");
    root
}

/// Get the root directory of the demo rekordbox library testdata.
pub fn testdata_demo_dir() -> PathBuf {
    let root = testdata_dir();
    root.join("RBv6").join("demo")
}

/// Get the options.json file path of the demo rekordbox library testdata.
pub fn testdata_options_path() -> PathBuf {
    let root = testdata_demo_dir();
    root.join("rekordboxAgent")
        .join("storage")
        .join("options.json")
}

/// Get the root directory of the Anlz testdata.
pub fn testdata_anlz_dir() -> PathBuf {
    let root = testdata_demo_dir();
    root.join("rekordbox")
        .join("share")
        .join("PIONEER")
        .join("USBANLZ")
}

/// Get the root directory of the MySettings testdata.
pub fn testdata_settings_dir() -> PathBuf {
    let root = testdata_dir();
    root.join("mysettings")
}

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

/// Get the root directory of the temp files and create it if it doesn't exist.
pub fn get_temp_root_dir() -> PathBuf {
    let tmp_root = testdata_dir().join(".tmp");
    if !tmp_root.exists() {
        fs::create_dir_all(&tmp_root).expect("Failed to create temp root directory");
    }
    tmp_root
}

/// Remove the temp files and directories
fn remove_temp_dir() {
    let tmp_root = testdata_dir().join(".tmp");
    if !tmp_root.exists() {
        fs::remove_dir_all(&tmp_root).expect("Failed to remove temp root directory");
    }
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

fn create_rbv6_temp_db_dir() -> TempDir {
    let rb_dir = testdata_demo_dir().join("rekordbox");

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

/// Setup code for the database tests.
pub fn setup_master_db() -> MasterDb {
    let fixture_path = testdata_demo_dir().join("rekordbox").join("master.db");
    let tmp = create_temp_file(&fixture_path);
    MasterDb::new(tmp.path().to_str().unwrap()).expect("Failed to open database")
}

/// Setup code for the database tests that need all additional files.
pub fn setup_master_db_dir() -> (MasterDb, TempDir) {
    let tmp_root = create_rbv6_temp_db_dir();
    let db_path = tmp_root.path().join("rekordbox").join("master.db");
    let db = MasterDb::new(db_path.to_str().unwrap()).expect("Failed to open database");
    (db, tmp_root)
}

/// Setup code for the database tests.
pub fn setup_rekordbox_xml_path() -> NamedTempFile {
    let fixture_path = testdata_demo_dir().join("database.xml");
    create_temp_file(&fixture_path)
}

/// Setup code fot the MySettings tests.
pub fn setup_mysettings_path() -> PathBuf {
    let sett_dir = testdata_settings_dir();
    sett_dir
}

pub struct AnlzPaths {
    pub dat: PathBuf,
    pub ext: PathBuf,
    pub ex2: PathBuf,
}

pub struct AnlzFiles {
    pub dat: Anlz,
    pub ext: Anlz,
    pub ex2: Anlz,
}

fn get_anlz_dir() -> PathBuf {
    let root = testdata_anlz_dir();
    let anlz_dir = root.join("e35").join("fa187-3f34-47e2-9880-2b33cb8d1304");
    anlz_dir
}

fn create_anlz_temp_dir() -> TempDir {
    let root = testdata_anlz_dir();
    let anlz_dir = root.join("e35").join("fa187-3f34-47e2-9880-2b33cb8d1304");
    // Recreate the temp directory with all files
    let tmp_dir = create_temp_dir();
    copy_tree(&anlz_dir, tmp_dir.path()).expect("Failed to copy files");

    tmp_dir
}

/// Setup code for the Anlz tests.
pub fn setup_anlz_paths() -> (AnlzPaths, TempDir) {
    let tmp_dir = create_anlz_temp_dir();
    let dir = tmp_dir.path();
    let paths = AnlzPaths {
        dat: dir.join("ANLZ0000.DAT"),
        ext: dir.join("ANLZ0000.EXT"),
        ex2: dir.join("ANLZ0000.2EX"),
    };
    (paths, tmp_dir)
}

/// Setup code for the Anlz tests.
pub fn setup_anlz_files() -> AnlzFiles {
    let tmp_dir = create_anlz_temp_dir();
    let dir = tmp_dir.path();
    let paths = AnlzPaths {
        dat: dir.join("ANLZ0000.DAT"),
        ext: dir.join("ANLZ0000.EXT"),
        ex2: dir.join("ANLZ0000.2EX"),
    };
    let files = AnlzFiles {
        dat: Anlz::load(paths.dat).expect("Failed to load ANLZ0000.DAT"),
        ext: Anlz::load(paths.ext).expect("Failed to load ANLZ0000.EXT"),
        ex2: Anlz::load(paths.ex2).expect("Failed to load ANLZ0000.EX2"),
    };
    files
}
