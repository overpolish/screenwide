// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::edit::AnnotationEdit;
use crate::gesture::AnnotationGestureTarget;
use crate::snap::SnapModifiers;
use crate::{Annotation, AnnotationKind, AnnotationPoint, AnnotationShape};

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
  let mut edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::New,
    point(10.0, 10.0),
    None,
    Some(kind),
    &crate::edit::FreshAnnotation::default(),
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
