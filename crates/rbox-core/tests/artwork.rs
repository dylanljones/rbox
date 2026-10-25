// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Artwork output contracts: preserved originals, padding, transparency, and resize fidelity.

use std::fs;
use std::io::Cursor;

use image::imageops::{overlay, FilterType};
use image::{DynamicImage, ImageBuffer, ImageFormat, Luma, LumaA, Rgb, RgbImage, Rgba, RgbaImage};
use rbox_core::artwork::{save_cover_variants, ArtworkVariant};

fn encode(image: &DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, format).unwrap();
    bytes.into_inner()
}

#[test]
fn jpeg_original_is_byte_identical_and_thumbnails_are_centered() -> anyhow::Result<()> {
    for (width, height) in [(320, 160), (160, 320), (31, 31), (1, 300), (300, 1)] {
        let temp = tempfile::tempdir()?;
        let input =
            DynamicImage::ImageRgb8(RgbImage::from_pixel(width, height, Rgb([220, 90, 40])));
        let bytes = encode(&input, ImageFormat::Jpeg);
        let paths = save_cover_variants(temp.path(), &bytes, 90)?;
        assert_eq!(paths.original, temp.path().join("artwork.jpg"));
        assert_eq!(fs::read(paths.original)?, bytes);
        for (path, size) in [(paths.medium, 240), (paths.small, 80)] {
            let output = image::open(path)?.to_rgb8();
            assert_eq!(output.dimensions(), (size, size));
            if width != height {
                assert!(output.get_pixel(0, 0).0.iter().all(|&c| c <= 3));
            }
            if width > 1 && height > 1 {
                let center = output.get_pixel(size / 2, size / 2);
                for (actual, expected) in center.0.into_iter().zip([220u8, 90, 40]) {
                    assert!(actual.abs_diff(expected) <= 4);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn png_original_is_preserved_and_only_thumbnails_composite_alpha() -> anyhow::Result<()> {
    for alpha in [0, 128, 255] {
        let temp = tempfile::tempdir()?;
        let input =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(320, 160, Rgba([200, 100, 50, alpha])));
        let bytes = encode(&input, ImageFormat::Png);
        let paths = save_cover_variants(temp.path(), &bytes, 100)?;
        assert_eq!(paths.original, temp.path().join("artwork.png"));
        assert_eq!(fs::read(&paths.original)?, bytes);
        assert!(!temp.path().join("artwork.jpg").exists());
        assert!(matches!(
            ArtworkVariant::try_from_path(&paths.original),
            Some(ArtworkVariant::Original)
        ));
        for (path, dimensions) in [(paths.medium, (240, 240)), (paths.small, (80, 80))] {
            let bytes = fs::read(path)?;
            assert_eq!(image::guess_format(&bytes)?, ImageFormat::Jpeg);
            let output = image::load_from_memory(&bytes)?.to_rgb8();
            assert_eq!(output.dimensions(), dimensions);
            let pixel = output.get_pixel(dimensions.0 / 2, dimensions.1 / 2);
            for (actual, channel) in pixel.0.into_iter().zip([200u16, 100, 50]) {
                let expected = (channel * u16::from(alpha) / 255) as u8;
                assert!(actual.abs_diff(expected) <= 3, "{pixel:?}, alpha={alpha}");
            }
        }
    }
    Ok(())
}

#[test]
fn thumbnails_remain_close_to_reference_lanczos() -> anyhow::Result<()> {
    // Compare against the previous floating-point Lanczos pipeline. Allow small
    // rounding differences after JPEG encoding, but catch filtering/alpha regressions.
    for (width, height) in [(601, 399), (39, 61)] {
        for alpha in [false, true] {
            let input = DynamicImage::ImageRgba8(RgbaImage::from_fn(width, height, |x, y| {
                Rgba([
                    (x % 256) as u8,
                    (y % 256) as u8,
                    ((x ^ y) % 256) as u8,
                    if alpha { ((x + y) % 256) as u8 } else { 255 },
                ])
            }));
            let input = if alpha {
                input
            } else {
                DynamicImage::ImageRgb8(input.to_rgb8())
            };
            let fitted = input.resize(240, 240, FilterType::Lanczos3).to_rgba8();
            let mut square = RgbaImage::from_pixel(240, 240, Rgba([0, 0, 0, 255]));
            overlay(
                &mut square,
                &fitted,
                i64::from((240 - fitted.width()) / 2),
                i64::from((240 - fitted.height()) / 2),
            );
            let medium = DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(square).into_rgb8());
            let small = medium.resize(80, 80, FilterType::Lanczos3);
            let temp = tempfile::tempdir()?;
            let bytes = encode(&input, ImageFormat::Png);
            let paths = save_cover_variants(temp.path(), &bytes, 90)?;
            assert_eq!(fs::read(paths.original)?, bytes);
            for (path, reference) in [(paths.medium, medium), (paths.small, small)] {
                let actual = image::open(path)?.to_rgb8();
                let mut encoded = Vec::new();
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded, 90)
                    .encode_image(&reference)?;
                let expected = image::load_from_memory(&encoded)?.to_rgb8();
                let mean_error = actual
                    .as_raw()
                    .iter()
                    .zip(expected.as_raw())
                    .map(|(a, b)| f64::from(a.abs_diff(*b)))
                    .sum::<f64>()
                    / actual.as_raw().len() as f64;
                assert!(
                    mean_error < 2.0,
                    "mean channel error {mean_error}, {width}x{height}, alpha={alpha}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn invalid_input_is_rejected_before_creating_outputs() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let output = temp.path().join("absent");
    assert!(save_cover_variants(&output, b"not an image", 90).is_err());
    assert!(!output.exists());
    let input = encode(&DynamicImage::new_rgb8(8, 8), ImageFormat::Png);
    assert!(save_cover_variants(&output, &input[..16], 90).is_err());
    assert!(!output.exists());
    assert!(save_cover_variants(&output, &input, 101).is_err());
    assert!(!output.exists());
    Ok(())
}

#[test]
fn grayscale_and_high_bit_depth_pngs_keep_their_color_conversion() -> anyhow::Result<()> {
    for input in [
        DynamicImage::ImageLuma8(ImageBuffer::from_pixel(17, 29, Luma([100]))),
        DynamicImage::ImageLumaA8(ImageBuffer::from_pixel(17, 29, LumaA([200, 128]))),
        DynamicImage::ImageRgb16(ImageBuffer::from_pixel(17, 29, Rgb([25700, 12850, 51400]))),
        DynamicImage::ImageRgba16(ImageBuffer::from_pixel(
            17,
            29,
            Rgba([51400, 25700, 12850, 32896]),
        )),
    ] {
        let temp = tempfile::tempdir()?;
        let rgba = input.to_rgba8();
        let expected = rgba.get_pixel(0, 0).0;
        let bytes = encode(&input, ImageFormat::Png);
        let paths = save_cover_variants(temp.path(), &bytes, 100)?;
        assert_eq!(fs::read(paths.original)?, bytes);
        for (path, dimensions) in [(paths.medium, (240, 240)), (paths.small, (80, 80))] {
            let output = image::open(path)?.to_rgb8();
            assert_eq!(output.dimensions(), dimensions);
            let actual = output.get_pixel(dimensions.0 / 2, dimensions.1 / 2);
            for (actual, channel) in actual.0.into_iter().zip(expected[..3].iter().copied()) {
                let expected = (u16::from(channel) * u16::from(expected[3]) / 255) as u8;
                assert!(actual.abs_diff(expected) <= 3);
            }
        }
    }
    Ok(())
}

#[test]
fn other_formats_still_convert_the_original_to_jpeg() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let input = DynamicImage::ImageRgb8(RgbImage::from_pixel(17, 29, Rgb([100, 50, 200])));
    let paths = save_cover_variants(temp.path(), &encode(&input, ImageFormat::Bmp), 100)?;
    assert_eq!(paths.original, temp.path().join("artwork.jpg"));
    let bytes = fs::read(paths.original)?;
    assert_eq!(image::guess_format(&bytes)?, ImageFormat::Jpeg);
    let output = image::load_from_memory(&bytes)?.to_rgb8();
    assert_eq!(output.dimensions(), (17, 29));
    for (actual, expected) in output.get_pixel(8, 14).0.into_iter().zip([100u8, 50, 200]) {
        assert!(actual.abs_diff(expected) <= 3);
    }
    Ok(())
}
