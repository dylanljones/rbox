# rbox-anlz

Read, edit, and write Rekordbox ANLZ analysis files (`DAT`, `EXT`, and `2EX`).

The `Anlz` handler indexes sections when opening a file and decodes them on
demand. It provides access to individual sections and preserves section order
and unchanged section bytes when writing.

Part of [rbox](../../README.md), also available through `rbox::anlz`.
