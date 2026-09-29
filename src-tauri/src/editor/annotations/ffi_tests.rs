// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
  assert_eq!(
    prepared(8, [220.0, 40.0], 12.0, 2),
    ArrowGeometry::default()
  );
  assert_eq!(
    screenwide_annotation_travel(
      8,
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
