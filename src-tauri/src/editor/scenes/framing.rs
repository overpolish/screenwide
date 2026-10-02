// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What part of a picture fills its box in a scene. A scene keeps one framing
//! for the screen and one for the camera, each the point of the picture that
//! sits in the middle of its box and how far it is zoomed past only just
//! covering it. A framing never uncovers its box: at any zoom the picture is
//! held where it still fills it. The twin of `recording-scene-framing.ts`.

use serde::{Deserialize, Serialize};

use super::geometry::Rect;
use crate::editor::CameraOverlaySettings;

/// The furthest a scene zooms into a picture.
pub(crate) const MAX_ZOOM: f64 = 8.0;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneFraming {
  /// The point of the picture in the middle of its box, as shares of the
  /// picture's width and height.
  pub focus_x: f64,
  pub focus_y: f64,
  /// How far the picture is zoomed past the size that only just covers its
  /// box: one shows as much of it as the box's shape allows.
  pub zoom: f64,
}

/// The width a picture of `aspect` needs to cover `frame` exactly.
fn cover(frame: Rect, aspect: f64) -> f64 {
  frame.width.max(frame.height * aspect)
}

/// The camera's box in the recording's own composition as it is drawn, for a
/// camera picture of `aspect`: no larger than the picture and held inside it,
/// the way `bake_geometry` places it, since a stored box can sit partly
/// outside the picture. The twin of `drawnCameraFrame` in
/// `recording-scene-framing.ts`.
pub(crate) fn drawn_frame(overlay: &CameraOverlaySettings, aspect: f64) -> Rect {
  let camera_width = overlay.camera_width;
  let camera_height = camera_width / aspect.max(f64::EPSILON);
  let left = overlay.camera_x - camera_width / 2.0;
  let top = overlay.camera_y - camera_height / 2.0;
  let width = overlay.frame_width.min(camera_width);
  let height = overlay.frame_height.min(camera_height);
  Rect {
    x: overlay
      .frame_x
      .clamp(left, (left + camera_width - width).max(left)),
    y: overlay
      .frame_y
      .clamp(top, (top + camera_height - height).max(top)),
    width,
    height,
  }
}

/// The furthest `focus` may sit from the picture's middle, as a share of it,
/// where a picture `size` long still covers `length` of box.
fn reach(size: f64, length: f64) -> f64 {
  (size - length).max(0.0) / 2.0 / size.max(f64::EPSILON)
}

impl SceneFraming {
  /// The whole picture, centred: what a scene shows until it is reframed.
  pub(crate) const WHOLE: Self = Self {
    focus_x: 0.5,
    focus_y: 0.5,
    zoom: 1.0,
  };

  /// Whether this is a framing a scene can hold.
  pub(crate) fn is_valid(self) -> bool {
    (0.0..=1.0).contains(&self.focus_x)
      && (0.0..=1.0).contains(&self.focus_y)
      && (1.0..=MAX_ZOOM).contains(&self.zoom)
  }

  /// The framing `overlay` gives a camera picture of `aspect` in the
  /// recording's own composition, outside every scene.
  pub(crate) fn of_camera(overlay: &CameraOverlaySettings, aspect: f64) -> Self {
    let frame = drawn_frame(overlay, aspect);
    let width = overlay.camera_width.max(f64::EPSILON);
    let height = width / aspect;
    let (x, y) = frame.centre();
    Self {
      focus_x: (x - (overlay.camera_x - width / 2.0)) / width,
      focus_y: (y - (overlay.camera_y - height / 2.0)) / height,
      zoom: (width / cover(frame, aspect).max(f64::EPSILON)).max(1.0),
    }
  }

  /// This framing held where a picture of `aspect` still covers `frame`.
  pub(crate) fn within(self, frame: Rect, aspect: f64) -> Self {
    let zoom = self.zoom.clamp(1.0, MAX_ZOOM);
    let width = cover(frame, aspect) * zoom;
    let reach_x = reach(width, frame.width);
    let reach_y = reach(width / aspect, frame.height);
    Self {
      focus_x: self.focus_x.clamp(0.5 - reach_x, 0.5 + reach_x),
      focus_y: self.focus_y.clamp(0.5 - reach_y, 0.5 + reach_y),
      zoom,
    }
  }

  /// The centre and width of a picture of `aspect` that frames `frame` this
  /// way.
  pub(crate) fn place(self, frame: Rect, aspect: f64) -> (f64, f64, f64) {
    let held = self.within(frame, aspect);
    let width = cover(frame, aspect) * held.zoom;
    let (x, y) = frame.centre();
    (
      x - (held.focus_x - 0.5) * width,
      y - (held.focus_y - 0.5) * width / aspect,
      width,
    )
  }

  /// This framing with its picture, of `aspect`, dragged `delta` canvas pixels
  /// and zoomed by `scale` in `frame`. The focus is a share of the picture,
  /// so it moves by the pointer's share of the picture as it is drawn there.
  pub(crate) fn reframed(self, frame: Rect, aspect: f64, delta: (f64, f64), scale: f64) -> Self {
    let start = self.within(frame, aspect);
    let width = cover(frame, aspect) * start.zoom;
    Self {
      focus_x: start.focus_x - delta.0 / width,
      focus_y: start.focus_y - delta.1 * aspect / width,
      zoom: start.zoom * scale,
    }
    .within(frame, aspect)
  }
}

#[cfg(test)]
mod tests;
