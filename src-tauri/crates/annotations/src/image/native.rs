// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's retained draw record.
//!
//! `p0` is the picture's middle, `p1` the middle of its right side and `p2`
//! of its bottom side, all placed like any point and swayed where it sways,
//! so a placement carries its size and turn with it. `head` says whether it
//! is mirrored, `params[0]` carries its corner radius as a percentage of its
//! shorter side, which no placement touches, `p3` its turn in depth in
//! radians, which no placement touches either, and `flags` carries `SHADOW`
//! where it casts one. Its asset id travels as the record's text. It has no
//! colour of its own: the record's white carries only how present it is. An
//! image whose picture moves carries the frame it shows or starts on in
//! `params[1]`, how far into its clip the frame is in `params[2]` (negative
//! on a still), `ONCE` where it plays through once, and `STRETCHED` where it
//! is stretched rather than upscaled when drawn larger than its picture; only
//! the image atlas reads them.

use super::model::{axes, half_extents};
use super::{ImagePlay, ImageTilt};
use crate::flags::{ONCE, SHADOW, STRETCHED};
use crate::native::NativeAnnotation;
use crate::{AnnotationPoint, AnnotationStyle};

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
/// carries; `stretched` whether it is drawn larger by stretching rather than
/// by the upscaling model, which only the image atlas reads.
pub(crate) fn fill(
  record: &mut NativeAnnotation,
  style: &AnnotationStyle,
  (play, clock_ms): (Option<&ImagePlay>, Option<f64>),
  tilt: ImageTilt,
  stretched: bool,
) {
  record.p3 = tilt.radians();
  let (frame, clock) = play.map_or((0.0, -1.0), |play| {
    (
      play.frame as f32,
      clock_ms.map_or(-1.0, |clock| clock as f32),
    )
  });
  record.params = [style.radius as f32, frame, clock];
  let flag = |on: bool, flag: u32| if on { flag } else { 0 };
  record.flags = flag(style.shadow, SHADOW)
    | flag(play.is_some_and(|play| play.once), ONCE)
    | flag(stretched, STRETCHED);
  record.color = [1.0; 4];
}
