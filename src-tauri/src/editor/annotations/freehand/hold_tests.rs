// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a stroke held still at its end is taken for, and how the hold gives
//! way to a hand that moves on.

use std::f64::consts::TAU;
use std::time::{Duration, Instant};

use super::hold::{StrokeHold, HOLD};
use super::model::new_draw;
use super::recognise::{recognise, Recognised};
use crate::editor::annotations::{Annotation, AnnotationHead, AnnotationPoint, AnnotationShape};

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn near(a: AnnotationPoint, b: AnnotationPoint, within: f64) -> bool {
  (a.x - b.x).hypot(a.y - b.y) <= within
}

/// A hand's line from (0, 100) to (400, 100), a pixel or two either side.
fn wobbly_line() -> Vec<AnnotationPoint> {
  (0..=80)
    .map(|step| {
      let x = f64::from(step) * 5.0;
      point(x, 100.0 + 2.0 * (x / 23.0).sin())
    })
    .collect()
}

/// `count` points round the ellipse at (200, 200) with these radii, closing
/// a little short of where they began.
fn ellipse(rx: f64, ry: f64, count: u32) -> Vec<AnnotationPoint> {
  (0..count)
    .map(|step| {
      let angle = TAU * 0.97 * f64::from(step) / f64::from(count);
      point(200.0 + rx * angle.cos(), 200.0 + ry * angle.sin())
    })
    .collect()
}

/// The square (100, 100) to (300, 300) walked round from its top-left
/// corner, stopping just short of it.
fn square() -> Vec<AnnotationPoint> {
  let corners = [
    (100.0, 100.0),
    (300.0, 100.0),
    (300.0, 300.0),
    (100.0, 300.0),
    (100.0, 110.0),
  ];
  corners
    .windows(2)
    .flat_map(|pair| {
      let ((ax, ay), (bx, by)) = (pair[0], pair[1]);
      (0..20).map(move |step| {
        let t = f64::from(step) / 20.0;
        point(ax + (bx - ax) * t, ay + (by - ay) * t)
      })
    })
    .collect()
}

fn stroke(points: Vec<AnnotationPoint>) -> Annotation {
  let mut annotation = new_draw("s".to_owned(), points[0], None);
  annotation.style.color = "#ff0000".to_owned();
  annotation.style.width = 12.0;
  annotation.shape = AnnotationShape::Draw {
    points,
    smooth: false,
  };
  annotation
}

#[test]
fn a_line_is_taken_for_an_arrow_without_a_head_between_the_hands_ends() {
  let Some(Recognised::Arrow {
    start,
    control,
    end,
    head,
  }) = recognise(&wobbly_line(), 1.0)
  else {
    panic!("a line is an arrow");
  };
  assert!(!head);
  assert!(near(start, point(0.0, 100.0), 0.5));
  assert!(near(end, point(400.0, 100.0), 3.0));
  // Straight: the control sits on the chord's middle.
  assert!(near(control, point(200.0, (start.y + end.y) / 2.0), 0.5));
}

#[test]
fn an_even_curve_is_taken_for_a_bent_arrow() {
  let arc: Vec<_> = (0..=60)
    .map(|step| {
      let t = f64::from(step) / 60.0;
      point(400.0 * t, 200.0 - 240.0 * t * (1.0 - t))
    })
    .collect();
  let Some(Recognised::Arrow { control, head, .. }) = recognise(&arc, 1.0) else {
    panic!("an even curve is an arrow");
  };
  assert!(!head);
  assert!(near(control, point(200.0, 80.0), 3.0), "{control:?}");
}

#[test]
fn a_hook_at_the_end_is_an_arrowhead_and_the_shaft_ends_at_its_tip() {
  let mut points = wobbly_line();
  let tip = *points.last().unwrap();
  points.extend((1..=10).map(|step| {
    let t = f64::from(step) / 10.0;
    point(tip.x - 40.0 * t, tip.y - 30.0 * t)
  }));
  let Some(Recognised::Arrow { end, head, .. }) = recognise(&points, 1.0) else {
    panic!("a hooked line is an arrow");
  };
  assert!(head);
  assert!(near(end, tip, 0.5));
}

