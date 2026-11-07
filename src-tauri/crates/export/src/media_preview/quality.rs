// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_sidecar::ffmpeg_command;

/// The H.264 quality represented by the compression control.
///
/// Zero is deliberately not a CRF: it means the original encoded video is
/// copied without a generation loss. The remaining values are named quality
/// steps in the UI, so each maps to a stable encoder setting.
pub(super) fn compression_crf(compression: u8) -> Option<u16> {
  match compression {
    0 => None,
    1 => Some(20),
    2 => Some(24),
    3 => Some(28),
    _ => Some(32),
  }
}

/// The scale FFmpeg gives a recording it writes smaller: each side to the
/// nearest even, as `FrameFit::of` sizes the compositor's frame. FFmpeg can
/// only stretch to whole pixels, so the picture fills the frame, at most
/// half a pixel taller or wider at each edge than one scale would make it;
/// the compare is told so (see `FrameFit::canvas`'s callers).
pub(super) fn resolution_filter(
  source_scale_percent: u16,
  resolution_scale_percent: u16,
) -> Option<String> {
  (resolution_scale_percent < source_scale_percent).then(|| {
    let side = |input: &str| {
      format!("ceil({input}*{resolution_scale_percent}/{source_scale_percent}/2-0.5)*2")
    };
    format!("scale={}:{}:flags=lanczos", side("iw"), side("ih"))
  })
}

/// Resizing cannot stream-copy. High quality is deliberately used if a caller
/// requests a smaller resolution with Original compression.
pub(crate) fn export_crf(compression: u8, is_resizing: bool) -> Option<u16> {
  compression_crf(compression).or(is_resizing.then_some(20))
}

/// Whether this FFmpeg build carries the software H.264 encoder used to make
/// compression behave the same on macOS and Windows.
pub fn supports_compression() -> bool {
  static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

  *AVAILABLE.get_or_init(|| {
    ffmpeg_command()
      .args(["-hide_banner", "-encoders"])
      .stdin(Stdio::null())
      .output()
      .is_ok_and(|output| {
        output.status.success()
          && String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.split_whitespace().nth(1) == Some("libx264"))
      })
  })
}
