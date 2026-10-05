// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's grips, as the native chrome needs them.

use super::model::{bounds, half_extents};
use crate::editor::annotations::handles::{normalised_point, NativeAnnotationHandles};
use crate::editor::annotations::{AnnotationKind, AnnotationPoint};

/// An image reads the slots its own way: `start` and `end` are the corners
/// of the upright box it fits in, which the group's frame and the marquee
/// read; `middle` is its middle; `start_head` its turn in radians clockwise;
/// `end_head` and `width` its half width and half height as shares of the
/// drawn width; and `radius` its corner radius as a percentage of its
/// shorter side. The native side, which works in isotropic display points,
/// places the picture, its own frame and its grips from those, as it places
/// a counter's tail from its angle.
pub(crate) fn grips(
  center: AnnotationPoint,
  size: f64,
  angle: f64,
  aspect: f64,
  radius: f64,
  index: u32,
  source: (u32, u32),
) -> NativeAnnotationHandles {
  let (low, high) = bounds(center, size, angle, aspect);
  let (start_x, start_y) = normalised_point(low, source);
  let (end_x, end_y) = normalised_point(high, source);
  let (middle_x, middle_y) = normalised_point(center, source);
  let (half_width, half_height) = half_extents(size, aspect);
  let across = f64::from(source.0.max(1));
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Image.raw(),
    flags: 0,
    start_x,
    start_y,
    middle_x,
    middle_y,
    end_x,
    end_y,
    start_head: angle,
    end_head: half_width / across,
    width: half_height / across,
    radius,
  }
}