#[test]
fn the_jitter_of_a_resting_hand_is_no_hook() {
  let mut points = wobbly_line();
  let tip = *points.last().unwrap();
  points.extend([
    point(tip.x - 1.5, tip.y + 1.0),
    point(tip.x - 0.5, tip.y - 1.5),
  ]);
  assert!(matches!(
    recognise(&points, 1.0),
    Some(Recognised::Arrow { head: false, .. })
  ));
}

#[test]
fn a_closed_stroke_is_a_box_or_a_pill_round_the_oval_it_follows() {
  let Some(Recognised::Box { low, high }) = recognise(&square(), 1.0) else {
    panic!("a closed square is a box");
  };
  assert!(near(low, point(100.0, 100.0), 0.5) && near(high, point(300.0, 300.0), 0.5));
  // An oval lying across is a pill over the box round it.
  let Some(Recognised::Round { low, high }) = recognise(&ellipse(150.0, 90.0, 90), 1.0) else {
    panic!("an oval lying across is a pill");
  };
  assert!(near(low, point(50.0, 110.0), 6.0), "{low:?}");
  assert!(near(high, point(350.0, 290.0), 6.0), "{high:?}");
}

#[test]
fn a_leaning_oval_is_an_ellipse_turned_the_same_way() {
  let (sin, cos) = (-45f64).to_radians().sin_cos();
  let leaf: Vec<_> = ellipse(130.0, 70.0, 120)
    .into_iter()
    .map(|at| {
      let (x, y) = (at.x - 200.0, at.y - 200.0);
      point(200.0 + x * cos - y * sin, 200.0 + x * sin + y * cos)
    })
    .collect();
  let Some(Recognised::Ellipse(fitted)) = recognise(&shaky(leaf), 1.0) else {
    panic!("a leaning oval is an ellipse");
  };
  // Its long radius lies along the lean, either way along the line.
  let lean = fitted.angle.to_degrees().rem_euclid(180.0);
  assert!((lean - 135.0).abs() < 5.0, "{fitted:?}");
}

#[test]
fn a_loop_left_open_by_more_than_a_sliver_is_no_shape() {
  // Three quarters and a bit of a circle: round, but open.
  let open: Vec<_> = (0..=100)
    .map(|step| {
      let angle = TAU * 0.82 * f64::from(step) / 100.0;
      point(200.0 + 100.0 * angle.cos(), 200.0 + 100.0 * angle.sin())
    })
    .collect();
  assert!(!matches!(
    recognise(&open, 1.0),
    Some(Recognised::Box { .. } | Recognised::Ellipse(_))
  ));
}

/// `turns` round the superellipse |x|^n + |y|^n = 1 at (200, 200), 100
/// across each way. A mouse's circle comes out near n = 3, fuller in the
/// corners than a true one.
fn superellipse(n: f64, turns: f64) -> Vec<AnnotationPoint> {
  (0..=150)
    .map(|step| {
      let angle = TAU * turns * f64::from(step) / 150.0;
      let (sin, cos) = angle.sin_cos();
      point(
        200.0 + 100.0 * cos.signum() * cos.abs().powf(2.0 / n),
        200.0 + 100.0 * sin.signum() * sin.abs().powf(2.0 / n),
      )
    })
    .collect()
}

#[test]
fn a_squarish_circle_drawn_past_its_start_is_still_round() {
  assert!(matches!(
    recognise(&superellipse(3.0, 1.15), 1.0),
    Some(Recognised::Round { .. })
  ));
  // A box whose corners the hand has rounded is still a box. Its points
  // bunch towards the corners, so it is drawn a little past its start to
  // close.
  assert!(matches!(
    recognise(&superellipse(8.0, 1.02), 1.0),
    Some(Recognised::Box { .. })
  ));
}

