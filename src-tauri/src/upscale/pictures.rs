// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The stored pictures images show, drawn larger by the upscaling model once
//! each, for an image drawn at 1.25 times its picture's size or more, which
//! stretching would blur (`image_pictures.rs` in the compositor).
//!
//! The model keeps flat drawn art sharp (stickers, icons, a screen's
//! controls) but changes letters and paints photos over, so a picture with
//! text, as Text Recognition reads it, or one that looks photographed is
//! left to stretch, and so is one large enough that it is never drawn that
//! much larger. Each picture is decided once and the decision kept with the
//! store (`screenwide_images::store::upscaled`).

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use image::RgbaImage;
use screenwide_images::store;
use screenwide_imaging::CapturedImage;
use tauri::AppHandle;

/// The longest side an image is drawn at (`MAX_RASTER` in the image atlas).
const LONGEST_DRAWN: u32 = 2_048;
/// The longest a picture's own side may be for the model to draw it: drawn
/// at 1.25 times its size, a larger one would pass what an image is ever
/// drawn at.
const LARGEST_UPSCALED: u32 = 1_638;
/// The side a picture is looked at to judge whether it is drawn or
/// photographed.
const JUDGED_AT: u32 = 256;
/// How much of a drawn picture is flat colour or smooth shading. A photo's
/// grain leaves less of it: measured, a photo of a table came to 0.60, the
/// sticker with the most shading 0.67, icons and patterns 0.76 to 0.92.
const DRAWN_FLATNESS: f32 = 0.64;

/// Set while pictures are being decided, and asked for again meanwhile.
static RUNNING: AtomicBool = AtomicBool::new(false);
static AGAIN: AtomicBool = AtomicBool::new(false);

/// Decides about every stored picture not decided yet, in the background:
/// draws it larger, or notes it is to stretch. One run at a time; asked
/// while one runs, it runs again once that ends.
pub(crate) fn prepare(app: &AppHandle) {
  AGAIN.store(true, Ordering::Release);
  if RUNNING.swap(true, Ordering::AcqRel) {
    return;
  }
  let app = app.clone();
  tauri::async_runtime::spawn_blocking(move || {
    while AGAIN.swap(false, Ordering::AcqRel) {
      for (name, file) in store::undecided() {
        if let Err(error) = decide(&app, &name, &file) {
          eprintln!("Could not upscale a stored picture: {error}");
        }
      }
    }
    RUNNING.store(false, Ordering::Release);
  });
}

fn decide(app: &AppHandle, name: &str, file: &Path) -> Result<(), String> {
  let picture = image::open(file)
    .map_err(|error| format!("Could not read the picture: {error}"))?
    .into_rgba8();
  let (width, height) = picture.dimensions();
  if width.max(height) > LARGEST_UPSCALED {
    return store::skip_upscale(name, "large");
  }
  if !looks_drawn(&picture) {
    return store::skip_upscale(name, "photo");
  }
  if crate::text_recognition::shows_text(picture.as_raw(), width, height)? {
    return store::skip_upscale(name, "text");
  }
  let upscaled = super::upscaled_uncached(
    app,
    &CapturedImage {
      width,
      height,
      rgba: picture.into_raw(),
    },
  )?;
  let upscaled = RgbaImage::from_raw(upscaled.width, upscaled.height, upscaled.rgba)
    .ok_or("The upscaled picture does not fill its size")?;
  store::keep_upscaled(name, &within(upscaled, LONGEST_DRAWN))
}

/// `picture`, shrunk where needed so its longer side is no more than `most`.
fn within(picture: RgbaImage, most: u32) -> RgbaImage {
  let (width, height) = picture.dimensions();
  let long = width.max(height);
  if long <= most {
    return picture;
  }
  let fit = |side: u32| ((u64::from(side) * u64::from(most)) / u64::from(long)).max(1) as u32;
  image::imageops::resize(
    &picture,
    fit(width),
    fit(height),
    image::imageops::FilterType::Lanczos3,
  )
}

/// Whether `picture` looks drawn rather than photographed: drawn art is
/// mostly flat colour and smooth shading, where a photo's grain bends its
/// brightness almost everywhere.
fn looks_drawn(picture: &RgbaImage) -> bool {
  flatness(picture) >= DRAWN_FLATNESS
}

/// How much of `picture`'s opaque area is flat colour or smooth shading, 0
/// to 1, judged at a small size, which keeps a drawing's edges from
/// counting against it.
fn flatness(picture: &RgbaImage) -> f32 {
  let (width, height) = picture.dimensions();
  let long = width.max(height).max(1);
  let scale = (f64::from(JUDGED_AT) / f64::from(long)).min(1.0);
  let small = image::imageops::resize(
    picture,
    ((f64::from(width) * scale).round() as u32).max(1),
    ((f64::from(height) * scale).round() as u32).max(1),
    image::imageops::FilterType::Triangle,
  );
  let (width, height) = (small.width() as usize, small.height() as usize);
  let luminance: Vec<f32> = small
    .pixels()
    .map(|pixel| {
      (0.2126 * f32::from(pixel[0]) + 0.7152 * f32::from(pixel[1]) + 0.0722 * f32::from(pixel[2]))
        / 255.0
    })
    .collect();
  let at = |x: usize, y: usize| luminance[y * width + x];
  let (mut opaque, mut flat) = (0_u32, 0_u32);
  for y in 0..height {
    for x in 0..width {
      if small.get_pixel(x as u32, y as u32)[3] < 230 {
        continue;
      }
      opaque += 1;
      // How far the pixel's brightness bends from its four neighbours'.
      let around = at(x.saturating_sub(1), y)
        + at((x + 1).min(width - 1), y)
        + at(x, y.saturating_sub(1))
        + at(x, (y + 1).min(height - 1));
      if (4.0 * at(x, y) - around).abs() < 0.02 {
        flat += 1;
      }
    }
  }
  if opaque == 0 {
    return 0.0;
  }
  flat as f32 / opaque as f32
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_flat_drawing_looks_drawn() {
    let picture = RgbaImage::from_fn(64, 64, |x, _| {
      if x < 32 {
        image::Rgba([230, 40, 40, 255])
      } else {
        image::Rgba([40, 40, 230, 255])
      }
    });
    assert!(looks_drawn(&picture));
  }

  #[test]
  fn grain_does_not_look_drawn() {
    // Brightness that jumps from pixel to pixel, as a photo's grain does.
    let picture = RgbaImage::from_fn(64, 64, |x, y| {
      let value = ((x * 7919 + y * 104_729) % 97) as u8 + 80;
      image::Rgba([value, value, value, 255])
    });
    assert!(!looks_drawn(&picture));
  }

  #[test]
  fn a_large_picture_is_shrunk_to_the_longest_side() {
    let picture = RgbaImage::new(4_000, 2_000);
    assert_eq!(within(picture, 2_048).dimensions(), (2_048, 1_024));
  }
}
