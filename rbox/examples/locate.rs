// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

// Run via `cargo run --color=always --example locate --profile dev`

use rbox::{default_master_db_path, rekordbox_agent_app_dir, RekordboxOptions};

fn main() -> anyhow::Result<()> {
    let agent_dir = rekordbox_agent_app_dir();
    if agent_dir.is_err() {
        println!("Failed to locate Rekordbox agent directory");
        return Ok(());
    }
    let agent_dir = agent_dir?;
    println!("Rekordbox agent directory: {:?}", agent_dir);
    println!();

    println!("Reading options.json...");
    let opts = RekordboxOptions::open()?;
    println!("Database path:      {:?}", opts.db_path);
    println!("Analysis root:      {:?}", opts.analysis_root);
    println!("Settings root:      {:?}", opts.settings_root);
    println!("Database directory: {:?}", opts.get_db_dir());

    println!("");
    println!("Default locations:");
    let db_path = default_master_db_path();
    println!("Default database path: {:?}", db_path);
    Ok(())
}
