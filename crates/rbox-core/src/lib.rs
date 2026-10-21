// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared Rekordbox configuration, library discovery, and program lookup.
//!
//! This crate supplies the filesystem and process information needed to find a
//! local Rekordbox installation. It does not open or decrypt databases and does
//! not modify Rekordbox's preferences.

pub mod artwork;
pub mod error;
pub mod locate;
pub mod options;
pub mod pathlib;
pub mod process;
pub mod share;

#[cfg(feature = "pyo3")]
#[doc(hidden)]
pub mod python;

pub use locate::{
    find_programs, find_programs_in, locate_library, options_json_path, rekordbox_agent_app_dir,
    rekordbox_app_dir, LibraryLocation,
};
pub use options::RekordboxOptions;
pub use pathlib::PathExt;
pub use process::{get_rekordbox_pid, is_rekordbox_running};
pub use share::{anlz_dir_local, anlz_dir_usb, artwork_dir_local};

pub mod enums;
pub mod models;
pub use models::{Track, TrackMetadata};

pub use enums::FileType;
