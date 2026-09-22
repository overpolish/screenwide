// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Equal spacing: where a moving box makes a gap that matches one two other
//! boxes already have.
//!
//! Alignment lines say nothing about rhythm. Three counters down the side of
//! a screenshot read as a set only when the space between the first and the
//! second is the space between the second and the third, and no line in the
//! picture marks where that is. So every pair of static boxes whose
//! cross-axis projections the moving box shares offers its own gap as a
//! measure: the moving box can take that distance before the pair, after it,
//! or split the pair's own span in two by sitting in the middle of it.
//!
//! The reference is always between two boxes that are standing still. A gap
//! the moving box already has to a neighbour would move with it, so matching
//! it would mean matching nothing.
//!
//! Only boxes whose cross-axis span the moving box shares can offer a gap, so
//! those are gathered first and paired among themselves: the work is linear in
//! the document and quadratic only in the boxes level with the one in hand.

use super::{Axis, SnapBox};

/// Where the moving box sits in the run of three the two gaps are measured
/// over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GapPlace {
  /// Before both static boxes, taking the gap they have between them.
  Before,
  /// After both of them, taking the same gap.
  After,
  /// Between them, with its own two gaps made equal.
  Between,
}

/// One equal gap the chrome draws a bar across: where it starts and ends
/// along the snapped axis, and where the bar sits across it, which is the
/// middle of what the two boxes either side of it share.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GapSpan {
  pub(crate) from: f64,
  pub(crate) to: f64,
  pub(crate) cross: f64,
}

/// The two equal gaps one axis snapped to, in order along that axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GapSnap {
  pub(crate) spans: [GapSpan; 2],
}

/// A move that makes two gaps equal, and the pair of static boxes it was
/// measured against.
#[derive(Clone, Copy)]
pub(crate) struct GapCandidate {
  pub(crate) offset: f64,
  place: GapPlace,
  first: SnapBox,
  second: SnapBox,
}

impl GapCandidate {
  /// The two bars to draw, with the moving box already carried to where it
  /// landed in both axes - so a bar sits in the middle of what the boxes
  /// really share rather than of where the hand happened to be.
  pub(crate) fn spans(&self, moved: SnapBox, axis: Axis) -> GapSnap {
    let pair = match self.place {
      GapPlace::Before => [(moved, self.first), (self.first, self.second)],
      GapPlace::After => [(self.first, self.second), (self.second, moved)],
      GapPlace::Between => [(self.first, moved), (moved, self.second)],
    };
    GapSnap {
      spans: pair.map(|(near, far)| span(near, far, axis)),
    }
  }
}

/// The nearest move that lines the moving box up on equal spacing, within
/// `threshold`. Nearest wins, as everywhere else in the engine.
pub(crate) fn snap_gap(
  moving: SnapBox,
  boxes: &[SnapBox],
  axis: Axis,
  threshold: f64,
) -> Option<GapCandidate> {
  let cross = axis.other();
  let level: Vec<SnapBox> = boxes
    .iter()
    .copied()
    .filter(|other| overlaps(moving, *other, cross))
    .collect();
  let mut best: Option<GapCandidate> = None;
  for (index, left) in level.iter().enumerate() {
    for right in &level[index + 1..] {
      // Order the pair along the axis, and skip any that share space: two
      // boxes that overlap have no gap worth copying.
      let (first, second) = if left.min(axis) <= right.min(axis) {
        (*left, *right)
      } else {
        (*right, *left)
      };
      let gap = second.min(axis) - first.max(axis);
      if gap < 0.0 {
        continue;
      }
      let mut offer = |offset: f64, place: GapPlace| {
        if !(offset.abs() <= threshold) {
          return;
        }
        if best.is_none_or(|chosen| offset.abs() < chosen.offset.abs()) {
          best = Some(GapCandidate {
            offset,
            place,
            first,
            second,
          });
        }
      };
      offer(first.min(axis) - gap - moving.max(axis), GapPlace::Before);
      offer(second.max(axis) + gap - moving.min(axis), GapPlace::After);
      // Splitting the pair's own span only means anything where the moving
      // box is already inside it.
      if moving.min(axis) >= first.max(axis) && moving.max(axis) <= second.min(axis) {
        let middle = (first.max(axis) + second.min(axis)) / 2.0;
        offer(middle - moving.centre(axis), GapPlace::Between);
      }
    }
  }
  best
}

/// The gap between two boxes along `axis`, and where a bar across it sits.
fn span(near: SnapBox, far: SnapBox, axis: Axis) -> GapSpan {
  let cross = axis.other();
  GapSpan {
    from: near.max(axis),
    to: far.min(axis),
    cross: (near.min(cross).max(far.min(cross)) + near.max(cross).min(far.max(cross))) / 2.0,
  }
}

/// Whether two boxes share any space along `axis`. Boxes that merely touch
/// do not: there is nothing between them for a bar to sit in the middle of.
fn overlaps(one: SnapBox, other: SnapBox, axis: Axis) -> bool {
  one.min(axis) < other.max(axis) && other.min(axis) < one.max(axis)
}
