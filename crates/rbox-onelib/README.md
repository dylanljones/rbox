# rbox-onelib

Access Rekordbox One Library (Device Library Plus) export databases through
the `OneLibrary` handler or use its Diesel schema and row models directly.

> [!NOTE]
> This crate is part of [rbox](https://crates.io/crates/rbox), available through `rbox::onelib`.

> [!WARNING]
> This crate is still experimental and may contain bugs.
>
> Testing by the community would be appreciated.
> Back up existing databases and test generated exports on your hardware before relying on them.

`OneLibrary::new(directory)` opens `exportLibrary.db` in the supplied directory,
usually `PIONEER/rekordbox`. Use `OneLibrary::from_path(file)` for an explicit
database filename. `create` and `create_if_not_exists` take explicit file paths.

Includes relationship loading, ordered collection operations, SQLCipher
connection helpers, and schema initialization for new databases. The optional
`master-db` feature enables conversions from master database models.

The default `bundled-vendored-openssl` feature builds SQLCipher and OpenSSL.
The `bundled` feature builds SQLCipher using a system OpenSSL installation.

## Device tests

Device model families that read One Library exports.
Unchecked means untested, not incompatible.

- [ ] CDJ-3000X
- [ ] CDJ-1500X
- [ ] CDJ-3000 (see [CDJ-3000 firmware note][firmware])
- [ ] OPUS-QUAD
- [ ] OMNIS-DUO
- [ ] XDJ-AZ
- [ ] XDJ-AN

Community testing on additional hardware would be greatly appreciated; please share your device model, firmware version, and results.

Model references: [USB export compatibility][compatibility].
Hardware checks above cover the tested export, not every database operation.

[compatibility]: https://rekordbox.com/en/support/usb-export/
[firmware]: https://www.pioneerdj.com/en/news/2026/cdj-3000-firmware-ver330-important-notice
