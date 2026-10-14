// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Read-only discovery of Rekordbox programs, configuration, and library data.
//!
//! Use [`locate_library`] to find the selected library, [`master_db_path`] when an
//! existing database is required, and [`find_programs`] to enumerate installations.
//! Programs, preferences, and libraries have separate locations: moving a library
//! to an external drive does not move the application or its configuration.
//! Discovery reads configuration and filesystem metadata without changing them.
//!
//! # Configuration locations
//!
//! Let `BASE` denote the directory returned by [`app_dir`]:
//!
//! | Platform | `BASE` | Default library directories, in order |
//! | --- | --- | --- |
//! | Windows | `%APPDATA%` | `BASE/Pioneer/rekordbox` |
//! | macOS | `$HOME/Library/Application Support` | `$HOME/Library/Pioneer/rekordbox`, then `BASE/Pioneer/rekordbox` |
//! | Other platforms | Absolute `$XDG_DATA_HOME`, otherwise `$HOME/.local/share` | `BASE/Pioneer/rekordbox` |
//!
//! Preferences are read from `BASE/Pioneer/rekordbox6/rekordbox3.settings`, and
//! agent options from `BASE/Pioneer/rekordboxAgent/storage/options.json`. The XDG
//! layout supports explicit profiles; it does not imply a native Linux installation.
//!
//! # Library selection
//!
//! [`LookupPaths::locate`] searches in this order:
//!
//! 1. The last absolute `masterDbDirectory` in the XML preferences, with
//!    `master.db` appended.
//! 2. The absolute `db-path` database filename from [`RekordboxOptions`].
//! 3. The first default directory containing a regular `master.db` file, or the
//!    first default candidate with [`LibraryLocation::available`] set to `false`.
//!
//! A configured database takes precedence even when unavailable, for example on a
//! disconnected drive. [`locate_library`] preserves its path and [`LocationSource`];
//! [`master_db_path`] returns [`Error::FileNotFound`]. [`default_master_db_path`]
//! searches only platform defaults, ignoring the configured selection.
//!
//! Missing configuration files permit fallback; malformed files, missing required
//! options, and permission errors are returned. Agent options are loaded even when
//! preferences select the database, since they also supply related data paths.
//! Their analysis and settings roots are retained only when their database matches
//! the selected file. Otherwise, rbox uses `share` beside the database and the
//! preferences file's parent directory. Related directories are not checked for
//! existence, and configuration paths are not expanded for `~` or environment variables.
//!
//! Availability only means the database is a regular file at lookup time; it does
//! not validate its contents or open a database connection.
//!
//! ```no_run
//! use rbox_core::locate::locate_library;
//!
//! let library = locate_library()?;
//! if library.available {
//!     println!("Database: {}", library.db_path.display());
//!     println!("Analysis: {}", library.analysis_root.display());
//!     println!("Settings: {}", library.settings_root.display());
//! } else {
//!     println!("Selected via {:?}, but unavailable: {}",
//!         library.source, library.db_path.display());
//! }
//! # Ok::<(), rbox_core::error::Error>(())
//! ```
//!
//! # Custom profiles
//!
//! [`LookupPaths::under`] constructs paths beneath an application-data base
//! (before the `Pioneer` component) without reading the environment or filesystem.
//! Customize its fields for other layouts; keep at least one default directory.
//!
//! ```no_run
//! use rbox_core::locate::LookupPaths;
//!
//! let base = std::env::current_dir()?.join("fixtures/profile");
//! let mut paths = LookupPaths::under(&base);
//! paths.settings_file = base.join("preferences/rekordbox3.settings");
//! paths.default_library_dirs = vec![base.join("library"), base.join("legacy")];
//! let library = paths.locate()?;
//! # Ok::<(), rbox_core::error::Error>(())
//! ```
//!
//! # Program installations
//!
//! [`find_programs`] searches Windows program directories or macOS `/Applications`
//! and `$HOME/Applications`. Other platforms return [`Error::UnsupportedPlatform`];
//! use [`find_programs_in`] to search explicit installation roots on any platform.
//! Scanning is bounded to three levels of recognized vendor and Rekordbox directories,
//! skips symlinks, and recognizes Windows executables and macOS application bundles.
//! Results are deduplicated and sorted by versions parsed from directory names,
//! newest first, with unknown versions last. Running-process checks are provided
//! separately by [`crate::process`].
//!
//! ```no_run
//! use rbox_core::locate::find_programs;
//!
//! for program in find_programs()? {
//!     println!("{:?}: {}", program.version, program.executable.display());
//! }
//! # Ok::<(), rbox_core::error::Error>(())
//! ```

