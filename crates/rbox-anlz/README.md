# rbox-anlz

Read, edit, and write Rekordbox ANLZ analysis files (`DAT`, `EXT`, and `2EX`).

> [!NOTE]
> This crate is part of [rbox](https://crates.io/crates/rbox), available through `rbox::anlz`.

The `Anlz` handler indexes sections when opening a file and decodes them on
demand. It provides access to individual sections and preserves section order
and unchanged section bytes when writing.
