<p align="center">
  <h1 align="center">rbox</h1>
</p>

<p align="center">
  <i>rbox</i> gives you full control over your Rekordbox data.
</p>

<p align="center">
  <a title="GitHub" target="_blank" href="https://github.com/dylanljones/rbox"><img alt="Crates.io" src="https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github" style="height:20px;"></a>
  <a title="Rust" target="_blank" href="https://crates.io/crates/rbox"><img alt="Crates.io" src="https://img.shields.io/crates/v/rbox.svg?style=for-the-badge&color=fc8d62&logo=rust" style="height:20px;"></a>
  <a title="Docs" target="_blank" href="https://docs.rs/rbox"><img src="https://img.shields.io/badge/docs.rs-rbox-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" style="height:20px;"></a>
</p>

> **⚠️ Disclaimer**: This project is **not** affiliated with Pioneer DJ, AlphaTheta Corp., or any related entities.
> The maintainers and contributors assume no liability for any data loss or damage to your Rekordbox library.
> "Rekordbox" is a registered trademark of AlphaTheta Corporation.

**rbox** is a high-performance Rust library for seamlessly interacting with Pioneer DJ's Rekordbox software data.
It supports the following Rekordbox files:

- **Rekordbox database**: Query and update the Rekordbox v6/v7 master.db database through a type-safe ORM
- **One Library**: Read and write Rekordbox One Library (Device Library Plus) export files
- **Device Library**: Read and write Rekordbox DeviceSql (.pdb) database files
- **Analysis Files**: Read and write ANLZ files containing waveforms, beat grids, hot cues and more
- **Settings Access**: Read and write My-Setting files
- **XML database**: Read and write Rekordbox XML database files

## 🔧 Installation

rbox is available on Cargo. Run the following Cargo command in your project directory:
```shell
cargo add rbox
```
or add the following line to your Cargo.toml:
```toml
rbox = "0.2.0"
```

### Features

The library is organized into modular components that can be enabled individually:

- `full` (default): Enables all component features
- `master-db`: Access to the Rekordbox 6/7 master.db database (enables `anlz` feature)
- `one-library`: Access to the One Library (Device Library Plus) database
- `device-library`: Access to the Device Library (DeviceSQL) database
- `anlz`: Support for ANLZ analysis files
- `xml`: Support for Rekordbox XML database files
- `settings`: Support for My-Setting files

To use only selected components, disable the default features and enable the ones
you need. For example, to use `rbox::anlz`:

```toml
rbox = { version = "0.2.0", default-features = false, features = ["anlz"] }
```

Shared core APIs remain available. Add more component names to `features` as needed.

#### SQLCipher Features

The `master-db` and `one-library` features require SQLCipher for working with Rekordbox's
encrypted database files.
To make building easier, there are two optional features for choosing SQLCipher's installation:

- `bundled`: Uses a bundled version of SQLCipher.
  This searches for and links against a system-installed crypto library to provide the crypto implementation.
