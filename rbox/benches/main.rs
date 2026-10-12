// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Main entry point for the benchmark tests.

use criterion::criterion_main;

mod benchmarks;

#[cfg(feature = "anlz")]
criterion_main! {
    benchmarks::options::benches,
    benchmarks::anlz::benches,
}

#[cfg(not(feature = "anlz"))]
criterion_main! {
    benchmarks::options::benches,
}
