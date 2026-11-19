// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a layer was captured at, and the point sizes measured against it.

use super::ScreenshotOutputSettings;

/// The logical size a source was captured at. Zero where it is unknown.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Capture {
  /// The whole source's width in logical points.
  pub width_points: f64,
  /// Source pixels per logical point.
  pub scale: f64,
  /// How round the source's own corners are, in its pixels: a window
  /// recorded on its own, whose movie has them black. Zero for a square
  /// source.
  pub corner_radius: f64,
}

impl Capture {
  /// Output pixels per point of annotation size.
  pub fn size_scale(self) -> f64 {
    size_scale(self.scale)
  }
}

impl ScreenshotOutputSettings {
  /// Records the capture this layer draws, which its annotations are sized
  /// against and its picture is rounded to.
  pub fn stamp_capture(&mut self, capture: Capture) {
    self.capture_width_points = capture.width_points;
    self.capture_scale = capture.scale;
    self.capture_corner_radius = capture.corner_radius;
  }

  /// Output pixels per point of annotation size.
  pub fn size_scale(&self) -> f64 {
    size_scale(self.capture_scale)
  }

  /// The image's drawn width in points of annotation size: what turns a
  /// size into source pixels, or into a share of the picture.
  pub fn size_image_width(&self) -> f64 {
    self.image_width / self.size_scale()
  }

  /// Canvas pixels per point of the capture as the layer draws its picture:
  /// what the corners' and padding's points are drawn at, growing and
  /// shrinking with the picture. One where no capture was stamped.
  pub fn canvas_pixels_per_point(&self) -> f64 {
    if self.capture_width_points.is_finite() && self.capture_width_points > 0.0 {
      self.image_width.max(0.0) / self.capture_width_points
    } else {
      1.0
    }
  }

  /// The frame's corner radius in canvas pixels, on a frame `width` by
  /// `height` canvas pixels.
  pub fn corner_radius_pixels(&self, width: f64, height: f64) -> f64 {
    self
      .radius
      .radius_pixels(width.min(height), self.canvas_pixels_per_point())
  }

  /// The frame's corner radius as a percent of its own shorter side, as the
  /// selection's outline is rounded: the outline is drawn in the view's
  /// pixels, where the canvas's are not known.
  pub fn corner_radius_frame_percent(&self) -> f64 {
    let shortest = self.crop_width.min(self.crop_height);
    if shortest > 0.0 {
      self.corner_radius_pixels(self.crop_width, self.crop_height) / shortest * 100.0
    } else {
      0.0
    }
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
