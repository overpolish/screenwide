// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The edges of a detected UI element, and where a tip riding one lands.
//!
//! An arrow's tip takes a whole edge rather than a handful of points along it:
//! one axis is pinned and the other still follows the hand, so the arrow can
//! be aimed anywhere down a button's side and lands on a corner only where two
//! edges are both in reach. The edges belong to the detected box grown by
//! [`ELEMENT_PADDING`], because an arrowhead butted against the thing it
//! points at reads worse than one a hair off it.

use super::AnnotationPoint;
use crate::ruler::analysis::ComponentBox;

/// How far outside a detected element its snap edges sit, as a share of the
/// source's shorter side: the gap that keeps an arrowhead off the thing it is
/// aimed at. Measured against the picture rather than the screen so the
/// committed geometry does not change with the zoom it was drawn at.
const ELEMENT_PADDING: f64 = 0.005;

/// How large a rectangle can be and still be treated as a piece of something
/// bigger, and how far the piece it joins may grow, both as a share of the
/// source's shorter side and both measured on the padded rectangle. Roughly
/// 32 and 65 pixels on a 1080-high source: icon scale, and icon scale with
/// room for a badge beside it.
const FRAGMENT_SIZE: f64 = 0.03;
const CLUSTER_SIZE: f64 = 0.06;

/// A detected element grown by its padding, in source pixels. Fractional
/// because the padding is a share of the picture rather than a whole number of
/// its pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SnapBounds {
  pub(crate) x: f64,
  pub(crate) y: f64,
  pub(crate) width: f64,
  pub(crate) height: f64,
}

impl SnapBounds {
  /// One element's snap rectangle: its detected bounds, grown by `padding` on
  /// every side.
  pub(crate) fn padded(bounds: ComponentBox, padding: f64) -> Self {
    Self {
      x: f64::from(bounds.x) - padding,
      y: f64::from(bounds.y) - padding,
      width: f64::from(bounds.width) + padding * 2.0,
      height: f64::from(bounds.height) + padding * 2.0,
    }
  }

  pub(crate) fn right(&self) -> f64 {
    self.x + self.width
  }

  pub(crate) fn bottom(&self) -> f64 {
    self.y + self.height
  }

  /// Whether two rectangles touch at all. Closed, so two that share an edge
  /// exactly count: at that distance their edges are one target to a hand.
  pub(crate) fn intersects(&self, other: Self) -> bool {
    self.x <= other.right()
      && other.x <= self.right()
      && self.y <= other.bottom()
      && other.y <= self.bottom()
  }

  /// The smallest rectangle holding both.
  pub(crate) fn union(&self, other: Self) -> Self {
    let x = self.x.min(other.x);
    let y = self.y.min(other.y);
    Self {
      x,
      y,
      width: self.right().max(other.right()) - x,
      height: self.bottom().max(other.bottom()) - y,
    }
  }
}

/// The padding one source size gives every element it detects.
pub(crate) fn element_padding(source: (u32, u32)) -> f64 {
  f64::from(source.0.max(1)).min(f64::from(source.1.max(1))) * ELEMENT_PADDING
}

