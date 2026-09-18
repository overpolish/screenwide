// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::counter::new_counter;
use super::edit::AnnotationEdit;
use super::gesture::{AnnotationGestureTarget, AnnotationHandle, NewAnnotationKind};
use super::model::new_arrow;
use super::snap::{AnchorBoxes, SnapBounds, SnapField, SnapModifiers, SnapRequest, SnapResult};
use super::{AnnotationPoint, AnnotationShape, MAX_ANNOTATIONS};
use crate::ruler::analysis::ComponentBox;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// A sample with nothing held, which is every sample the older tests take.
fn plain() -> SnapModifiers {
  SnapModifiers::default()
}

fn positional() -> SnapModifiers {
  SnapModifiers::from_bits(0b10)
}

#[test]
fn cancelling_a_new_arrow_restores_the_original_list() {
  let original = vec![new_arrow(
    "existing".to_owned(),
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

#[test]
fn moving_a_counter_snaps_its_centre_to_another_counters_centre() {
  let mut annotations = vec![
    new_counter("moved".to_owned(), point(100.0, 100.0), 1, None, None),
    new_counter("fixed".to_owned(), point(600.0, 400.0), 2, None, None),
  ];
  let field = SnapField::new((1920, 1080), &annotations, "moved");
  let edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::Existing {
      index: 0,
      handle: AnnotationHandle::Body,
    },
    point(100.0, 100.0),
    None,
    NewAnnotationKind::Counter,
    None,
  )
  .unwrap();
  let request = SnapRequest {
    field: &field,
    threshold: 16.0,
  };
  // The drag arrives 6 pixels short of the other counter in x and 9 short in
  // y, both inside the reach, so the centre lands exactly on it.
  let result = edit.update(
    &mut annotations,
    point(594.0, 391.0),
    positional(),
    Some(request),
  );
  let AnnotationShape::Counter { center, .. } = annotations[0].shape else {
    unreachable!()
  };
  assert_eq!(center, point(600.0, 400.0));
  assert!(result.guide_x.is_some_and(|guide| guide.object));
  assert!(result.guide_y.is_some_and(|guide| guide.object));
  assert!(result.anchor.is_none());
  // Letting the modifier go frees the same sample again.
  let released = edit.update(
    &mut annotations,
    point(594.0, 391.0),
    plain(),
    Some(request),
  );
  let AnnotationShape::Counter { center, .. } = annotations[0].shape else {
    unreachable!()
  };
  assert_eq!(center, point(594.0, 391.0));
  assert_eq!(released, SnapResult::default());
}

#[test]
fn dragging_an_arrow_tip_snaps_it_to_a_detected_element() {
  let element = ComponentBox {
    x: 400,
    y: 300,
    width: 200,
    height: 100,
  };
  let mut annotations = vec![
    new_arrow(
      "arrow".to_owned(),
      point(10.0, 10.0),
      point(80.0, 80.0),
      None,
    ),
    // A counter's centre is an axis candidate, and an arrow must ignore it.
    new_counter("counter".to_owned(), point(398.0, 302.0), 1, None, None),
  ];
  let mut field = SnapField::new((1920, 1080), &annotations, "arrow");
  field.anchors = Some(std::sync::Arc::new(AnchorBoxes::new(
    1920,
    1080,
    vec![element],
  )));
  let edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::Existing {
      index: 0,
      handle: AnnotationHandle::End,
    },
    point(80.0, 80.0),
    None,
    NewAnnotationKind::Arrow,
    None,
  )
  .unwrap();
  let request = SnapRequest {
    field: &field,
    threshold: 16.0,
  };
  // Beside the element's left edge, well down its side: the padded edge takes
  // the tip's x and leaves its y alone, so the arrow can be aimed anywhere
  // along that side. Padding is 0.5% of the 1080-high source, so the edge
  // sits 5.4 pixels outside the detected 400.
  let result = edit.update(
    &mut annotations,
    point(402.0, 350.0),
    positional(),
    Some(request),
  );
  let AnnotationShape::Arrow { start, end, .. } = annotations[0].shape else {
    unreachable!()
  };
  assert_eq!(start, point(10.0, 10.0));
  assert_eq!(end, point(394.6, 350.0));
  assert_eq!(
    result.anchor.map(|anchor| anchor.bounds),
    Some(SnapBounds::padded(element, 5.4))
  );
  // An arrow never takes an axis guide, however close a counter's centre is.
  assert!(result.guide_x.is_none() && result.guide_y.is_none());
}

#[test]
fn the_bend_and_the_shaft_of_an_arrow_never_snap() {
  let element = ComponentBox {
    x: 400,
    y: 300,
    width: 200,
    height: 100,
  };
  let annotations = vec![new_arrow(
    "arrow".to_owned(),
    point(300.0, 200.0),
    point(500.0, 200.0),
    None,
  )];
  let mut field = SnapField::new((1920, 1080), &annotations, "arrow");
  field.anchors = Some(std::sync::Arc::new(AnchorBoxes::new(
    1920,
    1080,
    vec![element],
  )));
  let request = SnapRequest {
    field: &field,
    threshold: 16.0,
  };
  for handle in [AnnotationHandle::Middle, AnnotationHandle::Body] {
    let mut working = annotations.clone();
    let edit = AnnotationEdit::begin(
      &mut working,
      AnnotationGestureTarget::Existing { index: 0, handle },
      point(400.0, 200.0),
      None,
      NewAnnotationKind::Arrow,
      None,
    )
    .unwrap();
    let result = edit.update(
      &mut working,
      point(406.0, 308.0),
      positional(),
      Some(request),
    );
    assert_eq!(result, SnapResult::default());
  }
}
