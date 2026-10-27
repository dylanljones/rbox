// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Preserve JPEG/PNG originals and save centered square JPEG artwork thumbnails.

use fast_image_resize::{ResizeOptions, Resizer};
use image::codecs::jpeg::JpegEncoder;
use image::error::{ImageError, ParameterError, ParameterErrorKind};
use image::imageops::{overlay, FilterType};
use image::{DynamicImage, ImageEncoder, ImageFormat, RgbImage, Rgba, RgbaImage};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

// Background color for padding artwork images to a square
const BG: [u8; 3] = [0, 0, 0];

/// Represents the artwork variant.
pub enum ArtworkVariant {
    /// Full-resolution artwork, without resizing or padding.
    Original,
    /// Medium processed JPEG artwork file (240x240)
    Medium,
    /// Small processed JPEG artwork file (80x80)
    Small,
}

impl ArtworkVariant {
    /// Recognize standard artwork filenames, including JPEG and PNG originals.
    pub fn try_from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        match path.as_ref().file_name()?.to_str()? {
            "artwork.jpg" | "artwork.png" => Some(Self::Original),
            "artwork_m.jpg" => Some(Self::Medium),
            "artwork_s.jpg" => Some(Self::Small),
            _ => None,
        }
    }

    /// Returns the size of the artwork
    pub fn size(&self) -> Option<u32> {
        match self {
            Self::Original => None,
            Self::Medium => Some(240),
            Self::Small => Some(80),
        }
    }

    /// Returns the JPEG filename of the artwork variant.
    ///
    /// PNG originals use `artwork.png`; use [`ArtworkPaths::original`] for the
    /// actual original path returned by [`save_cover_variants`].
    pub fn name(&self) -> &'static str {
        match self {
            Self::Original => "artwork.jpg",
            Self::Medium => "artwork_m.jpg",
            Self::Small => "artwork_s.jpg",
        }
    }
}

/// Paths to the saved original and JPEG thumbnails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtworkPaths {
    /// Full-resolution original: `artwork.png` for PNG input, `artwork.jpg` otherwise.
    pub original: PathBuf,
    /// Square 240 by 240 pixel JPEG.
    pub medium: PathBuf,
    /// Square 80 by 80 pixel JPEG.
    pub small: PathBuf,
}

/// Save full-resolution artwork and aspect-preserving square JPEG thumbnails.
///
/// Pass original file bytes or `Picture::data()`, not decoded pixels. The original
/// JPEG or PNG input is copied byte for byte to `artwork.jpg` or `artwork.png`,
/// retaining metadata and PNG transparency/bit depth. Other formats are converted
/// to JPEG at their original dimensions, without padding or source metadata.
/// Thumbnails scale up or down to fit, center the image, and pad with black.
/// Transparent pixels in JPEG outputs are composited over black. Quality must be in `1..=100`
/// and affects thumbnails and originals that require conversion.
///
/// Creates the directory and overwrites matching filenames. Invalid image data
/// or quality is rejected before writing. I/O failures can leave partial outputs.
/// Other files (including old originals with different extensions) are left alone.
pub fn save_cover_variants<P: AsRef<Path>>(
    output_dir: P,
    bytes: &[u8],
    quality: u8,
) -> Result<ArtworkPaths, ImageError> {
    use ArtworkVariant::*;
    if quality > 100 {
        return Err(ImageError::Parameter(ParameterError::from_kind(
            ParameterErrorKind::Generic("JPEG quality must be in 1..=100".into()),
        )));
    }
    let format = image::guess_format(bytes)?;
    let image = image::load_from_memory_with_format(bytes, format)?;
    let output_dir = output_dir.as_ref();

    let paths = ArtworkPaths {
        original: output_dir.join(if format == ImageFormat::Png {
            "artwork.png"
        } else {
            Original.name()
        }),
        medium: output_dir.join(Medium.name()),
        small: output_dir.join(Small.name()),
    };

    std::fs::create_dir_all(output_dir)?;
    if matches!(format, ImageFormat::Jpeg | ImageFormat::Png) {
        std::fs::write(&paths.original, bytes)?;
    } else {
        // Borrow decoded RGB/RGBA pixels: expanding an opaque RGB cover to RGBA
        // and converting it back used two unnecessary full-resolution buffers.
        match &image {
            DynamicImage::ImageRgb8(rgb) => save_jpeg(rgb, &paths.original, quality)?,
            DynamicImage::ImageRgba8(rgba) => {
                save_jpeg(&rgba_to_rgb_over_bg(rgba, BG), &paths.original, quality)?;
            }
            _ if !image.color().has_alpha() => {
                save_jpeg(&image.to_rgb8(), &paths.original, quality)?;
            }
            _ => {
                save_jpeg(
                    &rgba_to_rgb_over_bg(&image.to_rgba8(), BG),
                    &paths.original,
                    quality,
                )?;
            }
        }
    }

    // Pad and resize to square thumbnails. The original is not resized or padded.
    let mut resizer = Resizer::new();
    let medium = render_square_variant_rgb(&image, Medium.size().unwrap(), &mut resizer)?;
    save_jpeg(&medium, &paths.medium, quality)?;

    // Resize padded square thumbnails
    let size = Small.size().unwrap();
    let small = resize_artwork(&DynamicImage::ImageRgb8(medium), size, &mut resizer)?.into_rgb8();
    save_jpeg(&small, &paths.small, quality)?;

    Ok(paths)
}

