
<p align="center">
  <h1 align="center">rbox</h1>
</p>

<p align="center">
  <i>rbox</i> gives you full control over your Rekordbox data.
</p>

<p align="center">
  <a title="Tests" target="_blank" href="https://github.com/dylanljones/rbox/actions/workflows/tests.yml"><img src="https://img.shields.io/github/actions/workflow/status/dylanljones/rbox/tests.yml?style=for-the-badge&label=tests&labelColor=555555" style="height:20px;"></a>
  <a title="Rust" target="_blank" href="https://crates.io/crates/rbox"><img alt="Crates.io" src="https://img.shields.io/crates/v/rbox.svg?style=for-the-badge&color=fc8d62&logo=rust" style="height:20px;"></a>
  <a title="Python" target="_blank" href="https://pypi.org/project/rbox/"><img alt="PyPI" src="https://img.shields.io/pypi/v/rbox.svg?style=for-the-badge&color=fc8d62&logo=python" style="height:20px;"></a>
  <a title="JavaScript" target="_blank" href="https://www.npmjs.com/package/rbox-js"><img alt="JS" src="https://img.shields.io/npm/v/rbox-js.svg?style=for-the-badge&color=fc8d62&logo=npm" style="height:20px;"></a>
  <a title="Docs" target="_blank" href="https://docs.rs/rbox"><img src="https://img.shields.io/badge/docs.rs-rbox-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" style="height:20px;"></a>
</p>

---

> [!IMPORTANT]
> This project is **not** affiliated with Pioneer DJ, AlphaTheta Corp., or any related entities.
> The maintainers and contributors assume no liability for any data loss or damage to your Rekordbox library.
> "Rekordbox" is a registered trademark of AlphaTheta Corporation.

**rbox** is a high-performance Rust library for seamlessly interacting with Pioneer DJ's Rekordbox software data.
It supports the following Rekordbox files:

- **Rekordbox database**: Query and update the Rekordbox v6/v7 master.db database through a type-safe ORM
- **One Library**: Read and write Rekordbox One Library (Device Library Plus) export files
- **XML database**: Read and write Rekordbox XML database files
- **Analysis Files**: Read and write ANLZ files containing waveforms, beat grids, hot cues and more
- **Settings Access**: Read and write My-Setting files

| File               | Read | Update | Create |
|:-------------------|:----:|-------:|-------:|
| Rekordbox database |  ✓   |      ✓ |      ❌ |
| One Library        |  ✓   |      ✓ |      ✓ |
| XML database       |  ✓   |      ✓ |      ✓ |
| Analysis Files     |  ✓   |      ✓ |      ❌ |
| Settings Files     |  ✓   |      ✓ |      ✓ |


#### Language support

rbox is built with Rust for maximum performance and safety, with additional language bindings:
- **🦀 [Rust](rbox)**: The core library, providing fast and efficient access to Rekordbox data
- **🐍 [Python](bindings/python)**: Integrate with Python applications for easy data manipulation
- **✨ [JavaScript](bindings/javascript)**: Integrate with electron applications and Node.js environments


## Installation

### 🦀 Rust

rbox is available on [crates.io]. Run the following Cargo command in your project directory:
```shell
cargo add rbox
```
or add the following line to your Cargo.toml:
```shell
rbox = "0.1.5"
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

Licensed under either of [Apache License, Version 2.0][LICENSE-APACHE] or [MIT license][LICENSE-MIT] at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.


## FAQ

<details>
<summary>
Can I run the analysis provided by Rekordbox with rbox?
</summary>

> No, `rbox` is only an interface to Rekordbox's data. It can read and write Rekordbox analysis
> files (waveforms, beat grids, hot cues, etc.), but it does not implement or run Rekordbox’s audio
> analysis algorithms.

</details>

<details>

<summary>
Can I use rbox to prepare device exports without Rekordbox?
</summary>

> Although `rbox` does support the new One Library (Device Library Plus) export format, the library
> cannot run the analysis provided by Rekordbox.
> Once files have been analyzed, the analysis data can be exported to the device.

</details>

<details>
<summary>
Does this library support the old Device Library export files?
</summary>

> `rbox` does not support the old device library export files and support is not planned anytime soon.
>
> Check out [rekordcrate](https://github.com/Holzhaus/rekordcrate) for a library focused more on export files
> and their awesome work of reverse engineering the `.pdb` format.

</details>



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
[CONTRIBUTING]: https://github.com/dylanljones/rbox/blob/main/CONTRIBUTING.md
[LICENSE-MIT]: https://github.com/dylanljones/rbox/blob/main/LICENSE-MIT
[LICENSE-APACHE]: https://github.com/dylanljones/rbox/blob/main/LICENSE-APACHE
[CHANGELOG]: https://github.com/dylanljones/rbox/blob/main/CHANGELOG.md
