// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Read-only access to the Rekordbox Agent's `options.json` path configuration.
//!
//! # File format and required values
//!
//! The agent stores options as arrays of strings inside an `options` array:
//!
//! ```json
//! {
//!   "options": [
//!     ["db-path", "/Volumes/Music/PIONEER/Master/master.db"],
//!     ["analysis-data-root-path", "/Volumes/Music/PIONEER/Master/share"],
//!     ["settings-root-path", "/Users/dj/Library/Application Support/Pioneer/rekordbox6"]
//!   ]
//! }
//! ```
//!
//! [`RekordboxOptions::load`] requires all three named values to be present and
//! nonempty. Entries with fewer than two strings and unknown option names are
//! ignored; extra strings after a value are ignored. For repeated recognized
//! names, the last usable pair wins, including an empty value that subsequently
//! fails validation. Non-string array elements are JSON deserialization errors.
//! Other top-level fields are ignored.
//!
//! `db-path` names a file; the other two options name directories. Loading copies
//! their text into native paths without checking whether those paths exist,
//! resolving relative paths against the options file, expanding `~`, or replacing
//! environment variables. A profile containing Windows paths is interpreted using
//! the host's path semantics when loaded on another operating system.
//!
//! # Agent options versus the active library
//!
//! [`RekordboxOptions::open`] finds the agent file through
//! [`crate::locate::options_json_path`]. Use [`RekordboxOptions::load`] for a
//! specific profile or backup. Both return the contents of that file, which may
//! lag behind application preferences after a library move. To resolve the
//! selected library, use [`crate::locate::locate_library`]: it gives
//! `masterDbDirectory` in the application settings precedence over `db-path`,
//! and accepts agent analysis paths only when the databases match.
//!
//! This type exposes only the three data paths. It does not preserve every agent
//! option, retrieve database credentials, or write `options.json`.
//!
//! # Read the installed agent options
//!
//! ```no_run
//! use rbox_core::RekordboxOptions;
//!
//! let options = RekordboxOptions::open()?;
//! println!("Agent database: {}", options.db_path.display());
//! println!("Analysis root: {}", options.analysis_root.display());
//! println!("Settings root: {}", options.settings_root.display());
//! # Ok::<(), rbox_core::error::Error>(())
//! ```
//!
//! # Read a profile or construct paths in memory
//!
//! ```no_run
//! use rbox_core::RekordboxOptions;
//!
//! let profile = RekordboxOptions::load("profile/options.json")?;
//! let playlist_xml = profile.get_db_dir().join("masterPlaylists6.xml");
//! println!("Playlist XML candidate: {}", playlist_xml.display());
//! # Ok::<(), rbox_core::error::Error>(())
//! ```
//!
//! ```
//! use rbox_core::RekordboxOptions;
//! use std::path::Path;
//!
//! let options = RekordboxOptions::new("library/master.db", "library/share", "settings")?;
//! assert_eq!(options.get_db_dir(), Path::new("library"));
//! # Ok::<(), rbox_core::error::Error>(())
//! ```

use serde::Deserialize;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::locate::options_json_path;

/// The three library-related paths recorded by the Rekordbox Agent.
///
/// Fields preserve configured paths and are editable in memory. Changing them
/// does not update the agent file or switch the library selected by Rekordbox.
#[derive(Debug, Clone, PartialEq)]
pub struct RekordboxOptions {
    /// Agent `db-path`: the database filename, normally ending in `master.db`.
    pub db_path: PathBuf,
    /// Agent `analysis-data-root-path`: root of the library's analysis data.
    pub analysis_root: PathBuf,
    /// Agent `settings-root-path`: directory containing application settings.
    pub settings_root: PathBuf,
}

impl RekordboxOptions {
    /// Copy three supplied paths into an options value without filesystem access.
    ///
    /// Performs no validation, including for empty or relative paths. Currently
    /// always returns `Ok`; the `Result` signature is retained for compatibility.
    /// All three arguments must use the same generic path type.
    pub fn new<P: AsRef<Path> + AsRef<OsStr>>(
        db_path: P,
        analysis_root: P,
        settings_root: P,
    ) -> Result<Self> {
        Ok(Self {
            db_path: Path::new(&db_path).to_path_buf(),
            analysis_root: Path::new(&analysis_root).to_path_buf(),
            settings_root: Path::new(&settings_root).to_path_buf(),
        })
    }

    /// Read and validate the three required path options from a JSON file.
    ///
    /// Relative values are preserved; they are not rebased against `path`.
    /// Existence and access to the referenced library are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if opening or reading fails, [`Error::Json`] for
    /// deserialization failures (including errors reported by the JSON reader),
    /// or [`Error::OptionError`] if a required path value is absent or empty.
    pub fn load<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> Result<Self> {
        #[derive(Debug, Deserialize)]
        struct OptionsRaw {
            options: Vec<Vec<String>>,
        }

        let file = std::fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);
        let raw: OptionsRaw = serde_json::from_reader(reader)?;

        let mut db_path: Option<String> = None;
        // let mut port_opt: Option<String> = None;
        let mut analysis_root_path: Option<String> = None;
        let mut settings_root_path: Option<String> = None;

        for opt in raw.options {
            if opt.len() < 2 {
                continue; // Skip invalid entries
            }
            match opt[0].as_str() {
                "db-path" => db_path = Some(opt[1].clone()),
                "analysis-data-root-path" => analysis_root_path = Some(opt[1].clone()),
                "settings-root-path" => settings_root_path = Some(opt[1].clone()),
                _ => {}
            }
        }
        if db_path.as_ref().is_none_or(|value| value.is_empty()) {
            return Err(Error::OptionError(
                "Database path not found in options".to_string(),
            ));
        }
        if analysis_root_path
            .as_ref()
            .is_none_or(|value| value.is_empty())
        {
            return Err(Error::OptionError(
                "Analysis root path not found in options".to_string(),
            ));
        }
        if settings_root_path
            .as_ref()
            .is_none_or(|value| value.is_empty())
        {
            return Err(Error::OptionError(
                "Settings root path not found in options".to_string(),
            ));
        }
        Self::new(
            db_path.unwrap(),
            analysis_root_path.unwrap(),
            settings_root_path.unwrap(),
        )
    }

    /// Locate and load the current user's agent options file.
    ///
    /// Uses [`crate::locate::options_json_path`] and then [`Self::load`]. There is
    /// no fallback to application preferences or default library paths here.
    ///
    /// # Errors
    ///
    /// Propagates environment and file-location errors from discovery, and all
    /// parsing and validation errors from [`Self::load`].
    pub fn open() -> Result<Self> {
        let file = options_json_path()?;
        Self::load(&file)
    }

    /// Return the database path's parent, or `.` when no nonempty parent exists.
    ///
    /// This is a lexical operation: it does not canonicalize or inspect the path.
    /// Relative inputs produce relative outputs. The same fallback applies to
    /// empty or root-only paths supplied through [`Self::new`] or public fields.
    pub fn get_db_dir(&self) -> PathBuf {
        let path = self
            .db_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        path.to_path_buf()
    }
}
