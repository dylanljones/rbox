// Author: Dylan Jones
// Date:   2025-05-13

use dirs::data_dir;
use serde::Deserialize;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct OptionsRaw {
    options: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct RekordboxOptions {
    pub db_path: PathBuf,
    pub analysis_root: PathBuf,
    pub settings_root: PathBuf,
}

impl RekordboxOptions {
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

    pub fn load<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> anyhow::Result<Self> {
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

    pub fn get_db_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self
            .db_path
            .parent()
            .expect("Failed to get database directory");
        Ok(path.to_path_buf())
    }
}
