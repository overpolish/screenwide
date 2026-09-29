// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::{freehand_body_distance, freehand_distance};
use super::gesture::{drag, drag_new};
use super::model::new_draw;
use super::path::{chain, chain_distance, curves, fitted, simplify, MAX_CURVES};
use crate::editor::annotations::box_gesture::{EDGE_BOTTOM, EDGE_RIGHT};
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn stroke(points: &[(f64, f64)]) -> Annotation {
  let mut annotation = new_draw("s".to_owned(), point(points[0].0, points[0].1), None);
  if let AnnotationShape::Draw { points: held, .. } = &mut annotation.shape {
    *held = points.iter().map(|&(x, y)| point(x, y)).collect();
  }
  annotation
}

fn points_of(annotation: &Annotation) -> Vec<(f64, f64)> {
  match &annotation.shape {
    AnnotationShape::Draw { points, .. } => points.iter().map(|p| (p.x, p.y)).collect(),
    _ => unreachable!(),
  }
}

/// A wobbly line from (0, 100) to (400, 100), a few pixels either side.
fn wobble() -> Vec<AnnotationPoint> {
  (0..=200)
    .map(|index| {
      let x = f64::from(index) * 2.0;
      point(x, 100.0 + 3.0 * (x * 0.3).sin())
    })
    .collect()
}

#[test]
fn the_fitted_line_starts_and_ends_where_the_hand_did_and_bends_smoothly() {
  let square = [[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]];
  for rounding in [3.0, f32::INFINITY] {
    let drawn = chain(&square, rounding);
    assert_eq!(drawn.first(), Some(&[0.0, 0.0]));
    assert_eq!(drawn.last(), Some(&[0.0, 100.0]));
    // Where one curve ends the next leaves in the direction it arrived in.
    for curve in 1..curves(&drawn) {
      let joint = drawn[curve * 2];
      let into = [
        joint[0] - drawn[curve * 2 - 1][0],
        joint[1] - drawn[curve * 2 - 1][1],
      ];
      let out = [
        drawn[curve * 2 + 1][0] - joint[0],
        drawn[curve * 2 + 1][1] - joint[1],
      ];
      assert!(
        (into[0] * out[1] - into[1] * out[0]).abs() < 1e-3,
        "kinked at {joint:?}"
      );
    }
  }
  // A drawn corner is rounded only near it: the line passes a few pixels
  // inside the corner, where smoothing it cuts right across.
  let drawn = chain(&square[..3], 3.0);
  let smooth = chain(&square[..3], f32::INFINITY);
  assert!(chain_distance([100.0, 0.0], &drawn) < 2.0);
  assert!(chain_distance([100.0, 0.0], &smooth) > 15.0);
  // A dot is one curve that goes nowhere.
  assert_eq!(chain(&[[5.0, 5.0]], 3.0), vec![[5.0, 5.0]; 3]);
}

#[test]
fn thinning_keeps_the_corners_and_drops_the_points_on_the_way() {
  let line = [
    [0.0, 0.0],
    [10.0, 0.2],
    [20.0, 0.0],
    [20.0, 10.0],
    [20.0, 20.0],
  ];
  assert_eq!(
    simplify(&line, 0.5),
    vec![[0.0, 0.0], [20.0, 0.0], [20.0, 20.0]]
  );
}

#[test]
fn a_smoothed_stroke_is_drawn_with_far_fewer_curves_than_as_drawn() {
  let points = wobble();
  let drawn = curves(&fitted(&points, false));
  let smooth = curves(&fitted(&points, true));
  assert!(smooth * 4 < drawn, "{smooth} against {drawn}");
  // Loose, but still the line that was drawn.
  let chain = fitted(&points, true);
  assert!(freehand_distance([200.0, 100.0], &chain, 1.0) < 4.0);
}

