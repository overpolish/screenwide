// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The scrub strip: frames along the edit, side by side in one picture, for
//! the project browser's card to scrub through as the pointer crosses it.
//! Composed by the editor through the same still compositor as the preview,
//! so it shows the edit, annotations and all; only the editor holds one.

use image::imageops::FilterType;
use tauri::Emitter;

use super::*;

/// More than a card can show apart, few enough to compose in a moment.
const MAX_FRAMES: usize = 24;
const JPEG_QUALITY: u8 = 80;

pub(super) fn check_positions(positions_ms: &[u64]) -> Result<(), String> {
  if positions_ms.is_empty() || positions_ms.len() > MAX_FRAMES {
    return Err(format!(
      "A scrub strip takes between 1 and {MAX_FRAMES} frames"
    ));
  }
  Ok(())
}

/// Keeps `frames`, in order along the edit, as the scrub strip of the
/// project whose manifest is `project`, and tells the browser.
pub(super) async fn keep_strip(
  app: &AppHandle,
  project: PathBuf,
  frames: Vec<CapturedImage>,
) -> Result<(), String> {
  let target = crate::project::scrub_strip_path(&project);
  tauri::async_runtime::spawn_blocking(move || write_strip(frames, &target))
    .await
    .map_err(|error| error.to_string())??;
  let _ = app.emit(crate::project::library::STILL_EVENT, &project);
  Ok(())
}

/// Lays `frames` side by side, each [`crate::project::SCRUB_FRAME_WIDTH`]
/// wide, and writes them as one JPEG. A frame is laid over black where it is
/// see-through, as an exported movie, which has no transparency, shows it.
fn write_strip(frames: Vec<CapturedImage>, target: &Path) -> Result<(), String> {
  let first = frames
    .first()
    .ok_or_else(|| "There are no frames to keep".to_owned())?;
  let width = crate::project::SCRUB_FRAME_WIDTH;
  let height =
    (u64::from(first.height) * u64::from(width) / u64::from(first.width.max(1))).max(1) as u32;
  let count = u32::try_from(frames.len()).map_err(|error| error.to_string())?;
  let mut strip = image::RgbImage::new(width * count, height);
  for (index, frame) in (0..count).zip(frames) {
    let picture = image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba)
      .ok_or_else(|| "A composed frame is not the size it says".to_owned())?;
    let smaller = image::imageops::resize(&picture, width, height, FilterType::Triangle);
    for (x, y, pixel) in smaller.enumerate_pixels() {
      let [red, green, blue, alpha] = pixel.0;
      let over_black = |channel: u8| ((u16::from(channel) * u16::from(alpha)) / 255) as u8;
      strip.put_pixel(
        index * width + x,
        y,
        image::Rgb([over_black(red), over_black(green), over_black(blue)]),
      );
    }
  }
  write_atomically(target, true, |partial| {
    let file = std::fs::File::create(partial).map_err(|error| error.to_string())?;
    image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(file), JPEG_QUALITY)
      .encode_image(&strip)
      .map_err(|error| error.to_string())
  })
}
