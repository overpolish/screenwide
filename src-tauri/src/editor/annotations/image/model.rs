// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making an image, and where it reaches.
//!
//! An image is held by its middle, its longer side and its turn, so moving,
//! sizing and turning it each touch one number. Its picture's own proportions
//! ride beside it as `aspect`, which only choosing another picture changes.

use crate::editor::annotations::{
  Annotation, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};
use serde::{Deserialize, Serialize};

/// What an image shows: its picture by library id, and that picture's width
/// over its height. What the editor hands over as the picture the next
/// image is made with, along with how many pixels long the picture is on
/// its longer side, and how it plays where it moves. The twin of
/// `ImageArt` in `src/features/editor/annotations/annotations.ts`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageArt {
  pub asset: String,
  pub aspect: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub pixels: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub play: Option<super::ImagePlay>,
}

impl ImageArt {
  /// The art as an image can carry it: a picture with no usable proportions
  /// is drawn square rather than not at all, one with no usable size at
  /// [`NEW_IMAGE_POINTS`], and one with no usable animation still.
  fn held(&self) -> Self {
    Self {
      asset: self.asset.clone(),
      aspect: if self.aspect.is_finite() && self.aspect > 0.0 {
        self.aspect
      } else {
        1.0
      },
      pixels: self
        .pixels
        .filter(|pixels| pixels.is_finite() && *pixels >= MIN_IMAGE_SIZE),
      play: self.play.filter(super::ImagePlay::is_usable),
    }
  }

  /// The art for an image on a picture `source` pixels across: a picture
  /// of its own keeps its size where it fits, and is shrunk to the largest
  /// that does where it does not.
  pub(crate) fn fitted(&self, (width, height): (u32, u32)) -> Self {
    let (width, height) = (f64::from(width), f64::from(height));
    let aspect = self.held().aspect;
    // The longest the longer side may be with both sides inside the picture.
    let room = if aspect >= 1.0 {
      width.min(height * aspect)
    } else {
      height.min(width / aspect)
    };
    Self {
      pixels: self
        .pixels
        .map(|pixels| if room > 0.0 { pixels.min(room) } else { pixels }),
      ..self.clone()
    }
  }
}

/// How long a fresh image's longer side is, in points of the capture, so it
/// lands the same size on a 2x capture as on a 1x one.
pub(crate) const NEW_IMAGE_POINTS: f64 = 96.0;

/// The shortest an image's longer side may be dragged to, in source pixels.
pub(crate) const MIN_IMAGE_SIZE: f64 = 8.0;

/// The dress a fresh image wears before anything has been chosen: square
/// corners and no shadow. It has no colour or pen of its own, and carries
/// the least width every clip must, as a spotlight does. The twin of the
/// image's row in `ANNOTATION_SIZES`.
pub(crate) fn default_image_style() -> AnnotationStyle {
  AnnotationStyle {
    align: Default::default(),
    blur: false,
    color: crate::editor::annotations::model::NEW_ANNOTATION_COLOR.to_owned(),
    head: AnnotationHead::None,
    hand_drawn: false,
    manual: false,
    tint: false,
    radius: 0.0,
    redaction: Default::default(),
    shadow: false,
    softness: 0.0,
    strength: 0.0,
    width: 1.0,
  }
}

/// An image showing `art`, centred on `center` and upright: the picture at
/// its own size, or with its longer side [`NEW_IMAGE_POINTS`] points long
/// where its size is not known. `source_per_size` turns points into source
/// pixels; zero or less where the workspace has not said, which takes a point
/// as a source pixel.
pub(crate) fn new_image(
  id: String,
  center: AnnotationPoint,
  art: &ImageArt,
  style: Option<&AnnotationStyle>,
  source_per_size: f64,
) -> Annotation {
  let art = art.held();
  let scale = if source_per_size.is_finite() && source_per_size > 0.0 {
    source_per_size
  } else {
    1.0
  };
  Annotation {
    above_camera: false,
    animated: true,
    id,
    held: None,
    pen: false,
    reveal: crate::editor::annotations::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Image {
      center,
      size: art.pixels.unwrap_or(NEW_IMAGE_POINTS * scale),
      angle: 0.0,
      aspect: art.aspect,
      flip: false,
      asset: art.asset,
      play: art.play,
      sway: None,
      clock_ms: None,
    },
    style: style.cloned().unwrap_or_else(default_image_style),
  }
}

