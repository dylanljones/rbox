# rbox-onelib

Access Rekordbox One Library (Device Library Plus) export databases through
the `OneLibrary` handler or use its Diesel schema and row models directly.

> [!NOTE]
> This crate is part of [rbox](https://crates.io/crates/rbox), available through `rbox::onelib`.

`OneLibrary::new(directory)` opens `exportLibrary.db` in the supplied directory,
usually `PIONEER/rekordbox`. Use `OneLibrary::from_path(file)` for an explicit
database filename. `create` and `create_if_not_exists` take explicit file paths.

Includes relationship loading, ordered collection operations, SQLCipher
connection helpers, and schema initialization for new databases. The optional
`master-db` feature enables conversions from master database models.

The default `bundled-vendored-openssl` feature builds SQLCipher and OpenSSL.
The `bundled` feature builds SQLCipher using a system OpenSSL installation.
