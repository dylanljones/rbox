// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! End-to-end artwork preparation, including decoding, resizing, encoding, and writes.
//! Inputs and temporary directories are prepared outside the timed loop. Each
//! iteration overwrites three files; this measures warm filesystem writes, not
//! durable USB throughput. No real artwork or Rekordbox library is modified.
//! JPEG and PNG originals are copied unchanged; only thumbnails are JPEG-encoded.

use std::hint::black_box;
use std::io::Cursor;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use image::{DynamicImage, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};
use rbox_core::artwork::save_cover_variants;

fn input(width: u32, height: u32, format: ImageFormat, alpha: bool) -> Vec<u8> {
    // Gradients, edges, and fine detail rather than a trivially uniform image.
    let rgb = RgbImage::from_fn(width, height, |x, y| {
        Rgb([
            (x * 255 / width) as u8,
            (y * 255 / height) as u8,
            ((x ^ y) & 255) as u8,
        ])
    });
    let image = if alpha {
        DynamicImage::ImageRgba8(RgbaImage::from_fn(width, height, |x, y| {
            let pixel = rgb.get_pixel(x, y);
            Rgba([pixel[0], pixel[1], pixel[2], ((x + y) & 255) as u8])
        }))
    } else {
        DynamicImage::ImageRgb8(rgb)
    };
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, format).unwrap();
    bytes.into_inner()
}

fn artwork(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_cover_variants");
    group.sample_size(20);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    for (name, format, alpha) in [
        ("jpeg", ImageFormat::Jpeg, false),
        ("png_rgb", ImageFormat::Png, false),
        ("png_rgba", ImageFormat::Png, true),
    ] {
        for (width, height) in [(600, 600), (3000, 2000)] {
            let bytes = input(width, height, format, alpha);
            let directory = tempfile::tempdir().unwrap();
            group.bench_with_input(
                BenchmarkId::new(name, format!("{width}x{height}")),
                &bytes,
                |b, bytes| {
                    b.iter(|| {
                        black_box(
                            save_cover_variants(directory.path(), black_box(bytes), 90).unwrap(),
                        )
                    });
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, artwork);
criterion_main!(benches);
