# rbox-testing

Shared test utilities for the rbox workspace. This crate is unpublished and
intended for use as a path-only development dependency.

`rbox_testing::data` provides fixture paths and temporary copies using filesystem
utilities. Fixture-based tests require the repository's `.testdata` directory.

Enable the `database` feature to use `rbox_testing::ordering`, the shared ordering
contract tests for database models. This adds Diesel and `rbox-db` dependencies;
fixture-only consumers do not need them.

```toml
[dev-dependencies]
rbox-testing = { path = "../rbox-testing", features = ["database"] }
```

Part of [rbox](https://crates.io/crates/rbox).
