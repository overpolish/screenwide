// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a counter's tail tip lands, both when the counter is carried and
//! when it is turned.

use super::counter::new_counter;
use super::counter::silhouette::COUNTER_TAIL_REACH;
use super::edit::AnnotationEdit;
use super::gesture::{AnnotationGestureTarget, AnnotationHandle};
use super::snap::{
  element_padding, AnchorBoxes, SnapField, SnapModifiers, SnapRequest, SnapResult,
};
use super::{Annotation, AnnotationKind, AnnotationPoint, AnnotationShape};
use crate::ruler::analysis::ComponentBox;
use std::f64::consts::{FRAC_PI_4, PI};
use std::sync::Arc;

const SOURCE: (u32, u32) = (1920, 1080);
/// Drawn one output pixel per source pixel, so the default 56-pixel disc has
/// a 28-pixel radius and its tail reaches 42 pixels from the centre.
const IMAGE_WIDTH: f64 = 1920.0;
const RADIUS: f64 = 28.0;
const THRESHOLD: f64 = 16.0;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn positional() -> SnapModifiers {
  SnapModifiers::from_bits(0b10)
}

fn shifted() -> SnapModifiers {
  SnapModifiers::from_bits(0b11)
}

fn field_with(annotations: &[Annotation], edited: &str, elements: Vec<ComponentBox>) -> SnapField {
  let mut field = SnapField::new(SOURCE, annotations, edited, IMAGE_WIDTH);
  field.anchors = Some(Arc::new(AnchorBoxes::new(SOURCE.0, SOURCE.1, elements)));
  field
}

fn centre(annotation: &Annotation) -> AnnotationPoint {
  let AnnotationShape::Counter { center, .. } = annotation.shape else {
    unreachable!()
  };
  center
}

fn aim(annotation: &Annotation) -> f64 {
  let AnnotationShape::Counter { angle, .. } = annotation.shape else {
    unreachable!()
  };
  angle
}

fn close(value: f64, wanted: f64) -> bool {
  (value - wanted).abs() < 1e-9
}

/// A 200x100 element, whose padded left edge is the candidate the move tests
/// aim a tail at.
fn element() -> ComponentBox {
  ComponentBox {
    x: 400,
    y: 300,
    width: 200,
    height: 100,
  }
}

fn drag(
  annotations: &mut Vec<Annotation>,
  handle: AnnotationHandle,
  from: AnnotationPoint,
) -> AnnotationEdit {
  AnnotationEdit::begin(
    annotations,
    AnnotationGestureTarget::Existing { index: 0, handle },
    from,
    None,
    Some(AnnotationKind::Counter),
    None,
    0.0,
  )
  .unwrap()
}

#[test]
fn a_tail_tip_pins_one_axis_while_a_disc_line_pins_the_other() {
  let mut annotations = vec![
    // Aimed east, so the tip leads the disc by its 42-pixel reach.
    new_counter("moved".to_owned(), point(100.0, 100.0), 1, None, Some(0.0)),
    new_counter("fixed".to_owned(), point(900.0, 350.0), 2, None, None),
  ];
  let field = field_with(&annotations, "moved", vec![element()]);
  let edit = drag(
    &mut annotations,
    AnnotationHandle::Body,
    point(100.0, 100.0),
  );
  let result = edit.update(
    &mut annotations,
    point(350.0, 350.0),
    positional(),
    Some(SnapRequest {
      field: &field,
      threshold: THRESHOLD,
    }),
  );
  // The tip arrives at 392, two and a half pixels short of the element's
  // padded left edge, and nothing else is in reach in x: the whole counter
  // slides forward until the tip is on the edge.
  let left = 400.0 - element_padding(SOURCE);
  let center = centre(&annotations[0]);
  assert!(close(center.x, left - RADIUS * COUNTER_TAIL_REACH));
  // In y the other counter's disc lines up exactly, so that axis is pinned by
  // an alignment guide while the tip pins x. Both chromes have something to
  // draw.
  assert_eq!(center.y, 350.0);
  assert!(result.guide_x.is_none());
  assert!(result.guide_y.is_some_and(|guide| guide.object));
  let anchor = result.anchor.expect("the tip took the element's edge");
  assert!(close(anchor.point.x, left));
  assert_eq!(anchor.point.y, 350.0);
}

