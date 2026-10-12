// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Single source frames for stills and pins, decoded by Media Foundation.

use super::decoder::NativeVideoReader;

pub fn source_frame_image(
  path: &std::path::Path,
  position_ms: u64,
  duration_ms: u64,
) -> Result<screenwide_imaging::CapturedImage, String> {
  let position_ms = position_ms.min(duration_ms.saturating_sub(1));
  let mut reader = NativeVideoReader::open(path, 0, 0, position_ms)?;
  let frame = reader
    .frame_at(position_ms)?
    .ok_or_else(|| "Media Foundation returned no source frame".to_owned())?;
  Ok(frame)
}

/// The recording's frames at each of `times_ms`, in rising order, decoded in
/// one pass and handed to `each` as they come; the pass ends early once
/// `each` returns false.
pub fn each_source_frame(
  path: &std::path::Path,
  times_ms: &[u64],
  duration_ms: u64,
  mut each: impl FnMut(u64, &screenwide_imaging::CapturedImage) -> bool,
) -> Result<(), String> {
  let Some(&first) = times_ms.first() else {
    return Ok(());
  };
  let last = duration_ms.saturating_sub(1);
  let mut reader = NativeVideoReader::open(path, 0, 0, first.min(last))?;
  for &time in times_ms {
    if let Some(frame) = reader.frame_at(time.min(last))? {
      if !each(time, &frame) {
        break;
      }
    }
  }
  Ok(())
}
