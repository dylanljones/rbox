// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

#[path = "../../tests/common/mod.rs"]
mod common;

use criterion::{criterion_group, Criterion};
use std::hint::black_box;

use rbox::RekordboxOptions;

fn options(c: &mut Criterion) {
    let file = common::options_path(6);

    c.bench_function("read_options", |b| {
        b.iter(|| RekordboxOptions::load(black_box(&file)).unwrap())
    });
}

criterion_group!(benches, options);
