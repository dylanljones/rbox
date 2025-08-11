// Author: Dylan Jones
// Date:   2025-05-01

//! [![github]](https://github.com/dylanljones/rbox)&ensp;[![crates-io]](https://crates.io/crates/rbox)&ensp;[![docs-rs]](https://docs.rs/rbox)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! <br>
//!
//! > rbox gives you full control over your Rekordbox data.
//!
//! <br>
//!
//! This crate provides a comprehensive set of tools to interact with Rekordbox data, including:
//! - [`MasterDb`] for managing the local Rekordbox database.
//! - [`Anlz`] for reading and writing ANLZ files.
//! - [`RekordboxXml`] for parsing and manipulating Rekordbox XML files.
//! - [`Setting`] for managing Rekordbox settings.
//!
//! > **❗ Caution**:
//! > Please make sure to back up your Rekordbox collection before making changes to rekordbox data.
//! > The backup dialog can be found under "File" > "Library" > "Backup Library"
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
//!     let contents = db.get_content()?;
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
//!     let mut xml = RekordboxXml::load("database.xml");
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
//! rbox can parse and write all three analysis files.
//!
//! ```no_run
//! use rbox::prelude::*;
//!
//! fn main() -> anyhow::Result<()> {
//!     let mut anlz = Anlz::load("ANLZ0000.DAT")?;
//!     let grid = anlz.get_beat_grid().unwrap();
//!     for beat in grid {
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

pub mod anlz;
pub mod masterdb;
mod options;
mod pathlib;
pub mod prelude;
pub mod settings;
pub mod util;
pub mod xml;

pub use anlz::{Anlz, AnlzTag};
pub use masterdb::MasterDb;
pub use options::RekordboxOptions;
pub use pathlib::NormalizePath;
pub use settings::Setting;
pub use util::*;
pub use xml::RekordboxXml;
