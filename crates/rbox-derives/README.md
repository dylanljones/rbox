# rbox-derives

Procedural macros for rbox's Python bindings.

> [!NOTE]
> This crate is part of [rbox](https://crates.io/crates/rbox), primarily intended for its binding implementation.

The `PyMutableMapping` derive implements Python mapping operations for PyO3
models with named fields. Generated iterator code uses the helpers in
`rbox-core` with its `pyo3` feature enabled.
