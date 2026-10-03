// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use image::imageops::FilterType;

/// The formats the system's own decoder is asked for. Everything else goes to
/// the `image` crate, which is faster to reach and already correct for it.
#[cfg(target_os = "macos")]
const NATIVE_FORMATS: [&str; 2] = ["heic", "heif"];

#[cfg(target_os = "macos")]
fn native_decode(path: &str, max_pixel_size: u32) -> Option<image::RgbaImage> {
  let extension = std::path::Path::new(path)
    .extension()?
    .to_str()?
    .to_ascii_lowercase();
  if !NATIVE_FORMATS.contains(&extension.as_str()) {
    return None;
  }
  super::macos::image_decode::decode_rgba(path, max_pixel_size)
}

#[cfg(not(target_os = "macos"))]
fn native_decode(_path: &str, _max_pixel_size: u32) -> Option<image::RgbaImage> {
  None
}

/// A picture off disk, decoded no larger than the caller can use.
///
/// `max_pixel_size` is a ceiling on the longer edge rather than a size to
/// scale to: a picture already smaller than it comes back at its own size, and
/// only the system decoder honours it, since it is the one that would
/// otherwise hand back tens of megapixels for a swatch.
///
/// The format is read from the file's contents rather than its name: the
/// picture Windows puts on the desktop, `TranscodedWallpaper`, has no
/// extension.
pub(crate) fn load(path: &str, max_pixel_size: u32) -> Option<image::RgbaImage> {
  if let Some(picture) = native_decode(path, max_pixel_size) {
    return Some(picture);
  }
  let reader = image::ImageReader::open(path)
    .ok()?
    .with_guessed_format()
    .ok()?;
  Some(reader.decode().ok()?.into_rgba8())
}

/// A chosen picture, filled to the canvas.
///
/// The picture covers the canvas rather than fitting inside it: it is scaled
/// until both sides reach, and the overhang is trimmed evenly from the two
/// edges that overflow, so a background never letterboxes and never stretches.
/// `None` when the file cannot be read or decoded, which the caller answers
/// with the solid colour rather than with an error: a picture moved out from
/// under a saved canvas must not stop an export.
pub(crate) fn background_image_canvas(
  path: &str,
  width: u32,
  height: u32,
) -> Option<image::RgbaImage> {
  if width == 0 || height == 0 {
    return None;
  }
  // Twice the canvas is as much detail as a cover fit can use: the picture is
  // scaled down to it, never up, so anything past that is decoded and thrown
  // away.
  let source = load(path, width.max(height).saturating_mul(2))?;
  Some(cover_fit(&source, width, height))
}

/// A chosen picture at a swatch's size.
///
/// A tile is a few dozen pixels across, and the pictures the system draws for
/// itself are display sized PNGs of eight megabytes or so: decoding one whole
/// and scaling it down costs a fifth of a second per tile, where the system's
/// own scaler reads only as much of the file as the tile needs. So a swatch
/// asks that scaler for every format it can read rather than only for the
/// ones the `image` crate cannot, and falls back to the ordinary decode when
/// it answers with nothing.
#[cfg(target_os = "macos")]
pub(crate) fn background_image_swatch(
  path: &str,
  width: u32,
  height: u32,
) -> Option<image::RgbaImage> {
  if width == 0 || height == 0 {
    return None;
  }
  let cap = width.max(height).saturating_mul(2);
  let source = super::macos::image_decode::decode_rgba(path, cap).or_else(|| load(path, cap))?;
  Some(cover_fit(&source, width, height))
}

/// Elsewhere a swatch is the ordinary decode, held to the tile's size.
#[cfg(not(target_os = "macos"))]
pub(crate) fn background_image_swatch(
  path: &str,
  width: u32,
  height: u32,
) -> Option<image::RgbaImage> {
  background_image_canvas(path, width, height)
}

fn cover_fit(source: &image::RgbaImage, width: u32, height: u32) -> image::RgbaImage {
  let (source_width, source_height) = source.dimensions();
  if source_width == 0 || source_height == 0 {
    return image::RgbaImage::new(width, height);
  }
  if (source_width, source_height) == (width, height) {
    return source.clone();
  }
  let scale =
    (f64::from(width) / f64::from(source_width)).max(f64::from(height) / f64::from(source_height));
  let scaled_width = ((f64::from(source_width) * scale).ceil() as u32).max(width);
  let scaled_height = ((f64::from(source_height) * scale).ceil() as u32).max(height);
  let scaled = image::imageops::resize(source, scaled_width, scaled_height, FilterType::Lanczos3);
  image::imageops::crop_imm(
    &scaled,
    (scaled_width - width) / 2,
    (scaled_height - height) / 2,
    width,
    height,
  )
  .to_image()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn stripes(width: u32, height: u32) -> image::RgbaImage {
    image::RgbaImage::from_fn(width, height, |x, _| {
      if x < width / 2 {
        image::Rgba([255, 0, 0, 255])
      } else {
        image::Rgba([0, 0, 255, 255])
      }
    })
  }

  #[test]
  fn fills_the_canvas_without_stretching() {
    let filled = cover_fit(&stripes(200, 100), 100, 100);
    assert_eq!(filled.dimensions(), (100, 100));
    // A wide picture in a square canvas keeps its middle: the red half stays
    // on the left and the blue half on the right, trimmed at both edges.
    assert_eq!(filled.get_pixel(5, 50)[0], 255);
    assert_eq!(filled.get_pixel(95, 50)[2], 255);
  }

  #[test]
  fn keeps_a_picture_that_already_fits() {
    let filled = cover_fit(&stripes(64, 64), 64, 64);
    assert_eq!(filled.dimensions(), (64, 64));
    assert_eq!(filled.get_pixel(0, 0), &image::Rgba([255, 0, 0, 255]));
  }

  #[test]
  fn answers_nothing_for_a_file_that_is_not_there() {
    assert!(background_image_canvas("/no/such/background.png", 32, 32).is_none());
  }

  #[test]
  fn decodes_a_picture_without_an_extension() {
    let directory = std::env::temp_dir().join("screenwide-background-image-test");
    std::fs::create_dir_all(&directory).unwrap();
    let png = directory.join("png.png");
    image::RgbaImage::from_pixel(4, 2, image::Rgba([10, 200, 30, 255]))
      .save(&png)
      .unwrap();
    let bare = directory.join("TranscodedWallpaper");
    std::fs::rename(&png, &bare).unwrap();
    let picture = load(bare.to_str().unwrap(), 64).expect("the format came from the contents");
    assert_eq!(picture.dimensions(), (4, 2));
    let _ = std::fs::remove_dir_all(&directory);
  }
}
