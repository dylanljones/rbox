// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Read-only library and program discovery bindings.

use napi::Result;
use napi_derive::napi;
use rbox::core::locate::{self, LocationSource};

/// Selected library and its availability at discovery time. Paths are strings.
#[napi(object)]
pub struct LibraryLocation {
    pub db_path: String,
    pub analysis_root: String,
    pub settings_root: String,
    pub source: LocationSource,
    pub available: bool,
}
impl From<locate::LibraryLocation> for LibraryLocation {
    fn from(value: locate::LibraryLocation) -> Self {
        Self {
            db_path: value.db_path.to_string_lossy().into_owned(),
            analysis_root: value.analysis_root.to_string_lossy().into_owned(),
            settings_root: value.settings_root.to_string_lossy().into_owned(),
            source: value.source,
            available: value.available,
        }
    }
}
impl From<LibraryLocation> for locate::LibraryLocation {
    fn from(value: LibraryLocation) -> Self {
        Self {
            db_path: value.db_path.into(),
            analysis_root: value.analysis_root.into(),
            settings_root: value.settings_root.into(),
            source: value.source,
            available: value.available,
        }
    }
}

/// A discovered executable and version parsed from its directory name.
#[napi(object)]
pub struct ProgramLocation {
    pub executable: String,
    pub version: Vec<u32>,
}
impl From<locate::ProgramLocation> for ProgramLocation {
    fn from(value: locate::ProgramLocation) -> Self {
        Self {
            executable: value.executable.to_string_lossy().into_owned(),
            version: value.version,
        }
    }
}

/// Custom discovery inputs. Construction does not access the filesystem.
#[napi]
pub struct LookupPaths {
    inner: locate::LookupPaths,
}
#[napi]
impl LookupPaths {
    #[napi(constructor)]
    pub fn new(
        settings_file: String,
        options_file: String,
        default_library_dirs: Vec<String>,
    ) -> Self {
        Self {
            inner: locate::LookupPaths {
                settings_file: settings_file.into(),
                options_file: options_file.into(),
                default_library_dirs: default_library_dirs.into_iter().map(Into::into).collect(),
            },
        }
    }
    #[napi(factory)]
    pub fn under(base: String) -> Self {
        Self {
            inner: locate::LookupPaths::under(base),
        }
    }
    #[napi(factory)]
    pub fn installed() -> Result<Self> {
        Ok(Self {
            inner: locate::LookupPaths::installed().map_err(crate::util::binding_error)?,
        })
    }
    #[napi]
    pub fn locate(&self) -> Result<LibraryLocation> {
        self.inner
            .locate()
            .map(Into::into)
            .map_err(crate::util::binding_error)
    }
    #[napi(getter)]
    pub fn settings_file(&self) -> String {
        self.inner.settings_file.to_string_lossy().into_owned()
    }
    #[napi(setter)]
    pub fn set_settings_file(&mut self, value: String) {
        self.inner.settings_file = value.into();
    }
    #[napi(getter)]
    pub fn options_file(&self) -> String {
        self.inner.options_file.to_string_lossy().into_owned()
    }
    #[napi(setter)]
    pub fn set_options_file(&mut self, value: String) {
        self.inner.options_file = value.into();
    }
    #[napi(getter)]
    pub fn default_library_dirs(&self) -> Vec<String> {
        self.inner
            .default_library_dirs
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect()
    }
    #[napi(setter)]
    pub fn set_default_library_dirs(&mut self, value: Vec<String>) {
        self.inner.default_library_dirs = value.into_iter().map(Into::into).collect();
    }
}

/// Locate the selected library, including unavailable configured paths.
#[napi]
pub fn locate_library() -> Result<LibraryLocation> {
    locate::locate_library()
        .map(Into::into)
        .map_err(crate::util::binding_error)
}

/// Discover installed Rekordbox programs, newest version first.
#[napi]
pub fn find_programs() -> Result<Vec<ProgramLocation>> {
    locate::find_programs()
        .map(|values| values.into_iter().map(Into::into).collect())
        .map_err(crate::util::binding_error)
}

/// Discover Rekordbox programs beneath explicit roots without consulting the environment.
#[napi]
pub fn find_programs_in(roots: Vec<String>) -> Result<Vec<ProgramLocation>> {
    locate::find_programs_in(roots.into_iter().map(Into::into))
        .map(|values| values.into_iter().map(Into::into).collect())
        .map_err(crate::util::binding_error)
}

/// Return the platform application-data base directory without checking existence.
#[napi]
pub fn app_dir() -> Result<String> {
    locate::app_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(crate::util::binding_error)
}

/// Return an existing database in the platform default locations.
#[napi]
pub fn default_master_db_path() -> Result<String> {
    locate::default_master_db_path()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(crate::util::binding_error)
}

/// Return the selected database path, requiring an existing file.
#[napi]
pub fn master_db_path() -> Result<String> {
    locate::master_db_path()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(crate::util::binding_error)
}
