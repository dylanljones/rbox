<p align="center">
  <h1 align="center">rbox-js</h1>
</p>

<p align="center">
  <i>rbox</i> gives you full control over your Rekordbox data.
</p>

<p align="center">
  <a title="GitHub" target="_blank" href="https://github.com/dylanljones/rbox"><img alt="Crates.io" src="https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github" style="height:20px;"></a>
  <a title="JavaScript" target="_blank" href="https://www.npmjs.com/package/rbox-js"><img alt="JS" src="https://img.shields.io/npm/v/rbox-js.svg?style=for-the-badge&color=fc8d62&logo=npm" style="height:20px;"></a>
  <a title="Docs" target="_blank" href="https://docs.rs/rbox"><img src="https://img.shields.io/badge/docs.rs-rbox-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" style="height:20px;"></a>
</p>

> **⚠️ Disclaimer**: This project is **not** affiliated with Pioneer DJ, AlphaTheta Corp., or any related entities.
> The maintainers and contributors assume no liability for any data loss or damage to your Rekordbox library.
> "Rekordbox" is a registered trademark of AlphaTheta Corporation.

**rbox-js** provides JavaScript and TypeScript bindings to the Rust library for reading and modifying Rekordbox data.
It supports the following Rekordbox files:

- **Rekordbox database**: Query and update the Rekordbox v6/v7 master.db database through a type-safe ORM
- **One Library**: Read and write Rekordbox One Library (Device Library Plus) export files
- **Device Library**: Read and write Rekordbox DeviceSQL (.pdb) database files
- **Analysis Files**: Read and write ANLZ files containing waveforms, beat grids, hot cues and more
- **Settings Access**: Read and write My-Setting files
- **XML database**: Read and write Rekordbox XML database files

## 🔧 Installation

rbox-js is available on npm:
```shell
npm install rbox-js
```
or
```shell
yarn add rbox-js
```


## Quick-Start

> **❗ Caution**:
> Please make sure to back up your Rekordbox collection before making changes to rekordbox data.
> The backup dialog can be found under "File" > "Library" > "Backup Library"

### Rekordbox 6/7 database

Rekordbox 6 and 7 use a SQLite database for storing the collection content.

rbox can unlock the new Rekordbox `master.db` SQLite database and provides
an easy interface for accessing and updating the data stored in it.

```javascript
import { MasterDb } from 'rbox-js'

const db = MasterDb.open()
for (const content of db.getContents()) {
    console.log(content)
}
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

```javascript
import { OneLibrary } from 'rbox-js'

const db = new OneLibrary('PIONEER/rekordbox')
// For an explicit file: OneLibrary.fromPath('backups/exportLibrary.db')
for (const content of db.getContents()) {
    console.log(content)
}
```

### Device Library (DeviceSQL)

> **⚠️ Warning**: DeviceSQL support is experimental and may contain bugs. Back up existing databases
> and test generated exports on your hardware before relying on them.
> See [rbox-devlib](https://github.com/dylanljones/rbox/tree/main/crates/rbox-devlib) for more information.

Rekordbox stores Device Library exports in the binary `export.pdb` database and
an optional `exportExt.pdb` file under `PIONEER/rekordbox` on the export device.
Pass the database directory to `DeviceLibrary`. Pages are loaded on demand,
and typed models provide access to tracks, playlists, artists, and other tables:

```javascript
import { DeviceLibrary } from 'rbox-js'

const db = new DeviceLibrary('PIONEER/rekordbox')
for (const content of db.getContents()) {
    console.log(content)
}
```

### Rekordbox ANLZ files

Rekordbox stores analysis information of the tracks in the collection in specific files,
which also get exported to devices used by Pioneer professional DJ equipment. The files
have names like `ANLZ0000` and come with the extensions `.DAT`, `.EXT` or `.2EX`.
They include waveforms, beat grids (information about the precise time at which
each beat occurs), time indices to allow efficient seeking to specific positions
inside variable bit-rate audio streams, and lists of memory cues and loop points.

rbox indexes analysis files on opening and decodes sections on demand. Reading a
section can fail after opening, and a missing section returns `null`.
Getters return detached snapshots: apply edits with the corresponding setter,
such as `setBeatGrid(grid)`, then call `dump()` to save. Saving preserves section
order and copies untouched sections without decoding them.

For finer access, use `getSection(index)`. `intoData()` materializes a complete
snapshot while leaving the binding handler usable. Keep the backing file
unchanged externally while the handler is open.

```javascript
import { Anlz } from 'rbox-js'

const anlz = Anlz.load('ANLZ0000.DAT')
const grid = anlz.getBeatGrid()
if (grid !== null) {
    for (const beat of grid.beats) {
        console.log(`Beat: ${beat.beatNumber} Tempo: ${beat.tempo} Time: ${beat.time}`)
    }
}
```


### Rekordbox My-Settings

Rekordbox stores the user settings in `*SETTING.DAT` files, which get exported to USB
devices. These files are either in the `PIONEER` directory of a USB drive
(device exports), and are also present in local installations of Rekordbox.
The setting files store the settings found on the "DJ System" > "My Settings" page of
the Rekordbox preferences. These include language, LCD brightness, tempo fader range,
crossfader curve and other settings for Pioneer professional DJ equipment.

rbox supports both parsing and writing of My-Setting files.

```javascript
import { Setting, Quantize } from 'rbox-js'

const settings = Setting.load('MYSETTING.DAT')
console.log(`Quantize: ${settings.getQuantize()}`)
settings.setQuantize(Quantize.Off)
settings.dump()
```

### Rekordbox XML

The Rekordbox XML database is used for importing (and exporting) Rekordbox collections
including track metadata and playlists. They can also be used to share playlists
between two databases.

rbox can read and write Rekordbox XML databases.

```javascript
import { RekordboxXml } from 'rbox-js'

const xml = RekordboxXml.load('database.xml')
for (const track of xml.getTracks()) {
    console.log(track)
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

[LICENSE]: https://github.com/dylanljones/rbox/blob/main/LICENSE

[OPUS-QUAD]: https://www.pioneerdj.com/en/product/all-in-one-system/opus-quad/black/overview/
[OMNIS-DUO]: https://alphatheta.com/en/product/all-in-one-dj-system/omnis-duo/indigo/
[XDJ-AZ]: https://alphatheta.com/en/product/all-in-one-dj-system/xdj-az/black/
[XDJ-AN]: https://alphatheta.com/en/product/all-in-one-dj-system/xdj-an/black/
[CDJ-3000X]: https://alphatheta.com/en/product/player/cdj-3000x/black/
[CDJ-1500X]: https://alphatheta.com/en/product/player/cdj-1500x/black/
