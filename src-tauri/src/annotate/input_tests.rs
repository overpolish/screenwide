// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use super::*;
use crate::editor::annotations::AnnotationShape;

const PHASE_UP: u32 = 2;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// The arrow tool in hand. A counter's number and aim are never read for it.
fn arrow_tool() -> Tool {
  Tool {
    shape: AnnotationKind::Arrow,
    value: 0,
    angle: 0.0,
  }
}

fn counter_tool(value: u32, angle: f64) -> Tool {
  Tool {
    shape: AnnotationKind::Counter,
    value,
    angle,
  }
}

#[test]
fn a_drag_completes_one_arrow_between_its_ends() {
  let tool = arrow_tool();
  let started = Instant::now();
  let mut drawing = None;

  assert!(step(&mut drawing, PHASE_DOWN, started, point(10.0, 20.0), &tool).is_none());
  let dragged = started + Duration::from_millis(300);
  assert!(step(&mut drawing, PHASE_DRAG, dragged, point(40.0, 60.0), &tool).is_none());
  let lifted = started + Duration::from_millis(800);
  let (completed, at) = step(&mut drawing, PHASE_UP, lifted, point(50.0, 80.0), &tool).unwrap();

  assert!(drawing.is_none());
  let AnnotationShape::Arrow { start, end, .. } = completed.shape else {
    unreachable!()
  };
  assert_eq!((start.x, start.y), (10.0, 20.0));
  assert_eq!((end.x, end.y), (50.0, 80.0));
  // An arrow is drawn at the stroke preset, not at the counter's disc.
  assert_eq!(completed.style.width, 8.0);
  // Timed from the moment the stroke began, not from the moment it was
  // finished: the annotation was appearing on screen for the whole drag.
  assert_eq!(at, started);
}

#[test]
fn a_click_that_never_travelled_leaves_no_annotation() {
  let tool = arrow_tool();
  let at = Instant::now();
  let mut drawing = None;

  assert!(step(&mut drawing, PHASE_DOWN, at, point(10.0, 20.0), &tool).is_none());
  assert!(step(&mut drawing, PHASE_UP, at, point(10.0, 20.0), &tool).is_none());
  assert!(drawing.is_none());
}

#[test]
fn a_drag_that_never_began_completes_nothing() {
  let tool = arrow_tool();
  let at = Instant::now();
  let mut drawing = None;

  assert!(step(&mut drawing, PHASE_DRAG, at, point(10.0, 20.0), &tool).is_none());
  assert!(step(&mut drawing, PHASE_UP, at, point(30.0, 40.0), &tool).is_none());
}

#[test]
fn each_stroke_carries_its_own_identity() {
  let tool = arrow_tool();
  let at = Instant::now();
  let mut drawing = None;

  step(&mut drawing, PHASE_DOWN, at, point(0.0, 0.0), &tool);
  let (first, _) = step(&mut drawing, PHASE_UP, at, point(10.0, 10.0), &tool).unwrap();
  step(&mut drawing, PHASE_DOWN, at, point(0.0, 0.0), &tool);
  let (second, _) = step(&mut drawing, PHASE_UP, at, point(10.0, 10.0), &tool).unwrap();

  assert_ne!(first.id, second.id);
}

#[test]
fn a_click_drops_a_counter_numbered_and_aimed_as_the_tool_says() {
  let tool = counter_tool(3, std::f64::consts::FRAC_PI_2);
  let at = Instant::now();
  let mut drawing = None;

  assert!(step(&mut drawing, PHASE_DOWN, at, point(10.0, 20.0), &tool).is_none());
  let (completed, _) = step(&mut drawing, PHASE_UP, at, point(10.0, 20.0), &tool).unwrap();

  let AnnotationShape::Counter {
    center,
    value,
    angle,
  } = completed.shape
  else {
    unreachable!()
  };
  assert_eq!((center.x, center.y), (10.0, 20.0));
  assert_eq!(value, 3);
  assert_eq!(angle, std::f64::consts::FRAC_PI_2);
  // The disc, from the counter's own setting: an arrow's eight-pixel stroke
  // would be a disc too small to hold a number.
  assert_eq!(completed.style.width, 56.0);
}

#[test]
fn a_drag_carries_the_counter_it_dropped() {
  let tool = counter_tool(1, 0.0);
  let at = Instant::now();
  let mut drawing = None;

  step(&mut drawing, PHASE_DOWN, at, point(10.0, 20.0), &tool);
  step(&mut drawing, PHASE_DRAG, at, point(40.0, 60.0), &tool);
  let (completed, _) = step(&mut drawing, PHASE_UP, at, point(50.0, 80.0), &tool).unwrap();

  let AnnotationShape::Counter { center, .. } = completed.shape else {
    unreachable!()
  };
  assert_eq!((center.x, center.y), (50.0, 80.0));
}

#[test]
fn a_counter_is_on_screen_from_the_press_and_an_arrow_is_not() {
  let at = Instant::now();
  let mut drawing = None;

  step(
    &mut drawing,
    PHASE_DOWN,
    at,
    point(10.0, 20.0),
    &arrow_tool(),
  );
  assert!(!is_drawn(drawing.as_ref().unwrap()));

  step(
    &mut drawing,
    PHASE_DOWN,
    at,
    point(10.0, 20.0),
    &counter_tool(1, 0.0),
  );
  assert!(is_drawn(drawing.as_ref().unwrap()));
}
