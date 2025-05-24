# rbox

> **⚠️ Disclaimer**: This project is **not** affiliated with Pioneer DJ, AlphaTheta Corp., or any related entities.
> Rekordbox-IO is an independent project released under the **MIT license**.
> The maintainers and contributors assume no liability for any data loss or damage to your Rekordbox library.
> "Rekordbox" is a registered trademark of AlphaTheta Corporation.


**rbox** is a high-performance Rust library for seamlessly interacting with Pioneer DJ's Rekordbox software data.
It provides a comprehensive toolkit for DJs, developers, and music enthusiasts to read, analyze, and export data from
multiple Rekordbox formats with speed and precision. ✨

- 💾 **Database Access**: Query and update the Rekordbox v6/v7 master.db database through a type-safe ORM
- 📊 **XML Support**: Import and export data from Rekordbox XML collection files
- 📈 **Analysis Files**: Read and write ANLZ files containing waveforms, beat grids, hot cues and more
- ⚙️ **Settings Access**: Read and write My-Setting files (Coming soon!)

#### 🌐 Language support

rbox is built with Rust for maximum performance and safety, with additional language bindings:
- **🦀 Rust**: The core library, providing fast and efficient access to Rekordbox data
- **🐍 Python**: Integrate with Python applications for easy data manipulation
- **✨ JavaScript**: Integrate with electron applications and Node.js environments
- **💻 Command Line**: Command line interface for quick access to your data (Comming soon!)


## 🔧 Installation

rbox can be used as a library in your Rust project or through its powerful command-line interface.

<details>
<summary>🦀 Rust</summary>

Run the following Cargo command in your project directory:
```bash
cargo add rbox
```
or add the following line to your Cargo.toml:
```bash
rbox = "0.1.0"
```
</details>

<details>
<summary>🐍 Python</summary>

rbox is available on PyPI:

```bash
pip install rbox
```
</details>

<details>
<summary>✨ JavaScript</summary>

rbox is available on npm:

```bash
npm install rbox
```

</details>

<details>
<summary>💻 Command Line</summary>

Coming soon!

</details>

If you want to build the Rust library, you need a OpenSSL installation and have to set a environment variable to
the directory it is installed in:
````
OPENSSL_DIR = C:\Program Files\OpenSSL-Win64
````

## 🚀 Quick-Start

> [!CAUTION]
> Please make sure to back up your Rekordbox collection before making changes to rekordbox data.
> The backup dialog can be found under "File" > "Library" > "Backup Library"


### Rekordbox 6/7 database

Rekordbox 6 and 7 use a SQLite database for storing the collection content.
Unfortunatly, the `master.db` SQLite database is encrypted using
[SQLCipher][sqlcipher], which means it can't be used without the encryption key.
However, since your data is stored and used locally, the key must be present on the
machine running Rekordbox.

rbox can unlock the new Rekordbox `master.db` SQLite database and provides
an easy interface for accessing the data stored in it:

<details>
<summary>🦀 Rust</summary>

```rust
use rbox::masterdb::MasterDb;

const PATH: &str = r"C:\Users\user\AppData\Roaming\Pioneer\rekordbox\master.db";

fn main() -> anyhow::Result<()> {
    let mut db = MasterDb::new(PATH)?;
    let contents = db.get_content()?;
    for content in contents {
        println!("{:?}", content);
    }
    Ok(())
}
```
</details>

<details>
<summary>🐍 Python</summary>

```python
from rbox.masterdb import MasterDb

path = r"C:\Users\user\AppData\Roaming\Pioneer\rekordbox\master.db"

db = MasterDb(path)
contents = db.get_content()
for content in contents:
    print(content)
```
</details>

<details>
<summary>✨ JavaScript</summary>

```javascript
import { MasterDb } from 'rbox'

const path = "C:\\Users\\user\\AppData\\Roaming\\Pioneer\\rekordbox\\master.db"

const db = new MasterDb(path);
const contents = db.getContent();
for (const content of contents) {
    console.log(content)
}
```
</details>


[sqlcipher]: https://www.zetetic.net/sqlcipher/open-source/