/// Strokes a hand drew with a mouse or a trackpad and held, as the editor
/// logged them, and what the hand wanted from each: wobbly and egg-shaped
/// circles and ovals, bowed boxes, one left open at a corner, arrows whose
/// heads double back past the tip, curves with a jerk in them, and a loop
/// whose end stops short of the line it heads for.
#[test]
fn a_hands_held_strokes_are_taken_for_what_it_wanted() {
  #[derive(serde::Deserialize)]
  struct Held {
    want: String,
    unit: f64,
    points: Vec<[f64; 2]>,
  }
  let held: Vec<Held> = serde_json::from_str(include_str!("held_strokes.json")).unwrap();
  for (index, stroke) in held.iter().enumerate() {
    let points: Vec<_> = stroke.points.iter().map(|&[x, y]| point(x, y)).collect();
    let taken = recognise(&points, stroke.unit);
    let closes = |points: &[AnnotationPoint]| points.first() == points.last();
    let wanted = match (stroke.want.as_str(), &taken) {
      ("round" | "closed", Some(Recognised::Round { .. } | Recognised::Ellipse(_))) => true,
      ("box" | "closed", Some(Recognised::Box { .. })) => true,
      ("arrow with head", Some(Recognised::Arrow { head: true, .. })) => true,
      ("curve", Some(Recognised::Curve { points })) => !closes(points),
      ("closed", Some(Recognised::Lines { points } | Recognised::Curve { points })) => {
        closes(points)
      }
      _ => false,
    };
    assert!(
      wanted,
      "stroke {index} wanted {} but was taken for {taken:?}",
      stroke.want
    );
  }
}

/// The corners walked in turn at `per` points a leg: round the loop, stopping
/// a step short of where it began, where `closed`, and otherwise from the
/// first corner to the last.
fn legs(corners: &[(f64, f64)], per: u32, closed: bool) -> Vec<AnnotationPoint> {
  let sides = if closed {
    corners.len()
  } else {
    corners.len() - 1
  };
  let mut points: Vec<_> = (0..sides)
    .flat_map(|side| {
      let ((ax, ay), (bx, by)) = (corners[side], corners[(side + 1) % corners.len()]);
      (0..per).map(move |step| {
        let t = f64::from(step) / f64::from(per);
        point(ax + (bx - ax) * t, ay + (by - ay) * t)
      })
    })
    .collect();
  if closed {
    points.pop();
  } else {
    let (x, y) = corners[corners.len() - 1];
    points.push(point(x, y));
  }
  points
}

/// `points` with a hand's jitter, a pixel and a half one way and then the
/// other.
fn shaky(points: Vec<AnnotationPoint>) -> Vec<AnnotationPoint> {
  points
    .into_iter()
    .enumerate()
    .map(|(index, at)| {
      let jitter = if index % 2 == 0 { 1.5 } else { -1.5 };
      point(at.x + 0.6 * jitter, at.y - jitter)
    })
    .collect()
}

fn lines_of(recognised: Option<Recognised>) -> Vec<AnnotationPoint> {
  match recognised {
    Some(Recognised::Lines { points }) => points,
    other => panic!("straight lines, not {other:?}"),
  }
}

/// Whether every one of `corners` has a vertex within `within` of it.
fn has_corners(vertices: &[AnnotationPoint], corners: &[(f64, f64)], within: f64) -> bool {
  corners.iter().all(|&(x, y)| {
    vertices
      .iter()
      .any(|vertex| near(*vertex, point(x, y), within))
  })
}

#[test]
fn a_leaning_box_is_upright_and_a_turned_one_or_a_triangle_is_its_lines() {
  // The way a mouse draws a box: every side leaning, none by much.
  let leaning = [(65.0, 45.0), (315.0, 50.0), (265.0, 240.0), (35.0, 210.0)];
  assert!(matches!(
    recognise(&shaky(legs(&leaning, 30, true)), 1.0),
    Some(Recognised::Box { .. })
  ));
  // A slanted box, whose obtuse corners turn little.
  let slanted = [(60.0, 160.0), (220.0, 40.0), (480.0, 40.0), (330.0, 170.0)];
  let vertices = lines_of(recognise(&shaky(legs(&slanted, 40, true)), 1.0));
  assert_eq!(vertices.len(), 5);
  assert!(has_corners(&vertices, &slanted, 3.0), "{vertices:?}");
  // A square turned a third of the way to a diamond keeps its turn.
  let (sin, cos) = 30f64.to_radians().sin_cos();
  let turned: Vec<_> = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
    .iter()
    .map(|&(x, y)| {
      (
        200.0 + 100.0 * (x * cos - y * sin),
        200.0 + 100.0 * (x * sin + y * cos),
      )
    })
    .collect();
  let vertices = lines_of(recognise(&shaky(legs(&turned, 40, true)), 1.0));
  assert_eq!(vertices.len(), 5, "four corners, closed back to the first");
  assert_eq!(vertices[0], vertices[4]);
  assert!(has_corners(&vertices, &turned, 3.0), "{vertices:?}");
  let triangle = [(100.0, 300.0), (300.0, 300.0), (200.0, 120.0)];
  let vertices = lines_of(recognise(&shaky(legs(&triangle, 40, true)), 1.0));
  assert_eq!(vertices.len(), 4);
  assert!(has_corners(&vertices, &triangle, 3.0), "{vertices:?}");
  // A hand's triangle, with a kink where the pen wavered near a corner.
  let wavering = [
    (30.0, 272.0),
    (295.0, 42.0),
    (395.0, 258.0),
    (360.0, 262.0),
    (365.0, 268.0),
    (48.0, 262.0),
  ];
  let vertices = lines_of(recognise(&shaky(legs(&wavering, 40, true)), 1.0));
  assert_eq!(vertices.len(), 4, "{vertices:?}");
}

