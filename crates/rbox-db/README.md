# rbox-db

Shared database infrastructure for rbox's Rekordbox database handlers.

> [!NOTE]
> This crate is part of [rbox](https://crates.io/crates/rbox), available through `rbox::db`.

Provides Diesel SQLite/SQLCipher connection options, connection and pool setup,
model traits, relationship loading, and ordering operations. Database-specific
schemas and models live in `rbox-masterdb` and `rbox-onelib`.

The `bundled` and `bundled-vendored-openssl` features control SQLCipher builds.
No features are enabled by default.
