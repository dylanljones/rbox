# rbox-derives

Procedural macros for rbox's Python and JavaScript bindings.

> [!NOTE]
> This crate is part of [rbox](https://crates.io/crates/rbox), primarily intended for its binding implementation.

The `PyMutableMapping` derive implements Python mapping operations for PyO3
models with named fields. Generated iterator code uses the helpers in
`rbox-core` with its `pyo3` feature enabled.


`PyConstructor` calls the Rust constructor with the required fields and applies
optional public-field keyword arguments, preserving defaults for omitted fields.

`NapiPublic` generates plain JavaScript objects from public Rust fields while
keeping private state native. Output models can also generate a partial-update
input with readonly fields excluded.
