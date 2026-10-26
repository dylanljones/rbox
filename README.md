
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
Its goal is one comprehensive, general-purpose package that other projects can
build on. It currently supports the following Rekordbox files:

- **Rekordbox database**: Query and update the Rekordbox v6/v7 master.db database through a type-safe ORM
- **One Library**: Read and write Rekordbox One Library (Device Library Plus) export files
- **Device Library**: Read and write Rekordbox DeviceSql (.pdb) database files
- **Analysis Files**: Read and write ANLZ files containing waveforms, beat grids, hot cues and more
- **Settings Access**: Read and write My-Setting files
- **XML database**: Read and write Rekordbox XML database files

| File               | Read | Update | Create |
|:-------------------|:----:|-------:|-------:|
| Rekordbox database |  ✓  |     ✓ |     ❌ |
| One Library        |  ✓  |     ✓ |     ✓ |
| Device Library     |  ✓  |     ✓ |     ✓ |
| Analysis Files     |  ✓  |     ✓ |     ✓ |
| Settings Files     |  ✓  |     ✓ |     ✓ |
| XML database       |  ✓  |     ✓ |     ✓ |


> [!TIP]
> This project is still being actively developed and is in early stages, so expect some rough edges,
> API changes in the future and missing features.
> If you are looking for a more mature and battle-tested library, check the original [pyrekordbox] Python library.


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
rbox = "0.2.0"
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


## FAQ

<details>
<summary>
Can rbox analyze audio and generate analysis files?
</summary>

> Not yet. `rbox` can read and modify existing analysis files, but it cannot currently generate
> analysis data from audio. A goal of the project is to support audio processing and generate
> Rekordbox-compatible analysis files, including waveforms. This is planned work, with no release
> timeline yet.

</details>

<details>

<summary>
Can I use rbox to prepare device exports without Rekordbox?
</summary>

> Yes. `rbox` can create and update both original Device Library (`.pdb`) and One Library
> (Device Library Plus) databases, using existing analysis files for the export. A complete
> workflow starting from unanalysed audio still requires analysis-file generation, which is
> a project goal.

</details>


<details>
<summary>
Does rbox support Pro DJ Link?
</summary>

> Not yet. Pro DJ Link support is planned for the future, but there is no release timeline yet.

</details>


## Thank you

Thank you to everyone contributing to rbox through code, bug reports, testing,
documentation, and ideas. Your contributions and shared knowledge help make
these tools better for the community.

A special thank you to everyone who has contributed to [pyrekordbox]. Your code,
bug reports, testing, documentation, and ideas helped build the foundation that
rbox builds on today.

We are also grateful to the projects that have documented and reverse-engineered
Rekordbox's formats and shared their findings openly. Their documentation,
reference implementations, and sample files have been invaluable:

- [Deep Symmetry's crate-digger](https://github.com/Deep-Symmetry/crate-digger) and
  [Rekordbox Export Analysis](https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/)
  for detailed format documentation and binary schemas.
- [rekordcrate](https://github.com/Holzhaus/rekordcrate) for its extensive work on Rekordbox's binary formats.
- [fourfour](https://github.com/morizkraemer/fourfour) for additional export-format
  research and implementation references.

## Sponsor

If rbox has helped you or saved you time, consider supporting its development - every coffee makes a difference!

[![BuyMeACoffee](https://raw.githubusercontent.com/pachadotdev/buymeacoffee-badges/main/bmc-white.svg)](https://www.buymeacoffee.com/dylanljones)


## License

This project is licensed under the
[GNU General Public License, version 3.0][LICENSE].

Earlier releases may have been available under different terms.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you shall be licensed under the same terms as
this project, without any additional terms or conditions.


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
[LICENSE]: https://github.com/dylanljones/rbox/blob/main/LICENSE
[CHANGELOG]: https://github.com/dylanljones/rbox/blob/main/CHANGELOG.md

[pyrekordbox]: https://github.com/dylanljones/pyrekordbox
