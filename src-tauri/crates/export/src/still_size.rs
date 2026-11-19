// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A composed still made smaller, for a preview that shows it small.
//!
//! Here rather than beside its callers: the resize is generic, so it is
//! compiled into the crate that calls it, and this crate is optimized in dev
//! builds. Compiled into the app crate, shrinking a 3.5K frame took about a
//! third of a second in a dev build instead of about ten milliseconds.

use screenwide_imaging::CapturedImage;

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
