// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A spotlight's grips, as the native chrome needs them.

use super::model::share;
use crate::editor::annotations::handles::{normalised_point, NativeAnnotationHandles};
use crate::editor::annotations::redact::model::{corners, redact_box};
use crate::editor::annotations::{AnnotationKind, AnnotationPoint, AnnotationStyle};

/// A spotlight reads the slots as a redaction does: `start` is the box's
/// top-left corner and `end` its bottom-right, normalised over the source,
/// `middle` its centre and `start_head` the corner radius, from which the
/// native side places the eight grips of the layer selection's box and its
/// radius dot.
pub(crate) fn grips(
  start: AnnotationPoint,
  end: AnnotationPoint,
  style: &AnnotationStyle,
  index: u32,
  source: (u32, u32),
) -> NativeAnnotationHandles {
  let (top_left, bottom_right) = corners(redact_box(start, end));
  let (start_x, start_y) = normalised_point(top_left, source);
  let (end_x, end_y) = normalised_point(bottom_right, source);
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Spotlight.raw(),
    padding: 0,
    start_x,
    start_y,
    middle_x: (start_x + end_x) / 2.0,
    middle_y: (start_y + end_y) / 2.0,
    end_x,
    end_y,
    start_head: share(style.radius),
    end_head: 0.0,
    width: 0.0,
  }
}
