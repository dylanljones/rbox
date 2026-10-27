<p align="center">
  <h1 align="center">rbox</h1>
</p>

<p align="center">
  <i>rbox</i> gives you full control over your Rekordbox data.
</p>

<p align="center">
  <a title="GitHub" target="_blank" href="https://github.com/dylanljones/rbox"><img alt="Crates.io" src="https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github" style="height:20px;"></a>
  <a title="Python" target="_blank" href="https://pypi.org/project/rbox/"><img alt="PyPI" src="https://img.shields.io/pypi/v/rbox.svg?style=for-the-badge&color=fc8d62&logo=python" style="height:20px;"></a>
  <a title="Docs" target="_blank" href="https://docs.rs/rbox"><img src="https://img.shields.io/badge/docs.rs-rbox-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" style="height:20px;"></a>
</p>


> **⚠️ Disclaimer**: This project is **not** affiliated with Pioneer DJ, AlphaTheta Corp., or any related entities.
> The maintainers and contributors assume no liability for any data loss or damage to your Rekordbox library.
> "Rekordbox" is a registered trademark of AlphaTheta Corporation.

**rbox** provides Python bindings to the Rust library for reading and modifying Rekordbox data.
It supports the following Rekordbox files:

- **Rekordbox database**: Query and update the Rekordbox v6/v7 master.db database through a type-safe ORM
- **One Library**: Read and write Rekordbox One Library (Device Library Plus) export files
- **Device Library**: Read and write Rekordbox DeviceSQL (.pdb) database files
- **Analysis Files**: Read and write ANLZ files containing waveforms, beat grids, hot cues and more
- **Settings Access**: Read and write My-Setting files
- **XML database**: Read and write Rekordbox XML database files

## 🔧 Installation

rbox is available on PyPI:
```bash
pip install rbox
```


## 🚀 Quick-Start

> **❗ Caution**:
> Please make sure to back up your Rekordbox collection before making changes to rekordbox data.
> The backup dialog can be found under "File" > "Library" > "Backup Library"

### Rekordbox 6/7 database

Rekordbox 6 and 7 use a SQLite database for storing the collection content.

rbox can unlock the new Rekordbox `master.db` SQLite database and provides
an easy interface for accessing and updating the data stored in it.

```python
from rbox import MasterDb

db = MasterDb.open()
for content in db.get_contents():
    print(content)
```

### One Library (Device Library Plus)

> **⚠️ Warning**: One Library support is experimental and may contain bugs. Back up existing databases
> and test generated exports on your hardware before relying on them.
> See [rbox-onelib](https://github.com/dylanljones/rbox/tree/main/crates/rbox-onelib) for more information.

For newer generation Pioneer DJ devices, Rekordbox exports a new library format to the USB storage
device (or SD card), called ``One Library`` (formerly known as ``Device Library Plus``).
As of 2026, this format is only supported by the [CDJ-3000X], [CDJ-1500X], [OPUS-QUAD], [OMNIS-DUO],
[XDJ-AZ] and [XDJ-AN] devices.
The database schema is similar to the main Rekordbox database. It contains a selection of tables
from the main database, with similar columns and data types.

rbox can unlock the new Rekordbox `exportLibrary.db` One Library database and provides
an easy interface for accessing the data stored in it:
```python
from rbox.onelib import OneLibrary

db = OneLibrary("PIONEER/rekordbox")
# For an explicit file: OneLibrary.from_path("backups/exportLibrary.db")
for content in db.get_contents():
    print(content)
```

### Device Library (DeviceSQL)

> **⚠️ Warning**: DeviceSQL support is experimental and may contain bugs. Back up existing databases
> and test generated exports on your hardware before relying on them.
> See [rbox-devlib](https://github.com/dylanljones/rbox/tree/main/crates/rbox-devlib) for more information.

Rekordbox stores Device Library exports in the binary `export.pdb` database and
an optional `exportExt.pdb` file under `PIONEER/rekordbox` on the export device.
Pass the database directory to `DeviceLibrary`. Pages are loaded on demand,
and typed models provide access to tracks, playlists, artists, and other tables:

```python
from rbox import DeviceLibrary

db = DeviceLibrary("PIONEER/rekordbox")
for content in db.get_contents():
    print(content)
```

### Rekordbox ANLZ files

Rekordbox stores analysis information of the tracks in the collection in specific files,
which also get exported to devices used by Pioneer professional DJ equipment. The files
have names like `ANLZ0000` and come with the extensions `.DAT`, `.EXT` or `.2EX`.
They include waveforms, beat grids (information about the precise time at which
each beat occurs), time indices to allow efficient seeking to specific positions
inside variable bit-rate audio streams, and lists of memory cues and loop points.

rbox indexes analysis files on opening and decodes sections on demand. Reading a
section can fail after opening, and a missing section returns `None`.
Getters return detached snapshots: apply edits with the corresponding setter,
such as `set_beat_grid(grid)`, then call `dump()` to save. Saving preserves section
order and copies untouched sections without decoding them.

For finer access, use `get_section(index)`. `into_data()` materializes a complete
snapshot while leaving the binding handler usable. Keep the backing file
unchanged externally while the handler is open.

```python
from rbox import Anlz

anlz = Anlz.load("ANLZ0000.DAT")
grid = anlz.get_beat_grid()
if grid is not None:
    for beat in grid.beats:
        print(f"Beat: {beat.beat_number} Tempo: {beat.tempo} Time: {beat.time}")
```


### Rekordbox My-Settings

Rekordbox stores the user settings in `*SETTING.DAT` files, which get exported to USB
devices. These files are either in the `PIONEER` directory of a USB drive
(device exports), but are also present for on local installations of Rekordbox.
The setting files store the settings found on the "DJ System" > "My Settings" page of
the Rekordbox preferences. These include language, LCD brightness, tempo fader range,
crossfader curve and other settings for Pioneer professional DJ equipment.

rbox supports both parsing and writing of My-Setting files.

```python
from rbox import Setting
from rbox.settings import Quantize

settings = Setting.load("MYSETTING.DAT")
print(f"Quantize: {settings.get_quantize()}")
settings.set_quantize(Quantize.Off)
settings.dump()
```

### Rekordbox XML

The Rekordbox XML database is used for importing (and exporting) Rekordbox collections
including track metadata and playlists. They can also be used to share playlists
between two databases.

rbox can read and write Rekordbox XML databases.

```python
from rbox import RekordboxXml

xml = RekordboxXml("database.xml")
for track in xml.get_tracks():
    print(track)
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

[LICENSE]: https://github.com/dylanljones/rbox/blob/main/LICENSE

[OPUS-QUAD]: https://www.pioneerdj.com/en/product/all-in-one-system/opus-quad/black/overview/
[OMNIS-DUO]: https://alphatheta.com/en/product/all-in-one-dj-system/omnis-duo/indigo/
[XDJ-AZ]: https://alphatheta.com/en/product/all-in-one-dj-system/xdj-az/black/
[XDJ-AN]: https://alphatheta.com/en/product/all-in-one-dj-system/xdj-an/black/
[CDJ-3000X]: https://alphatheta.com/en/product/player/cdj-3000x/black/
[CDJ-1500X]: https://alphatheta.com/en/product/player/cdj-1500x/black/
