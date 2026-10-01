// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A highlight laid by hand: the box a drag spans, gone over in strokes of one
//! marker's width. A marker does not grow to fit what it covers, so a box
//! taller than one stroke takes several, each just over the one before. The
//! compositor draws an edge one stroke shares with the next straight, so the
//! small overlap never opens onto the page.

use super::model::HighlightBand;
use crate::editor::annotations::AnnotationPoint;

/// How far apart strokes are laid, as a share of their width: neighbours
/// overlap by the rest, enough to join without stacking up.
const STRIDE: f64 = 0.92;

/// The strokes covering the box between `start` and `end`, top to bottom,
/// each `width` tall and spanning the box. A drag no taller than one stroke is
/// one stroke, centred on the drag; at exactly one stroke the two lay the same
/// band, so a box grows out of a single stroke without a jump.
pub(crate) fn strokes(
  start: AnnotationPoint,
  end: AnnotationPoint,
  width: f64,
) -> Vec<HighlightBand> {
  let width = if width.is_finite() {
    width.max(1.0)
  } else {
    1.0
  };
  let span = HighlightBand::between(start, end);
  let tall = span.bottom - span.top;
  let stroke = |top: f64| HighlightBand {
    top,
    bottom: top + width,
    ..span
  };
  if tall <= width {
    return vec![stroke((span.top + span.bottom - width) * 0.5)];
  }
  // The first stroke lies along the box's top and the last along its bottom;
  // the rest share the height between them evenly.
  let count = ((tall - width) / (width * STRIDE)).ceil() as usize + 1;
  let step = (tall - width) / (count - 1) as f64;
  (0..count)
    .map(|index| stroke(span.top + step * index as f64))
    .collect()
}

/// How far down the box its start grip (`is_start`) or its end grip sits:
/// half way down its first or its last stroke, where the chrome draws them.
pub(crate) fn grip(top: f64, bottom: f64, width: f64, is_start: bool) -> f64 {
  let width = if width.is_finite() {
    width.max(1.0)
  } else {
    1.0
  };
  if bottom - top <= width {
    (top + bottom) * 0.5
  } else if is_start {
    top + width * 0.5
  } else {
    bottom - width * 0.5
  }
}

/// The edge that puts the start grip (`is_start`) or the end grip at `at`,
/// the opposite edge staying at `fixed`: the inverse of [`grip`], so a grip
/// stays under the hand that drags it. A box's grip moves with its edge; one
/// stroke's sits between both edges and moves half as far.
pub(crate) fn edge_for_grip(fixed: f64, at: f64, width: f64, is_start: bool) -> f64 {
  let half = if width.is_finite() {
    width.max(1.0)
  } else {
    1.0
  } * 0.5;
  match is_start {
    true if at <= fixed - half => at - half,
    false if at >= fixed + half => at + half,
    _ => 2.0 * at - fixed,
  }
}

/// The box's corners in reading order, the top-left first, whichever way it
/// was drawn. A grip moves one of these, and the other stays where it is.
pub(crate) fn corners(
  start: AnnotationPoint,
  end: AnnotationPoint,
) -> (AnnotationPoint, AnnotationPoint) {
  let span = HighlightBand::between(start, end);
  (
    AnnotationPoint {
      x: span.left,
      y: span.top,
    },
    AnnotationPoint {
      x: span.right,
      y: span.bottom,
    },
  )
}

#[cfg(test)]
mod tests;
