// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

pub mod file;
pub mod sections;
mod string;
mod xor;

pub use file::Anlz;
pub use sections::*;
pub use string::AnlzString;

use crate::error::Result;

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
            match extension.to_ascii_uppercase().to_str().unwrap() {
                "DAT" => dat = Some(file.clone()),
                "EXT" => ext = Some(file.clone()),
                "2EX" => ex2 = Some(file.clone()),
                &_ => {}
            }
        }
    }
    if dat.is_none() {
        return Ok(None);
    }
    let dat = dat.unwrap();
    let files = AnlzPaths { dat, ext, ex2 };
    Ok(Some(files))
}
