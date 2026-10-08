// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Opening a reader where playback starts. A settled seek rewinds
//! [`super::SETTLED_SEEK_PREROLL_MS`] and decodes forward, which costs several
//! times what a seek with no rewind does, all of it before the first frame.
//! Media Foundation normally lands on the keyframe at or before the request,
//! so playback starts from a seek with no rewind and pays for the settled one
//! only when that seek demonstrably missed: it landed past the start, or the
//! stream ended before reaching it.

use std::{path::Path, sync::Arc};

use super::GpuVideoReader;
use crate::editor::preview_platform::RecordingPreviewSurface;

impl GpuVideoReader {
  /// A reader at `start_ms` with the frame there already decoded.
  pub(crate) fn open_for_playback(
    path: &Path,
    start_ms: u64,
    surface: Arc<RecordingPreviewSurface>,
  ) -> Result<Self, String> {
    let device = surface.device();
    let mut reader = Self::open_with_device(path, start_ms, true, device.clone())?;
    if reader.decode_from_seek_to(start_ms)? {
      return Ok(reader);
    }
    // A fresh reader, as for any settled seek: repeated seeks can leave the
    // source reader at a premature end of stream.
    drop(reader);
    let mut reader = Self::open_with_device(path, start_ms, false, device)?;
    reader.frame_at(start_ms)?;
    Ok(reader)
  }

  /// Decodes forward from the last seek to `target_ms`, returning whether
  /// the seek landed at or before it and the stream reached it.
  fn decode_from_seek_to(&mut self, target_ms: u64) -> Result<bool, String> {
    let Some((frame, sample)) = self.read_frame()? else {
      return Ok(false);
    };
    let landed_ms = frame.timestamp_ms;
    self.pending_frame = Some(frame);
    self.pending_sample = Some(sample);
    let shown_ms = self.frame_at(target_ms)?.map(|frame| frame.timestamp_ms);
    // `frame_at` shows the frame before the target once it has decoded the
    // one past it, which it keeps pending; at the end of the stream it shows
    // the last frame with nothing pending.
    let decoded_past = self.pending_frame.is_some();
    Ok(seek_reached(landed_ms, shown_ms, decoded_past, target_ms))
  }
}

/// The 2 ms matches the allowance `frame_at` gives a frame stamped either
/// side of the request.
fn seek_reached(landed_ms: u64, shown_ms: Option<u64>, decoded_past: bool, target_ms: u64) -> bool {
  landed_ms <= target_ms.saturating_add(2)
    && shown_ms.is_some_and(|shown_ms| decoded_past || shown_ms.saturating_add(2) >= target_ms)
}

#[cfg(test)]
mod tests {
  use super::seek_reached;

  #[test]
  fn a_seek_that_lands_on_an_earlier_keyframe_and_decodes_to_the_start_is_kept() {
    assert!(seek_reached(2_500, Some(2_883), true, 2_890));
    assert!(seek_reached(2_890, Some(2_890), false, 2_890));
  }

  #[test]
  fn a_seek_that_lands_past_the_start_falls_back() {
    assert!(!seek_reached(3_000, Some(3_000), true, 2_890));
  }

  #[test]
  fn a_seek_whose_stream_ends_before_the_start_falls_back() {
    assert!(!seek_reached(2_500, Some(2_700), false, 2_890));
    assert!(!seek_reached(2_500, None, false, 2_890));
  }
}