#[test]
fn a_hand_that_turns_clearly_between_straight_legs_draws_those_lines() {
  let l = [(50.0, 50.0), (50.0, 300.0), (350.0, 300.0)];
  let vertices = lines_of(recognise(&shaky(legs(&l, 60, false)), 1.0));
  assert_eq!(vertices.len(), 3);
  assert!(has_corners(&vertices, &l, 3.0), "{vertices:?}");
  let zigzag = [(0.0, 0.0), (200.0, 150.0), (400.0, 0.0), (600.0, 150.0)];
  let vertices = lines_of(recognise(&shaky(legs(&zigzag, 50, false)), 1.0));
  assert_eq!(vertices.len(), 4);
  assert!(has_corners(&vertices, &zigzag, 3.0), "{vertices:?}");
  // Back along the same line: the lines cannot cross, so the turn stays
  // where the hand made it.
  let mut back = wobbly_line();
  back.extend(wobbly_line().into_iter().rev().skip(1).take(50));
  let vertices = lines_of(recognise(&back, 1.0));
  assert!(near(vertices[1], point(400.0, 100.0), 8.0), "{vertices:?}");
}

fn curve_of(recognised: Option<Recognised>) -> Vec<AnnotationPoint> {
  match recognised {
    Some(Recognised::Curve { points }) => points,
    other => panic!("a curve, not {other:?}"),
  }
}

#[test]
fn anything_else_is_a_curve_through_the_stroke() {
  // A wave: open, and no single curve or straight lines follow it.
  let wave: Vec<_> = (0..=80)
    .map(|step| {
      let x = f64::from(step) * 5.0;
      point(x, 100.0 + 60.0 * (x / 60.0).sin())
    })
    .collect();
  let through = curve_of(recognise(&wave, 1.0));
  assert!(near(through[0], wave[0], 0.5) && near(through[through.len() - 1], wave[80], 0.5));
  assert!(through
    .iter()
    .all(|at| (at.y - 100.0 - 60.0 * (at.x / 60.0).sin()).abs() < 3.0));
  // A deltoid: three sharp cusps, but legs that curve in between them,
  // which no straight line follows. A closed curve ends where it began.
  let deltoid: Vec<_> = (0..240)
    .map(|step| {
      let t = TAU * f64::from(step) / 240.0;
      point(
        200.0 + 60.0 * (2.0 * t.cos() + (2.0 * t).cos()),
        200.0 + 60.0 * (2.0 * t.sin() - (2.0 * t).sin()),
      )
    })
    .collect();
  let through = curve_of(recognise(&deltoid, 1.0));
  assert_eq!(through[0], through[through.len() - 1]);
}

#[test]
fn a_star_drawn_with_straight_legs_keeps_every_point() {
  let star: Vec<_> = (0..10)
    .map(|corner| {
      let reach = if corner % 2 == 0 { 100.0 } else { 45.0 };
      let angle = TAU * f64::from(corner) / 10.0;
      (200.0 + reach * angle.cos(), 200.0 + reach * angle.sin())
    })
    .collect();
  let vertices = lines_of(recognise(&legs(&star, 15, true), 1.0));
  assert_eq!(vertices.len(), 11);
  assert!(has_corners(&vertices, &star, 3.0), "{vertices:?}");
}

