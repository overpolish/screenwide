// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a magnifier, and the dress a fresh one wears.
//!
//! The zoom area is a box held by two corners in source pixels, `start` the
//! top-left and `end` the bottom-right; every edit writes them back in that
//! order. The loupe is the same box enlarged, centred on `loupe` and `size`
//! source pixels along its longer side, so its zoom is that size over the
//! zoom area's longer side, and resizing the zoom area leaves the loupe as
//! big as it was. The style rounds both by its `radius` and says in `shadow`
//! whether the loupe casts one; the rim is always drawn at a fresh one's pen.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::snap::SnapBox;
use crate::editor::annotations::AnnotationPoint;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::{Annotation, AnnotationHead, AnnotationShape, AnnotationStyle};

/// The zoom a fresh loupe is set out at, beside the zoom area drawn for it.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_MAGNIFY_ZOOM: f64 = 2.0;

/// The least and most a loupe enlarges. Below the least the loupe hardly
/// differs from what it covers; past the most one pixel fills the loupe.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const MIN_MAGNIFY_ZOOM: f64 = 1.25;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const MAX_MAGNIFY_ZOOM: f64 = 8.0;

/// The rim every loupe is drawn with, in points: an arrow's pen. The twin of
/// the magnifier's `defaultSize` in `widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_MAGNIFY_WIDTH: f64 = 8.0;

/// A fresh magnifier is round: rounded all the way, a square is a circle.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
const NEW_MAGNIFY_RADIUS: f64 = 50.0;

/// The dress a fresh magnifier wears before anything has been chosen: a white
/// rim, which reads against any picture its shadow falls on, round, casting a
/// shadow.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_magnify_style() -> AnnotationStyle {
  AnnotationStyle {
    align: Default::default(),
    blur: false,
    color: "#ffffff".to_owned(),
    head: AnnotationHead::None,
    hand_drawn: false,
    manual: false,
    tint: false,
    radius: NEW_MAGNIFY_RADIUS,
    redaction: Default::default(),
    shadow: true,
    softness: 0.0,
    strength: 0.0,
    width: NEW_MAGNIFY_WIDTH,
  }
}

/// A magnifier with its zoom area at `point`, in `style` or in the tool's own
/// first dress. A fresh one in the editor starts with both corners at the
/// press and no loupe, for the drag that follows to pull the zoom area out
/// and set the loupe beside it.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn new_magnify(
  id: String,
  point: AnnotationPoint,
  style: Option<&AnnotationStyle>,
) -> Annotation {
  Annotation {
    above_camera: false,
    animated: true,
    id,
    held: None,
    pen: false,
    reveal: crate::editor::annotations::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Magnify {
      start: point,
      end: point,
      loupe: point,
      size: 0.0,
    },
    style: style.cloned().unwrap_or_else(default_magnify_style),
  }
}

/// Whether a magnifier is somewhere it can be drawn.
pub(crate) fn placed(
  start: AnnotationPoint,
  end: AnnotationPoint,
  loupe: AnnotationPoint,
  size: f64,
) -> bool {
  size.is_finite()
    && size >= 0.0
    && [start, end, loupe]
      .iter()
      .all(|point| point.x.is_finite() && point.y.is_finite())
}

/// The zoom area's longer side, in source pixels.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn longest(start: AnnotationPoint, end: AnnotationPoint) -> f64 {
  (end.x - start.x).abs().max((end.y - start.y).abs())
}

/// The zoom area aligns by its box, its edges and its centre, as a spotlight
/// does. The loupe aligns to nothing: it is placed beside what it enlarges.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn field_box(start: AnnotationPoint, end: AnnotationPoint) -> Option<SnapBox> {
  Some(crate::editor::annotations::redact::model::redact_box(
    start, end,
  ))
}
