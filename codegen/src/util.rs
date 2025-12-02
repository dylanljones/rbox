// Author: Dylan Jones
// Date:   2025-11-22

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

pub fn walk_rust_files<P: AsRef<Path>>(root: P) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let mut parents = VecDeque::new();
    parents.push_back(root.as_ref().to_path_buf());
    while let Some(parent) = parents.pop_front() {
        let entries = std::fs::read_dir(parent)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                parents.push_back(path.clone());
            } else if path.is_file() {
                if path.extension().unwrap() == "rs" {
                    files.push(path);
                }
            }
        }
    }
    Ok(files)
}
