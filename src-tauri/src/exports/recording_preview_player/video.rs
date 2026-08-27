// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Playback frame plumbing that is the same on every platform.
//!
//! The frame *contents* are platform-owned - see
//! [`super::platform::VideoFramePayload`] - because a backend that composites
//! on the GPU hands over decoded surfaces while a fallback backend hands over
//! encoded bytes for the webview to draw.

use super::platform::VideoFramePayload;

#[cfg_attr(target_os = "windows", allow(dead_code))]
pub(super) const OUTPUT_FPS: u64 = 60;

pub(super) fn source_position_ms(start_ms: u64, output_frame: u64, playback_rate: f64) -> u64 {
  start_ms.saturating_add(
    ((output_frame as f64 * 1_000.0 * playback_rate) / OUTPUT_FPS as f64).round() as u64,
  )
}

pub(super) const fn presentation_elapsed_ms(output_frame: u64) -> u64 {
  output_frame.saturating_mul(1_000) / OUTPUT_FPS
}

pub(super) struct VideoFrame {
  pub payload: VideoFramePayload,
  pub presentation_elapsed_ms: u64,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn output_cadence_advances_source_by_the_effective_rate() {
    assert_eq!(source_position_ms(1_000, 60, 0.5), 1_500);
    assert_eq!(source_position_ms(1_000, 60, 1.0), 2_000);
    assert_eq!(source_position_ms(1_000, 60, 2.0), 3_000);
    assert_eq!(presentation_elapsed_ms(60), 1_000);
  }
}
