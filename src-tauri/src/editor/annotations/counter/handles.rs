// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A counter's grips, as the native chrome needs them.

use crate::editor::annotations::handles::{
  normalised_point, stroke_width, NativeAnnotationHandles,
};
use crate::editor::annotations::{AnnotationKind, AnnotationPoint, AnnotationStyle};

/// A counter reads the arrow's slots differently: every point is the disc's
/// centre, `start_head` is where its tail points in radians clockwise from
/// east, `end_head` is zero, and `width` is the disc's diameter as a share of
/// the drawn width.
///
/// The tail's tip is not sent as a point. A normalised point is a share of
/// the image in each axis, and those shares differ on a picture that is not
/// square, so a circular offset does not survive the trip. The angle does,
/// and the native side - which works in isotropic display points - places the
/// tip from it and the disc.
pub(crate) fn grips(
  center: AnnotationPoint,
  angle: f64,
  style: &AnnotationStyle,
  index: u32,
  source: (u32, u32),
  image_width: f64,
) -> NativeAnnotationHandles {
  let (start_x, start_y) = normalised_point(center, source);
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Counter.raw(),
    padding: 0,
    start_x,
    start_y,
    middle_x: start_x,
    middle_y: start_y,
    end_x: start_x,
    end_y: start_y,
    start_head: angle,
    end_head: 0.0,
    width: stroke_width(style, image_width),
  }
}
