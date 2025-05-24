// Author: Dylan Jones
// Date:   2025-05-06

#![allow(dead_code)]

use rbox::{Anlz, MasterDb};
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn testdata_demo_dir() -> anyhow::Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let root = current_dir
        .parent()
        .unwrap()
        .join(".testdata")
        .join("RBv6")
        .join("demo");
    Ok(root)
}

pub fn testdata_anlz_dir() -> anyhow::Result<PathBuf> {
    let root = testdata_demo_dir()?.join("rekordbox");
    let dir = root.join("share").join("PIONEER").join("USBANLZ");
    Ok(dir)
}

/// Setup code for the database tests.
pub fn setup_master_db() -> anyhow::Result<MasterDb> {
    let root = testdata_demo_dir()?.join("rekordbox");
    let demo_db_src = root.join("master.db");
    let demo_db = root.join("master-copy.db");

    // Create a clean copy of the demo database
    fs::copy(&demo_db_src, &demo_db).expect("Failed to copy database file");

    let db = MasterDb::new(demo_db.as_os_str().to_str().unwrap())?;
    Ok(db)
}

/// Setup code for the database tests.
pub fn setup_master_playlist_xml() -> anyhow::Result<String> {
    let root = testdata_demo_dir()?.join("rekordbox");
    let src = root.join("masterPlaylists6-tmplt.xml");
    let dst = root.join("masterPlaylists6.xml");

    // Create a clean copy of the demo database
    fs::copy(&src, &dst).expect("Failed to copy playlist xml file");

    Ok(dst.to_str().unwrap().to_string())
}

/// Setup code for the database tests.
pub fn setup_rekordbox_xml_path() -> anyhow::Result<String> {
    let root = testdata_demo_dir()?;
    let demo_xml_src = root.join("database.xml");
    let demo_xml = root.join("database-copy.xml");

    // Create a clean copy of the demo database
    fs::copy(&demo_xml_src, &demo_xml).expect("Failed to copy xml file");

    let path = demo_xml.to_str().unwrap();
    Ok(path.to_string())
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

fn get_anlz_dir() -> anyhow::Result<PathBuf> {
    let root = testdata_anlz_dir()?;
    let dir = root.join("e35").join("fa187-3f34-47e2-9880-2b33cb8d1304");
    Ok(dir)
}

/// Setup code for the Anlz tests.
pub fn setup_anlz_paths() -> anyhow::Result<AnlzPaths> {
    let dir = get_anlz_dir()?;
    let paths = AnlzPaths {
        dat: dir.join("ANLZ0000.DAT"),
        ext: dir.join("ANLZ0000.EXT"),
        ex2: dir.join("ANLZ0000.2EX"),
    };
    Ok(paths)
}

/// Setup code for the Anlz tests.
pub fn setup_anlz_files() -> anyhow::Result<AnlzFiles> {
    let dir = get_anlz_dir()?;
    let paths = AnlzPaths {
        dat: dir.join("ANLZ0000.DAT"),
        ext: dir.join("ANLZ0000.EXT"),
        ex2: dir.join("ANLZ0000.2EX"),
    };
    let files = AnlzFiles {
        dat: Anlz::load(paths.dat)?,
        ext: Anlz::load(paths.ext)?,
        ex2: Anlz::load(paths.ex2)?,
    };
    Ok(files)
}
