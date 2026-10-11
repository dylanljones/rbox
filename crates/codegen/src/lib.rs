// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Rust source inspection for code generation.
//!
//! Parses source files into structured descriptions of modules, types, and
//! functions, and provides a file walker for discovering Rust sources.

pub mod models;
mod parser;
mod util;

pub use models::*;
pub use parser::{parse_rust_file, parse_rust_source};
pub use util::walk_rust_files;
