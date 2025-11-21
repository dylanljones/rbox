// Author: Dylan Jones
// Date:   2025-05-05

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use std::env;
use std::path::PathBuf;
use sysinfo::System;

use super::error::{Error, Result};

/// Determines the application directory path based on the operating system.
///
/// The returned value depends on the operating system:
///
/// |Platform | Value                                    |
/// | ------- | ---------------------------------------- |
/// | macOS   | `$HOME`/Library/Application Support      |
/// | Windows | `%APPDATA%`                              |
/// | Linux   | `$XDG_DATA_HOME` or `$HOME`/.local/share |
///
/// # Errors
/// Returns `Error::EnvionmentError` if the required environment variable is not set or is empty.
pub fn app_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        match env::var_os("HOME") {
            Some(h) if !h.is_empty() => {
                let home = PathBuf::from(h);
                Ok(home.join("Library").join("Application Support"))
            }
            _ => Err(Error::EnvionmentNotFound("$HOME".into())),
        }
    }
    #[cfg(target_os = "windows")]
    {
        match env::var_os("APPDATA") {
            Some(p) if !p.is_empty() => Ok(PathBuf::from(p)),
            _ => Err(Error::EnvionmentNotFound("%APPDATA%".into())),
        }
    }
    #[cfg(target_os = "linux")]
    {
        match env::var_os("XDG_DATA_HOME") {
            Some(p) if !p.is_empty() => Ok(PathBuf::from(p)),
            _ => {
                // Fallback to the default location
                match env::var_os("HOME") {
                    Some(h) if !h.is_empty() => {
                        let home = PathBuf::from(h);
                        Ok(home.join(".local").join("share"))
                    }
                    _ => Err(Error::EnvionmentNotFound("$XDG_DATA_HOME or $HOME".into())),
                }
            }
        }
    }
}

/// Retrieves the path to the `Library` directory on macOS.
///
/// The `Library` directory is typically located at `$HOME/Library`.
///
/// # Returns
/// The path to the `Library` directory if the `$HOME` environment variable is set.
///
/// # Errors
/// Returns `Error::EnvionmentError` if the required environment variable is not set or is empty.
#[cfg(target_os = "macos")]
pub fn library_dir() -> Result<PathBuf> {
    match env::var_os("HOME") {
        Some(h) if !h.is_empty() => {
            let home = PathBuf::from(h);
            Ok(home.join("Library"))
        }
        _ => Err(Error::EnvionmentNotFound("$HOME".into())),
    }
}

/// Retrieves the path to the Rekordbox Agent directory.
///
/// The Rekordbox Agent directory is typically located within the application directory
/// under the `Pioneer/rekordboxAgent` subdirectory:
/// - Windows: `C:\Users\dylan\AppData\Roaming\Pioneer\rekordboxAgent`.
/// - MacOS: `~/Library/Application Support/Pioneer/rekordboxAgent`.
///
/// # Returns
/// The path to the Rekordbox Agent directory if it exists.
///
/// # Errors
/// * `Error::EnvionmentError` if the required environment variable is not set or is empty.
/// * `Error::FileNotFound` if the directory does not exist.
pub fn rekordbox_agent_app_dir() -> Result<PathBuf> {
    let app_dir = app_dir()?;
    let path = app_dir.join("Pioneer").join("rekordboxAgent");
    if !path.exists() {
        Err(Error::FileNotFound(path.display().to_string()))
    } else {
        Ok(path)
    }
}

/// Retrieves the path to the Rekordbox application directory.
///
/// The Rekordbox application directory is typically located within the application directory
/// under the `Pioneer/rekordbox` subdirectory:
/// - Windows: `C:\Users\dylan\AppData\Roaming\Pioneer\rekordbox`.
/// - MacOS: `~/Library/Application Support/Pioneer/rekordbox`.
///
/// # Returns
/// The path to the Rekordbox application directory if it exists.
///
/// # Errors
/// * `Error::EnvionmentError` if the required environment variable is not set or is empty.
/// * `Error::FileNotFound` if the directory does not exist.
pub fn rekordbox_app_dir() -> Result<PathBuf> {
    let app_dir = app_dir()?;
    let path = app_dir.join("Pioneer").join("rekordbox");
    if !path.exists() {
        Err(Error::FileNotFound(path.display().to_string()))
    } else {
        Ok(path)
    }
}

/// Retrieves the path to the `options.json` file used by the Rekordbox Agent.
///
/// The `options.json` file is typically located in the `storage` subdirectory
/// within the Rekordbox Agent application directory.
///
/// # Returns
/// The path to the `options.json` file if it exists.
///
/// # Errors
/// * `Error::EnvionmentError` if the required environment variable is not set or is empty.
/// * `Error::FileNotFound` if the `options.json` file does not exist.
pub fn options_json_path() -> Result<PathBuf> {
    let app_dir = rekordbox_agent_app_dir()?;
    let path = app_dir.join("storage").join("options.json");
    if !path.exists() {
        Err(Error::FileNotFound(path.display().to_string()))
    } else {
        Ok(path)
    }
}

