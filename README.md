
<p align="center">
  <h1 align="center">rbox</h1>
</p>

<p align="center">
  <i>rbox</i> gives you full control over your Rekordbox data.
</p>

<p align="center">
  <a title="Rust" target="_blank" href="https://crates.io/crates/rbox"><img alt="Crates.io" src="https://img.shields.io/crates/v/rbox?style=flat-square"></a>
  <a title="Python" target="_blank" href="https://pypi.org/project/rbox/"><img alt="PyPI" src="https://img.shields.io/pypi/v/rbox?style=flat-square"></a>
  <a title="JavaScript" target="_blank" href="https://www.npmjs.com/package/rbox-js"><img alt="JS" src="https://img.shields.io/npm/v/rbox-js?style=flat-square"></a>
  <a title="MIT" target="_blank" href="https://github.com/dylanljones/rbox/blob/main/LICENSE"><img src="http://img.shields.io/badge/license-MIT-lightgrey.svg?style=flat-square"></a>
</p>

---

> **⚠️ Disclaimer**: This project is **not** affiliated with Pioneer DJ, AlphaTheta Corp., or any related entities.
> rbox is an independent project released under the **MIT license**.
> The maintainers and contributors assume no liability for any data loss or damage to your Rekordbox library.
> "Rekordbox" is a registered trademark of AlphaTheta Corporation.

**rbox** is a high-performance Rust library for seamlessly interacting with Pioneer DJ's Rekordbox software data.
It supports the following Rekordbox files:

- **Rekordbox database**: Query and update the Rekordbox v6/v7 master.db database through a type-safe ORM
- **XML database**: Read and write Rekordbox XML database files
- **Analysis Files**: Read and write ANLZ files containing waveforms, beat grids, hot cues and more
- **Settings Access**: Read and write My-Setting files

#### 🌐 Language support

rbox is built with Rust for maximum performance and safety, with additional language bindings:
- **🦀 [Rust](rbox/README.md)**: The core library, providing fast and efficient access to Rekordbox data
- **🐍 [Python](bindings/python/README.md)**: Integrate with Python applications for easy data manipulation
- **✨ [JavaScript](bindings/javascript/README.md)**: Integrate with electron applications and Node.js environments


## 🔧 Installation

### 🦀 Rust

rbox is available on [crates.io]. Run the following Cargo command in your project directory:
```shell
cargo add rbox
```
or add the following line to your Cargo.toml:
```shell
rbox = "0.1.2"
```

### 🐍 Python

rbox is available on [PyPI][pypi]. You can install it using pip:

```shell
pip install rbox
```

The Python bindings can also be built manually via [pyo3] and [maturin]:
```shell
cd bindings/python
pip install -r requirements.txt
maturin develop
```

### ✨ JavaScript

rbox is available on [npm]:

```shell
npm install rbox-js
```
or
```shell
yarn add rbox-js
```

The JS bindings can also be built manually via [napi-rs]:
```shell
cd bindings/javascript
yarn install
yarn build
```

[crates.io]: https://crates.io/crates/rbox
[pypi]: https://pypi.org/project/rbox/
[npm]: https://www.npmjs.com/package/rbox-js

[pyo3]: https://github.com/PyO3/pyo3
[maturin]: https://github.com/PyO3/maturin
[napi-rs]: https://napi.rs/