use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::options::RekordboxOptions;

fn environment_path(name: &'static str) -> Result<PathBuf> {
    env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .ok_or(Error::EnvironmentNotFound(name))
}

/// Platform application-data base directory (not the program installation).
///
/// Uses `%APPDATA%` on Windows, `$HOME/Library/Application Support` on macOS,
/// and absolute `$XDG_DATA_HOME` or `$HOME/.local/share` elsewhere. Does not
/// check existence or create directories. A relative XDG value is ignored.
///
/// # Errors
/// Returns [`Error::EnvironmentNotFound`] when a required variable is unset or empty.
pub fn app_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        environment_path("APPDATA")
    }
    #[cfg(target_os = "macos")]
    {
        Ok(library_dir()?.join("Application Support"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        match environment_path("XDG_DATA_HOME") {
            Ok(path) if path.is_absolute() => Ok(path),
            _ => Ok(environment_path("HOME")?.join(".local/share")),
        }
    }
}

/// The current user's macOS Library directory.
#[cfg(target_os = "macos")]
pub fn library_dir() -> Result<PathBuf> {
    Ok(environment_path("HOME")?.join("Library"))
}

fn require_path(path: PathBuf, directory: bool) -> Result<PathBuf> {
    match std::fs::metadata(&path) {
        Ok(metadata)
            if if directory {
                metadata.is_dir()
            } else {
                metadata.is_file()
            } =>
        {
            Ok(path)
        }
        Ok(_) => Err(Error::FileNotFound(path)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(Error::FileNotFound(path))
        }
        Err(error) => Err(error.into()),
    }
}

/// Existing agent application-data directory.
///
/// Requires `Pioneer/rekordboxAgent` beneath [`app_dir`] to be a directory.
/// Returns [`Error::FileNotFound`] for an absent path or wrong path kind;
/// environment and other I/O errors are propagated.
pub fn rekordbox_agent_app_dir() -> Result<PathBuf> {
    require_path(app_dir()?.join("Pioneer/rekordboxAgent"), true)
}

/// Existing default library directory; configured libraries can be elsewhere.
///
/// Searches the platform defaults in module-documented order and requires only
/// a directory, not a database inside it. Use [`locate_library`] for configured
/// external libraries. Returns [`Error::FileNotFound`] if no directory exists.
pub fn rekordbox_app_dir() -> Result<PathBuf> {
    let paths = LookupPaths::installed()?;
    for path in &paths.default_library_dirs {
        match require_path(path.clone(), true) {
            Ok(path) => return Ok(path),
            Err(Error::FileNotFound(_)) => (),
            Err(error) => return Err(error),
        }
    }
    Err(Error::FileNotFound(paths.default_library_dirs[0].clone()))
}

/// Existing agent options file, independent of library selection.
///
/// Requires `Pioneer/rekordboxAgent/storage/options.json` beneath [`app_dir`] to
/// be a regular file. Does not parse it. Returns [`Error::FileNotFound`] for an
/// absent file or wrong path kind, propagating environment and other I/O errors.
pub fn options_json_path() -> Result<PathBuf> {
    require_path(
        app_dir()?.join("Pioneer/rekordboxAgent/storage/options.json"),
        false,
    )
}

/// Explicit discovery inputs, allowing custom profiles and isolated tests.
///
/// Constructors compute candidates without inspecting configuration. Call
/// [`Self::locate`] to read them. Supply absolute paths for portable behavior
/// across working directories; relative custom paths use the process working directory.
#[derive(Debug, Clone)]
pub struct LookupPaths {
    /// Application settings file; missing files are skipped.
    pub settings_file: PathBuf,
    /// Agent options file, parsed with [`RekordboxOptions`].
    pub options_file: PathBuf,
    /// Default library directories in preference order. Must contain an entry.
    pub default_library_dirs: Vec<PathBuf>,
}

/// Where the selected library path came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyo3::pyclass(from_py_object, eq, eq_int))]
#[cfg_attr(feature = "napi", napi_derive::napi)]
pub enum LocationSource {
    /// The `masterDbDirectory` application setting.
    Settings,
    /// The agent's `db-path` option.
    Options,
    /// A platform default library location.
    Default,
}

/// A selected library, including a missing configured external drive.
///
/// A successful lookup does not guarantee `available == true`. Related roots
/// are returned without existence checks. This value is a snapshot and owns no
/// database connection or filesystem lock.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryLocation {
    /// Selected master database, even when currently unavailable.
    pub db_path: PathBuf,
    /// Matching agent analysis path, or `share` next to the selected database.
    pub analysis_root: PathBuf,
    /// Matching agent settings path, or the application settings directory.
    pub settings_root: PathBuf,
    /// Configuration source that selected the database.
    pub source: LocationSource,
    /// True only when the selected database is an existing regular file.
    pub available: bool,
}