#[test]
fn a_dot_is_left_as_drawn() {
  let dot = vec![point(10.0, 10.0), point(14.0, 12.0), point(12.0, 16.0)];
  assert_eq!(recognise(&dot, 1.0), None);
  // The same stroke is no dot drawn zoomed far in, where a screen point is a
  // tenth of a source pixel.
  assert!(recognise(&dot, 0.1).is_some());
}

#[test]
fn a_held_stroke_becomes_an_arrow_in_its_own_colour_and_pen_and_moving_on_gives_it_back() {
  let start = Instant::now();
  let points = wobbly_line();
  let rest = *points.last().unwrap();
  let mut annotation = stroke(points.clone());
  let mut hold = StrokeHold::new(rest, start);
  assert!(!hold.hold(
    &mut annotation,
    1.0,
    start + HOLD - Duration::from_millis(1)
  ));
  assert!(hold.hold(&mut annotation, 1.0, start + HOLD));
  assert!(matches!(annotation.shape, AnnotationShape::Arrow { .. }));
  assert_eq!(annotation.style.head, AnnotationHead::None);
  assert_eq!(annotation.style.color, "#ff0000");
  assert_eq!(annotation.style.width, 12.0);
  assert_eq!(annotation.id, "s");
  assert!(
    annotation.pen,
    "Clear all takes what the pen made, whatever it became"
  );
  // A hand still resting keeps what the stroke was taken for.
  assert!(!hold.sample(
    &mut annotation,
    point(rest.x + 2.0, rest.y),
    1.0,
    start + HOLD
  ));
  assert!(matches!(annotation.shape, AnnotationShape::Arrow { .. }));
  // Moving on gives the stroke back to carry on, and the new rest is timed
  // afresh.
  let later = start + HOLD * 2;
  assert!(hold.sample(&mut annotation, point(rest.x + 30.0, rest.y), 1.0, later));
  assert_eq!(annotation.shape, stroke(points).shape);
  assert!(!hold.hold(&mut annotation, 1.0, later + HOLD / 2));
}

#[test]
fn a_held_box_or_circle_is_a_shape_and_an_oval_a_stroke_round_it() {
  let start = Instant::now();
  let mut boxy = stroke(square());
  assert!(StrokeHold::new(point(100.0, 110.0), start).hold(&mut boxy, 1.0, start + HOLD));
  assert!(matches!(boxy.shape, AnnotationShape::Shape { .. }));
  assert_eq!(boxy.style.radius, 0.0);
  // A circle is the shape tool's: a square box rounded fully, round the
  // circle the hand drew.
  let mut round = stroke(ellipse(120.0, 115.0, 90));
  assert!(StrokeHold::new(point(0.0, 0.0), start).hold(&mut round, 1.0, start + HOLD));
  let AnnotationShape::Shape {
    start: low,
    end: high,
    ..
  } = round.shape
  else {
    panic!("a circle is a shape");
  };
  assert_eq!(round.style.radius, 50.0);
  assert!(
    (high.x - low.x - (high.y - low.y)).abs() < 1e-9,
    "a square box"
  );
  assert!(near(
    point((low.x + high.x) / 2.0, (low.y + high.y) / 2.0),
    point(200.0, 200.0),
    5.0
  ));
  assert!((high.x - low.x - 235.0).abs() < 8.0);
  // An oval that leans has no shape to stand in for it, so it is a stroke
  // round the oval, turned as the hand turned it.
  let (sin, cos) = 45f64.to_radians().sin_cos();
  let leaning: Vec<_> = ellipse(150.0, 80.0, 90)
    .into_iter()
    .map(|at| {
      let (x, y) = (at.x - 200.0, at.y - 200.0);
      point(200.0 + x * cos - y * sin, 200.0 + x * sin + y * cos)
    })
    .collect();
  let mut oval = stroke(leaning);
  assert!(StrokeHold::new(point(0.0, 0.0), start).hold(&mut oval, 1.0, start + HOLD));
  let AnnotationShape::Draw { points, smooth } = &oval.shape else {
    panic!("a leaning oval is a stroke round it");
  };
  assert!(*smooth);
  assert!(points.iter().all(|at| {
    let (x, y) = (at.x - 200.0, at.y - 200.0);
    let (along, across) = ((x * cos + y * sin) / 150.0, (y * cos - x * sin) / 80.0);
    (along.hypot(across) - 1.0).abs() < 0.06
  }));
}
