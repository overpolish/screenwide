// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::measure::{measure_or_bare_percent, Measure};

/// A baked camera's placement, in the screen output's own pixels: the camera
/// image by its centre and width, and the crop window that frames it.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CameraOverlaySettings {
  pub camera_x: f64,
  pub camera_y: f64,
  pub camera_width: f64,
  pub frame_height: f64,
  pub frame_width: f64,
  pub frame_x: f64,
  pub frame_y: f64,
  /// The corners: a share of the canvas as the screen's are, or of the
  /// frame for a circle. Saved before as a bare percent of the frame.
  #[serde(alias = "radiusPercent", deserialize_with = "measure_or_bare_percent")]
  pub radius: Measure,
  /// Set where the camera follows the face found in it: the face is held
  /// in the middle of the crop window at the zoom the crop gives, wherever
  /// the picture sits behind it, and a scene follows it too unless it says
  /// otherwise.
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  #[ts(type = "true")]
  pub follow_face: bool,
}

impl CameraOverlaySettings {
  /// The corner radius in canvas pixels on a canvas whose shorter side is
  /// `canvas` pixels.
  pub fn radius_pixels(&self, canvas: f64) -> f64 {
    self
      .radius
      .radius_pixels(self.frame_width.min(self.frame_height), canvas)
  }

  /// The corner radius in percent of the frame's shorter side, as the
  /// selection's outline is rounded.
  pub fn radius_frame_percent(&self, canvas: f64) -> f64 {
    self
      .radius
      .radius_frame_percent(self.frame_width.min(self.frame_height), canvas)
  }
}
