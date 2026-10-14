# rbox-masterdb

Access Rekordbox's local `master.db` database through the `MasterDb` handler
or use its Diesel schema and row models for direct queries.

Includes relationship loading, ordered collection operations, SQLCipher
connection helpers, intelligent playlist definitions, and support for the
companion `masterPlaylists6.xml` file.

The default `bundled-vendored-openssl` feature builds SQLCipher and OpenSSL.
The `bundled` feature builds SQLCipher using a system OpenSSL installation.

Part of [rbox](https://crates.io/crates/rbox), also available through `rbox::masterdb`.
