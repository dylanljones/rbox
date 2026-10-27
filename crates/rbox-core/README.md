# rbox-core

Shared utilities for locating and working with Rekordbox data.

> [!NOTE]
> This crate is part of [rbox](https://crates.io/crates/rbox), available through `rbox::core`.

Includes program and library discovery, `RekordboxOptions`, process lookup,
artwork and analysis paths, and the `PathExt` trait for normalizing paths and
converting them to Unix paths or strings.

Shared database enums are available without Diesel. Enable the optional `diesel`
feature for their SQLite integer conversions; `rbox-masterdb` and `rbox-onelib` enable it automatically.
