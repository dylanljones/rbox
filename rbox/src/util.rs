// Author: Dylan Jones
// Date:   2025-05-05

use dirs::data_dir;
use std::path::PathBuf;

pub fn get_rekordbox_app_dir_path() -> anyhow::Result<PathBuf> {
    let binding = data_dir().expect("Failed to get app data directory");
    let app_dir = binding.as_path();
    let pioneer_app_dir = app_dir.join("Pioneer");
    if !pioneer_app_dir.exists() {
        return Err(anyhow::anyhow!("Pioneer directory not found!"));
    };
    let rekordbox_app_dir = pioneer_app_dir.join("rekordbox");
    if !rekordbox_app_dir.exists() {
        return Err(anyhow::anyhow!("Rekordbox directory not found!"));
    };
    Ok(rekordbox_app_dir)
}

pub fn get_rekordbox_share_path() -> anyhow::Result<PathBuf> {
    let app_dir = get_rekordbox_app_dir_path()?;
    let path = app_dir.join("share");
    if !path.exists() {
        return Err(anyhow::anyhow!("Share directory not found!"));
    };
    Ok(path)
}

pub fn get_master_playlist_xml_path() -> anyhow::Result<PathBuf> {
    let app_dir = get_rekordbox_app_dir_path()?;
    let path = app_dir.join("masterPlaylists6.xml");
    if !path.exists() {
        return Err(anyhow::anyhow!("Share directory not found!"));
    };
    Ok(path)
}

pub fn get_master_db_path() -> anyhow::Result<PathBuf> {
    let app_dir = get_rekordbox_app_dir_path()?;
    let path = app_dir.join("master.db");
    if !path.exists() {
        return Err(anyhow::anyhow!("Master database not found!"));
    };
    Ok(path)
}
