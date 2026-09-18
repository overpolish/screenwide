// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::bend::curve_midpoint;
use super::counter::new_counter;
use super::gesture::{
  annotation_mode, drag_handle, next_annotation_id, AnnotationDragOrigin, AnnotationHandle,
  NewAnnotationKind, MODE_ARROW, MODE_COUNTER, MODE_NONE, MODE_SELECT,
};
use crate::editor::annotations::{
  Annotation, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};

/// A drag that began on `annotation` at `point`, which is what a whole-arrow
/// move measures its travel against.
fn origin(annotation: &Annotation, x: f64, y: f64) -> AnnotationDragOrigin {
  AnnotationDragOrigin::new(AnnotationPoint { x, y }, &annotation.shape)
}

fn curve_midpoint_of(annotation: &Annotation) -> AnnotationPoint {
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = annotation.shape
  else {
    unreachable!()
  };
  curve_midpoint(start, control, end)
}

/// The curve's midpoint in the chord's own terms, measured here rather than
/// asked of the code under test: how far along the chord it sits, and how far
/// off it, both as fractions of the chord. A test that asked `arrow_bend` for
/// the along would be asking the clamp to confirm itself.
fn bend(annotation: &Annotation) -> (f64, f64) {
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = annotation.shape
  else {
    unreachable!()
  };
  let (chord_x, chord_y) = (end.x - start.x, end.y - start.y);
  let length = chord_x.hypot(chord_y);
  let middle = curve_midpoint(start, control, end);
  let (dx, dy) = (middle.x - start.x, middle.y - start.y);
  (
    (dx * chord_x + dy * chord_y) / (length * length),
    (dx * -chord_y + dy * chord_x) / (length * length),
  )
}