/// Retrieves the default path to the Rekordbox master database file.
///
/// On macOS, the function first checks for the database file in the `~/Library/Pioneer/rekordbox`
/// directory. If not found, it falls back to the `~/Library/Application Support/Pioneer/rekordbox`
/// directory. On other platforms, it directly checks the `master.db` file in the Rekordbox
/// application directory.
///
/// # Returns
/// The path to the `master.db` file if it exists.
///
/// # Errors
/// - Returns `Error::EnvionmentError` if the required environment variable is not set or is empty.
/// - Returns `Error::FileNotFound` if the `master.db` file is not found in the expected locations.
pub fn default_master_db_path() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let lib_dir = library_dir()?;
        let path = lib_dir.join("Pioneer").join("rekordbox").join("master.db");
        if !path.exists() {
            let path = lib_dir
                .join("Application Support")
                .join("Pioneer")
                .join("rekordbox")
                .join("master.db");
            if !path.exists() {
                Err(Error::FileNotFound(path.display().to_string()))
            } else {
                Ok(path)
            }
        } else {
            Ok(path)
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let app_dir = rekordbox_app_dir()?;
        let path = app_dir.join("master.db");
        if !path.exists() {
            Err(Error::FileNotFound(path.display().to_string()))
        } else {
            Ok(path)
        }
    }
}

/// Retrieves the path to the Rekordbox "share" directory.
///
/// The "share" directory is typically located within the Rekordbox application directory.
///
/// # Returns
/// The path to the "share" directory.
///
/// # Errors
/// * `Error::EnvionmentError` if the required environment variable is not set or is empty.
/// * `Error::FileNotFound` if the directory does not exist.
#[deprecated(since = "0.1.5", note = "use the `RekordboxOptions` handler instead!")]
pub fn get_rekordbox_share_path() -> Result<PathBuf> {
    let app_dir = rekordbox_app_dir()?;
    let path = app_dir.join("share");
    if !path.exists() {
        return Err(Error::FileNotFound(path.display().to_string()));
    };
    Ok(path)
}

/// Retrieves the path to the Rekordbox master playlists XML file.
///
/// # Returns
/// The path to the `masterPlaylists6.xml` file.
///
/// # Errors
/// * `Error::EnvionmentError` if the required environment variable is not set or is empty.
/// * `Error::FileNotFound` if the file does not exist.
#[deprecated(since = "0.1.5", note = "use the `RekordboxOptions` handler instead!")]
pub fn get_master_playlist_xml_path() -> Result<PathBuf> {
    let app_dir = rekordbox_app_dir()?;
    let path = app_dir.join("masterPlaylists6.xml");
    if !path.exists() {
        return Err(Error::FileNotFound(path.display().to_string()));
    };
    Ok(path)
}

/// Retrieves the path to the Rekordbox master database file.
///
/// # Returns
/// The path to the `master.db` file.
///
/// # Errors
/// * `Error::EnvionmentError` if the required environment variable is not set or is empty.
/// * `Error::FileNotFound` if the file does not exist.
#[deprecated(
    since = "0.1.5",
    note = "use `RekordboxOptions` or `default_master_db_path` instead"
)]
pub fn get_master_db_path() -> Result<PathBuf> {
    let app_dir = rekordbox_app_dir()?;
    let path = app_dir.join("master.db");
    if !path.exists() {
        return Err(Error::FileNotFound(path.display().to_string()));
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
/// A vector of `Process` structs representing Rekordbox processes.
pub fn get_rekordbox_processes() -> Vec<Process> {
    let s = System::new_all();
    let mut processes: Vec<Process> = Vec::new();
    for (pid, process) in s.processes() {
        if let Some(name) = process.name().to_str() {
            let name: &str = name.trim_end_matches(".exe");
            if name.contains::<&str>("rekordbox".as_ref()) {
                let proc = Process {
                    pid: pid.as_u32(),
                    name: name.to_string(),
                };
                processes.push(proc);
            }
        }
    }
    processes
}

/// Retrieves the PID of the Rekordbox process if it is running.
///
/// # Returns
/// A `Option` containing the PID of the Rekordbox process, or `None` if it is not running.
pub fn get_rekordbox_pid() -> Option<i32> {
    let s = System::new_all();
    for (pid, process) in s.processes() {
        if let Some(name) = process.name().to_str() {
            let name: &str = name.trim_end_matches(".exe");
            if name.to_lowercase() == "rekordbox" {
                return Some(pid.as_u32() as i32);
            }
        }
    }
    None
}

/// Checks if the Rekordbox application is currently running.
///
/// # Returns
/// `true` if Rekordbox is running, `false` otherwise
pub fn is_rekordbox_running() -> bool {
    get_rekordbox_pid().is_some()
}

/// String iterator used for Python bindings.
#[cfg(feature = "pyo3")]
#[pyclass]
pub struct PyStrIter {
    pub inner: std::vec::IntoIter<String>,
}

#[cfg(feature = "pyo3")]
impl PyStrIter {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            inner: items.into_iter(),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyStrIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<String> {
        slf.inner.next()
    }
}

/// Object iterator used for Python bindings.
#[cfg(feature = "pyo3")]
#[pyclass]
pub struct PyObjectIter {
    pub inner: std::vec::IntoIter<Py<PyAny>>,
}

#[cfg(feature = "pyo3")]
impl PyObjectIter {
    pub fn new(items: Vec<Py<PyAny>>) -> Self {
        Self {
            inner: items.into_iter(),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyObjectIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Py<PyAny>> {
        slf.inner.next()
    }
}

/// Items iterator used for Python bindings.
#[cfg(feature = "pyo3")]
#[pyclass]
pub struct PyItemsIter {
    pub inner: std::vec::IntoIter<(String, Py<PyAny>)>,
}

#[cfg(feature = "pyo3")]
impl PyItemsIter {
    pub fn new(items: Vec<(String, Py<PyAny>)>) -> Self {
        Self {
            inner: items.into_iter(),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyItemsIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(String, Py<PyAny>)> {
        slf.inner.next()
    }
}
