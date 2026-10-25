# rbox-pdb

Read, create, and edit Rekordbox DeviceSQL databases (`export.pdb` and `exportExt.pdb`).
Includes lazy reads, typed models, transactional binary edits, and selective page writes.

> [!NOTE]
> This crate is part of [rbox](https://crates.io/crates/rbox) (available through `rbox::pdb` soon).

> [!WARNING]
> This crate is still experimental and may contain bugs.
>
> Testing by the community would be appreciated.
> Back up existing databases and test generated exports on your hardware before relying on them.

## Design

- `binary`: on-disk structures and codecs, preserving unknown fields.
- `devicesql`: storage engine for lazy reads, caching, transactions, binary edits, and file writes.
- `models`: high-level abstraction layer over `devicesql`, following the masterdb and OneLibrary pattern.

## CDJ output tests

CDJ model families that read Device Library exports.
Unchecked means untested, not incompatible.

- [ ] CDJ-350
- [ ] CDJ-850
- [ ] CDJ-900
- [ ] CDJ-900NXS
- [ ] CDJ-2000
- [x] CDJ-2000NXS
- [ ] CDJ-2000NXS2
- [ ] CDJ-3000 (see [CDJ-3000 firmware note][firmware])


Community testing on additional hardware would be greatly appreciated; please share your device model, firmware version, and results.


Model references: [USB export compatibility][compatibility].
Hardware checks above cover the tested export, not every database operation.

[compatibility]: https://rekordbox.com/en/support/usb-export/
[firmware]: https://www.pioneerdj.com/en/news/2026/cdj-3000-firmware-ver330-important-notice