- `bundled-vendored-openssl` (default): Uses `bundled` with a vendored version of OpenSSL.
  This is the most portable option and requires no system dependencies but increases compilation time.
  On Windows, a Perl installation such as [Strawberry Perl](https://strawberryperl.com/) is required
  to build OpenSSL.

For example, to enable only One Library support and build both SQLCipher and OpenSSL:

```toml
rbox = { version = "0.2.0", default-features = false, features = ["one-library", "bundled-vendored-openssl"] }
```

Replace `bundled-vendored-openssl` with `bundled` to use a system OpenSSL installation.
Keep `default-features = false`: Cargo features are additive, so enabling `bundled`
alone does not disable the default vendored OpenSSL build. Omitting both build
features requires a system SQLCipher installation.

#### Language Bindings

The library can be compiled with bindings for other programming languages.
These features are used in the bindings and *should not be enabled manually*:

- `pyo3`: Enables Python bindings
- `napi`: Enables Node.js/JavaScript bindings through N-API

## 🚀 Quick-Start

> **❗ Caution**:
> Please make sure to back up your Rekordbox collection before making changes to rekordbox data.
> The backup dialog can be found under "File" > "Library" > "Backup Library"

### Rekordbox 6/7 database

Rekordbox 6 and 7 use a SQLite database for storing the collection content.

rbox can unlock the new Rekordbox `master.db` SQLite database and provides
an easy interface for accessing and updating the data stored in it.

```rust
use rbox::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut db = MasterDb::open()?;
    let contents = db.get_contents()?;
    for content in contents {
        println!("{:?}", content);
    }
    Ok(())
}
```

### One Library (Device Library Plus)

For newer generation Pioneer DJ devices, Rekordbox exports a new library format to the USB storage
device (or SD card), called ``One Library`` (formerly known as ``Device Library Plus``).
As of 2025, this format is only supported by the [OPUS-QUAD], [OMNIS-DUO], [XDJ-AZ] and [CDJ-3000X] devices.
The database schema is similar to the main Rekordbox database. It contains a selection of tables
from the main database, with similar columns and data types.

rbox can unlock the new Rekordbox `exportLibrary.db` One Library database and provides
an easy interface for accessing the data stored in it:
```rust
use rbox::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut db = OneLibrary::new("exportLibrary.db")?;
    let contents = db.get_contents()?;
    for content in contents {
        println!("{:?}", content);
    }
    Ok(())
}
```

### Device Library (DeviceSQL)

> [!WARNING]
> DeviceSQL support is experimental and may contain bugs. Back up existing databases
> and test generated exports on your hardware before relying on them.
> See the `rbox-devlib` crate for more information.

Rekordbox stores Device Library exports in the binary `export.pdb` database and
an optional `exportExt.pdb` file under `PIONEER/rekordbox` on the export device.
Pass the database directory to `DeviceLibrary::new`. Pages are loaded on demand,
and typed models provide access to tracks, playlists, artists, and other tables:

```rust
use rbox::prelude::*;
use rbox::devlib::models::Content;

fn main() -> anyhow::Result<()> {
    let mut db = DeviceLibrary::new("PIONEER/rekordbox")?;
    let contents = Content::all(db.standard_mut())?;
    for content in contents {
        println!("{:?}", content);
    }
    Ok(())
}
```

Use `DeviceLibrary::create` with `LibraryCreateOptions` to initialize a new pair
of database files. Edits stay in memory until explicitly saved: `save()` replaces
changed files, `save_pages()` writes changed pages, and `save_as(directory)` copies
the databases to a new destination.

### Rekordbox ANLZ files

Rekordbox stores analysis information of the tracks in the collection in specific files,
which also get exported to decives used by Pioneer professional DJ equipment. The files
have names like `ANLZ0000` and come with the extensions `.DAT`, `.EXT` or `.2EX`.
They include waveforms, beat grids (information about the precise time at which
each beat occurs), time indices to allow efficient seeking to specific positions
inside variable bit-rate audio streams, and lists of memory cues and loop points.

rbox indexes analysis files on opening and decodes sections on demand. Getters
return `Result` because reading a section can fail after opening the file.
Use `get_*` for shared access and `get_mut_*` for mutable access. Saving preserves
section order and copies untouched sections without decoding them.

For finer access, use `get_section(index)` or `get_mut_section(index)`.
`into_data()` consumes the handler and materializes an `AnlzData`.
Keep the backing file unchanged externally while the handler is open.

```rust
use rbox::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut anlz = Anlz::load("ANLZ0000.DAT")?;
    let grid = anlz.get_beat_grid()?.unwrap();
    for beat in &grid.beats {
        println!("Beat: {} Tempo: {} Time: {}", beat.beat_number, beat.tempo, beat.time);
    }
    Ok(())
}
```


### Rekordbox My-Settings

Rekordbox stores the user settings in `*SETTING.DAT` files, which get exported to USB
devices. These files are either in the `PIONEER`directory of a USB drive
(device exports), but are also present for on local installations of Rekordbox 6.
The setting files store the settings found on the "DJ System" > "My Settings" page of
the Rekordbox preferences. These include language, LCD brightness, tempo fader range,
crossfader curve and other settings for Pioneer professional DJ equipment.

rbox supports both parsing and writing of My-Setting files.

```rust
use rbox::prelude::*;
use rbox::settings::Quantize;

fn main() -> anyhow::Result<()> {
    let mut sett = Setting::load("MYSETTING.DAT")?;
    println!("Quantize: {}", sett.get_quantize()?);
    sett.set_quantize(Quantize::Off)?;
    sett.dump()?;
    Ok(())
}
```

### Rekordbox XML

The Rekordbox XML database is used for importing (and exporting) Rekordbox collections
including track metadata and playlists. They can also be used to share playlists
between two databases.

rbox can read and write Rekordbox XML databases.

```rust
use rbox::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut xml = RekordboxXml::load("database.xml");
    let tracks = xml.get_tracks();
    for track in tracks {
        println!("{:?}", track);
    }
    Ok(())
}
```


## License

This project is licensed under the
[GNU General Public License, version 3.0][LICENSE].

Earlier releases may have been available under different terms.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you shall be licensed under the same terms as
this project, without any additional terms or conditions.


## Sponsor

If rbox has helped you or saved you time, consider supporting its development - every coffee makes a difference!

[![BuyMeACoffee](https://raw.githubusercontent.com/pachadotdev/buymeacoffee-badges/main/bmc-white.svg)](https://www.buymeacoffee.com/dylanljones)

[sqlcipher]: https://www.zetetic.net/sqlcipher/open-source/
[LICENSE]: https://github.com/dylanljones/rbox/blob/main/LICENSE

[OPUS-QUAD]: https://www.pioneerdj.com/en/product/all-in-one-system/opus-quad/black/overview/
[OMNIS-DUO]: https://alphatheta.com/en/product/all-in-one-dj-system/omnis-duo/indigo/
[XDJ-AZ]: https://alphatheta.com/en/product/all-in-one-dj-system/xdj-az/black/
[CDJ-3000X]: https://alphatheta.com/en/product/player/cdj-3000x/black/
