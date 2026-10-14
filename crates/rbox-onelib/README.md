# rbox-onelib

Access Rekordbox One Library (Device Library Plus) export databases through
the `OneLibrary` handler or use its Diesel schema and row models directly.

Includes relationship loading, ordered collection operations, SQLCipher
connection helpers, and schema initialization for new databases. The optional
`master-db` feature enables conversions from master database models.

The default `bundled-vendored-openssl` feature builds SQLCipher and OpenSSL.
The `bundled` feature builds SQLCipher using a system OpenSSL installation.

Part of [rbox](https://crates.io/crates/rbox), also available through `rbox::one_library`.
