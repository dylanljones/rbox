
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

#### Language support

rbox is built with Rust for maximum performance and safety, with additional language bindings:
- **🦀 [Rust](rbox/README.md)**: The core library, providing fast and efficient access to Rekordbox data
- **🐍 [Python](bindings/python/README.md)**: Integrate with Python applications for easy data manipulation
- **✨ [JavaScript](bindings/javascript/README.md)**: Integrate with electron applications and Node.js environments


## Installation

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

The JS bindings can also be built manually via [napi-rs]:
```shell
cd bindings/javascript
yarn install
yarn build
```


## Development

If you encounter an issue or want to contribute to rbox, please feel free to get in touch,
[open an issue][issue] or create a new [pull request][pulls]! A guide for contributing to
rbox and the commit-message style can be found in
[CONTRIBUTING].

For general questions or discussions about the library or Rekordbox,
please use [GitHub Discussions][discussions] instead of opening an issue.


## License

This project is licensed under the [MIT license][LICENSE].

## Sponsor

If rbox has helped you or saved you time, consider supporting its development - every coffee makes a difference!

[![BuyMeACoffee](https://raw.githubusercontent.com/pachadotdev/buymeacoffee-badges/main/bmc-white.svg)](https://www.buymeacoffee.com/dylanljones)


[crates.io]: https://crates.io/crates/rbox
[pypi]: https://pypi.org/project/rbox/
[npm]: https://www.npmjs.com/package/rbox-js

[pyo3]: https://github.com/PyO3/pyo3
[maturin]: https://github.com/PyO3/maturin
[napi-rs]: https://napi.rs/

[issue]: https://github.com/dylanljones/rbox/issues
[pulls]: https://github.com/dylanljones/rbox/pulls
[discussions]: https://github.com/dylanljones/rbox/discussions
[CONTRIBUTING]: https://github.com/dylanljones/pyrekordbox/blob/main/CONTRIBUTING.md
[LICENSE]: https://github.com/dylanljones/pyrekordbox/blob/main/LICENSE
[CHANGELOG]: https://github.com/dylanljones/pyrekordbox/blob/main/CHANGELOG.md