/// Alpha-composite an RGBA image over a solid RGB background to produce an opaque RGB image.
///
/// Uses integer u16 arithmetic instead of floats and iterates the raw pixel buffer directly
/// to avoid per-pixel bounds-checking overhead. The common case of fully-opaque pixels
/// (alpha = 255) is handled as a fast path that copies channels without any multiply.
fn rgba_to_rgb_over_bg(rgba: &RgbaImage, bg: [u8; 3]) -> RgbImage {
    let (w, h) = rgba.dimensions();
    let rgba_buf = rgba.as_raw();
    let mut rgb_buf = Vec::with_capacity((w * h * 3) as usize);

    for chunk in rgba_buf.chunks_exact(4) {
        let (r, g, b, a) = (chunk[0], chunk[1], chunk[2], chunk[3]);
        if a == 255 {
            rgb_buf.push(r);
            rgb_buf.push(g);
            rgb_buf.push(b);
        } else {
            let a16 = a as u16;
            let inv = 255 - a16;
            rgb_buf.push((r as u16 * a16 / 255 + bg[0] as u16 * inv / 255) as u8);
            rgb_buf.push((g as u16 * a16 / 255 + bg[1] as u16 * inv / 255) as u8);
            rgb_buf.push((b as u16 * a16 / 255 + bg[2] as u16 * inv / 255) as u8);
        }
    }

    RgbImage::from_raw(w, h, rgb_buf).expect("buffer size matches dimensions")
}

/// Renders a square RGB cover at the given target size.
///
/// This preserves aspect ratio, up/downscales to fit within the square,
/// and pads with the configured background color.
fn render_square_variant_rgb(
    image: &DynamicImage,
    size: u32,
    resizer: &mut Resizer,
) -> Result<RgbImage, ImageError> {
    let fitted_rgba = resize_artwork(image, size, resizer)?.into_rgba8();
    let mut square_rgba = RgbaImage::from_pixel(size, size, Rgba([BG[0], BG[1], BG[2], 255]));
    let x_offset = ((size - fitted_rgba.width()) / 2) as i64;
    let y_offset = ((size - fitted_rgba.height()) / 2) as i64;
    overlay(&mut square_rgba, &fitted_rgba, x_offset, y_offset);
    // Overlay onto an opaque background has already flattened the alpha channel.
    Ok(DynamicImage::ImageRgba8(square_rgba).into_rgb8())
}

/// Keep Lanczos3 filtering and the existing fit/alpha semantics while using SIMD
/// for common 8-bit RGB/RGBA covers. Other pixel formats retain image's resizer.
fn resize_artwork(
    image: &DynamicImage,
    size: u32,
    resizer: &mut Resizer,
) -> Result<DynamicImage, ImageError> {
    if !matches!(
        image,
        DynamicImage::ImageRgb8(_) | DynamicImage::ImageRgba8(_)
    ) {
        return Ok(image.resize(size, size, FilterType::Lanczos3));
    }
    // Match DynamicImage::resize: round to nearest, with at least one pixel on
    // each axis even for extremely narrow covers. Never crop the source.
    let ratio = f64::from(size) / f64::from(image.width().max(image.height()));
    let width = ((f64::from(image.width()) * ratio).round() as u32).max(1);
    let height = ((f64::from(image.height()) * ratio).round() as u32).max(1);
    let mut output = DynamicImage::new(width, height, image.color());
    // The existing pipeline resamples straight channels and composites over
    // black afterwards; preserve that behavior instead of changing alpha math.
    let options = ResizeOptions::new().use_alpha(false);
    resizer
        .resize(image, &mut output, &options)
        .map_err(|error| {
            ImageError::Parameter(ParameterError::from_kind(ParameterErrorKind::Generic(
                format!("artwork resize failed: {error}"),
            )))
        })?;
    Ok(output)
}

/// Saves an `RgbImage` to disk as a JPEG file using the requested compression quality.
///
/// This function creates (or truncates) the destination file at `path` and encodes
/// the provided RGB pixels as JPEG.
///
/// # Parameters
/// - `rgb`: Source image buffer in 8-bit RGB format.
/// - `path`: Output file path where the JPEG will be written.
/// - `quality`: JPEG quality setting in the range of 1-100.
///
/// # Returns
/// - `Ok(())` if the image was successfully encoded and written.
/// - `Err(ImageError)` if file creation or JPEG encoding fails.
///
/// # Errors
/// Returns an error when:
/// - the file cannot be created/written (I/O failure), or
/// - the JPEG encoder fails while writing image data.
pub fn save_jpeg<P: AsRef<Path>>(rgb: &RgbImage, path: P, quality: u8) -> Result<(), ImageError> {
    let mut file = BufWriter::new(File::create(path)?);
    let enc = JpegEncoder::new_with_quality(&mut file, quality);
    enc.write_image(
        rgb,
        rgb.width(),
        rgb.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    file.flush()?;
    Ok(())
}
