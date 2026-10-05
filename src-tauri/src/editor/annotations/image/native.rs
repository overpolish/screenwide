// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's retained draw record.
//!
//! `p0` is the picture's middle, `p1` the middle of its right side and `p2`
//! of its bottom side, all placed like any point and swayed where it sways,
//! so a placement carries its size and turn with it. `head` says whether it
//! is mirrored, `params[0]` carries its corner radius as a percentage of its
//! shorter side, which no placement touches, and `flags` carries `SHADOW`
//! where it casts one. Its
//! asset id travels as the record's text. It has no colour of its own: the
//! record's white carries only how present it is. An image whose picture
//! moves carries the frame it shows or starts on in `params[1]`, how far
//! into its clip the frame is in `params[2]` (negative on a still), and
//! `ONCE` where it plays through once; only the image atlas reads them.

use super::model::{axes, half_extents};
use super::ImagePlay;
use crate::editor::annotations::flags::{ONCE, SHADOW};
use crate::editor::annotations::native::NativeAnnotation;
use crate::editor::annotations::{AnnotationPoint, AnnotationStyle};

pub(crate) fn draw_points(
  center: AnnotationPoint,
  size: f64,
  angle: f64,
  aspect: f64,
) -> [[f32; 2]; 3] {
  let (half_width, half_height) = half_extents(size, aspect);
  let (across, down) = axes(angle);
  let at = |axis: [f64; 2], reach: f64| {
    [
      (center.x + axis[0] * reach) as f32,
      (center.y + axis[1] * reach) as f32,
    ]
  };
  [
    [center.x as f32, center.y as f32],
    at(across, half_width),
    at(down, half_height),
  ]
}

/// Writes what an image's record carries beyond its points. `clock_ms` is
/// how far into its clip the frame is, which only a moving picture's record
/// carries.
pub(crate) fn fill(
  record: &mut NativeAnnotation,
  style: &AnnotationStyle,
  play: Option<&ImagePlay>,
  clock_ms: Option<f64>,
) {
  let (frame, clock) = play.map_or((0.0, -1.0), |play| {
    (
      play.frame as f32,
      clock_ms.map_or(-1.0, |clock| clock as f32),
    )
  });
  record.params = [style.radius as f32, frame, clock];
  record.flags = if style.shadow { SHADOW } else { 0 }
    | if play.is_some_and(|play| play.once) {
      ONCE
    } else {
      0
    };
  record.color = [1.0; 4];
}
