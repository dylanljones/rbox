// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

#[path = "../../tests/common/mod.rs"]
mod common;

use criterion::{criterion_group, Criterion};
use std::hint::black_box;

use rbox::anlz::Anlz;

fn anlz(c: &mut Criterion) {
    let mut group = c.benchmark_group("anlz");
    group.measurement_time(std::time::Duration::from_secs(35));

    let root = common::anlz_dir(6);
    let anlz_dir = root.join("e35").join("fa187-3f34-47e2-9880-2b33cb8d1304");

    let dat_file = anlz_dir.join("ANLZ0000.DAT");
    let ext_file = anlz_dir.join("ANLZ0000.EXT");
    let ex2_file = anlz_dir.join("ANLZ0000.2EX");

    group.bench_function("read_anlz_dat", |b| {
        b.iter(|| Anlz::load(black_box(&dat_file)).unwrap())
    });

    group.bench_function("read_anlz_ext", |b| {
        b.iter(|| Anlz::load(black_box(&ext_file)).unwrap())
    });

    group.bench_function("read_anlz_ex2", |b| {
        b.iter(|| Anlz::load(black_box(&ex2_file)).unwrap())
    });
}

criterion_group!(benches, anlz);
