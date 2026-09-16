// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use super::*;
use crate::editor::annotations::AnnotationShape;

const PHASE_UP: u32 = 2;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn test_style() -> AnnotationStyle {
  AnnotationStyle {
    color: "#ffcc00".to_owned(),
    head: AnnotationHead::End,
    width: 8.0,
  }
}

#[test]
fn a_drag_completes_one_arrow_between_its_ends() {
  let style = test_style();
  let started = Instant::now();
  let mut drawing = None;

  assert!(step(&mut drawing, PHASE_DOWN, started, point(10.0, 20.0), &style).is_none());
  let dragged = started + Duration::from_millis(300);
  assert!(step(&mut drawing, PHASE_DRAG, dragged, point(40.0, 60.0), &style).is_none());
  let lifted = started + Duration::from_millis(800);
  let (completed, at) = step(&mut drawing, PHASE_UP, lifted, point(50.0, 80.0), &style).unwrap();

  assert!(drawing.is_none());
  let AnnotationShape::Arrow { start, end, .. } = completed.shape;
  assert_eq!((start.x, start.y), (10.0, 20.0));
  assert_eq!((end.x, end.y), (50.0, 80.0));
  assert_eq!(completed.style.width, 8.0);
  // Timed from the moment the stroke began, not from the moment it was
  // finished: the annotation was appearing on screen for the whole drag.
  assert_eq!(at, started);
}

#[test]
fn a_click_that_never_travelled_leaves_no_annotation() {
  let style = test_style();
  let at = Instant::now();
  let mut drawing = None;

  assert!(step(&mut drawing, PHASE_DOWN, at, point(10.0, 20.0), &style).is_none());
  assert!(step(&mut drawing, PHASE_UP, at, point(10.0, 20.0), &style).is_none());
  assert!(drawing.is_none());
}

#[test]
fn a_drag_that_never_began_completes_nothing() {
  let style = test_style();
  let at = Instant::now();
  let mut drawing = None;

  assert!(step(&mut drawing, PHASE_DRAG, at, point(10.0, 20.0), &style).is_none());
  assert!(step(&mut drawing, PHASE_UP, at, point(30.0, 40.0), &style).is_none());
}

#[test]
fn each_stroke_carries_its_own_identity() {
  let style = test_style();
  let at = Instant::now();
  let mut drawing = None;

  step(&mut drawing, PHASE_DOWN, at, point(0.0, 0.0), &style);
  let (first, _) = step(&mut drawing, PHASE_UP, at, point(10.0, 10.0), &style).unwrap();
  step(&mut drawing, PHASE_DOWN, at, point(0.0, 0.0), &style);
  let (second, _) = step(&mut drawing, PHASE_UP, at, point(10.0, 10.0), &style).unwrap();

  assert_ne!(first.id, second.id);
}
