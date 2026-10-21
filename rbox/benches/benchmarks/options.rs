// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Benchmark options loading through the facade using the shared Rekordbox fixture.

use criterion::{criterion_group, Criterion};
use std::hint::black_box;

use rbox_testing::data;

use rbox::RekordboxOptions;

fn options(c: &mut Criterion) {
    let file = data::options_path(6);

    c.bench_function("read_options", |b| {
        b.iter(|| RekordboxOptions::load(black_box(&file)).unwrap())
    });
}

criterion_group!(benches, options);