/// Collapse icon-scale fragments into the single element they make up.
///
/// A dashed or dotted glyph is detected as one small box per dash, each a
/// valid shape but none of them a thing worth aiming at. Fragments whose
/// padded rectangles touch merge into their union, so a run of dashes becomes
/// one rectangle the width of the glyph. The padding is already the gap that
/// makes two edges indistinguishable to a hand, so borrowing it as the merge
/// distance loses no candidate the pointer could actually reach.
///
/// Only fragments (both dimensions under [`FRAGMENT_SIZE`]) participate, and a
/// cluster stops growing once either dimension reaches [`CLUSTER_SIZE`]. The
/// cap is what keeps single-linkage chaining from swallowing a whole row of
/// closely spaced icons into one undifferentiated box; anything above
/// fragment size passes through untouched, so buttons and cards keep their
/// edges.
///
/// Seeds grow left to right over fragments sorted by `x`. The inner scan
/// starts past the seed and stops at the first fragment whose `x` is past the
/// cluster's right edge, which keeps this near-linear in the number of
/// fragments rather than quadratic.
pub(crate) fn merge_fragments(bounds: Vec<SnapBounds>, source: (u32, u32)) -> Vec<SnapBounds> {
  let side = f64::from(source.0.max(1)).min(f64::from(source.1.max(1)));
  let fragment = side * FRAGMENT_SIZE;
  let cluster = side * CLUSTER_SIZE;

  let mut kept: Vec<SnapBounds> = Vec::new();
  let mut fragments: Vec<SnapBounds> = Vec::new();
  for item in bounds {
    if item.width <= fragment && item.height <= fragment {
      fragments.push(item);
    } else {
      kept.push(item);
    }
  }
  fragments.sort_by(|a, b| a.x.total_cmp(&b.x));

  let mut merged: Vec<SnapBounds> = Vec::with_capacity(fragments.len());
  let mut claimed = vec![false; fragments.len()];
  for seed in 0..fragments.len() {
    if claimed[seed] {
      continue;
    }
    claimed[seed] = true;
    let mut rect = fragments[seed];
    loop {
      let grown = rect;
      for i in seed + 1..fragments.len() {
        if fragments[i].x > rect.right() {
          break;
        }
        if claimed[i] || !rect.intersects(fragments[i]) {
          continue;
        }
        let joined = rect.union(fragments[i]);
        if joined.width <= cluster && joined.height <= cluster {
          rect = joined;
          claimed[i] = true;
        }
      }
      if rect == grown {
        break;
      }
    }
    merged.push(rect);
  }
  kept.extend(merged);
  kept
}

/// The nearest element edge to `tip` within `threshold`, the tip moved onto
/// it, and the element it belongs to.
///
/// The nearest edge decides which element the tip is riding; the two
/// perpendicular edges of that same element are then offered as well, so a tip
/// near a corner lands on it and a tip anywhere else down a side pins only
/// that side and keeps following the hand along it. An edge is out of reach
/// unless the tip lies within its span, grown by the threshold so the corners
/// are still reachable from outside the rectangle.
///
/// This is a linear scan over four edges of every detected element, which a 5K
/// frame at subtle tolerance can make thousands of. It is cheap enough per
/// sample only because the padded rectangles were computed once, when the
/// boxes landed.
pub(crate) fn snap_edges(
  tip: AnnotationPoint,
  bounds: &[SnapBounds],
  threshold: f64,
) -> Option<(AnnotationPoint, SnapBounds)> {
  let mut chosen: Option<(f64, SnapBounds, bool, f64)> = None;
  for rect in bounds {
    let mut consider = |vertical: bool, edge: f64| {
      let (position, span) = if vertical {
        (tip.x, (tip.y - rect.y).min(rect.bottom() - tip.y))
      } else {
        (tip.y, (tip.x - rect.x).min(rect.right() - tip.x))
      };
      let distance = (edge - position).abs();
      if span < -threshold || distance > threshold {
        return;
      }
      if chosen.is_none_or(|(best, ..)| distance < best) {
        chosen = Some((distance, *rect, vertical, edge));
      }
    };
    consider(true, rect.x);
    consider(true, rect.right());
    consider(false, rect.y);
    consider(false, rect.bottom());
  }
  let (_, rect, vertical, edge) = chosen?;
  let across = if vertical {
    nearest(tip.y, rect.y, rect.bottom(), threshold)
  } else {
    nearest(tip.x, rect.x, rect.right(), threshold)
  };
  Some((
    if vertical {
      AnnotationPoint {
        x: edge,
        y: across.unwrap_or(tip.y),
      }
    } else {
      AnnotationPoint {
        x: across.unwrap_or(tip.x),
        y: edge,
      }
    },
    rect,
  ))
}

/// Whichever of an element's two perpendicular edges `position` is nearest, if
/// either is in reach. This is what turns riding a side into landing on a
/// corner.
fn nearest(position: f64, near: f64, far: f64, threshold: f64) -> Option<f64> {
  [near, far]
    .into_iter()
    .map(|edge| ((edge - position).abs(), edge))
    .filter(|(distance, _)| *distance <= threshold)
    .min_by(|a, b| a.0.total_cmp(&b.0))
    .map(|(_, edge)| edge)
}
