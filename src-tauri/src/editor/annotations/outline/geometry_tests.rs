// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::f32::consts::FRAC_PI_4;

use super::*;

const WHOLE: AnnotationReveal = AnnotationReveal {
  low: 0.0,
  high: 1.0,
  scale: 1.0,
  opacity: 1.0,
  previous: [0.0, 1.0, 1.0, 1.0],
};

const PEN: f32 = 8.0;

fn clean(radius: f32) -> ArrowGeometry {
  prepare_shape([100.0, 100.0], [300.0, 200.0], radius, 0.0, PEN, WHOLE)
}

fn hand_drawn(radius: f32, seed: u32) -> ArrowGeometry {
  prepare_shape(
    [100.0, 100.0],
    [300.0, 200.0],
    radius,
    seed as f32 + 1.0,
    PEN,
    WHOLE,
  )
}

/// The point `along` the clean outline, relative to the box's centre, and
/// the outward normal there: the walk the pen follows, written out plainly
/// so the distance can be held to it. A square corner's one point faces out
/// along the diagonal.
fn walk(along: f32, half: [f32; 2], rounding: f32) -> ([f32; 2], [f32; 2]) {
  let inner = [half[0] - rounding, half[1] - rounding];
  let (across, down, quarter) = (2.0 * inner[0], 2.0 * inner[1], FRAC_PI_2 * rounding);
  let arc = |corner: [f32; 2], from: f32, run: f32| {
    let turned = if rounding > 0.0 {
      (run / rounding).clamp(0.0, FRAC_PI_2)
    } else {
      FRAC_PI_4
    };
    let normal = [(from + turned).cos(), (from + turned).sin()];
    (add(corner, scale(normal, rounding)), normal)
  };
  let mut s = along.rem_euclid(perimeter(half, rounding));
  if s < across {
    return ([s - inner[0], -half[1]], [0.0, -1.0]);
  }
  s -= across;
  if s < quarter {
    return arc([inner[0], -inner[1]], -FRAC_PI_2, s);
  }
  s -= quarter;
  if s < down {
    return ([half[0], s - inner[1]], [1.0, 0.0]);
  }
  s -= down;
  if s < quarter {
    return arc([inner[0], inner[1]], 0.0, s);
  }
  s -= quarter;
  if s < across {
    return ([inner[0] - s, half[1]], [0.0, 1.0]);
  }
  s -= across;
  if s < quarter {
    return arc([-inner[0], inner[1]], FRAC_PI_2, s);
  }
  s -= quarter;
  if s < down {
    return ([-half[0], inner[1] - s], [-1.0, 0.0]);
  }
  s -= down;
  arc([-inner[0], -inner[1]], PI, s)
}

/// Where the pen is `along` the stroke, in the space it was prepared in.
fn pen_at(geometry: &ArrowGeometry, along: f32) -> [f32; 2] {
  let centre = scale(add(geometry.a, geometry.b), 0.5);
  let half = scale(subtract(geometry.b, geometry.a), 0.5);
  let (on, normal) = walk(geometry.c[0] + along, half, geometry.rounding);
  add(centre, add(on, scale(normal, wander(geometry, along))))
}

/// A clean shape is picked on its pen line all the way round, and not from
/// inside the box it outlines.
#[test]
fn a_clean_shape_is_its_line_and_not_its_inside() {
  for radius in [0.0, 25.0, 50.0] {
    let geometry = clean(radius);
    assert_eq!(shape_distance([200.0, 100.0], &geometry), -4.0);
    assert_eq!(shape_distance([200.0, 200.0], &geometry), -4.0);
    assert_eq!(shape_distance([100.0, 150.0], &geometry), -4.0);
    assert!((shape_distance([200.0, 90.0], &geometry) - 6.0).abs() < 1e-4);
    assert!(shape_distance([200.0, 150.0], &geometry) > 40.0);
  }
  // A square corner is drawn round the pen, not mitred.
  let square = clean(0.0);
  let out = 4.0 / std::f32::consts::SQRT_2;
  assert!(shape_distance([100.0 - out, 100.0 - out], &square).abs() < 1e-3);
}

/// Rounding a square all the way makes a circle.
#[test]
fn a_square_rounded_all_the_way_is_a_circle() {
  let circle = prepare_shape([0.0, 0.0], [100.0, 100.0], 50.0, 0.0, 4.0, WHOLE);
  for step in 0..16 {
    let angle = TAU * step as f32 / 16.0;
    let on = [50.0 + 50.0 * angle.cos(), 50.0 + 50.0 * angle.sin()];
    assert!((shape_distance(on, &circle) + 2.0).abs() < 1e-3, "{angle}");
  }
}

