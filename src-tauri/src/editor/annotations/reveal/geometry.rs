// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Arc-length reveal windows mapped into curve parameters.

use super::AnnotationReveal;

/// How many segments the arc-length table measures the curve in. A quadratic
/// at annotation scale is flat enough between sixty-fourths that the linear
/// inversion inside one segment is worth less than a tenth of a pixel.
const ARC_SAMPLES: usize = 64;

/// A prepared reveal, in the curve's own parameter and in canvas pixels.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnnotationRevealGeometry {
  /// The parameter domain the shaft is drawn over.
  pub low: f32,
  pub high: f32,
  /// Where each head's tip is. The heads ride on the shaft's ends, outside
  /// them, so a head is never standing over the shaft it is joined to.
  pub start_tip: f32,
  pub end_tip: f32,
  /// The annotation's size: its stroke, its heads and their rounding, at most
  /// one.
  pub scale: f32,
}

/// The whole path at full size, which is what a static annotation has always
/// drawn.
const WHOLE_GEOMETRY: AnnotationRevealGeometry = AnnotationRevealGeometry {
  low: 0.0,
  high: 1.0,
  start_tip: 0.0,
  end_tip: 1.0,
  scale: 1.0,
};

pub(super) fn curve_point(a: [f32; 2], b: [f32; 2], c: [f32; 2], t: f32) -> [f32; 2] {
  let inverse = 1.0 - t;
  [
    inverse * inverse * a[0] + 2.0 * inverse * t * b[0] + t * t * c[0],
    inverse * inverse * a[1] + 2.0 * inverse * t * b[1] + t * t * c[1],
  ]
}

/// Cumulative length along the curve at each of `ARC_SAMPLES` + 1 parameters.
fn arc_table(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> [f32; ARC_SAMPLES + 1] {
  let mut table = [0.0; ARC_SAMPLES + 1];
  let mut previous = a;
  for index in 1..=ARC_SAMPLES {
    let point = curve_point(a, b, c, index as f32 / ARC_SAMPLES as f32);
    let step = (point[0] - previous[0]).hypot(point[1] - previous[1]);
    table[index] = table[index - 1] + step;
    previous = point;
  }
  table
}

/// The parameter at `length` along the curve, interpolated inside the segment
/// the table lands in. This is the whole reason the table exists: the arrow's
/// midpoint by length is not its midpoint by parameter.
pub(super) fn parameter_at(table: &[f32; ARC_SAMPLES + 1], length: f32) -> f32 {
  let total = table[ARC_SAMPLES];
  if !total.is_finite() || total <= 0.0 {
    return 0.0;
  }
  let length = length.clamp(0.0, total);
  let mut index = 1;
  while index < ARC_SAMPLES && table[index] < length {
    index += 1;
  }
  let span = table[index] - table[index - 1];
  let within = if span > 0.0 {
    (length - table[index - 1]) / span
  } else {
    0.0
  };
  ((index - 1) as f32 + within) / ARC_SAMPLES as f32
}

/// Prepare one exposure sample in canvas pixels.
///
/// The window is the *shaft's*: it runs over the path less the room the heads
/// take, and each head rides on its end of the shaft, outside it, the way a
/// trimmed stroke with an arrowhead parented to its end does. So the head is
/// there from the first frame and moving with the stroke, it never stands over
/// the shaft, and its length never counts against the travel - a heavy
/// annotation's head is four wide strokes long, and measured against the path
/// it is still there when a fine one's is long gone. Growing and shrinking, a
/// head does so from its base on the shaft's end, which is where it is joined.
///
pub(crate) fn reveal_geometry(
  a: [f32; 2],
  b: [f32; 2],
  c: [f32; 2],
  stroke: f32,
  heads: f32,
  window: AnnotationReveal,
) -> AnnotationRevealGeometry {
  if window.is_whole() {
    return WHOLE_GEOMETRY;
  }
  let table = arc_table(a, b, c);
  let total = table[ARC_SAMPLES];
  let head = (stroke.max(0.0) * 4.0).min(total * 0.5);
  let start_head = if heads >= 2.0 { head } else { 0.0 };
  let end_head = if heads >= 1.0 { head } else { 0.0 };
  let shaft = (total - start_head - end_head).max(0.0);
  let scale = window.scale.clamp(0.0, 1.0);
  let low = start_head + window.low.clamp(0.0, 1.0) * shaft;
  let high = (start_head + window.high.clamp(0.0, 1.0) * shaft).max(low);
  // A stroke's round end reaches half its width past where it stops. On a
  // bare tail that is the annotation's own end and belongs there - except while
  // the tail has all but reached the head's base, where nothing covers it and
  // it shows as a knob on the back of the head. So a bare tail stops inside
  // the window by the cap it would otherwise spend, which puts the round
  // end's edge on the tail itself, and gets that back over the first few
  // strokes of shaft: a stroke with room to be seen is untouched, and the end
  // never moves by more than a fraction of a frame's travel.
  let drawn = if heads >= 2.0 {
    low
  } else {
    let width = stroke.max(0.0) * scale;
    let given = ((high - low) / (width * 4.0).max(1e-4)).min(1.0);
    low + width * 0.5 * (1.0 - given)
  };
  AnnotationRevealGeometry {
    low: parameter_at(&table, drawn),
    high: parameter_at(&table, high),
    start_tip: parameter_at(&table, low - start_head * scale),
    end_tip: parameter_at(&table, high + end_head * scale),
    scale,
  }
}

/// The prepared reveal for one annotation, called from the compositor's own
/// preparation so every path - preview, still export and video export -
/// animates through the same maths.
///
/// # Safety
/// `out` must point at one writable [`AnnotationRevealGeometry`].
#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn screenwide_annotation_reveal_geometry(
  ax: f32,
  ay: f32,
  bx: f32,
  by: f32,
  cx: f32,
  cy: f32,
  stroke: f32,
  heads: f32,
  window: AnnotationReveal,
  out: *mut AnnotationRevealGeometry,
) {
  if out.is_null() {
    return;
  }
  *out = reveal_geometry([ax, ay], [bx, by], [cx, cy], stroke, heads, window);
}
