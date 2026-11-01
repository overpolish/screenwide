// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A lens's grips, as the native chrome needs them.

use crate::handles::{normalised_point, stroke_width, NativeAnnotationHandles};
use crate::outline::native::radius;
use crate::redact::model::{corners, redact_box};
use crate::{AnnotationKind, AnnotationPoint, AnnotationStyle};

/// A lens reads the slots as a shape does: `start` is its box's top-left
/// corner and `end` its bottom-right, normalised over the source, `middle`
/// its centre, `start_head` the corner radius as a percentage of its shorter
/// side and `end_head` its zoom; the native side places the eight grips of the
/// layer selection's box from its corners, and reads what lies under it
/// through it. `width` is the lens's width, which its halo reaches by.
pub(crate) fn grips(
  start: AnnotationPoint,
  end: AnnotationPoint,
  zoom: f64,
  style: &AnnotationStyle,
  index: u32,
  source: (u32, u32),
  image_width: f64,
) -> NativeAnnotationHandles {
  let (top_left, bottom_right) = corners(redact_box(start, end));
  let (start_x, start_y) = normalised_point(top_left, source);
  let (end_x, end_y) = normalised_point(bottom_right, source);
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Magnify.raw(),
    flags: 0,
    start_x,
    start_y,
    middle_x: (start_x + end_x) / 2.0,
    middle_y: (start_y + end_y) / 2.0,
    end_x,
    end_y,
    start_head: radius(style),
    end_head: zoom,
    width: stroke_width(style, image_width),
    radius: 0.0,
    tilt_x: 0.0,
    tilt_y: 0.0,
  }
}
