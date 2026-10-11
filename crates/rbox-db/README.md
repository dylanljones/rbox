# rbox-db

Shared database infrastructure for rbox's Rekordbox database handlers.

Provides Diesel SQLite/SQLCipher connection options, connection and pool setup,
model traits, relationship loading, and ordering operations. Database-specific
schemas and models live in `rbox-masterdb` and `rbox-onelib`.

The `bundled` and `bundled-vendored-openssl` features control SQLCipher builds.
No features are enabled by default.

Part of [rbox](../../README.md).
