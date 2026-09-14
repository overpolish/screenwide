// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::annotation::curve_midpoint;
use super::annotation_target::{drag_handle, next_annotation_id, AnnotationHandle};
use crate::screenshots::{
  Annotation, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};

fn arrow(id: &str) -> Annotation {
  Annotation {
    above_camera: false,
    id: id.to_owned(),
    shape: AnnotationShape::Arrow {
      start: AnnotationPoint { x: 0.0, y: 0.0 },
      control: AnnotationPoint { x: 50.0, y: 0.0 },
      end: AnnotationPoint { x: 100.0, y: 0.0 },
    },
    style: AnnotationStyle {
      color: "#ff0000".to_owned(),
      head: AnnotationHead::End,
      width: 6.0,
    },
  }
}

#[test]
fn dragging_the_middle_handle_bends_the_curve_through_it() {
  let mut annotation = arrow("a");
  drag_handle(
    &mut annotation,
    AnnotationHandle::Middle,
    AnnotationPoint { x: 50.0, y: 80.0 },
  );
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = annotation.shape;
  assert_eq!(control, AnnotationPoint { x: 50.0, y: 160.0 });
  assert_eq!(
    curve_midpoint(start, control, end),
    AnnotationPoint { x: 50.0, y: 80.0 }
  );
}

#[test]
fn dragging_a_tip_keeps_the_control_point() {
  let mut annotation = arrow("a");
  drag_handle(
    &mut annotation,
    AnnotationHandle::End,
    AnnotationPoint { x: 10.0, y: 10.0 },
  );
  let AnnotationShape::Arrow { control, end, .. } = annotation.shape;
  assert_eq!(end, AnnotationPoint { x: 10.0, y: 10.0 });
  assert_eq!(control, AnnotationPoint { x: 50.0, y: 0.0 });
}

#[test]
fn a_press_on_the_shaft_changes_nothing() {
  let mut annotation = arrow("a");
  let before = annotation.clone();
  drag_handle(
    &mut annotation,
    AnnotationHandle::Body,
    AnnotationPoint { x: 10.0, y: 10.0 },
  );
  assert_eq!(annotation, before);
}

#[test]
fn handles_read_back_from_their_raw_codes() {
  assert_eq!(AnnotationHandle::from_raw(0), Some(AnnotationHandle::Start));
  assert_eq!(
    AnnotationHandle::from_raw(1),
    Some(AnnotationHandle::Middle)
  );
  assert_eq!(AnnotationHandle::from_raw(2), Some(AnnotationHandle::End));
  assert_eq!(AnnotationHandle::from_raw(3), Some(AnnotationHandle::Body));
  assert_eq!(AnnotationHandle::from_raw(4), None);
}

#[test]
fn fresh_ids_do_not_repeat() {
  assert_ne!(next_annotation_id(), next_annotation_id());
}
