// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::editor::annotations::edit::AnnotationEdit;
use crate::editor::annotations::gesture::{
  AnnotationGestureTarget, AnnotationHandle, RADIUS_HANDLE,
};
use crate::editor::annotations::snap::SnapModifiers;
use crate::editor::annotations::{Annotation, AnnotationKind, AnnotationPoint, AnnotationShape};

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

const RELEASED: SnapModifiers = SnapModifiers {
  shift: false,
  position: false,
};

/// Every kind held by a box and its eight grips.
const BOXED: [AnnotationKind; 2] = [AnnotationKind::Redact, AnnotationKind::Shape];

/// A box of `kind` pulled from (10, 10) to (110, 60): fifty source pixels on
/// its shorter side.
fn boxed(kind: AnnotationKind) -> Vec<Annotation> {
  let mut annotations = Vec::new();
  let edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::New,
    point(10.0, 10.0),
    None,
    Some(kind),
    None,
    0.0,
  )
  .unwrap();
  edit.update(&mut annotations, point(110.0, 60.0), RELEASED, None);
  annotations
}

fn corners(annotation: &Annotation) -> (AnnotationPoint, AnnotationPoint) {
  match annotation.shape {
    AnnotationShape::Redact { start, end, .. } | AnnotationShape::Shape { start, end, .. } => {
      (start, end)
    }
    _ => unreachable!(),
  }
}

/// The radius dot dragged to `to`, on a picture drawn at `source_per_point`
/// source pixels a screen point.
fn rounded(annotations: &mut Vec<Annotation>, to: AnnotationPoint, source_per_point: f64) {
  let target = AnnotationGestureTarget::Existing {
    index: 0,
    handle: AnnotationHandle::Radius,
  };
  let mut edit = AnnotationEdit::begin(
    annotations,
    target,
    point(30.0, 30.0),
    None,
    None,
    None,
    0.0,
  )
  .unwrap();
  edit.set_source_per_point(Some(source_per_point));
  edit.update(annotations, to, RELEASED, None);
}

/// Where the native chrome draws the dot for `percent`: in from the corner
/// by ten points plus 0.55 of the radius, both in screen points.
fn dot(percent: f64, source_per_point: f64) -> AnnotationPoint {
  let shortest_points = 50.0 / source_per_point;
  let inset = (shortest_points * percent / 100.0 * 0.55 + 10.0) * source_per_point;
  point(10.0 + inset, 10.0 + inset)
}

#[test]
fn a_fresh_box_is_pulled_out_from_the_press_to_the_hand() {
  for kind in BOXED {
    let annotations = boxed(kind);
    assert_eq!(annotations[0].shape.kind(), kind);
    assert_eq!(
      corners(&annotations[0]),
      (point(10.0, 10.0), point(110.0, 60.0)),
      "{kind:?}"
    );
  }
}

#[test]
fn the_radius_dot_reads_back_the_radius_it_is_drawn_at() {
  for kind in BOXED {
    for source_per_point in [0.5, 2.0] {
      for percent in [0.0, 12.5, 50.0] {
        let mut annotations = boxed(kind);
        rounded(
          &mut annotations,
          dot(percent, source_per_point),
          source_per_point,
        );
        let radius = annotations[0].style.radius;
        assert!(
          (radius - percent).abs() < 1e-9,
          "{kind:?} {percent} at {source_per_point}: {radius}"
        );
      }
    }
  }
}

#[test]
fn the_radius_dot_rounds_the_corners_without_moving_the_box() {
  for kind in BOXED {
    let mut annotations = boxed(kind);
    rounded(&mut annotations, point(500.0, 500.0), 1.0);
    assert_eq!(annotations[0].style.radius, 50.0);
    assert_eq!(
      corners(&annotations[0]),
      (point(10.0, 10.0), point(110.0, 60.0))
    );
    rounded(&mut annotations, point(0.0, 0.0), 1.0);
    assert_eq!(annotations[0].style.radius, 0.0);
  }
}

#[test]
fn the_radius_dot_reads_back_from_its_native_number() {
  assert_eq!(
    AnnotationHandle::from_raw(RADIUS_HANDLE),
    Some(AnnotationHandle::Radius)
  );
}
