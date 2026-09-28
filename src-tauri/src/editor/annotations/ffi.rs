// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A kind's prepared geometry, as the native platforms reach it.
//!
//! Both backends prepare an annotation from the same Rust: the D3D11 one calls
//! each kind's `geometry` module directly, and the Metal compositor and the
//! macOS chrome call [`screenwide_annotation_prepare`], which dispatches to
//! those same functions. `geometry.h` declares what is here and carries no
//! arithmetic of its own, so a kind's draw geometry is written once.

use super::arrow::distance::prepared_arrow_distance;
use super::arrow::geometry::prepare_arrow;
use super::counter::geometry::prepare_counter;
use super::counter::silhouette::prepared_counter_distance;
use super::exposure::{annotation_travel, highlight_travel};
use super::geometry::ArrowGeometry;
use super::highlight::geometry::{flow_distance, prepare_highlight, HighlightFlow};
use super::outline::geometry::{prepare_shape, shape_distance};
use super::redact::geometry::{prepare_redact, redact_distance};
use super::reveal::AnnotationReveal;
use super::text::geometry::{prepare_text, text_distance};
use super::AnnotationKind;

/// One annotation's draw geometry, in the pixel space its points are given in:
/// canvas pixels for the compositor, display points for the chrome. An arrow
/// solves its curve from the three points; a counter places its disc at `p0`
/// and aims its tail with `p1x`, which is the angle `native.rs` keeps there
/// rather than a point, so no placement touches it; a text box places its
/// corner at `p0` and its pointer's tip at `p1`, and reads its text block's
/// size out of `p2`, which is a size rather than a point for the same reason.
/// A highlight is placed by where `p0` and `p1` land - the source's origin and
/// its pixel `(1, 1)` - and reads its tone out of `p2`. A shape's box runs
/// from `p0` to `p2`, and `p1` carries its radius and its hand rather than a
/// point, as `outline::native` keeps them.
///
/// A number no kind owns prepares nothing. It cannot come from a retained
/// annotation, and drawing it as an arrow would draw some future kind as a
/// curve rather than leaving it out.
///
/// # Safety
/// `out` must point at one writable [`ArrowGeometry`].
#[cfg(target_os = "macos")]
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn screenwide_annotation_prepare(
  kind: u32,
  p0x: f32,
  p0y: f32,
  p1x: f32,
  p1y: f32,
  p2x: f32,
  p2y: f32,
  width: f32,
  head: u32,
  reveal: AnnotationReveal,
  out: *mut ArrowGeometry,
) {
  if out.is_null() {
    return;
  }
  *out = match AnnotationKind::from_raw(kind) {
    Some(AnnotationKind::Arrow) => {
      prepare_arrow([p0x, p0y], [p1x, p1y], [p2x, p2y], width, head, reveal)
    }
    Some(AnnotationKind::Counter) => prepare_counter([p0x, p0y], width, p1x, reveal),
    Some(AnnotationKind::Text) => {
      prepare_text([p0x, p0y], [p1x, p1y], [p2x, p2y], width, head, reveal)
    }
    Some(AnnotationKind::Redact) => prepare_redact([p0x, p0y], [p2x, p2y], width),
    Some(AnnotationKind::Highlight) => {
      prepare_highlight([p0x, p0y], [p1x, p1y], [p2x, p2y], reveal)
    }
    Some(AnnotationKind::Shape) => prepare_shape([p0x, p0y], [p2x, p2y], p1x, p1y, width, reveal),
    None => ArrowGeometry::default(),
  };
}

