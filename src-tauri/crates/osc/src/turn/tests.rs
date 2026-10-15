// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const RADIUS: f64 = MAX_RING_RADIUS;

fn turn(x: f64, y: f64, z: f64) -> SceneRotation {
  SceneRotation { x, y, z }
}

/// Ring `axis`'s point at angle `t`, in display points with Y down, and its
/// depth toward the viewer.
fn ring_point(rotation: SceneRotation, axis: usize, t: f64) -> [f64; 3] {
  let basis = ring_basis(rotation);
  let (u, v) = (basis[axis], basis[axis + 3]);
  let (sin, cos) = t.sin_cos();
  [0, 1, 2].map(|i| RADIUS * (cos * u[i] + sin * v[i]))
}

#[test]
fn an_upright_gizmo_shows_its_z_ring_round_the_middle() {
  // Upright, the X and Y rings are seen edge on, as lines through the
  // middle, so only the Z ring passes half way between them.
  let reach = RADIUS * std::f64::consts::FRAC_1_SQRT_2;
  let hit = ring_at(turn(0.0, 0.0, 0.0), RADIUS, (reach, -reach)).expect("the Z ring");
  assert_eq!(hit.axis, 2);
  assert!(ring_at(turn(0.0, 0.0, 0.0), RADIUS, (reach * 0.5, -reach * 0.5)).is_none());
}

#[test]
fn the_rings_keep_to_a_share_of_a_small_layer() {
  assert_eq!(ring_radius(2000.0), MAX_RING_RADIUS);
  assert_eq!(ring_radius(100.0), 30.0);
  assert_eq!(ring_radius(10.0), MIN_RING_RADIUS);
}

#[test]
fn only_the_near_half_of_a_ring_is_taken_hold_of() {
  // Tipped 60 degrees about X, the Z ring's top half comes toward the viewer
  // and its bottom half goes away.
  let tipped = turn(60.0, 0.0, 0.0);
  let quarter = std::f64::consts::FRAC_PI_2;
  let (top, bottom) = (
    ring_point(tipped, 2, quarter),
    ring_point(tipped, 2, -quarter),
  );
  assert!(top[1] < 0.0 && top[2] > 0.0);
  assert!(bottom[1] > 0.0 && bottom[2] < 0.0);
  assert_eq!(
    ring_at(tipped, RADIUS, (top[0], top[1])).map(|hit| hit.axis),
    Some(2)
  );
  assert_ne!(
    ring_at(tipped, RADIUS, (bottom[0], bottom[1])).map(|hit| hit.axis),
    Some(2)
  );
}

/// A press on an upright gizmo's Z ring a sixth of a turn round from its
/// right, clear of the X and Y rings, and the way along the ring from there,
/// in display points with Y down.
fn z_ring_press(rotation: SceneRotation) -> (RingDrag, (f64, f64)) {
  let (sin, cos) = std::f64::consts::FRAC_PI_3.sin_cos();
  let hit = ring_at(rotation, RADIUS, (RADIUS * cos, -RADIUS * sin)).expect("the Z ring");
  assert_eq!(hit.axis, 2);
  (
    RingDrag::begin(rotation, RADIUS, hit.axis, hit.angle),
    (-sin, -cos),
  )
}

#[test]
fn dragging_the_z_ring_along_itself_turns_the_layer_counter_clockwise() {
  let (mut drag, (x, y)) = z_ring_press(turn(0.0, 0.0, 0.0));
  // The arc of a twelfth of a turn.
  let travel = RADIUS * std::f64::consts::PI / 6.0;
  let turned = drag.turned((x * travel, y * travel), false);
  assert!((turned.z - 30.0).abs() < 1e-6, "{turned:?}");
  assert!(turned.x.abs() < 1e-9 && turned.y.abs() < 1e-9);
}

#[test]
fn shift_lands_the_dragged_axis_on_a_whole_step() {
  // From 7 degrees, 19 points along the ring reach about 29.7.
  let (mut drag, (x, y)) = z_ring_press(turn(0.0, 0.0, 7.0));
  let turned = drag.turned((x * 19.0, y * 19.0), true);
  assert!((turned.z - 30.0).abs() < 1e-6, "{turned:?}");
}

#[test]
fn a_sweep_past_a_quarter_turn_stays_continuous() {
  let mut drag = RingDrag {
    axis: 1,
    press: turn(0.0, 80.0, 0.0).radians(),
    last: turn(0.0, 80.0, 0.0).radians(),
    tangent: [1.0, 0.0],
    radius: RADIUS,
  };
  let mut previous = 80.0;
  for step in 1..=40 {
    let turned = drag.turned((-f64::from(step), 0.0), false);
    assert!(
      (turned.y - previous).abs() < 1.5,
      "{turned:?} jumped from {previous}"
    );
    assert!(turned.x.abs() < 1e-6 && turned.z.abs() < 1e-6, "{turned:?}");
    previous = turned.y;
  }
  assert!(previous > 100.0);
}

#[test]
fn a_turn_within_a_turned_frame_is_recovered_from_the_turn_seen() {
  let outer = turn(30.0, -20.0, 15.0);
  let inner = turn(25.0, 40.0, -70.0);
  let seen = composed(outer, inner);
  // The rings show neither turn alone.
  assert!((seen.x - inner.x).abs() > 1.0 || (seen.y - inner.y).abs() > 1.0);
  let back = within(outer, seen, inner);
  for (got, want) in [(back.x, inner.x), (back.y, inner.y), (back.z, inner.z)] {
    assert!((got - want).abs() < 1e-9, "{back:?} against {inner:?}");
  }
}

#[test]
fn within_an_upright_frame_the_turn_seen_is_the_turn_itself() {
  let inner = turn(10.0, -35.0, 400.0);
  let seen = composed(SceneRotation::UPRIGHT, inner);
  // Whole turns about Z are kept rather than wrapped.
  assert!((seen.z - 400.0).abs() < 1e-9, "{seen:?}");
  assert!((seen.x - 10.0).abs() < 1e-9 && (seen.y + 35.0).abs() < 1e-9);
}
