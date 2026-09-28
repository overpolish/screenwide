// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A highlight's prepared record, and the distance that picks it from its
//! grips.
//!
//! A highlight has as many bands as it has lines, which no fixed record holds,
//! so its bands ride in the side buffer in source pixels and the shader places
//! each one itself. The prepared record says how: `a` is where the source's
//! origin lands in the pixels it is drawn in and `b` how far one source pixel
//! reaches across and down. `c` is its tone - the page's brightness and its
//! ink's - `low` and `high` its reveal window, which the shader shares out
//! between its lines, and `width` one, since both passes skip a record of no
//! width.

use super::model::HighlightTone;
use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::text::geometry::rounded_box_distance;

/// The retained draw record's three points: the source's origin and the
/// source pixel `(1, 1)`, which every placement carries into the pixels the
/// highlight is drawn in and the shader places the bands by, and the tone,
/// which no placement touches.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn draw_points(tone: HighlightTone) -> [[f32; 2]; 3] {
  [
    [0.0, 0.0],
    [1.0, 1.0],
    [tone.surface as f32, tone.ink as f32],
  ]
}

/// The record for a highlight whose source origin lands at `origin` and whose
/// source pixel `(1, 1)` lands at `unit`, in the caller's pixels.
pub(crate) fn prepare_highlight(
  origin: [f32; 2],
  unit: [f32; 2],
  tone: [f32; 2],
  reveal: AnnotationReveal,
) -> ArrowGeometry {
  ArrowGeometry {
    a: origin,
    b: [unit[0] - origin[0], unit[1] - origin[1]],
    c: tone,
    width: 1.0,
    low: reveal.low.clamp(0.0, 1.0),
    high: reveal.high.clamp(0.0, 1.0),
    ..ArrowGeometry::default()
  }
}

/// A highlight as its grips' record carries it, in the caller's pixels: the
/// first band's top-left corner and bottom edge, the last band's top edge and
/// bottom-right corner, and how far left and right the block of bands
/// reaches. The twin of the record `handles::grips` fills.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HighlightFlow {
  pub(crate) start: [f32; 2],
  pub(crate) first_bottom: f32,
  pub(crate) last_top: f32,
  pub(crate) end: [f32; 2],
  pub(crate) block: [f32; 2],
}

#[cfg(target_os = "windows")]
impl HighlightFlow {
  /// Where the grip at the selection's start sits: the first band's left
  /// end, half way down it.
  pub(crate) fn start_grip(&self) -> [f32; 2] {
    [self.start[0], (self.start[1] + self.first_bottom) * 0.5]
  }

  /// Where the grip at the selection's end sits: the last band's right end,
  /// half way down it.
  pub(crate) fn end_grip(&self) -> [f32; 2] {
    [self.end[0], (self.last_top + self.end[1]) * 0.5]
  }
}

/// How far `point` falls outside a highlight read from its grips' record:
/// the first line from its start to the block's right, the lines between
/// across the whole block, and the last line from the block's left to its
/// end. Negative inside. A ragged line end is picked a little past its ink,
/// which the hand never notices; the drawn bands are the shader's.
pub(crate) fn flow_distance(point: [f32; 2], flow: &HighlightFlow) -> f32 {
  let [left, right] = flow.block;
  let first = rounded_box_distance(
    point,
    flow.start,
    [right.max(flow.start[0]), flow.first_bottom],
    0.0,
  );
  let last = rounded_box_distance(point, [left.min(flow.end[0]), flow.last_top], flow.end, 0.0);
  let between = if flow.last_top > flow.first_bottom {
    rounded_box_distance(
      point,
      [left, flow.first_bottom],
      [right, flow.last_top],
      0.0,
    )
  } else {
    f32::INFINITY
  };
  first.min(last).min(between)
}
