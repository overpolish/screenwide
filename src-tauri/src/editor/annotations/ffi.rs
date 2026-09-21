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
use super::exposure::annotation_travel;
use super::geometry::ArrowGeometry;
use super::reveal::AnnotationReveal;
use super::AnnotationKind;

/// One annotation's draw geometry, in the pixel space its points are given in:
/// canvas pixels for the compositor, display points for the chrome. An arrow
/// solves its curve from the three points; a counter places its disc at `p0`
/// and aims its tail with `p1x`, which is the angle `native.rs` keeps there
/// rather than a point, so no placement touches it.
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
    None => f32::INFINITY,
  }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
  use super::*;
  use crate::editor::annotations::arrow::distance::shaft_distance;
  use crate::editor::annotations::geometry::bezier;

  fn prepared(kind: u32, p1: [f32; 2], width: f32, head: u32) -> ArrowGeometry {
    let mut out = ArrowGeometry::default();
    unsafe {
      screenwide_annotation_prepare(
        kind,
        100.0,
        100.0,
        p1[0],
        p1[1],
        300.0,
        200.0,
        width,
        head,
        AnnotationReveal::WHOLE,
        &mut out,
      );
    }
    out
  }

  #[test]
  fn each_kind_prepares_what_its_own_module_does() {
    assert_eq!(
      prepared(0, [220.0, 40.0], 12.0, 2),
      prepare_arrow(
        [100.0, 100.0],
        [220.0, 40.0],
        [300.0, 200.0],
        12.0,
        2,
        AnnotationReveal::WHOLE
      )
    );
    // A counter reads its aim out of `p1x` and ignores the rest of the curve.
    assert_eq!(
      prepared(1, [0.75, 3.0], 40.0, 0),
      prepare_counter([100.0, 100.0], 40.0, 0.75, AnnotationReveal::WHOLE)
    );
  }

  #[test]
  fn a_kind_no_number_owns_prepares_nothing() {
    assert_eq!(prepared(7, [220.0, 40.0], 12.0, 2), ArrowGeometry::default());
    assert_eq!(
      screenwide_annotation_travel(
        7,
        100.0,
        100.0,
        220.0,
        40.0,
        300.0,
        200.0,
        1.0,
        1.0,
        12.0,
        AnnotationReveal::WHOLE
      ),
      0.0
    );
  }

  #[test]
  fn an_arrow_is_picked_by_its_stroke_and_its_heads() {
    let width = 12.0;
    let arrow = prepared(0, [200.0, 100.0], width, 1);
    let probe = |x: f32, y: f32| unsafe { screenwide_annotation_distance(0, x, y, &arrow) };
    // The curve's own midpoint is the middle of the stroke.
    let middle = bezier([100.0, 100.0], [200.0, 100.0], [300.0, 200.0], 0.5);
    assert_eq!(probe(middle[0], middle[1]), 0.0);
    // The stroke is picked out to its edge and no further: the tolerance is
    // the width the annotation shows, not a box around it.
    assert_eq!(probe(middle[0], middle[1] - width * 0.5 + 0.5), 0.0);
    assert!(probe(middle[0], middle[1] - width * 0.5 - 1.0) > 0.0);
    // A head is four strokes long and four wide, so its corners stand well
    // outside the stroke: they are picked by the head's own triangle.
    let corner = arrow.end_head.b;
    assert!(shaft_distance(corner, arrow.a, arrow.b, arrow.c) - arrow.width * 0.5 > 0.0);
    assert_eq!(probe(corner[0], corner[1]), 0.0);
    // And a step further out from the head's edge is nothing at all.
    let outward = [
      corner[0] + (corner[0] - arrow.end_head.a[0]),
      corner[1] + (corner[1] - arrow.end_head.a[1]),
    ];
    assert!(probe(outward[0], outward[1]) > 0.0);
  }

  #[test]
  fn a_counter_is_picked_by_its_disc_and_its_tail() {
    let counter = prepared(1, [0.0, 3.0], 40.0, 0);
    let probe = |x: f32, y: f32| unsafe { screenwide_annotation_distance(1, x, y, &counter) };
    assert!(probe(100.0, 100.0) < 0.0);
    // The tail's tip is the far end of the silhouette, and the grip sits on
    // it: the record carries it in `b`.
    assert_eq!(counter.b, [130.0, 100.0]);
    assert!(probe(counter.b[0], counter.b[1]).abs() < 1e-4);
    assert!(probe(counter.b[0] + 1.0, counter.b[1]) > 0.0);
  }

  #[test]
  fn nothing_prepared_is_nowhere() {
    let arrow = prepared(0, [200.0, 100.0], 12.0, 1);
    assert_eq!(
      unsafe { screenwide_annotation_distance(7, 100.0, 100.0, &arrow) },
      f32::INFINITY
    );
    assert_eq!(
      unsafe { screenwide_annotation_distance(0, 100.0, 100.0, std::ptr::null()) },
      f32::INFINITY
    );
  }
}
