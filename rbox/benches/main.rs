// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::options::benches,
    benchmarks::anlz::benches,
}
