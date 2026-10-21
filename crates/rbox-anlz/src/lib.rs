// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Lazy reading, editing, and writing of Rekordbox ANLZ analysis files.
//!
//! [`Anlz`] indexes section boundaries when opening a file and decodes individual
//! sections on demand. The binary section models are available from [`sections`],
//! while [`AnlzError`] describes filesystem, format, decoding, and writing failures.

pub mod error;
mod file;
pub mod sections;
mod string;
mod xor;

pub use error::{AnlzError, InvalidFileReason, Result};
pub use file::Anlz;
pub use sections::*;
pub use string::AnlzString;

/// Collection of ANLZ file paths.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnlzPaths {
    pub dat: std::path::PathBuf,
    pub ext: Option<std::path::PathBuf>,
    pub ex2: Option<std::path::PathBuf>,
}

/// Collection of ANLZ files.
pub struct AnlzFiles {
    pub dat: Anlz,
    pub ext: Option<Anlz>,
    pub ex2: Option<Anlz>,
}

/// Scan a directory for ANLZ files.
pub fn find_anlz_files<P: AsRef<std::path::Path>>(root: P) -> Result<Option<AnlzPaths>> {
    let root = root.as_ref();
    let mut dat: Option<std::path::PathBuf> = None;
    let mut ext: Option<std::path::PathBuf> = None;
    let mut ex2: Option<std::path::PathBuf> = None;

    let paths = std::fs::read_dir(root)?;
    for path in paths {
        let file = path?.path();
        let extension = file.extension();
        if let Some(extension) = extension {
            match extension.to_ascii_uppercase().to_str() {
                Some("DAT") => dat = Some(file.clone()),
                Some("EXT") => ext = Some(file.clone()),
                Some("2EX") => ex2 = Some(file.clone()),
                _ => {}
            }
        }
    }
    let Some(dat) = dat else {
        return Ok(None);
    };
    let files = AnlzPaths { dat, ext, ex2 };
    Ok(Some(files))
}
