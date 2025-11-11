// Author: Dylan Jones
// Date:   2025-11-02

#[path = "../../tests/common/mod.rs"]
mod common;

use criterion::{criterion_group, Criterion};
use std::hint::black_box;

use rbox::RekordboxOptions;

fn options(c: &mut Criterion) {
    let file = common::testdata_options_path();

    c.bench_function("read_options", |b| {
        b.iter(|| RekordboxOptions::load(black_box(&file)).unwrap())
    });
}

criterion_group!(benches, options);
