// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Recognising what a cursor is from its picture.
//!
//! macOS names no cursor another application sets: it hands over an image and
//! a hotspot. The system's own cursors are recognised by their exact bytes,
//! but an application drawing an older or its own design - the classic 9x18
//! I-beam most text views still show - matches none of them. What an I-beam,
//! an arrow or a crosshair *is* survives any redesign, so this reads the
//! silhouette instead: where the hotspot sits in what is drawn, and how wide
//! each part of it is. Nothing here depends on size, colour, outline or
//! scale, and a shape that fits no rule stays [`CursorStyle::Custom`].
//!
//! The grabbing hands are left to the exact match: the system draws them as
//! one silhouette whose outline joins the fingers, so an open hand and a
//! closed one, or either and a blob, cannot be told apart by shape.

use super::format::CursorStyle;

/// The opaque part of a cursor's picture, cropped to what is drawn, and its
/// hotspot as a share of that crop across and down.
struct Silhouette {
  width: usize,
  height: usize,
  opaque: Vec<bool>,
  across: f64,
  down: f64,
}

impl Silhouette {
  /// Only what is drawn solidly counts: a soft shadow or an antialiased rim
  /// would blur every proportion measured below.
  fn new(width: usize, height: usize, alpha: &[u8], hotspot: (f64, f64)) -> Option<Self> {
    let solid = |x: usize, y: usize| alpha.get(y * width + x).is_some_and(|&a| a >= 128);
    let (mut left, mut top, mut right, mut bottom) = (usize::MAX, usize::MAX, 0, 0);
    for y in 0..height {
      for x in 0..width {
        if solid(x, y) {
          (left, top) = (left.min(x), top.min(y));
          (right, bottom) = (right.max(x + 1), bottom.max(y + 1));
        }
      }
    }
    if left >= right || top >= bottom {
      return None;
    }
    let (cropped_width, cropped_height) = (right - left, bottom - top);
    let opaque = (top..bottom)
      .flat_map(|y| (left..right).map(move |x| (x, y)))
      .map(|(x, y)| solid(x, y))
      .collect();
    Some(Self {
      width: cropped_width,
      height: cropped_height,
      opaque,
      across: (hotspot.0 - left as f64) / cropped_width as f64,
      down: (hotspot.1 - top as f64) / cropped_height as f64,
    })
  }

  /// The same silhouette turned on its side, so a rule for a vertical shape
  /// reads its horizontal twin.
  fn transposed(&self) -> Self {
    let opaque = (0..self.width)
      .flat_map(|x| (0..self.height).map(move |y| (x, y)))
      .map(|(x, y)| self.at(x, y))
      .collect();
    Self {
      width: self.height,
      height: self.width,
      opaque,
      across: self.down,
      down: self.across,
    }
  }

  fn at(&self, x: usize, y: usize) -> bool {
    self.opaque[y * self.width + x]
  }

  fn aspect(&self) -> f64 {
    self.width as f64 / self.height as f64
  }

  fn fill(&self) -> f64 {
    self.opaque.iter().filter(|&&solid| solid).count() as f64 / self.opaque.len() as f64
  }

  /// How much of row `y` is drawn, as a share of the width.
  fn row(&self, y: usize) -> f64 {
    (0..self.width).filter(|&x| self.at(x, y)).count() as f64 / self.width as f64
  }

  /// How many separate strokes row `y` crosses.
  fn runs(&self, y: usize) -> usize {
    (0..self.width)
      .filter(|&x| self.at(x, y) && (x == 0 || !self.at(x - 1, y)))
      .count()
  }

  /// The rows from `from` to `to` down the silhouette, as shares of its
  /// height; never empty.
  fn rows(&self, from: f64, to: f64) -> std::ops::Range<usize> {
    let first = ((from * self.height as f64).floor() as usize).min(self.height - 1);
    let last = ((to * self.height as f64).ceil() as usize).clamp(first + 1, self.height);
    first..last
  }

  fn widest(&self, from: f64, to: f64) -> f64 {
    self.rows(from, to).map(|y| self.row(y)).fold(0.0, f64::max)
  }

  fn narrowest(&self, from: f64, to: f64) -> f64 {
    self.rows(from, to).map(|y| self.row(y)).fold(1.0, f64::min)
  }

  fn hotspot_row(&self) -> usize {
    ((self.down * self.height as f64) as usize).min(self.height - 1)
  }

  fn hotspot_within(&self, from: f64, to: f64) -> bool {
    (from..=to).contains(&self.across) && (from..=to).contains(&self.down)
  }
}