/// The travel one annotation's exposure covers, for the compositor's sample
/// count. `sx` and `sy` carry a point from the space the points are given in
/// into the pixels the travel is measured in.
#[cfg(target_os = "macos")]
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn screenwide_annotation_travel(
  kind: u32,
  p0x: f32,
  p0y: f32,
  p1x: f32,
  p1y: f32,
  p2x: f32,
  p2y: f32,
  sx: f32,
  sy: f32,
  width: f32,
  reveal: AnnotationReveal,
) -> f32 {
  match AnnotationKind::from_raw(kind) {
    Some(kind) => annotation_travel(
      kind,
      [p0x, p0y],
      [p1x, p1y],
      [p2x, p2y],
      [sx, sy],
      width,
      reveal,
    ),
    None => 0.0,
  }
}

/// The travel a highlight's exposure covers, for the compositor's sample
/// count: `p0` and `p1` are its record's source origin and pixel `(1, 1)`,
/// `sx` and `sy` carry them into the pixels the travel is measured in, and
/// `sweep` is its record's `params[2]`.
#[cfg(target_os = "macos")]
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn screenwide_highlight_travel(
  p0x: f32,
  p0y: f32,
  p1x: f32,
  p1y: f32,
  sx: f32,
  sy: f32,
  sweep: f32,
  reveal: AnnotationReveal,
) -> f32 {
  highlight_travel([p0x, p0y], [p1x, p1y], [sx, sy], sweep, reveal)
}

/// How far a point falls from one prepared annotation's drawn shape, in the
/// space it was prepared in. Zero anywhere the annotation is painted, which
/// is what picks it and what the halo is drawn around: the tolerance is the
/// stroke's edge rather than its centreline, so a press lands on the
/// annotation exactly where the annotation looks like it is.
///
/// Nothing prepared, and nothing a kind owns, is nowhere: a press never lands
/// on it.
///
/// # Safety
/// `geometry` must point at one readable [`ArrowGeometry`].
#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn screenwide_annotation_distance(
  kind: u32,
  px: f32,
  py: f32,
  geometry: *const ArrowGeometry,
) -> f32 {
  let Some(geometry) = geometry.as_ref() else {
    return f32::INFINITY;
  };
  match AnnotationKind::from_raw(kind) {
    Some(AnnotationKind::Arrow) => prepared_arrow_distance([px, py], geometry),
    Some(AnnotationKind::Counter) => {
      prepared_counter_distance((f64::from(px), f64::from(py)), geometry) as f32
    }
    Some(AnnotationKind::Text) => text_distance([px, py], geometry),
    Some(AnnotationKind::Redact) => redact_distance([px, py], geometry),
    Some(AnnotationKind::Shape) => shape_distance([px, py], geometry),
    // A highlight's record places bands it does not carry; the chrome picks
    // it through [`screenwide_highlight_distance`] instead.
    Some(AnnotationKind::Highlight) | None => f32::INFINITY,
  }
}

/// How far a point falls outside a prepared shape's stroke or the box it
/// outlines: what picks a shape from anywhere inside it.
///
/// # Safety
/// `geometry` must point at one readable [`ArrowGeometry`].
#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn screenwide_shape_body_distance(
  px: f32,
  py: f32,
  geometry: *const ArrowGeometry,
) -> f32 {
  geometry.as_ref().map_or(f32::INFINITY, |geometry| {
    super::outline::geometry::shape_body_distance([px, py], geometry)
  })
}

/// How far a point falls outside a highlight, from what its grips' record
/// carries placed in the chrome's display points: the first band's top-left
/// corner and bottom, the last band's top and bottom-right corner, and the
/// block's left and right. Negative inside.
#[cfg(target_os = "macos")]
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn screenwide_highlight_distance(
  px: f32,
  py: f32,
  start_x: f32,
  start_y: f32,
  first_bottom: f32,
  last_top: f32,
  end_x: f32,
  end_y: f32,
  block_left: f32,
  block_right: f32,
) -> f32 {
  flow_distance(
    [px, py],
    &HighlightFlow {
      start: [start_x, start_y],
      first_bottom,
      last_top,
      end: [end_x, end_y],
      block: [block_left, block_right],
    },
  )
}

#[cfg(all(test, target_os = "macos"))]
#[path = "ffi_tests.rs"]
mod tests;
