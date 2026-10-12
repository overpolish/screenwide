// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
  pub radius_percent: f64,
}
