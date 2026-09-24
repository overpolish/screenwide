// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What one pointer sample lands on, through the whole edit path.

use super::arrow::new_arrow;
use super::counter::new_counter;
use super::edit::AnnotationEdit;
use super::gesture::{AnnotationGestureTarget, AnnotationHandle};
use super::snap::{AnchorBoxes, SnapBounds, SnapField, SnapModifiers, SnapRequest, SnapResult};
use super::{Annotation, AnnotationKind, AnnotationPoint, AnnotationShape};
use crate::ruler::analysis::ComponentBox;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn plain() -> SnapModifiers {
  SnapModifiers::default()
}

fn positional() -> SnapModifiers {
  SnapModifiers::from_bits(0b10)
}

/// The default disc is 56 output pixels across, and these tests draw a
/// 1920-wide source 1920 output pixels across, so a disc's edges sit 28
/// source pixels either side of its centre.
const IMAGE_WIDTH: f64 = 1920.0;

/// A drag of the whole counter at `index`, which began where it stands.
fn counter_move(
  annotations: &mut Vec<Annotation>,
  index: usize,
  from: AnnotationPoint,
) -> AnnotationEdit {
  AnnotationEdit::begin(
    annotations,
    AnnotationGestureTarget::Existing {
      index,
      handle: AnnotationHandle::Body,
    },
    from,
    None,
    Some(AnnotationKind::Counter),
    None,
    0.0,
  )
  .unwrap()
}

fn centre(annotation: &Annotation) -> AnnotationPoint {
  let AnnotationShape::Counter { center, .. } = annotation.shape else {
    unreachable!()
  };
  center
}

#[test]
fn moving_a_counter_snaps_its_centre_to_another_counters_centre() {
  let mut annotations = vec![
    new_counter("moved".to_owned(), point(100.0, 100.0), 1, None, None),
    new_counter("fixed".to_owned(), point(600.0, 400.0), 2, None, None),
  ];
  let field = SnapField::new((1920, 1080), &annotations, "moved", IMAGE_WIDTH);
  let edit = counter_move(&mut annotations, 0, point(100.0, 100.0));
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
  assert_eq!(centre(&annotations[0]), point(600.0, 400.0));
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
  assert_eq!(centre(&annotations[0]), point(594.0, 391.0));
  assert_eq!(released, SnapResult::default());
}

#[test]
fn a_discs_edge_lines_up_with_another_discs_edge() {
  let mut annotations = vec![
    new_counter("moved".to_owned(), point(100.0, 100.0), 1, None, None),
    new_counter("fixed".to_owned(), point(600.0, 400.0), 2, None, None),
  ];
  let field = SnapField::new((1920, 1080), &annotations, "moved", IMAGE_WIDTH);
  let edit = counter_move(&mut annotations, 0, point(100.0, 100.0));
  let request = SnapRequest {
    field: &field,
    threshold: 16.0,
  };
  // The fixed disc's left edge is at 572. The drag arrives with the moving
  // disc's right edge at 577, five pixels past it and nearer than any centre
  // pairing, so the whole disc backs up by five and its centre stops at 544 -
  // nowhere near the other centre.
  let result = edit.update(
    &mut annotations,
    point(549.0, 700.0),
    positional(),
    Some(request),
  );
  assert_eq!(centre(&annotations[0]), point(544.0, 700.0));
  assert_eq!(
    result.guide_x.map(|guide| (guide.position, guide.object)),
    Some((572.0, true))
  );
  // Nothing is in reach in y, so that axis still follows the hand.
  assert!(result.guide_y.is_none());
}

#[test]
fn a_discs_edge_lands_on_the_canvas_inset() {
  let mut annotations = vec![new_counter(
    "moved".to_owned(),
    point(100.0, 100.0),
    1,
    None,
    None,
  )];
  let field = SnapField::new((1920, 1080), &annotations, "moved", IMAGE_WIDTH);
  let edit = counter_move(&mut annotations, 0, point(100.0, 100.0));
  let request = SnapRequest {
    field: &field,
    threshold: 16.0,
  };
  // The inset is 2% of the shorter side: 21.6 pixels in. The drag arrives
  // with the disc's left edge at 26, so the disc slides out to sit against
  // the inset rather than putting its centre on it.
  let result = edit.update(
    &mut annotations,
    point(54.0, 700.0),
    positional(),
    Some(request),
  );
  let center = centre(&annotations[0]);
  assert!(
    (center.x - 49.6).abs() < 1e-9,
    "the disc's centre landed at {}",
    center.x
  );
  assert_eq!(center.y, 700.0);
  assert!(result.guide_x.is_some_and(|guide| !guide.object));
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
    // A counter's disc is an axis candidate, and an arrow must ignore it.
    new_counter("counter".to_owned(), point(398.0, 302.0), 1, None, None),
  ];
  let mut field = SnapField::new((1920, 1080), &annotations, "arrow", IMAGE_WIDTH);
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
    Some(AnnotationKind::Arrow),
    None,
    0.0,
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
  // An arrow never takes an axis guide, however close a counter's disc is.
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
  let mut field = SnapField::new((1920, 1080), &annotations, "arrow", IMAGE_WIDTH);
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
      Some(AnnotationKind::Arrow),
      None,
      0.0,
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
