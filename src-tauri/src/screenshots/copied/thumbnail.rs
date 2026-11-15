// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The copied shot's picture on the island: scaled down to the size it is
//! shown at, as a file in the cache the island can load.

use std::path::{Path, PathBuf};

use image::{imageops, ImageBuffer, ImageFormat, Rgba};
use tauri::{AppHandle, Manager};

use super::CapturedImage;

/// The largest the picture is shown under the island's row, in points. A
/// smaller shot is shown at its own size.
const LARGEST: (f64, f64) = (320.0, 200.0);
const DIRECTORY: &str = "screenshot-previews";

pub(super) fn directory(app: &AppHandle) -> Result<PathBuf, String> {
  app
    .path()
    .app_cache_dir()
    .map(|cache| cache.join(DIRECTORY))
    .map_err(|error| format!("The cache directory is not available: {error}"))
}

/// The size `image`, taken at `scale`, is shown at, in points.
fn shown_size(image: &CapturedImage, scale: f64) -> (f64, f64) {
  let scale = if scale.is_finite() && scale > 0.0 {
    scale
  } else {
    1.0
  };
  let points = (
    f64::from(image.width) / scale,
    f64::from(image.height) / scale,
  );
  let fit = (LARGEST.0 / points.0).min(LARGEST.1 / points.1).min(1.0);
  (points.0 * fit, points.1 * fit)
}

/// Writes `image` scaled down, at `scale` pixels to the point, as shot `id`'s
/// picture, and says where and at what size in points it is shown. Pictures
/// of earlier shots left behind go first.
pub(super) fn write(
  directory: &Path,
  id: u32,
  image: &CapturedImage,
  scale: f64,
) -> Result<(PathBuf, f64, f64), String> {
  let (width, height) = shown_size(image, scale);
  let pixels = (
    ((width * scale).round() as u32).max(1),
    ((height * scale).round() as u32).max(1),
  );
  let source = ImageBuffer::<Rgba<u8>, &[u8]>::from_raw(image.width, image.height, &image.rgba)
    .ok_or_else(|| "The screenshot's pixels do not match its size".to_owned())?;
  let small = imageops::thumbnail(&source, pixels.0, pixels.1);
  if let Ok(entries) = std::fs::read_dir(directory) {
    for entry in entries.flatten() {
      let _ = std::fs::remove_file(entry.path());
    }
  }
  std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
  let path = directory.join(format!("{id}.png"));
  small
    .save_with_format(&path, ImageFormat::Png)
    .map_err(|error| error.to_string())?;
  Ok((path, width, height))
}

pub(super) fn remove(path: &str) {
  let _ = std::fs::remove_file(path);
}
