// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A redaction's grips, as the native chrome needs them.

use super::model::{corners, redact_box};
use crate::editor::annotations::handles::{normalised_point, NativeAnnotationHandles};
use crate::editor::annotations::{AnnotationKind, AnnotationPoint};

/// A redaction reads the arrow's slots as its box: `start` is the top-left
/// corner and `end` the bottom-right, both normalised over the source, and
/// `middle` the centre. The native side places the eight grips of the layer
/// selection's own box from those, and its radius dot from `start_head`, the
/// corner radius as a percentage of the box's shorter side.
pub(crate) fn grips(
  start: AnnotationPoint,
  end: AnnotationPoint,
  radius: f64,
  index: u32,
  source: (u32, u32),
) -> NativeAnnotationHandles {
  let (top_left, bottom_right) = corners(redact_box(start, end));
  let (start_x, start_y) = normalised_point(top_left, source);
  let (end_x, end_y) = normalised_point(bottom_right, source);
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Redact.raw(),
    padding: 0,
    start_x,
    start_y,
    middle_x: (start_x + end_x) / 2.0,
    middle_y: (start_y + end_y) / 2.0,
    end_x,
    end_y,
    start_head: if radius.is_finite() {
      radius.clamp(0.0, 50.0)
    } else {
      0.0
    },
    end_head: 0.0,
    width: 0.0,
  }
}
