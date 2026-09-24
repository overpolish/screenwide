// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::arrow::new_arrow;
use super::edit::AnnotationEdit;
use super::gesture::{AnnotationGestureTarget, AnnotationHandle};
use super::snap::SnapModifiers;
use super::text::snap::text_box;
use super::{Annotation, AnnotationKind, AnnotationPoint, AnnotationShape};

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
    Some(AnnotationKind::Arrow),
    None,
    0.0,
  )
  .unwrap();
  edit.update(&mut annotations, point(80.0, 60.0), plain(), None);
  assert_eq!(annotations.len(), 2);
  assert_eq!(annotations[0], original[0]);
  edit.cancel(&mut annotations);
  assert_eq!(annotations, original);
}

/// A fresh text box is centred on the press, so its caret starts under the
/// hand, and a drag while it is being placed keeps it centred there.
#[test]
fn a_new_text_box_is_centred_on_the_press_and_carried_there() {
  const SOURCE_PER_OUTPUT: f64 = 2.0;
  let centre = |annotation: &Annotation| {
    let AnnotationShape::Text { origin, text, .. } = &annotation.shape else {
      unreachable!()
    };
    let bounds = text_box(*origin, text, annotation.style.width, SOURCE_PER_OUTPUT);
    assert!(bounds.width > 0.0 && bounds.height > 0.0);
    point(
      bounds.x + bounds.width / 2.0,
      bounds.y + bounds.height / 2.0,
    )
  };
  let near =
    |a: AnnotationPoint, b: AnnotationPoint| (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9;
  let mut annotations = Vec::new();
  let edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::New,
    point(200.0, 150.0),
    None,
    Some(AnnotationKind::Text),
    None,
    SOURCE_PER_OUTPUT,
  )
  .unwrap();
  assert!(near(centre(&annotations[0]), point(200.0, 150.0)));
  edit.update(&mut annotations, point(260.0, 190.0), plain(), None);
  assert!(near(centre(&annotations[0]), point(260.0, 190.0)));
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
    Some(AnnotationKind::Arrow),
    None,
    0.0,
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
    Some(AnnotationKind::Arrow),
    None,
    0.0,
  )
  .unwrap();
  edit.update(&mut annotations, point(60.0, 10.0), plain(), None);
  drop(edit);
  assert_ne!(annotations[0], original[0]);
  assert_eq!(annotations[1], original[1]);
}

#[test]
fn selection_and_a_missing_annotation_do_not_open_an_edit() {
  let mut annotations = vec![new_arrow(
    "a".to_owned(),
    point(0.0, 0.0),
    point(100.0, 0.0),
    None,
  )];
  let before = annotations.clone();
  for target in [
    AnnotationGestureTarget::Select { index: 0 },
    AnnotationGestureTarget::None,
    AnnotationGestureTarget::Existing {
      index: 1,
      handle: AnnotationHandle::End,
    },
  ] {
    assert!(AnnotationEdit::begin(
      &mut annotations,
      target,
      point(0.0, 0.0),
      None,
      Some(AnnotationKind::Arrow),
      None,
      0.0
    )
    .is_none());
    assert_eq!(annotations, before);
  }
}

/// A document has no ceiling: a press on empty picture still draws a new
/// annotation however many are already there.
#[test]
fn a_long_document_still_takes_a_new_annotation() {
  let mut annotations =
    vec![new_arrow("a".to_owned(), point(0.0, 0.0), point(100.0, 0.0), None); 5_000];
  let edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::New,
    point(10.0, 10.0),
    None,
    Some(AnnotationKind::Arrow),
    None,
    0.0,
  );
  assert!(edit.is_some());
  assert_eq!(annotations.len(), 5_001);
}
