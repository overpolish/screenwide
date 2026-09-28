// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a layer was captured at, and the point sizes measured against it.

use super::ScreenshotOutputSettings;

/// The logical size a source was captured at. Zero where it is unknown.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Capture {
  /// The whole source's width in logical points.
  pub(crate) width_points: f64,
  /// Source pixels per logical point.
  pub(crate) scale: f64,
}

impl Capture {
  /// Output pixels per point of annotation size.
  pub(crate) fn size_scale(self) -> f64 {
    size_scale(self.scale)
  }
}

impl ScreenshotOutputSettings {
  /// Records the capture this layer draws, which its annotations are sized
  /// against.
  pub(crate) fn stamp_capture(&mut self, capture: Capture) {
    self.capture_width_points = capture.width_points;
    self.capture_scale = capture.scale;
  }

  /// Output pixels per point of annotation size.
  pub(crate) fn size_scale(&self) -> f64 {
    size_scale(self.capture_scale)
  }

  /// The image's drawn width in points of annotation size: what turns a
  /// size into source pixels, or into a share of the picture.
  pub(crate) fn size_image_width(&self) -> f64 {
    self.image_width / self.size_scale()
  }
}

/// Output pixels per point of annotation size for a capture taken at
/// `scale`, reading an unknown scale as one.
fn size_scale(scale: f64) -> f64 {
  if scale.is_finite() && scale > 0.0 {
    scale
  } else {
    1.0
  }
}