impl LookupPaths {
    /// Conventional locations beneath an application-data base directory.
    ///
    /// Appends `Pioneer/rekordbox6/rekordbox3.settings`,
    /// `Pioneer/rekordboxAgent/storage/options.json`, and `Pioneer/rekordbox`.
    /// Does not add the macOS legacy library location; [`Self::installed`] does.
    /// No files are read or created, and the environment is not consulted.
    pub fn under(base: impl AsRef<Path>) -> Self {
        let pioneer = base.as_ref().join("Pioneer");
        Self {
            settings_file: pioneer.join("rekordbox6/rekordbox3.settings"),
            options_file: pioneer.join("rekordboxAgent/storage/options.json"),
            default_library_dirs: vec![pioneer.join("rekordbox")],
        }
    }

    /// Current platform paths without requiring Rekordbox to be installed.
    ///
    /// Uses [`app_dir`] and adds the macOS `Library/Pioneer/rekordbox` default
    /// before its Application Support fallback. The resulting paths need not
    /// exist. Returns environment errors if a required base cannot be resolved.
    pub fn installed() -> Result<Self> {
        let paths = Self::under(app_dir()?);
        #[cfg(target_os = "macos")]
        let paths = {
            let mut paths = paths;
            paths
                .default_library_dirs
                .insert(0, library_dir()?.join("Pioneer/rekordbox"));
            paths
        };
        Ok(paths)
    }

