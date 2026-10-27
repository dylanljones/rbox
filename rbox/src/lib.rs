// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! [![github]](https://github.com/dylanljones/rbox)&ensp;[![crates-io]](https://crates.io/crates/rbox)&ensp;[![docs-rs]](https://docs.rs/rbox)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! <br>
//!
//! > rbox gives you full control of your Rekordbox data.
//!
//! <br>
//!
//! This crate provides a comprehensive set of tools to interact with Rekordbox data, including
//! - [`MasterDb`] for managing the local Rekordbox database.
//! - [`OneLibrary`] for managing One Library (Device Library Plus) SQLite databases.
//! - [`Anlz`] for reading and writing ANLZ files.
//! - [`RekordboxXml`] for parsing and manipulating Rekordbox XML files.
//! - [`Setting`] for managing Rekordbox settings.
//!
//! > **❗ Caution**:
//! > Please make sure to back up your Rekordbox collection before making changes to rekordbox data.
//! > The backup dialog can be found under "File" > "Library" > "Backup Library"
//!
//! ## Sub-crate documentation
//!
//! rbox re-exports the following crates. Their documentation covers the underlying APIs in detail.
//!
//! | Crate | Purpose |
//! | --- | --- |
//! | [`rbox-core`](rbox_core) | Shared track models, paths, library discovery, and options |
//! | [`rbox-db`](rbox_db) | Database connections, model traits, ordering, and relationships |
//! | [`rbox-masterdb`](rbox_masterdb) | Rekordbox `master.db`, playlists, and smart lists |
//! | [`rbox-onelib`](rbox_onelib) | One Library (Device Library Plus) export databases |
//! | [`rbox-anlz`](rbox_anlz) | Analysis files, waveforms, beat grids, and cues |
//! | [`rbox-xml`](rbox_xml) | Rekordbox XML libraries |
//! | [`rbox-setting`](rbox_setting) | Rekordbox and device settings files |
//!
//! ## Features
//!
//! By default, rbox enables all file and database components (`full`) and builds SQLCipher
//! with vendored OpenSSL (`bundled-vendored-openssl`). Core models, paths, and library
//! discovery are always available, including with `default-features = false`.
//!
//! | Feature | Effect | Enabled by default |
//! | --- | --- | --- |
//! | `full` | Enables `master-db`, `one-library`, `anlz`, `xml`, and `settings` | Yes |
//! | `master-db` | Enables `masterdb`, shared `db` APIs, and `anlz`; adds master database integration to `onelib` when it is also enabled | Yes, via `full` |
//! | `one-library` | Enables `onelib` and shared `db` APIs | Yes, via `full` |
//! | `anlz` | Enables analysis file APIs in `anlz` | Yes, via `full` |
//! | `xml` | Enables XML library APIs in `xml` | Yes, via `full` |
//! | `settings` | Enables settings file APIs in `settings` | Yes, via `full` |
//! | `bundled` | Builds SQLCipher from source without vendoring OpenSSL | No |
//! | `bundled-vendored-openssl` | Builds both SQLCipher and OpenSSL from source | Yes |
//! | `pyo3` | Enables Python binding support in core and enabled database components | No |
//! | `napi` | Enables JavaScript binding support in core and enabled database components | No |
//!
//! The SQLCipher build features require `master-db` or `one-library`; they do not enable
//! either component themselves. With database support enabled and neither build feature
//! selected, a system SQLCipher installation is required. Binding features also configure
//! only the components already enabled. `full` does not select a SQLCipher build strategy
//! or enable binding support.
//!
//! Disable the defaults to select only the components and build options you need:
//!
//! ```toml
//! [dependencies]
//! # Analysis and XML files only; no database dependencies.
//! rbox = { version = "0.2", default-features = false, features = ["anlz", "xml"] }
//! ```
//!
//! For example, use `features = ["master-db", "bundled-vendored-openssl"]` with defaults
//! disabled for master database support, its required analysis APIs, and bundled native
//! dependencies. Cargo features are additive: selecting `bundled` alone does not turn off
//! the default `bundled-vendored-openssl` feature.
//!
//! ## Rekordbox 6/7 database
//!
//! rbox can unlock the new Rekordbox `master.db` SQLite database and provides
//! an easy interface for accessing and updating the data stored in it.
//!
//! ```no_run
//! use rbox::prelude::*;
//!
//! fn main() -> anyhow::Result<()> {
//!     let mut db = MasterDb::open()?;
//!     let contents = db.get_contents()?;
//!     for content in contents {
//!         println!("{:?}", content);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Rekordbox XML
//!
//! rbox can read and write Rekordbox XML databases.
//!
//! ```no_run
//! use rbox::prelude::*;
//!
//! fn main() -> anyhow::Result<()> {
//!     let mut xml = RekordboxXml::load("database.xml")?;
//!     let tracks = xml.get_tracks();
//!     for track in tracks {
//!         println!("{:?}", track);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Rekordbox ANLZ files
//!
//! rbox lazily reads all three analysis formats. Opening indexes section headers;
//! getters decode sections on demand and can return I/O or parsing errors.
//! Shared `get_*` access leaves sections untouched; mutable `get_mut_*` access marks
//! them for serialization. Saves stream all other sections from the source file
//! in their original order. Do not edit that file externally while it is open.
//! Use `get_section(index)` / `get_mut_section(index)` for low-level access,
//! or `into_data()` to materialize the complete file.
//!
//! ```no_run
//! use rbox::prelude::*;
//!
//! fn main() -> anyhow::Result<()> {
//!     let mut anlz = Anlz::load("ANLZ0000.DAT")?;
//!     let grid = anlz.get_beat_grid()?.unwrap();
//!     for beat in &grid.beats {
//!         println!("Beat: {} Tempo: {} Time: {}", beat.beat_number, beat.tempo, beat.time);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Rekordbox Settings
//!
//! rbox supports both parsing and writing of Setting files
//!
//! ```no_run
//! use rbox::prelude::*;
//! use rbox::settings::Quantize;
//!
//! fn main() -> anyhow::Result<()> {
//!     let mut sett = Setting::load("MYSETTING.DAT")?;
//!     println!("Quantize: {}", sett.get_quantize()?);
//!     sett.set_quantize(Quantize::Off)?;
//!     sett.dump()?;
//!     Ok(())
//! }
//! ```
//!

