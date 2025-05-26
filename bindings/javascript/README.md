# rbox JavaScript binding

> **⚠️ Disclaimer**: This project is **not** affiliated with Pioneer DJ, AlphaTheta Corp., or any related entities.
> rbox is an independent project released under the **MIT license**.
> The maintainers and contributors assume no liability for any data loss or damage to your Rekordbox library.
> "Rekordbox" is a registered trademark of AlphaTheta Corporation.

## 🔧 Installation

rbox-js is available on npm:

```bash
npm install rbox-js
```

The JS bindings can also be built manually via [napi-rs]:
```bash
yarn build
```

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

```javascript
import { MasterDb } from 'rbox-js'

const db = MasterDb.open();
const contents = db.getContent();
for (const content of contents) {
    console.log(content)
}
```

[napi-rs]: https://napi.rs/
[sqlcipher]: https://www.zetetic.net/sqlcipher/open-source/
