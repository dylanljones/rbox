// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Compare lazy and eager ANLZ operations using shared fixtures and temporary outputs.

use binrw::{BinRead, BinWrite};
use criterion::{criterion_group, Criterion};
use std::fs::File;
use std::hint::black_box;

use rbox_testing::data;

use rbox::anlz::sections::AnlzData;
use rbox::anlz::{self, Anlz};

fn anlz(c: &mut Criterion) {
    let mut group = c.benchmark_group("anlz");
    group.measurement_time(std::time::Duration::from_secs(35));

    let root = data::anlz_dir(6);
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

    group.bench_function("eager_ext", |b| {
        b.iter(|| AnlzData::read(&mut File::open(black_box(&ext_file)).unwrap()).unwrap())
    });
    group.bench_function("lazy_ext_path", |b| {
        b.iter(|| {
            Anlz::load(black_box(&ext_file))
                .unwrap()
                .get_path()
                .unwrap()
        })
    });
    group.bench_function("eager_ext_path", |b| {
        b.iter(|| {
            let data = AnlzData::read(&mut File::open(black_box(&ext_file)).unwrap()).unwrap();
            black_box(data.sections.iter().find(|s| s.tag == anlz::AnlzTag::Path));
        })
    });
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("copy.EXT");
    group.bench_function("lazy_ext_edit_save", |b| {
        b.iter(|| {
            let mut data = Anlz::load(black_box(&ext_file)).unwrap();
            data.set_path("/Music/benchmark.flac").unwrap();
            data.dump_copy(&output).unwrap();
        })
    });
    group.bench_function("eager_ext_edit_save", |b| {
        b.iter(|| {
            let mut data = AnlzData::read(&mut File::open(black_box(&ext_file)).unwrap()).unwrap();
            for section in &mut data.sections {
                if let anlz::sections::Content::Path(p) = &mut section.content {
                    p.set("/Music/benchmark.flac");
                    break;
                }
            }
            let mut file = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
            data.write(&mut file).unwrap();
            file.as_file().sync_all().unwrap();
            file.persist(&output).unwrap();
        })
    });
}

criterion_group!(benches, anlz);
