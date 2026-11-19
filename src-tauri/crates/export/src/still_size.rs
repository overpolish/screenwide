// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A composed still made smaller, for a preview that shows it small, and
//! kept as a JPEG.
//!
//! Here rather than beside its callers: the resize and the pixel loops are
//! compiled into the crate that runs them, and this crate is optimized in
//! dev builds. Compiled into the app crate, shrinking a 3.5K frame took
//! about a third of a second in a dev build instead of about ten
//! milliseconds.

use std::io::Write;

use screenwide_imaging::CapturedImage;

/// `frame`, whose colour is premultiplied by its alpha, as a JPEG of
/// `quality` laid over black where it is see-through, written to `out`. Not
/// generic over the writer, so it stays compiled here.
pub fn write_jpeg(frame: &CapturedImage, quality: u8, out: &mut dyn Write) -> Result<(), String> {
  let rgb: Vec<u8> = frame
    .rgba
    .as_chunks::<4>()
    .0
    .iter()
    .flat_map(|&[red, green, blue, _]| [red, green, blue])
    .collect();
  image::codecs::jpeg::JpegEncoder::new_with_quality(out, quality)
    .encode(
      &rgb,
      frame.width,
      frame.height,
      image::ExtendedColorType::Rgb8,
    )
    .map_err(|error| error.to_string())
}

/// `frame` at most `max_width` wide, its proportions kept, averaged by area.
pub fn shrunk(frame: CapturedImage, max_width: u32) -> Result<CapturedImage, String> {
  if max_width >= frame.width {
    return Ok(frame);
  }
  let height =
    (u64::from(frame.height) * u64::from(max_width) / u64::from(frame.width.max(1))).max(1) as u32;
  let picture = image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba)
    .ok_or_else(|| "A composed frame is not the size it says".to_owned())?;
  let smaller = image::imageops::thumbnail(&picture, max_width, height);
  Ok(CapturedImage {
    rgba: smaller.into_raw(),
    width: max_width,
    height,
  })
}
