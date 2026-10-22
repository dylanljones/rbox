# rbox-xml

Read, edit, and write Rekordbox XML collection exports.

> [!NOTE]
> This crate is part of [rbox](https://crates.io/crates/rbox), available through `rbox::xml`.

The `RekordboxXml` handler manages tracks and playlist trees, with models for
track metadata, cue points, and beat grids. Parsing, file access, and invalid
operations are reported through `XmlError`.
