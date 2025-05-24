// Author: Dylan Jones
// Date:   2025-05-05

use anyhow::Result;
use dirs::data_dir;
use std::path::PathBuf;
use sysinfo::System;

/// Retrieves the path to the Rekordbox application directory.
///
/// The Rekordbox application directory is typically:
/// Windows: `C:\Users\dylan\AppData\Roaming\Pioneer\rekordbox`.
/// MacOS: `~/Library/Application Support/Pioneer/rekordbox`.
///
/// # Returns
/// - `Ok(PathBuf)`: The path to the Rekordbox application directory.
/// - `Err(anyhow::Error)`: If the directory does not exist or cannot be accessed.
pub fn get_rekordbox_app_dir_path() -> Result<PathBuf> {
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

/// Retrieves the path to the Rekordbox "share" directory.
///
/// The "share" directory is typically located within the Rekordbox application directory.
///
/// # Returns
/// - `Ok(PathBuf)`: The path to the "share" directory.
/// - `Err(anyhow::Error)`: If the directory does not exist or cannot be accessed.
pub fn get_rekordbox_share_path() -> Result<PathBuf> {
    let app_dir = get_rekordbox_app_dir_path()?;
    let path = app_dir.join("share");
    if !path.exists() {
        return Err(anyhow::anyhow!("Share directory not found!"));
    };
    Ok(path)
}

/// Retrieves the path to the Rekordbox master playlists XML file.
///
/// # Returns
/// - `Ok(PathBuf)`: The path to the `masterPlaylists6.xml` file.
/// - `Err(anyhow::Error)`: If the file does not exist or cannot be accessed.
pub fn get_master_playlist_xml_path() -> Result<PathBuf> {
    let app_dir = get_rekordbox_app_dir_path()?;
    let path = app_dir.join("masterPlaylists6.xml");
    if !path.exists() {
        return Err(anyhow::anyhow!("Share directory not found!"));
    };
    Ok(path)
}

/// Retrieves the path to the Rekordbox master database file.
///
/// # Returns
/// - `Ok(PathBuf)`: The path to the `master.db` file.
/// - `Err(anyhow::Error)`: If the file does not exist or cannot be accessed.
pub fn get_master_db_path() -> Result<PathBuf> {
    let app_dir = get_rekordbox_app_dir_path()?;
    let path = app_dir.join("master.db");
    if !path.exists() {
        return Err(anyhow::anyhow!("Master database not found!"));
    };
    Ok(path)
}

/// Represents a process with a PID and name.
#[derive(Debug, Eq, PartialEq)]
pub struct Process {
    /// The process ID.
    pub pid: u32,
    /// The name of the process.
    pub name: String,
}

/// Retrieves a list of Rekordbox processes currently running on the system.
///
/// # Returns
/// - `Vec<Process>`: A vector of `Process` structs representing Rekordbox processes.
pub fn get_rekordbox_processes() -> Vec<Process> {
    let s = System::new_all();
    let mut processes: Vec<Process> = Vec::new();
    for (pid, process) in s.processes() {
        let name: &str = process
            .name()
            .to_str()
            .expect("Failed to convert process name to string")
            .trim_end_matches(".exe");
        if name.contains::<&str>("rekordbox".as_ref()) {
            let proc = Process {
                pid: pid.as_u32(),
                name: name.to_string(),
            };
            processes.push(proc);
        }
    }
    processes
}

/// Retrieves the PID of the Rekordbox process if it is running.
///
/// # Returns
/// - `Some(i32)`: The PID of the Rekordbox process.
/// - `None`: If no Rekordbox process is found.
pub fn get_rekordbox_pid() -> Option<i32> {
    let s = System::new_all();
    for (pid, process) in s.processes() {
        let name: &str = process
            .name()
            .to_str()
            .expect("Failed to convert process name to string")
            .trim_end_matches(".exe");
        if name.to_lowercase() == "rekordbox" {
            return Some(pid.as_u32() as i32);
        }
    }
    None
}

/// Checks if the Rekordbox application is currently running.
///
/// # Returns
/// - `true`: If Rekordbox is running.
/// - `false`: If Rekordbox is not running.
pub fn is_rekordbox_running() -> bool {
    get_rekordbox_pid().is_some()
}
