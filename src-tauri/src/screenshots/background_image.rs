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
  super::image_decode_macos::decode_rgba(path, max_pixel_size)
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
pub(crate) fn load(path: &str, max_pixel_size: u32) -> Option<image::RgbaImage> {
  if let Some(picture) = native_decode(path, max_pixel_size) {
    return Some(picture);
  }
  Some(image::open(path).ok()?.into_rgba8())
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
  let source = super::image_decode_macos::decode_rgba(path, cap).or_else(|| load(path, cap))?;
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

/// The native canvas shader samples the picture straight from a Metal
/// texture, so the decode happens once per file and the pixels are uploaded
/// at their own size. Cover-fitting belongs to the shader there, not here.
#[cfg(target_os = "macos")]
pub(crate) mod native {
  use std::sync::atomic::{AtomicU32, Ordering};
  use std::sync::Mutex;

  use super::super::ScreenshotOutputSettings;

  unsafe extern "C" {
    fn screenwide_gpu_register_background_image(
      id: u32,
      rgba: *const u8,
      width: u32,
      height: u32,
    ) -> i32;
  }

  struct Registered {
    id: u32,
    key: String,
  }

  /// The canvas never samples more than this across, so a desktop picture is
  /// decoded to it rather than to its own tens of megapixels.
  const TEXTURE_EDGE: u32 = 4096;

  /// The same two entries the native side keeps, evicted oldest first, so a
  /// cached id always names a texture the compositor still holds.
  const LIMIT: usize = 2;

  static CACHE: Mutex<Vec<Registered>> = Mutex::new(Vec::new());
  static NEXT_ID: AtomicU32 = AtomicU32::new(1);

  /// The canvas uniforms' `(has_background_image, background_image_id)` for
  /// these settings. A background that is not a picture, or a picture that
  /// cannot be read, answers `(0, 0)`, which leaves the canvas on its mesh or
  /// solid colour: a picture moved out from under a saved canvas must not
  /// stop an export.
  pub(crate) fn canvas_picture(settings: &ScreenshotOutputSettings) -> (u32, u32) {
    if settings.background_type != "image" {
      return (0, 0);
    }
    settings
      .background_image_path
      .as_deref()
      .and_then(register_background_image)
      .map_or((0, 0), |id| (1, id))
  }

  /// Decodes the picture once per path and modification time and answers the
  /// id the canvas uniforms carry.
  fn register_background_image(path: &str) -> Option<u32> {
    let key = cache_key(path);
    let mut cache = CACHE.lock().ok()?;
    if let Some(entry) = cache.iter().find(|entry| entry.key == key) {
      return Some(entry.id);
    }
    let picture = super::load(path, TEXTURE_EDGE)?;
    let (width, height) = picture.dimensions();
    if width == 0 || height == 0 {
      return None;
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    // SAFETY: the pixels stay alive for the whole call and the native side
    // only reads `width * height * 4` bytes from them, which is exactly what
    // the decoded buffer holds.
    let uploaded = unsafe {
      screenwide_gpu_register_background_image(id, picture.as_raw().as_ptr(), width, height)
    };
    if uploaded == 0 {
      return None;
    }
    while cache.len() >= LIMIT {
      cache.remove(0);
    }
    cache.push(Registered { id, key });
    Some(id)
  }

  /// A picture edited in place keeps its path, so the modification time is
  /// part of what identifies it.
  fn cache_key(path: &str) -> String {
    let stamp = std::fs::metadata(path)
      .and_then(|metadata| metadata.modified())
      .ok()
      .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
      .map_or(0, |since| since.as_nanos());
    format!("{path}|{stamp}")
  }
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

  /// The system's own desktop pictures are HEIC, which the `image` crate here
  /// cannot read: this proves the picture goes through the system decoder and
  /// comes back at the size the caller asked for rather than its own.
  #[cfg(target_os = "macos")]
  #[test]
  fn decodes_a_system_heic_wallpaper_within_its_cap() {
    let Some(wallpaper) = crate::settings::wallpapers::system_wallpapers(None)
      .into_iter()
      .find(|one| one.path.to_ascii_lowercase().ends_with(".heic"))
    else {
      return;
    };
    let picture = load(&wallpaper.path, 256).expect("the system decoded its own wallpaper");
    let (width, height) = picture.dimensions();
    assert!(width > 0 && height > 0);
    assert!(
      width <= 256 && height <= 256,
      "{width}x{height} past the cap"
    );
  }

  #[test]
  fn answers_nothing_for_a_file_that_is_not_there() {
    assert!(background_image_canvas("/no/such/background.png", 32, 32).is_none());
  }
}