#[test]
fn a_loop_joins_its_ends_as_smoothly_as_any_other_corner() {
  let square = [
    [0.0, 0.0],
    [100.0, 0.0],
    [100.0, 100.0],
    [0.0, 100.0],
    [0.0, 0.0],
  ];
  let loop_chain = chain(&square, f32::INFINITY);
  let (first, last) = (loop_chain[0], loop_chain[loop_chain.len() - 1]);
  assert_eq!(first, last, "a loop ends where it begins");
  // The last curve arrives heading the way the first leaves.
  let leaving = [loop_chain[1][0] - first[0], loop_chain[1][1] - first[1]];
  let control = loop_chain[loop_chain.len() - 2];
  let arriving = [last[0] - control[0], last[1] - control[1]];
  let cross = leaving[0] * arriving[1] - leaving[1] * arriving[0];
  let dot = leaving[0] * arriving[0] + leaving[1] * arriving[1];
  assert!(cross.abs() < 1e-3 && dot > 0.0, "{loop_chain:?}");
}

#[test]
fn a_long_scribble_is_thinned_until_it_fits() {
  let points: Vec<AnnotationPoint> = (0..4000)
    .map(|index| {
      let at = f64::from(index);
      point(at * 0.5, 200.0 + 40.0 * (at * 0.7).sin())
    })
    .collect();
  assert!(curves(&fitted(&points, false)) <= MAX_CURVES);
}

#[test]
fn a_stroke_is_picked_on_its_line_and_its_box_only_once_chosen() {
  let chain = fitted(&[point(0.0, 0.0), point(100.0, 100.0)], false);
  // On the line, inside the pen, and off it.
  assert!(freehand_distance([50.0, 50.0], &chain, 8.0) < 0.0);
  assert!(freehand_distance([52.0, 48.0], &chain, 8.0) < 0.0);
  assert!(freehand_distance([80.0, 20.0], &chain, 8.0) > 30.0);
  // The box takes a press anywhere inside it.
  assert!(freehand_body_distance([80.0, 20.0], &chain, 8.0, [0.0, 0.0], [100.0, 100.0]) <= 0.0);
}

#[test]
fn a_stroke_being_drawn_keeps_a_sample_only_once_the_hand_has_moved() {
  let mut annotation = stroke(&[(0.0, 0.0)]);
  let mut origin = AnnotationDragOrigin::new(point(0.0, 0.0), &annotation.shape);
  origin.source_per_point = 2.0;
  for sample in [(0.5, 0.0), (1.5, 0.0), (2.5, 0.0), (2.6, 0.2), (5.0, 0.0)] {
    drag_new(&mut annotation, point(sample.0, sample.1), &origin);
  }
  assert_eq!(
    points_of(&annotation),
    vec![(0.0, 0.0), (2.5, 0.0), (5.0, 0.0)]
  );
}

#[test]
fn a_stroke_is_scaled_into_the_box_its_grip_pulls_out_and_carried_by_its_body() {
  let before = stroke(&[(10.0, 10.0), (30.0, 20.0), (50.0, 30.0)]);
  let origin = AnnotationDragOrigin::new(point(50.0, 30.0), &before.shape);
  let mut resized = before.clone();
  drag(
    &mut resized,
    AnnotationHandle::Edges(EDGE_RIGHT | EDGE_BOTTOM),
    point(90.0, 50.0),
    &origin,
    false,
    None,
  );
  assert_eq!(
    points_of(&resized),
    vec![(10.0, 10.0), (50.0, 30.0), (90.0, 50.0)]
  );
  let origin = AnnotationDragOrigin::new(point(30.0, 20.0), &before.shape);
  let mut moved = before.clone();
  drag(
    &mut moved,
    AnnotationHandle::Body,
    point(35.0, 17.0),
    &origin,
    false,
    None,
  );
  assert_eq!(
    points_of(&moved),
    vec![(15.0, 7.0), (35.0, 17.0), (55.0, 27.0)]
  );
}

#[test]
fn a_stroke_round_trips_through_a_document() {
  let annotation = stroke(&[(1.0, 2.0), (3.0, 4.0)]);
  let json = serde_json::to_string(&annotation).unwrap();
  assert!(json.contains("\"kind\":\"draw\""), "{json}");
  assert_eq!(
    serde_json::from_str::<Annotation>(&json).unwrap(),
    annotation
  );
  // A document from before smoothing existed reads as drawn.
  let older = json.replace(",\"smooth\":false", "");
  assert_eq!(
    serde_json::from_str::<Annotation>(&older).unwrap(),
    annotation
  );
}