/// A text I-beam standing upright: a thin stem between two flat serifs, the
/// hotspot on the stem's middle. The classic design crosses its stem with a
/// short bar at the hotspot, so the stem is thin along most of its length
/// rather than all of it, and no part of it is as wide as the serifs.
fn i_beam(shape: &Silhouette) -> bool {
  let stem = shape.rows(0.25, 0.75);
  let thin = stem.clone().filter(|&y| shape.row(y) <= 0.45).count();
  shape.aspect() <= 0.625
    && shape.hotspot_within(0.3, 0.7)
    && shape.widest(0.0, 0.12) >= 0.6
    && shape.widest(0.88, 1.0) >= 0.6
    && thin as f64 >= stem.len() as f64 * 0.6
    && shape.widest(0.25, 0.75) <= 0.75
    && shape.narrowest(0.25, 0.75) > 0.0
}

/// A double-headed arrow standing upright: narrow tips, wide heads, a waist
/// between them.
fn resize(shape: &Silhouette) -> bool {
  shape.aspect() <= 0.9
    && shape.hotspot_within(0.3, 0.7)
    && shape.widest(0.0, 0.1) <= 0.5
    && shape.widest(0.9, 1.0) <= 0.5
    && shape.widest(0.0, 0.45) >= 0.6
    && shape.widest(0.55, 1.0) >= 0.6
    && shape.narrowest(0.3, 0.7) <= 0.5
}

/// Two thin strokes crossing at the hotspot.
fn crosshair(shape: &Silhouette) -> bool {
  let column = shape.transposed();
  (0.8..=1.25).contains(&shape.aspect())
    && shape.hotspot_within(0.35, 0.65)
    && shape.fill() <= 0.45
    && shape.row(shape.hotspot_row()) >= 0.8
    && column.row(column.hotspot_row()) >= 0.8
    && shape.widest(0.0, 0.25) <= 0.4
    && shape.widest(0.75, 1.0) <= 0.4
}

/// A ring struck through: three strokes across the middle each way, and
/// nothing in the corners.
fn not_allowed(shape: &Silhouette) -> bool {
  let column = shape.transposed();
  let corner = |x: usize, y: usize| shape.at(x, y);
  let (last_x, last_y) = (shape.width - 1, shape.height - 1);
  (0.85..=1.15).contains(&shape.aspect())
    && shape.hotspot_within(0.35, 0.65)
    && (0.25..=0.65).contains(&shape.fill())
    && !(corner(0, 0) || corner(last_x, 0) || corner(0, last_y) || corner(last_x, last_y))
    && shape.runs(shape.height / 2) >= 3
    && column.runs(column.height / 2) >= 3
}

/// The arrow: the hotspot at its tip in the top-left corner, a straight left
/// edge running down from it, and the body widening below.
fn arrow(shape: &Silhouette) -> bool {
  let edge = shape.rows(0.05, 0.5);
  let straight = edge
    .clone()
    .filter(|&y| (0..=shape.width / 5).any(|x| shape.at(x, y)))
    .count();
  shape.across <= 0.25
    && shape.down <= 0.2
    && shape.aspect() <= 1.0
    && straight as f64 >= edge.len() as f64 * 0.8
    && shape.widest(0.0, 0.15) <= 0.5
    && shape.widest(0.3, 0.8) >= 0.6
}

/// A pointing hand: the hotspot on one raised finger, over a broad hand.
fn pointing_hand(shape: &Silhouette) -> bool {
  shape.down <= 0.2
    && (0.15..=0.7).contains(&shape.across)
    && (0.6..=1.1).contains(&shape.aspect())
    && shape.fill() >= 0.45
    && shape.widest(0.0, 0.2) <= 0.45
    && shape.narrowest(0.55, 0.85) >= 0.6
}

/// What the cursor drawn with `alpha` - one byte a pixel, `width` by `height`
/// - and its hotspot at `hotspot` pixels is.
pub(crate) fn recognise(
  width: usize,
  height: usize,
  alpha: &[u8],
  hotspot: (f64, f64),
) -> CursorStyle {
  let Some(shape) = Silhouette::new(width, height, alpha, hotspot) else {
    return CursorStyle::Custom;
  };
  let lying = shape.transposed();
  if i_beam(&shape) {
    CursorStyle::IBeam
  } else if i_beam(&lying) {
    CursorStyle::VerticalIBeam
  } else if resize(&shape) {
    CursorStyle::ResizeVertical
  } else if resize(&lying) {
    CursorStyle::ResizeHorizontal
  } else if crosshair(&shape) {
    CursorStyle::Crosshair
  } else if not_allowed(&shape) {
    CursorStyle::NotAllowed
  } else if arrow(&shape) {
    CursorStyle::Arrow
  } else if pointing_hand(&shape) {
    CursorStyle::PointingHand
  } else {
    CursorStyle::Custom
  }
}

#[cfg(test)]
#[path = "shape_tests.rs"]
mod tests;
