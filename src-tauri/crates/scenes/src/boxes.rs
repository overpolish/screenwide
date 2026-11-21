// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a custom scene puts its panes: a box for each, as shares of the
//! canvas, moved and resized by the select tool.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::geometry::Rect;

/// One pane's box in a custom scene, as shares of the canvas's width and
/// height.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct SceneBox {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

/// The furthest a box may grow and shrink, as shares of the canvas. The twins
/// of the limits in `recording-scene-custom.ts`.
const MAX_BOX_SHARE: f64 = 4.0;
const MIN_BOX_SHARE: f64 = 0.02;

impl SceneBox {
  /// A box with some size whose middle stays on the canvas, so it can always
  /// be found and dragged back.
  pub(crate) fn is_valid(self) -> bool {
    let (x, y) = (self.x + self.width / 2.0, self.y + self.height / 2.0);
    (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) && self.is_drawable()
  }

  /// A box with some size and a place, wherever it lies. The preview draws a
  /// preset held where the crop tool left it, which may lie past the canvas's
  /// edge with the part of the picture it picks there.
  fn is_drawable(self) -> bool {
    self.x.is_finite()
      && self.y.is_finite()
      && self.width > 0.0
      && self.height > 0.0
      && self.width <= MAX_BOX_SHARE
      && self.height <= MAX_BOX_SHARE
  }

  /// The box on a `canvas` of output pixels.
  pub fn on(self, (width, height): (f64, f64)) -> Rect {
    Rect {
      x: self.x * width,
      y: self.y * height,
      width: self.width * width,
      height: self.height * height,
    }
  }

  /// How far a select gesture may scale the box: as far as keeps it between
  /// the smallest and largest a box may be.
  fn held_scale(self, scale: f64) -> f64 {
    let low = MIN_BOX_SHARE / self.width.min(self.height).max(f64::EPSILON);
    let high = MAX_BOX_SHARE / self.width.max(self.height).max(f64::EPSILON);
    scale.max(low).min(high)
  }

  /// The box a select gesture leaves: its corner moved `delta` shares of the
  /// canvas and its size scaled by `scale`, its shape kept. It stays between
  /// the smallest and largest a box may be, its middle on the canvas.
  pub fn moved(self, (dx, dy): (f64, f64), scale: f64) -> Self {
    let scale = self.held_scale(scale);
    let width = self.width * scale;
    let height = self.height * scale;
    Self {
      x: (self.x + dx).clamp(-width / 2.0, 1.0 - width / 2.0),
      y: (self.y + dy).clamp(-height / 2.0, 1.0 - height / 2.0),
      width,
      height,
    }
  }

  /// [`SceneBox::moved`] by a gesture on the box padded outwards by `pad`
  /// shares of the canvas on each side, where a scene's inset draws the
  /// handles. The padding scales with the box, so the box's corner travels
  /// the padding's growth further than the padded corner the gesture reports.
  pub fn moved_padded(self, (dx, dy): (f64, f64), scale: f64, (pad_x, pad_y): (f64, f64)) -> Self {
    let grown = self.held_scale(scale) - 1.0;
    self.moved((dx + pad_x * grown, dy + pad_y * grown), scale)
  }
}

/// Where a custom scene puts its panes in place of its preset's boxes, and
/// which of them is drawn in front. The camera keeps the recording's own box
/// where it has none, and is drawn in front unless `camera_behind` says
/// otherwise. Each pane is shown unless its `_hidden` says otherwise, and
/// keeps its place while hidden, so it comes back where it was and has
/// somewhere to arrive from as the scene leaves: a camera-only scene made
/// custom keeps the screen's box unseen, a screen-only one the camera in the
/// recording's own place.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SceneBoxes {
  pub screen: SceneBox,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub camera: Option<SceneBox>,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  #[ts(type = "true")]
  pub camera_behind: bool,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  #[ts(type = "true")]
  pub screen_hidden: bool,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  #[ts(type = "true")]
  pub camera_hidden: bool,
}

impl SceneBoxes {
  /// Whether every box has some size and its middle on the canvas, as a
  /// saved layout's must.
  pub(crate) fn is_valid(self) -> bool {
    [Some(self.screen), self.camera]
      .into_iter()
      .flatten()
      .all(SceneBox::is_valid)
  }

  /// Whether every box has some size and a place, as the held scene's must:
  /// a preset held while cropped is drawn at boxes that may lie past the
  /// canvas.
  pub(crate) fn is_drawable(self) -> bool {
    [Some(self.screen), self.camera]
      .into_iter()
      .flatten()
      .all(SceneBox::is_drawable)
  }
}