/// Whether an image is somewhere it can be drawn: a document read from disk
/// carries whatever it was written with.
pub(crate) fn placed(center: AnnotationPoint, size: f64, angle: f64, aspect: f64) -> bool {
  center.x.is_finite()
    && center.y.is_finite()
    && size.is_finite()
    && size > 0.0
    && angle.is_finite()
    && aspect.is_finite()
    && aspect > 0.0
}

/// Half the picture's width and height, from its longer side and its width
/// over its height.
pub(crate) fn half_extents(size: f64, aspect: f64) -> (f64, f64) {
  let half = size.max(0.0) * 0.5;
  if aspect >= 1.0 {
    (half, half / aspect)
  } else {
    (half * aspect, half)
  }
}

/// The picture's own right and down, turned `angle` radians clockwise.
pub(crate) fn axes(angle: f64) -> ([f64; 2], [f64; 2]) {
  let (sin, cos) = angle.sin_cos();
  ([cos, sin], [-sin, cos])
}

/// `point` in the picture's own frame: how far it lies along the picture's
/// own right and down from `center`, the picture turned `angle`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn to_local(
  center: AnnotationPoint,
  angle: f64,
  point: AnnotationPoint,
) -> AnnotationPoint {
  let (across, down) = axes(angle);
  let (x, y) = (point.x - center.x, point.y - center.y);
  AnnotationPoint {
    x: x * across[0] + y * across[1],
    y: x * down[0] + y * down[1],
  }
}

/// The point `local` names in the picture's own frame, back on the picture
/// it was taken from.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn from_local(
  center: AnnotationPoint,
  angle: f64,
  local: AnnotationPoint,
) -> AnnotationPoint {
  let (across, down) = axes(angle);
  AnnotationPoint {
    x: center.x + across[0] * local.x + down[0] * local.y,
    y: center.y + across[1] * local.x + down[1] * local.y,
  }
}

/// The upright box the turned picture fits in, as its top-left and
/// bottom-right corners.
pub(crate) fn bounds(
  center: AnnotationPoint,
  size: f64,
  angle: f64,
  aspect: f64,
) -> (AnnotationPoint, AnnotationPoint) {
  let (half_width, half_height) = half_extents(size, aspect);
  let (across, down) = axes(angle);
  let reach_x = (across[0] * half_width).abs() + (down[0] * half_height).abs();
  let reach_y = (across[1] * half_width).abs() + (down[1] * half_height).abs();
  (
    AnnotationPoint {
      x: center.x - reach_x,
      y: center.y - reach_y,
    },
    AnnotationPoint {
      x: center.x + reach_x,
      y: center.y + reach_y,
    },
  )
}

/// The turn that stands the image's top towards `point`: its turning grip
/// sits straight above its middle. A press exactly on the middle keeps the
/// turn it had. `snap` holds it to the eighth turns the counter's tail and
/// the angle control snap to.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn image_turn(
  center: AnnotationPoint,
  point: AnnotationPoint,
  current: f64,
  snap: bool,
) -> f64 {
  let (dx, dy) = (point.x - center.x, point.y - center.y);
  if dx == 0.0 && dy == 0.0 {
    return current;
  }
  let angle = dx.atan2(-dy);
  if !snap {
    return angle;
  }
  let step = crate::editor::annotations::counter::model::COUNTER_SNAP_RADIANS;
  (angle / step).round() * step
}

/// What an image offers the snap engine: the upright box it fits in.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn field_box(
  center: AnnotationPoint,
  size: f64,
  angle: f64,
  aspect: f64,
) -> Option<crate::editor::annotations::snap::SnapBox> {
  let (low, high) = bounds(center, size, angle, aspect);
  Some(crate::editor::annotations::snap::SnapBox {
    x: low.x,
    y: low.y,
    width: high.x - low.x,
    height: high.y - low.y,
  })
}
