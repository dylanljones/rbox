// Author: Dylan Jones
// Date:   2025-05-13

//! Rekordbox options loading and management.
//!
//! This module provides functionality to load and manage Rekordbox application options
//! from the JSON configuration files created by the application. It handles parsing
//! options such as database paths, analysis data locations, and settings directories.
//!
//! The main entry point is the `RekordboxOptions` struct which provides methods for
//! creating options instances, loading from files, and accessing specific paths.
//!
//! # Example
//!
//! ```no_run
//! use rbox::options::RekordboxOptions;
//!
//! // Open the default Rekordbox options file
//! match RekordboxOptions::open() {
//!     Ok(options) => {
//!         println!("Database path: {:?}", options.db_path);
//!         println!("Analysis root: {:?}", options.analysis_root);
//!         println!("Settings root: {:?}", options.settings_root);
//!
//!         // Get the database directory
//!         if let Ok(db_dir) = options.get_db_dir() {
//!             println!("Database directory: {:?}", db_dir);
//!         }
//!     },
//!     Err(e) => println!("Failed to load Rekordbox options: {}", e),
//! }
//! ```

use dirs::data_dir;
use serde::Deserialize;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Represents the Rekordbox options, including paths for the database, analysis data, and settings.
#[derive(Debug)]
pub struct RekordboxOptions {
    /// Path to the Rekordbox database.
    pub db_path: PathBuf,
    /// Root path for analysis data.
    pub analysis_root: PathBuf,
    /// Root path for settings.
    pub settings_root: PathBuf,
}

impl RekordboxOptions {
    /// Creates a new `RekordboxOptions` instance.
    ///
    /// # Parameters
    /// - `db_path`: Path to the Rekordbox database.
    /// - `analysis_root`: Root path for analysis data.
    /// - `settings_root`: Root path for settings.
    ///
    /// # Returns
    /// - `Ok(Self)` if the instance is successfully created.
    /// - `Err(anyhow::Error)` if an error occurs.
    pub fn new<P: AsRef<Path> + AsRef<OsStr>>(
        db_path: P,
        analysis_root: P,
        settings_root: P,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            db_path: Path::new(&db_path).to_path_buf(),
            analysis_root: Path::new(&analysis_root).to_path_buf(),
            settings_root: Path::new(&settings_root).to_path_buf(),
        })
    }

    /// Loads Rekordbox options from a JSON file.
    ///
    /// # Parameters
    /// - `path`: Path to the JSON file containing the options.
    ///
    /// # Returns
    /// - `Ok(Self)` if the options are successfully loaded.
    /// - `Err(anyhow::Error)` if an error occurs during loading or parsing.
    pub fn load<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> anyhow::Result<Self> {
        #[derive(Debug, Deserialize)]
        struct OptionsRaw {
            options: Vec<Vec<String>>,
        }

        let file = std::fs::File::open(path).expect("File not found");
        let reader = std::io::BufReader::new(file);
        let raw: OptionsRaw = serde_json::from_reader(reader).expect("JSON was not well-formatted");

        let mut db_path_opt: Option<String> = None;
        // let mut port_opt: Option<String> = None;
        let mut analysis_root_path_opt: Option<String> = None;
        let mut settings_root_path_opt = None;

        for opt in raw.options {
            let name = opt.get(0).expect("No name");
            let value = opt.get(1).expect("No value");
            match name.as_str() {
                "db-path" => db_path_opt = Some(value.to_string()),
                // "port" => port_opt = Some(value.to_string()),
                "analysis-data-root-path" => analysis_root_path_opt = Some(value.to_string()),
                "settings-root-path" => settings_root_path_opt = Some(value.to_string()),
                &_ => {}
            }
        }
        Self::new(
            db_path_opt.expect("No db path"),
            analysis_root_path_opt.expect("No analysis root path"),
            settings_root_path_opt.expect("No settings root path"),
        )
    }

    /// Opens the Rekordbox options file from the default location.
    ///
    /// # Returns
    /// - `Ok(Self)` if the options file is successfully loaded.
    /// - `Err(anyhow::Error)` if the file or required directories are not found.
    pub fn open() -> anyhow::Result<Self> {
        let binding = data_dir().expect("Failed to get app data directory");
        let app_dir = binding.as_path();
        let pioneer_app_dir = app_dir.join("Pioneer");
        if !pioneer_app_dir.exists() {
            return Err(anyhow::anyhow!("Pioneer directory not found!"));
        };
        let file = pioneer_app_dir
            .join("rekordboxAgent")
            .join("storage")
            .join("options.json");
        if !file.exists() {
            return Err(anyhow::anyhow!("Rekordbox options.json not found!"));
        };
        Self::load(&file)
    }

    /// Retrieves the directory containing the Rekordbox database.
    ///
    /// # Returns
    /// - `Ok(PathBuf)` containing the database directory path.
    /// - `Err(anyhow::Error)` if the database directory cannot be determined.
    pub fn get_db_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self
            .db_path
            .parent()
            .expect("Failed to get database directory");
        Ok(path.to_path_buf())
    }
}