#[test]
fn a_discs_edge_never_takes_an_element_edge() {
  // Aimed west, so the tail points away from the element and its tip is out
  // of reach while the disc's own left edge sits two and a half pixels from
  // the element's padded edge. Element edges belong to the tip alone, so
  // nothing snaps.
  let mut annotations = vec![new_counter(
    "moved".to_owned(),
    point(100.0, 100.0),
    1,
    None,
    Some(PI),
  )];
  let field = field_with(&annotations, "moved", vec![element()]);
  let edit = drag(
    &mut annotations,
    AnnotationHandle::Body,
    point(100.0, 100.0),
  );
  let result = edit.update(
    &mut annotations,
    point(420.0, 350.0),
    positional(),
    Some(SnapRequest {
      field: &field,
      threshold: THRESHOLD,
    }),
  );
  assert_eq!(centre(&annotations[0]), point(420.0, 350.0));
  assert_eq!(result, SnapResult::default());
}

/// The counter the rotation tests turn, and the element whose padded top edge
/// its tail circle crosses.
fn turning() -> (Vec<Annotation>, ComponentBox) {
  (
    vec![new_counter(
      "turned".to_owned(),
      point(500.0, 500.0),
      1,
      None,
      Some(0.0),
    )],
    ComponentBox {
      x: 400,
      y: 540,
      width: 200,
      height: 100,
    },
  )
}

/// Where the tail circle crosses the element's padded top edge, on the side
/// the tests aim at, and the angle that puts the tip there.
fn crossing() -> (AnnotationPoint, f64) {
  let reach = RADIUS * COUNTER_TAIL_REACH;
  let down = 540.0 - element_padding(SOURCE) - 500.0;
  let across = (reach * reach - down * down).sqrt();
  (point(500.0 + across, 500.0 + down), down.atan2(across))
}

/// A pointer sample a hundred pixels out from the counter's centre, aimed at
/// `angle`.
fn aimed_at(angle: f64) -> AnnotationPoint {
  point(500.0 + 100.0 * angle.cos(), 500.0 + 100.0 * angle.sin())
}

#[test]
fn turning_a_tail_lands_its_tip_on_an_element_edge() {
  let (mut annotations, element) = turning();
  let field = field_with(&annotations, "turned", vec![element]);
  let edit = drag(
    &mut annotations,
    AnnotationHandle::Tail,
    point(542.0, 500.0),
  );
  let result = edit.update(
    &mut annotations,
    aimed_at(1.0),
    positional(),
    Some(SnapRequest {
      field: &field,
      threshold: THRESHOLD,
    }),
  );
  let (tip, angle) = crossing();
  assert!(close(aim(&annotations[0]), angle));
  let anchor = result.anchor.expect("the tip took the element's edge");
  assert!(close(anchor.point.x, tip.x) && close(anchor.point.y, tip.y));
  // A turn moves nothing but the aim, so there is no axis guide to show.
  assert!(result.guide_x.is_none() && result.guide_y.is_none());
}

#[test]
fn an_element_edge_in_reach_beats_shifts_eighth_turns() {
  let (mut annotations, element) = turning();
  let field = field_with(&annotations, "turned", vec![element]);
  let edit = drag(
    &mut annotations,
    AnnotationHandle::Tail,
    point(542.0, 500.0),
  );
  edit.update(
    &mut annotations,
    aimed_at(1.0),
    shifted(),
    Some(SnapRequest {
      field: &field,
      threshold: THRESHOLD,
    }),
  );
  let (_, angle) = crossing();
  assert!(close(aim(&annotations[0]), angle));
  assert!(!close(aim(&annotations[0]), FRAC_PI_4));
}

#[test]
fn with_no_element_in_reach_shift_still_quantises_the_aim() {
  let (mut annotations, element) = turning();
  let field = field_with(&annotations, "turned", vec![element]);
  let edit = drag(
    &mut annotations,
    AnnotationHandle::Tail,
    point(542.0, 500.0),
  );
  let request = SnapRequest {
    field: &field,
    threshold: THRESHOLD,
  };
  // Aimed up and to the right, where the circle crosses nothing.
  let result = edit.update(&mut annotations, aimed_at(-1.0), shifted(), Some(request));
  assert!(close(aim(&annotations[0]), -FRAC_PI_4));
  assert_eq!(result, SnapResult::default());
  // Without Shift the same sample keeps the hand's own aim.
  edit.update(
    &mut annotations,
    aimed_at(-1.0),
    positional(),
    Some(request),
  );
  assert!(close(aim(&annotations[0]), -1.0));
}
