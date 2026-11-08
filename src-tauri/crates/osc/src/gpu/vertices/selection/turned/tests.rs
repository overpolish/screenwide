// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_scenes::rotation::{apply, turned_homography};
use screenwide_scenes::SceneRotation;

/// Carries `point` on the upright picture through a strong turn.
fn turned(point: (f64, f64)) -> Point {
  let turn = turned_homography(
    SceneRotation {
      x: -50.0,
      y: -5.0,
      z: 30.0,
    },
    (570.0, 377.0),
    760.0,
  );
  let (x, y) = apply(turn, point).expect("the point stays in front of the eye");
  Point { x, y }
}

#[test]
fn a_turned_frame_draws_its_side_pills_where_the_turn_carries_the_sides_middles() {
  // A tall box, as a redaction stretched down a turned screenshot is.
  let (left, top, right, bottom) = (436.0, 127.0, 755.0, 676.0);
  let corners = [(left, top), (right, top), (right, bottom), (left, bottom)].map(turned);
  let expected = [
    ((left + right) / 2.0, top),
    (right, (top + bottom) / 2.0),
    ((left + right) / 2.0, bottom),
    (left, (top + bottom) / 2.0),
  ]
  .map(turned);

  let mut furthest_halfway: f64 = 0.0;
  for (side, (drawn, middle)) in side_middles(corners).iter().zip(expected).enumerate() {
    let halfway = Point {
      x: (corners[side].x + corners[(side + 1) % 4].x) / 2.0,
      y: (corners[side].y + corners[(side + 1) % 4].y) / 2.0,
    };
    let off = |point: Point| (point.x - middle.x).hypot(point.y - middle.y);
    furthest_halfway = furthest_halfway.max(off(halfway));
    assert!(
      off(*drawn) < 1e-6,
      "side {side} is drawn {} off",
      off(*drawn)
    );
  }
  // Halfway between the drawn corners of a long side is well off its true
  // middle, further than a pill can be pressed from.
  assert!(furthest_halfway > 8.0, "halfway is {furthest_halfway} off");
}

#[test]
fn a_frame_seen_flat_draws_its_side_pills_halfway() {
  let corners =
    [(10.0, 20.0), (210.0, 60.0), (190.0, 160.0), (-10.0, 120.0)].map(|(x, y)| Point { x, y });
  let middles = side_middles(corners);
  for side in 0..4 {
    let (from, to) = (corners[side], corners[(side + 1) % 4]);
    assert!((middles[side].x - (from.x + to.x) / 2.0).abs() < 1e-9);
    assert!((middles[side].y - (from.y + to.y) / 2.0).abs() < 1e-9);
  }
}
