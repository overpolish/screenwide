// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a lens, the dress a fresh one wears, and reading one a document
//! kept from before the lens.
//!
//! The lens is a box held by two corners in source pixels, `start` the
//! top-left and `end` the bottom-right; every edit writes them back in that
//! order. It shows what lies under it `zoom` times larger about its own
//! middle, so resizing it shows more or less at the same zoom. The zoom is
//! one of [`MAGNIFY_ZOOMS`]. The style rounds it by its `radius` and says in
//! `shadow` whether it casts one; the width is always a fresh one's pen.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::snap::SnapBox;
use crate::AnnotationPoint;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::{Annotation, AnnotationHead, AnnotationShape, AnnotationStyle};
use serde::{Deserialize, Deserializer};

/// The zooms a lens is offered, as looks rather than a scale: a handful read
/// apart at a glance, and the lens's size, not its zoom, says how much it
/// shows. The twin of `LENS_ZOOMS` in
/// `src/features/editor/annotations/annotation-shape-parsers.ts`.
pub const MAGNIFY_ZOOMS: [f64; 4] = [1.5, 2.0, 3.0, 4.0];

/// The zoom a fresh lens, and one a document left without any, is drawn at.
pub const NEW_MAGNIFY_ZOOM: f64 = 2.0;

/// The width every lens is drawn at, in points: an arrow's pen. The glass
/// has an edge of its own, so nothing is drawn by it but the hover halo's
/// reach. The twin of the magnifier's `defaultSize` in `widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_MAGNIFY_WIDTH: f64 = 8.0;

/// A fresh lens is round: rounded all the way, a square is a circle.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
const NEW_MAGNIFY_RADIUS: f64 = 50.0;

/// The dress a fresh lens wears before anything has been chosen: round, and
/// casting a shadow.
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

/// A lens at `point`, in `style` or in the tool's own first dress, at `zoom`
/// or a fresh one's. A fresh one in the editor starts with both corners at
/// the press, for the drag that follows to pull it out as a shape's box is.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub fn new_magnify(
  id: String,
  point: AnnotationPoint,
  style: Option<&AnnotationStyle>,
  zoom: Option<f64>,
) -> Annotation {
  Annotation {
    above_camera: false,
    animated: true,
    id,
    held: None,
    pen: false,
    reveal: crate::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Magnify {
      start: point,
      end: point,
      zoom: zoom.map_or(NEW_MAGNIFY_ZOOM, nearest_zoom),
    },
    style: style.cloned().unwrap_or_else(default_magnify_style),
  }
}

/// The look nearest `zoom`, measured as a ratio, since a zoom is a factor:
/// 2.5× is nearer 3× than 2×. A zoom that is not a number is a fresh one.
pub fn nearest_zoom(zoom: f64) -> f64 {
  if !(zoom.is_finite() && zoom > 0.0) {
    return NEW_MAGNIFY_ZOOM;
  }
  MAGNIFY_ZOOMS
    .into_iter()
    .min_by(|a, b| (zoom / a).ln().abs().total_cmp(&(zoom / b).ln().abs()))
    .unwrap_or(NEW_MAGNIFY_ZOOM)
}

/// Whether a lens is somewhere it can be drawn.
pub(crate) fn placed(start: AnnotationPoint, end: AnnotationPoint, zoom: f64) -> bool {
  zoom.is_finite()
    && zoom > 0.0
    && [start, end]
      .iter()
      .all(|point| point.x.is_finite() && point.y.is_finite())
}

/// A lens aligns by its box, its edges and its centre, as a shape does.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn field_box(start: AnnotationPoint, end: AnnotationPoint) -> Option<SnapBox> {
  Some(crate::redact::model::redact_box(start, end))
}

/// A lens as a document keeps it, or a magnifier as one kept it before the
/// lens: a zoom area from `start` to `end` and a loupe `size` along its
/// longer side, centred elsewhere.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredLens {
  start: AnnotationPoint,
  end: AnnotationPoint,
  #[serde(default)]
  zoom: Option<f64>,
  #[serde(default)]
  size: Option<f64>,
}

/// Reads a lens's fields, `start`, `end` and `zoom`, from a document. One
/// written before the lens is read by [`lens_from_loupe`]; one with no zoom
/// at all takes a fresh one's.
pub(crate) fn stored_lens<'de, D: Deserializer<'de>>(
  deserializer: D,
) -> Result<(AnnotationPoint, AnnotationPoint, f64), D::Error> {
  let stored = StoredLens::deserialize(deserializer)?;
  Ok(match (stored.zoom, stored.size) {
    (Some(zoom), _) => (stored.start, stored.end, zoom),
    (None, Some(size)) => lens_from_loupe(stored.start, stored.end, size),
    (None, None) => (stored.start, stored.end, NEW_MAGNIFY_ZOOM),
  })
}

/// The lens a magnifier from before the lens becomes: over its zoom area's
/// centre, as big as its loupe was, which was the zoom area's shape `size`
/// along its longer side, at the look nearest the zoom that size gave it.
/// Its corners, its dress and its reveal are the annotation's, and carry
/// over untouched.
pub fn lens_from_loupe(
  start: AnnotationPoint,
  end: AnnotationPoint,
  size: f64,
) -> (AnnotationPoint, AnnotationPoint, f64) {
  let longest = (end.x - start.x).abs().max((end.y - start.y).abs());
  let scale = if longest > 0.0 && size.is_finite() && size > 0.0 {
    size / longest
  } else {
    1.0
  };
  let centre = ((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
  let half = (
    (end.x - start.x).abs() / 2.0 * scale,
    (end.y - start.y).abs() / 2.0 * scale,
  );
  (
    AnnotationPoint {
      x: centre.0 - half.0,
      y: centre.1 - half.1,
    },
    AnnotationPoint {
      x: centre.0 + half.0,
      y: centre.1 + half.1,
    },
    nearest_zoom(scale),
  )
}
