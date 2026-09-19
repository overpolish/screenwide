// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::edit::AnnotationEdit;
use super::gesture::{AnnotationGestureTarget, AnnotationHandle, NewAnnotationKind};
use super::model::new_arrow;
use super::snap::SnapModifiers;
use super::{AnnotationPoint, AnnotationShape, MAX_ANNOTATIONS};

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// A sample with nothing held, which is every sample these tests take.
fn plain() -> SnapModifiers {
  SnapModifiers::default()
}

#[test]
fn cancelling_a_new_arrow_restores_the_original_list() {
  let original = vec![new_arrow(
    "a".to_owned(),
    point(0.0, 0.0),
    point(100.0, 0.0),
    None,
  )];
  let mut annotations = original.clone();
  let edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::New,
    point(20.0, 30.0),
    None,
    NewAnnotationKind::Arrow,
    None,
  )
  .unwrap();
  edit.update(&mut annotations, point(80.0, 60.0), plain(), None);
  assert_eq!(annotations.len(), 2);
  assert_eq!(annotations[0], original[0]);
  edit.cancel(&mut annotations);
  assert_eq!(annotations, original);
}

#[test]
fn completing_or_cancelling_an_existing_drag_only_changes_its_annotation() {
  let original = vec![
    new_arrow("a".to_owned(), point(0.0, 0.0), point(100.0, 0.0), None),
    new_arrow("b".to_owned(), point(0.0, 50.0), point(100.0, 50.0), None),
  ];
  let mut annotations = original.clone();
  let edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::Existing {
      index: 0,
      handle: AnnotationHandle::Body,
    },
    point(50.0, 0.0),
    None,
    NewAnnotationKind::Arrow,
    None,
  )
  .unwrap();
  edit.update(&mut annotations, point(80.0, 20.0), plain(), None);
  edit.update(&mut annotations, point(60.0, 10.0), plain(), None);
  assert_eq!(annotations[1], original[1]);
  assert_eq!(
    annotations[0].shape,
    AnnotationShape::Arrow {
      start: point(10.0, 10.0),
      control: point(60.0, 10.0),
      end: point(110.0, 10.0)
    }
  );
  edit.cancel(&mut annotations);
  assert_eq!(annotations, original);
  let edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::Existing {
      index: 0,
      handle: AnnotationHandle::Body,
    },
    point(50.0, 0.0),
    None,
    NewAnnotationKind::Arrow,
    None,
  )
  .unwrap();
  edit.update(&mut annotations, point(60.0, 10.0), plain(), None);
  drop(edit);
  assert_ne!(annotations[0], original[0]);
  assert_eq!(annotations[1], original[1]);
}

#[test]
fn selection_and_capacity_do_not_open_an_edit() {
  let mut annotations =
    vec![new_arrow("a".to_owned(), point(0.0, 0.0), point(100.0, 0.0), None); MAX_ANNOTATIONS];
  let before = annotations.clone();
  for target in [
    AnnotationGestureTarget::Select { index: 0 },
    AnnotationGestureTarget::None,
    AnnotationGestureTarget::New,
    AnnotationGestureTarget::Existing {
      index: MAX_ANNOTATIONS,
      handle: AnnotationHandle::End,
    },
  ] {
    assert!(AnnotationEdit::begin(
      &mut annotations,
      target,
      point(0.0, 0.0),
      None,
      NewAnnotationKind::Arrow,
      None
    )
    .is_none());
    assert_eq!(annotations, before);
  }
}