// Compile-time feature validation
#[cfg(all(
    any(feature = "bundled", feature = "bundled-vendored-openssl"),
    not(any(feature = "master-db", feature = "one-library"))
))]
compile_error!(
    "The 'bundled' or 'bundled-vendored-openssl' features require either 'master-db' or 'one-library' to be enabled"
);

pub mod error;
pub mod prelude;
pub use error::{Error, Result};

#[cfg(feature = "anlz")]
pub use rbox_anlz as anlz;
pub use rbox_core as core;
#[cfg(any(feature = "master-db", feature = "one-library"))]
pub use rbox_db as db;
#[cfg(feature = "master-db")]
pub use rbox_masterdb as masterdb;
#[cfg(feature = "one-library")]
pub use rbox_onelib as onelib;

/// Compatibility namespace for One Library APIs. Use [`onelib`] instead.
#[cfg(feature = "one-library")]
#[deprecated(since = "0.2.0", note = "use `rbox::onelib` instead")]
pub mod one_library {
    pub use rbox_onelib::*;
}

#[cfg(feature = "device-library")]
pub use rbox_devlib as devlib;
#[cfg(feature = "settings")]
pub use rbox_setting as settings;
#[cfg(feature = "xml")]
pub use rbox_xml as xml;

// Core exports
pub use core::*;

// anlz exports
#[cfg(feature = "anlz")]
pub use anlz::Anlz;

// shared db exports
#[cfg(any(feature = "master-db", feature = "one-library"))]
pub use db::{
    DbConn, DbPool, Model, ModelDelete, ModelInsert, ModelList, ModelTree, ModelUpdate, NodeRef,
    OrderedModel, TreeNode, TreeSeq,
};

// masterdb exports
#[cfg(feature = "master-db")]
pub use masterdb::{MasterDb, MasterPlaylistXml, SmartList};

// onelib exports
#[cfg(feature = "one-library")]
pub use onelib::OneLibrary;

// devlib exports
#[cfg(feature = "device-library")]
pub use devlib::DeviceLibrary;

// settings exports
#[cfg(feature = "settings")]
pub use settings::Setting;

// xml exports
#[cfg(feature = "xml")]
pub use xml::RekordboxXml;