    /// Resolve the active library without creating directories or modifying settings.
    ///
    /// Invalid relative settings paths are ignored. Agent database paths must be
    /// absolute. Malformed existing files and permission errors are propagated.
    /// Agent analysis paths are used only when its database matches the selection.
    ///
    /// Missing configured databases return a location with `available == false`.
    /// See the module documentation for the full precedence and related-root rules.
    ///
    /// # Errors
    /// Returns [`Error::OptionError`] for an empty default-directory list, a
    /// relative selected agent database path, or incomplete agent options.
    /// Propagates XML, JSON, and I/O errors from existing configuration and
    /// metadata queries. Invalid options fail even when settings select the database.
    pub fn locate(&self) -> Result<LibraryLocation> {
        let default = self
            .default_library_dirs
            .first()
            .ok_or_else(|| Error::OptionError("no default library directories supplied".into()))?;
        let configured = read_master_directory(&self.settings_file)?;
        let options = match RekordboxOptions::load(&self.options_file) {
            Ok(options) => Some(options),
            Err(Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        let (db_path, source) = if let Some(dir) = configured {
            (dir.join("master.db"), LocationSource::Settings)
        } else if let Some(options) = &options {
            if !options.db_path.is_absolute() {
                return Err(Error::OptionError("db-path must be absolute".into()));
            }
            (options.db_path.clone(), LocationSource::Options)
        } else {
            let mut selected = default.join("master.db");
            for dir in &self.default_library_dirs {
                let candidate = dir.join("master.db");
                if file_available(&candidate)? {
                    selected = candidate;
                    break;
                }
            }
            (selected, LocationSource::Default)
        };
        let available = file_available(&db_path)?;
        let matching = options
            .as_ref()
            .filter(|options| same_path(&options.db_path, &db_path));
        let analysis_root = matching
            .map(|o| o.analysis_root.clone())
            .unwrap_or_else(|| db_path.parent().unwrap_or(default).join("share"));
        let settings_root = matching
            .map(|o| o.settings_root.clone())
            .unwrap_or_else(|| {
                self.settings_file
                    .parent()
                    .unwrap_or(Path::new("."))
                    .to_path_buf()
            });
        Ok(LibraryLocation {
            db_path,
            analysis_root,
            settings_root,
            source,
            available,
        })
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    left == right
        || match (left.canonicalize(), right.canonicalize()) {
            (Ok(left), Ok(right)) => left == right,
            _ => false,
        }
}

fn file_available(path: &Path) -> Result<bool> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn read_master_directory(path: &Path) -> Result<Option<PathBuf>> {
    #[derive(Deserialize)]
    #[serde(rename = "PROPERTIES")]
    struct Properties {
        #[serde(rename = "VALUE", default)]
        values: Vec<Value>,
    }
    #[derive(Deserialize)]
    struct Value {
        #[serde(rename = "@name", default)]
        name: String,
        #[serde(rename = "@val")]
        value: Option<String>,
    }
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let properties: Properties = quick_xml::de::from_reader(std::io::BufReader::new(file))?;
    Ok(properties
        .values
        .into_iter()
        .filter(|v| v.name == "masterDbDirectory")
        .filter_map(|v| v.value)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .next_back())
}

/// Resolve the current user's active library, including availability information.
///
/// Convenience wrapper for [`LookupPaths::installed`] followed by
/// [`LookupPaths::locate`], propagating errors from both. Use explicit
/// [`LookupPaths`] when operating on another profile.
pub fn locate_library() -> Result<LibraryLocation> {
    LookupPaths::installed()?.locate()
}

/// Find an existing database at the platform defaults, ignoring configured overrides.
///
/// Does not read preferences or agent options. Returns [`Error::FileNotFound`]
/// naming the first candidate when no regular database file exists, and
/// propagates environment and other filesystem errors.
pub fn default_master_db_path() -> Result<PathBuf> {
    let mut paths = LookupPaths::installed()?;
    let first = paths.default_library_dirs[0].join("master.db");
    for directory in paths.default_library_dirs.drain(..) {
        let path = directory.join("master.db");
        if file_available(&path)? {
            return Ok(path);
        }
    }
    Err(Error::FileNotFound(first))
}

/// Find the selected database, reporting an unavailable configured drive as an error.
///
/// Resolves [`locate_library`] and requires `available == true`. Propagates
/// discovery errors and returns [`Error::FileNotFound`] for the selected missing
/// path. Does not validate the contents or open a database connection.
pub fn master_db_path() -> Result<PathBuf> {
    let location = locate_library()?;
    if location.available {
        Ok(location.db_path)
    } else {
        Err(Error::FileNotFound(location.db_path))
    }
}

/// Find the active database, honoring configured locations.
#[deprecated(note = "use master_db_path or locate_library")]
pub fn get_master_db_path() -> Result<PathBuf> {
    master_db_path()
}

/// Find the active library's analysis directory.
#[deprecated(note = "use locate_library for all configured data paths")]
pub fn get_rekordbox_share_path() -> Result<PathBuf> {
    require_path(locate_library()?.analysis_root, true)
}

/// Find the playlist XML beside the selected database.
#[deprecated(note = "use locate_library to resolve the selected database directory")]
pub fn get_master_playlist_xml_path() -> Result<PathBuf> {
    let location = locate_library()?;
    let parent = location
        .db_path
        .parent()
        .ok_or_else(|| Error::OptionError("selected database has no parent directory".into()))?;
    require_path(parent.join("masterPlaylists6.xml"), false)
}

/// An installed program and the numeric version parsed from its containing directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramLocation {
    /// Windows executable or executable inside a macOS application bundle.
    pub executable: PathBuf,
    /// Directory version components; empty if no numeric version was found.
    pub version: Vec<u32>,
}

/// Scan installation roots for Windows executables and macOS bundles.
///
/// Accepts an executable, an app bundle, a version directory, or a vendor root.
/// Traversal is limited to three levels and does not follow directory symlinks.
/// Results are deduplicated and sorted newest first; unversioned entries come last.
///
/// A directly supplied executable must be named `rekordbox.exe`; for macOS,
/// supply the `.app` bundle or its parent. Only Rekordbox and vendor-named child
/// entries are traversed. Version values come from path names, not binary metadata.
///
/// # Errors
/// Missing roots are skipped. Other directory or metadata failures return
/// [`Error::Io`]; partial results are not returned. No matches produce an empty vector.
pub fn find_programs_in(roots: impl IntoIterator<Item = PathBuf>) -> Result<Vec<ProgramLocation>> {
    fn scan(path: &Path, depth: usize, found: &mut Vec<ProgramLocation>) -> Result<()> {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        if metadata.file_type().is_symlink() {
            return Ok(());
        }
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if metadata.is_file() && name == "rekordbox.exe" {
            found.push(program(path));
        } else if metadata.is_dir() {
            if name.starts_with("rekordbox") && name.ends_with(".app") {
                let executable = path.join("Contents/MacOS/rekordbox");
                if file_available(&executable)? {
                    found.push(program(&executable));
                }
            } else if depth > 0 {
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if name.starts_with("rekordbox") || name == "pioneer" || name == "alphatheta" {
                        scan(&entry.path(), depth - 1, found)?;
                    }
                }
            }
        }
        Ok(())
    }
    fn program(path: &Path) -> ProgramLocation {
        let version = path
            .ancestors()
            .filter_map(|part| part.file_name())
            .filter_map(|part| {
                let name = part.to_string_lossy().to_lowercase();
                let suffix = name
                    .strip_prefix("rekordbox")?
                    .trim()
                    .trim_end_matches(".app");
                if suffix.is_empty() {
                    return None;
                }
                suffix
                    .split('.')
                    .map(str::parse::<u32>)
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .ok()
            })
            .next()
            .unwrap_or_default();
        ProgramLocation {
            executable: path.to_path_buf(),
            version,
        }
    }
    let mut found = Vec::new();
    for root in roots {
        scan(&root, 3, &mut found)?;
    }
    found.sort_by(|a, b| {
        b.version
            .cmp(&a.version)
            .then(a.executable.cmp(&b.executable))
    });
    found.dedup_by(|a, b| a.executable == b.executable);
    Ok(found)
}

/// Discover installations in platform program directories.
/// Custom locations can be passed to [`find_programs_in`].
///
/// Windows uses the nonempty Program Files environment paths, and macOS uses
/// `/Applications` and `$HOME/Applications`. No available Windows roots yields
/// an empty list. Results use the ordering and matching rules of [`find_programs_in`].
///
/// # Errors
/// Returns [`Error::UnsupportedPlatform`] outside Windows/macOS,
/// [`Error::EnvironmentNotFound`] for missing macOS `HOME`, or filesystem errors
/// from scanning. An empty successful result means no matching program was found.
pub fn find_programs() -> Result<Vec<ProgramLocation>> {
    #[cfg(target_os = "windows")]
    {
        let roots = ["ProgramW6432", "ProgramFiles", "ProgramFiles(x86)"]
            .into_iter()
            .filter_map(|name| environment_path(name).ok());
        find_programs_in(roots)
    }
    #[cfg(target_os = "macos")]
    {
        find_programs_in([
            PathBuf::from("/Applications"),
            environment_path("HOME")?.join("Applications"),
        ])
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err(Error::UnsupportedPlatform)
    }
}
