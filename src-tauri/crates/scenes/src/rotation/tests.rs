// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const CENTRE: (f64, f64) = (800.0, 450.0);
const EXTENT: f64 = 1_600.0;

fn turned(x: f64, y: f64, z: f64) -> Homography {
  turned_homography(SceneRotation { x, y, z }, CENTRE, EXTENT)
}

fn close(a: (f64, f64), b: (f64, f64)) -> bool {
  (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6
}

#[test]
fn an_upright_screen_is_drawn_where_it_stands() {
  let h = turned(0.0, 0.0, 0.0);
  for point in [(0.0, 0.0), (1_234.5, 67.0), CENTRE] {
    assert!(close(apply(h, point).unwrap(), point));
  }
}

#[test]
fn the_middle_stays_put_however_the_screen_turns() {
  let h = turned(35.0, -50.0, 120.0);
  assert!(close(apply(h, CENTRE).unwrap(), CENTRE));
}

#[test]
fn a_positive_z_turn_runs_counter_clockwise_on_screen() {
  let h = turned(0.0, 0.0, 90.0);
  // The point to the right of the middle turns to above it, Y running down.
  let (x, y) = apply(h, (CENTRE.0 + 100.0, CENTRE.1)).unwrap();
  assert!((x - CENTRE.0).abs() < 1e-6);
  assert!((y - (CENTRE.1 - 100.0)).abs() < 1e-6);
}

#[test]
fn the_side_turned_toward_the_eye_looms_larger() {
  // A positive X turn tips the top toward the viewer.
  let h = turned(30.0, 0.0, 0.0);
  let top = apply(h, (CENTRE.0, CENTRE.1 - 200.0)).unwrap();
  let bottom = apply(h, (CENTRE.0, CENTRE.1 + 200.0)).unwrap();
  assert!(CENTRE.1 - top.1 > bottom.1 - CENTRE.1);
  // A positive Y turn swings the right side away.
  let h = turned(0.0, 30.0, 0.0);
  let right = apply(h, (CENTRE.0 + 200.0, CENTRE.1)).unwrap();
  let left = apply(h, (CENTRE.0 - 200.0, CENTRE.1)).unwrap();
  assert!(right.0 - CENTRE.0 < CENTRE.0 - left.0);
}

#[test]
fn the_inverse_finds_the_point_a_turned_pixel_shows() {
  let h = turned(25.0, -40.0, 10.0);
  let inverse = inverted(h).unwrap();
  for point in [(100.0, 80.0), (1_500.0, 820.0), (640.0, 700.0)] {
    let drawn = apply(h, point).unwrap();
    assert!(close(apply(inverse, drawn).unwrap(), point));
  }
}

#[test]
fn a_screen_seen_edge_on_has_no_inverse() {
  assert!(inverted(turned(90.0, 0.0, 0.0)).is_none());
  assert!(inverted(turned(0.0, -90.0, 30.0)).is_none());
}

#[test]
fn a_point_turned_behind_the_eye_is_not_drawn() {
  // Turned almost edge-on, the near end of a box many canvases long passes
  // the eye, two canvas lengths in front of it.
  let h = turned(0.0, -89.0, 0.0);
  assert!(apply(h, (CENTRE.0 + 4.0 * EXTENT, CENTRE.1)).is_none());
}