/// Part way through arriving, only the stroke drawn so far is there.
#[test]
fn a_revealing_shape_draws_only_its_window() {
  let half_way = AnnotationReveal { high: 0.5, ..WHOLE };
  let geometry = prepare_shape([100.0, 100.0], [300.0, 200.0], 0.0, 0.0, PEN, half_way);
  // Clockwise from the top-left corner, the top side is drawn first and the
  // bottom one last.
  assert!(shape_distance([200.0, 100.0], &geometry) < 0.0);
  assert!(shape_distance([200.0, 200.0], &geometry) > 0.0);
  let unstarted = AnnotationReveal { high: 0.0, ..WHOLE };
  let geometry = prepare_shape([100.0, 100.0], [300.0, 200.0], 0.0, 0.0, PEN, unstarted);
  assert_eq!(shape_distance([100.0, 100.0], &geometry), f32::INFINITY);
}

/// Wherever the pen went, the stroke is drawn: no stretch of it goes
/// missing where its passes over one corner stray apart.
#[test]
fn a_hand_drawn_stroke_covers_its_whole_path() {
  for radius in [0.0, 20.0, 50.0] {
    for seed in 0..24 {
      let geometry = hand_drawn(radius, seed);
      let stroke = geometry.c[1];
      for step in 0..=2000 {
        let along = stroke * step as f32 / 2000.0;
        let distance = shape_distance(pen_at(&geometry, along), &geometry);
        assert!(
          distance < -PEN * 0.45,
          "{radius}% seed {seed} at {along}: {distance}"
        );
      }
    }
  }
}

/// A hand-drawn stroke finishes each of the four ways across seeds - crossing
/// its start, opening away from it, covering it, or stopping short of it -
/// and every tip is either clear of the other pass or wholly under it, so it
/// ends as a pen line rather than a lump on one.
#[test]
fn a_hand_drawn_stroke_finishes_every_way_with_clean_tips() {
  let clean = |gap: f32| gap.abs() >= PEN * 1.2 || gap.abs() <= PEN * 0.2;
  let [mut crossing, mut apart, mut covered, mut short] = [0; 4];
  for seed in 0..200 {
    let geometry = hand_drawn(0.0, seed);
    let lap = perimeter(scale(subtract(geometry.b, geometry.a), 0.5), 0.0);
    let overshoot = geometry.c[1] - lap;
    if overshoot < 0.0 {
      let ends = length(subtract(
        pen_at(&geometry, 0.0),
        pen_at(&geometry, geometry.c[1]),
      ));
      assert!(ends >= PEN * 1.2, "{seed}'s ends are {ends} apart");
      short += 1;
      continue;
    }
    let gap = |at: f32| wander(&geometry, lap + at) - wander(&geometry, at);
    for (tip, at) in [("start", 0.0), ("end", overshoot)] {
      assert!(clean(gap(at)), "{seed}'s {tip}: {}", gap(at));
    }
    if gap(0.0).abs() <= PEN * 0.2 {
      covered += 1;
    } else if gap(0.0).signum() != gap(overshoot).signum() {
      crossing += 1;
    } else {
      apart += 1;
    }
  }
  assert!(
    [crossing, apart, covered, short]
      .iter()
      .all(|&count| count > 0),
    "{crossing} crossing, {apart} apart, {covered} covered, {short} short"
  );
}

/// A thin pen round a large box still strays far enough to read as drawn by
/// hand: the wander follows the shape's size, not only the pen's.
#[test]
fn a_thin_pen_still_wanders_visibly() {
  for seed in 0..16 {
    let geometry = prepare_shape(
      [0.0, 0.0],
      [800.0, 600.0],
      0.0,
      seed as f32 + 1.0,
      2.0,
      WHOLE,
    );
    let lap = perimeter(scale(subtract(geometry.b, geometry.a), 0.5), 0.0);
    let widest = (0..400)
      .map(|step| wander(&geometry, lap * step as f32 / 400.0).abs())
      .fold(0.0, f32::max);
    assert!(widest >= 8.0, "{seed}: {widest}");
  }
}

/// The chrome prepares in display points and the compositor in canvas
/// pixels; the same shape in either is the same stroke at a different size.
#[test]
fn a_shape_prepared_larger_is_the_same_stroke_larger() {
  let small = prepare_shape([10.0, 10.0], [110.0, 60.0], 20.0, 42.0, 6.0, WHOLE);
  let large = prepare_shape([30.0, 30.0], [330.0, 180.0], 20.0, 42.0, 18.0, WHOLE);
  for point in [
    [60.0, 10.0],
    [110.0, 35.0],
    [12.0, 14.0],
    [60.0, 35.0],
    [5.0, 62.0],
  ] {
    let near = shape_distance(point, &small) * 3.0;
    let far = shape_distance(scale(point, 3.0), &large);
    assert!((near - far).abs() < 1e-2, "{point:?}: {near} and {far}");
  }
}
