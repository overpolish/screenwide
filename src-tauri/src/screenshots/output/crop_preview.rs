// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The crop tool's live result rectangle, in output pixels.
///
/// Crop mode previews the whole source so the part being cropped away stays
/// visible. This is where the cropped layer itself lands inside that canvas,
/// so the compositor can draw it a second time with its real corner radius
/// and drop shadow.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CropPreviewRect {
  pub height: f64,
  pub width: f64,
  pub x: f64,
  pub y: f64,
}
