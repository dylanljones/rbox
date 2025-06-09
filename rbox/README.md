# rbox

> **⚠️ Disclaimer**: This project is **not** affiliated with Pioneer DJ, AlphaTheta Corp., or any related entities.
> rbox is an independent project released under the **MIT license**.
> The maintainers and contributors assume no liability for any data loss or damage to your Rekordbox library.
> "Rekordbox" is a registered trademark of AlphaTheta Corporation.

**rbox** is a high-performance Rust library for seamlessly interacting with Pioneer DJ's Rekordbox software data.
It supports the following Rekordbox files:

- **Rekordbox database**: Query and update the Rekordbox v6/v7 master.db database through a type-safe ORM
- **XML database**: Read and write Rekordbox XML database files
- **Analysis Files**: Read and write ANLZ files containing waveforms, beat grids, hot cues and more
- **Settings Access**: Read and write My-Setting files (Coming soon!)

## 🔧 Installation

rbox is available on Cargo. Run the following Cargo command in your project directory:
```shell
cargo add rbox
```
or add the following line to your Cargo.toml:
```shell
rbox = "0.1.2"
```

## 🚀 Quick-Start

> **❗ Caution**:
> Please make sure to back up your Rekordbox collection before making changes to rekordbox data.
> The backup dialog can be found under "File" > "Library" > "Backup Library"

Rekordbox 6 and 7 use a SQLite database for storing the collection content.
Unfortunatly, the `master.db` SQLite database is encrypted using
[SQLCipher][sqlcipher], which means it can't be used without the encryption key.
However, since your data is stored and used locally, the key must be present on the
machine running Rekordbox.

rbox can unlock the new Rekordbox `master.db` SQLite database and provides
an easy interface for accessing the data stored in it:

```rust
use rbox::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut db = MasterDb::open()?;
    let contents = db.get_content()?;
    for content in contents {
        println!("{:?}", content);
    }
    Ok(())
}
```

[sqlcipher]: https://www.zetetic.net/sqlcipher/open-source/