fn arrow(id: &str) -> Annotation {
  Annotation {
    above_camera: false,
    animated: true,
    id: id.to_owned(),
    reveal: Default::default(),
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
  let from = origin(&annotation, 50.0, 0.0);
  drag_handle(
    &mut annotation,
    AnnotationHandle::Middle,
    AnnotationPoint { x: 50.0, y: 50.0 },
    &from,
    false,
    None,
  );
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = annotation.shape
  else {
    unreachable!()
  };
  assert_eq!(control, AnnotationPoint { x: 50.0, y: 100.0 });
  assert_eq!(
    curve_midpoint(start, control, end),
    AnnotationPoint { x: 50.0, y: 50.0 }
  );
}

/// The middle handle can be pulled anywhere; the curve follows only as far as
/// it can be drawn without closing into a loop.
#[test]
fn the_middle_handle_stops_at_the_hairpin() {
  let mut annotation = arrow("a");
  let from = origin(&annotation, 50.0, 0.0);
  drag_handle(
    &mut annotation,
    AnnotationHandle::Middle,
    AnnotationPoint { x: 50.0, y: 400.0 },
    &from,
    false,
    None,
  );
  // The chord is 100 long, so the midpoint stops 60 off it.
  assert_eq!(
    curve_midpoint_of(&annotation),
    AnnotationPoint { x: 50.0, y: 60.0 }
  );
}

/// The middle handle rides the chord's perpendicular bisector and nothing
/// else: pulled along the chord it does not move, because sliding the
/// midpoint that way is a kink rather than a bend and swings the head off the
/// shaft as the control point nears a tip.
#[test]
fn the_middle_handle_moves_only_across_the_chord() {
  let mut annotation = arrow("a");
  let from = origin(&annotation, 50.0, 0.0);
  drag_handle(
    &mut annotation,
    AnnotationHandle::Middle,
    AnnotationPoint { x: -500.0, y: 30.0 },
    &from,
    false,
    None,
  );
  assert_eq!(
    curve_midpoint_of(&annotation),
    AnnotationPoint { x: 50.0, y: 30.0 }
  );
}

/// A tip carries the bend with it: the curve keeps the same share of its own
/// chord rather than the same control point in the canvas, so it rotates and
/// scales with the shaft.
#[test]
fn dragging_a_tip_keeps_the_bend_against_the_chord() {
  let mut annotation = arrow("a");
  let bent = origin(&annotation, 50.0, 0.0);
  drag_handle(
    &mut annotation,
    AnnotationHandle::Middle,
    AnnotationPoint { x: 60.0, y: 40.0 },
    &bent,
    false,
    None,
  );
  let before = bend(&annotation);
  let from = origin(&annotation, 100.0, 0.0);
  drag_handle(
    &mut annotation,
    AnnotationHandle::End,
    AnnotationPoint { x: 10.0, y: 220.0 },
    &from,
    false,
    None,
  );
  let AnnotationShape::Arrow { end, .. } = annotation.shape else {
    unreachable!()
  };
  assert_eq!(end, AnnotationPoint { x: 10.0, y: 220.0 });
  let after = bend(&annotation);
  assert!((after.0 - before.0).abs() < 1e-9, "{before:?} {after:?}");
  assert!((after.1 - before.1).abs() < 1e-9, "{before:?} {after:?}");
}

/// A chord too short to point anywhere is drawn straight rather than bent
/// across a direction that does not exist.
#[test]
fn a_chord_with_no_direction_is_drawn_straight() {
  let mut annotation = arrow("a");
  let from = origin(&annotation, 100.0, 0.0);
  drag_handle(
    &mut annotation,
    AnnotationHandle::End,
    AnnotationPoint { x: 0.2, y: 0.0 },
    &from,
    false,
    None,
  );
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = annotation.shape
  else {
    unreachable!()
  };
  assert_eq!(
    control,
    AnnotationPoint {
      x: (start.x + end.x) / 2.0,
      y: (start.y + end.y) / 2.0,
    }
  );
}

/// However the handles are dragged, the curve keeps its one shape: the
/// midpoint halfway along the chord, no further off it than a hairpin. It can
/// therefore never double back on itself.
#[test]
fn no_sequence_of_drags_can_fold_the_curve() {
  let mut annotation = arrow("a");
  let pulls = [
    (AnnotationHandle::Middle, 400.0, 0.0),
    (AnnotationHandle::Middle, -600.0, 900.0),
    (AnnotationHandle::End, -400.0, 5.0),
    (AnnotationHandle::Middle, -900.0, -900.0),
    (AnnotationHandle::Start, 300.0, -200.0),
    (AnnotationHandle::Middle, 20.0, 4_000.0),
    (AnnotationHandle::End, 301.0, -199.0),
    (AnnotationHandle::Body, 40.0, 40.0),
  ];
  for (handle, x, y) in pulls {
    let from = origin(&annotation, 0.0, 0.0);
    drag_handle(
      &mut annotation,
      handle,
      AnnotationPoint { x, y },
      &from,
      false,
      None,
    );
    let (along, across) = bend(&annotation);
    assert!((along - 0.5).abs() < 1e-9, "{along} {handle:?}");
    assert!(across.abs() <= 0.6 + 1e-9, "{across} {handle:?}");
  }
}

#[test]
fn a_press_on_the_shaft_that_never_travels_changes_nothing() {
  let mut annotation = arrow("a");
  let before = annotation.clone();
  let from = origin(&annotation, 50.0, 0.0);
  drag_handle(
    &mut annotation,
    AnnotationHandle::Body,
    AnnotationPoint { x: 50.0, y: 0.0 },
    &from,
    false,
    None,
  );
  assert_eq!(annotation, before);
}

/// The whole arrow travels with the pointer, bend and all: every point moves
/// by the same delta, so the curve is carried rather than reshaped.
#[test]
fn dragging_the_shaft_moves_the_whole_arrow() {
  let mut annotation = Annotation {
    shape: AnnotationShape::Arrow {
      start: AnnotationPoint { x: 0.0, y: 0.0 },
      control: AnnotationPoint { x: 50.0, y: 40.0 },
      end: AnnotationPoint { x: 100.0, y: 0.0 },
    },
    ..arrow("a")
  };
  let from = origin(&annotation, 50.0, 20.0);
  drag_handle(
    &mut annotation,
    AnnotationHandle::Body,
    AnnotationPoint { x: 70.0, y: -5.0 },
    &from,
    false,
    None,
  );
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = annotation.shape
  else {
    unreachable!()
  };
  assert_eq!(start, AnnotationPoint { x: 20.0, y: -25.0 });
  assert_eq!(control, AnnotationPoint { x: 70.0, y: 15.0 });
  assert_eq!(end, AnnotationPoint { x: 120.0, y: -25.0 });
}

/// The move is measured from where the press landed, not from the last
/// sample, so a drag that doubles back lands exactly under the pointer.
#[test]
fn a_shaft_drag_is_measured_from_where_the_press_landed() {
  let mut annotation = arrow("a");
  let from = origin(&annotation, 50.0, 0.0);
  for point in [
    AnnotationPoint { x: 90.0, y: 30.0 },
    AnnotationPoint { x: 60.0, y: 10.0 },
  ] {
    drag_handle(
      &mut annotation,
      AnnotationHandle::Body,
      point,
      &from,
      false,
      None,
    );
  }
  let AnnotationShape::Arrow { start, end, .. } = annotation.shape else {
    unreachable!()
  };
  assert_eq!(start, AnnotationPoint { x: 10.0, y: 10.0 });
  assert_eq!(end, AnnotationPoint { x: 110.0, y: 10.0 });
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
  assert_eq!(AnnotationHandle::from_raw(4), Some(AnnotationHandle::Tail));
  assert_eq!(AnnotationHandle::from_raw(5), None);
}

#[test]
fn tool_names_read_back_as_modes() {
  assert_eq!(annotation_mode(Some("arrow")), MODE_ARROW);
  assert_eq!(annotation_mode(Some("counter")), MODE_COUNTER);
  assert_eq!(annotation_mode(Some("select")), MODE_SELECT);
  assert_eq!(annotation_mode(Some("crop")), MODE_NONE);
  assert_eq!(annotation_mode(None), MODE_NONE);
  assert_eq!(
    NewAnnotationKind::from_mode(MODE_COUNTER),
    NewAnnotationKind::Counter
  );
  assert_eq!(
    NewAnnotationKind::from_mode(MODE_ARROW),
    NewAnnotationKind::Arrow
  );
}

#[test]
fn a_counter_tail_drag_only_turns_it() {
  let mut counter = new_counter(
    "c".to_owned(),
    AnnotationPoint { x: 100.0, y: 100.0 },
    1,
    None,
    None,
  );
  let from = origin(&counter, 150.0, 100.0);
  drag_handle(
    &mut counter,
    AnnotationHandle::Tail,
    AnnotationPoint { x: 100.0, y: 40.0 },
    &from,
    false,
    None,
  );
  let AnnotationShape::Counter {
    center,
    value,
    angle,
  } = counter.shape
  else {
    unreachable!()
  };
  // Straight up in a y-down space is a quarter turn anticlockwise.
  assert!(
    (angle + std::f64::consts::FRAC_PI_2).abs() < 1e-9,
    "{angle}"
  );
  assert_eq!(center, AnnotationPoint { x: 100.0, y: 100.0 });
  assert_eq!(value, 1);
}

#[test]
fn a_counter_body_drag_carries_the_disc_by_the_travel() {
  let mut counter = new_counter(
    "c".to_owned(),
    AnnotationPoint { x: 100.0, y: 100.0 },
    2,
    None,
    None,
  );
  let from = origin(&counter, 110.0, 90.0);
  for point in [
    AnnotationPoint { x: 200.0, y: 200.0 },
    AnnotationPoint { x: 130.0, y: 110.0 },
  ] {
    drag_handle(
      &mut counter,
      AnnotationHandle::Body,
      point,
      &from,
      false,
      None,
    );
  }
  let AnnotationShape::Counter { center, angle, .. } = counter.shape else {
    unreachable!()
  };
  assert_eq!(center, AnnotationPoint { x: 120.0, y: 120.0 });
  assert_eq!(angle, 0.0);
}

#[test]
fn fresh_ids_do_not_repeat() {
  assert_ne!(next_annotation_id(), next_annotation_id());
}
