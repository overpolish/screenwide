// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{FacePath, FaceSighting};

const ASPECT: f64 = 16.0 / 9.0;

/// The face seen five times a second over `seconds`, centred where `at` says.
fn seen(seconds: f64, at: impl Fn(f64) -> Option<(f64, f64)>) -> Vec<FaceSighting> {
  (0..(seconds * 5.0) as usize)
    .filter_map(|index| {
      let ms = index as f64 * 200.0;
      at(ms).map(|(x, y)| FaceSighting {
        ms,
        x,
        y,
        size: 0.3,
      })
    })
    .collect()
}

#[test]
fn a_face_that_shifts_a_little_leaves_the_crop_still() {
  // Small movements either way of a point, as a person talking makes.
  let sightings = seen(20.0, |ms| Some((0.5 + 0.01 * (ms / 300.0).sin(), 0.45)));
  let path = FacePath::new(&sightings, ASPECT, 20_000.0).expect("a path");
  let first = path.at(1_000.0);
  for ms in (1_000..19_000).step_by(100) {
    let (x, y) = path.at(f64::from(ms));
    assert!(
      (x - first.0).abs() < 1e-9 && (y - first.1).abs() < 1e-9,
      "{ms}"
    );
  }
  assert!((first.0 - 0.5).abs() < 0.01 && (first.1 - 0.45).abs() < 1e-9);
}

/// Close enough that nobody could see the difference: a still stretch takes
/// in the first of a move, which tips its average a hair.
fn near(a: f64, b: f64) -> bool {
  (a - b).abs() < 0.005
}

#[test]
fn a_face_that_moves_is_followed_and_reached() {
  let sightings = seen(20.0, |ms| {
    Some((if ms < 10_000.0 { 0.4 } else { 0.6 }, 0.5))
  });
  let path = FacePath::new(&sightings, ASPECT, 20_000.0).expect("a path");
  assert!(near(path.at(5_000.0).0, 0.4));
  assert!(near(path.at(15_000.0).0, 0.6));
  // The crop glides across rather than jumping: halfway at the move.
  assert!((path.at(10_000.0).0 - 0.5).abs() < 0.05);
  let step = (path.at(10_033.0).0 - path.at(10_000.0).0).abs();
  assert!(step < 0.02, "{step}");
}

#[test]
fn where_the_face_is_lost_the_path_holds_where_it_was_last_seen() {
  // Seen on the left, lost for five seconds, then seen on the right.
  let sightings = seen(20.0, |ms| match ms {
    ms if ms < 8_000.0 => Some((0.3, 0.5)),
    ms if ms < 13_000.0 => None,
    _ => Some((0.7, 0.5)),
  });
  let path = FacePath::new(&sightings, ASPECT, 20_000.0).expect("a path");
  assert!(near(path.at(10_000.0).0, 0.3));
  assert!(near(path.at(18_000.0).0, 0.7));
  // Held at either end of the camera.
  assert!(near(path.at(-500.0).0, 0.3));
  assert!(near(path.at(60_000.0).0, 0.7));
}

#[test]
fn a_camera_with_no_face_has_no_path() {
  assert_eq!(FacePath::new(&[], ASPECT, 10_000.0), None);
}
