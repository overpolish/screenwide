// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{region, stroke, Gesture, Stroke};
use crate::glide::core::{GlideAction, GlideDetectorOptions, GlidePhase};

#[test]
fn row_release_is_cheaper_than_the_next_row_step() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(45.0, 0.0));
  gesture.flick(stroke(0.0, -45.0));
  assert_eq!(
    gesture.flick(stroke(0.0, 20.0)).region,
    Some(region(2, 1, 1, 0, 2))
  );
  assert_eq!(
    gesture.flick(stroke(0.0, 43.0)).region,
    Some(region(2, 1, 1, 0, 2))
  );
  assert_eq!(
    gesture.move_by(stroke(0.0, 1.0)).region,
    Some(region(2, 1, 1, 1, 1))
  );
}

#[test]
fn thirds_walk_the_full_ladder_and_clamp() {
  let mut gesture = Gesture::new();
  gesture.move_by(Stroke {
    delta_x: 45.0,
    thirds: true,
    ..Stroke::default()
  });
  let expected = [
    region(3, 1, 2, 0, 2),
    region(3, 1, 1, 0, 2),
    region(3, 0, 2, 0, 2),
    region(3, 0, 1, 0, 2),
    region(3, 0, 1, 0, 2),
  ];
  for expected_region in expected {
    let result = gesture.flick(Stroke {
      delta_x: -50.0,
      thirds: true,
      ..Stroke::default()
    });
    assert_eq!(result.region, Some(expected_region));
  }
}

#[test]
fn regridding_preserves_side_and_rows() {
  let mut gesture = Gesture::new();
  gesture.move_by(Stroke {
    delta_x: 45.0,
    thirds: true,
    ..Stroke::default()
  });
  gesture.flick(Stroke {
    delta_y: -44.0,
    thirds: true,
    ..Stroke::default()
  });
  let result = gesture.detector.set_thirds(false);
  assert!(result.changed);
  assert_eq!(result.region, Some(region(2, 1, 1, 0, 1)));
  assert!(!result.became_ready);
}

#[test]
fn minimize_arms_disarms_and_converts_to_bottom_row() {
  let mut disarm = Gesture::new();
  assert_eq!(
    disarm.move_by(stroke(0.0, 45.0)).pending,
    Some(GlideAction::Minimize)
  );
  let result = disarm.flick(stroke(0.0, -50.0));
  assert_eq!(result.pending, None);
  assert_eq!(result.region, None);

  let mut convert = Gesture::new();
  convert.move_by(stroke(0.0, 45.0));
  let result = convert.flick(stroke(0.0, 50.0));
  assert_eq!(result.pending, None);
  assert_eq!(result.region, Some(region(2, 0, 2, 1, 1)));
}

#[test]
fn bottom_edge_rearms_minimize_over_the_retained_region() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(-45.0, 0.0));
  gesture.flick(stroke(0.0, 50.0));
  let armed = gesture.flick(stroke(0.0, 50.0));
  assert_eq!(armed.pending, Some(GlideAction::Minimize));
  assert_eq!(armed.region, Some(region(2, 0, 1, 1, 1)));

  let disarmed = gesture.flick(stroke(0.0, -50.0));
  assert_eq!(disarmed.pending, None);
  assert_eq!(disarmed.region, Some(region(2, 0, 1, 1, 1)));
}

#[test]
fn sideways_escape_drops_the_arm_and_steps_the_retained_region() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(-45.0, 0.0));
  gesture.flick(stroke(0.0, 50.0));
  gesture.flick(stroke(0.0, 50.0));
  let escaped = gesture.flick(stroke(60.0, 0.0));
  assert_eq!(escaped.pending, None);
  assert_eq!(escaped.region, Some(region(2, 0, 2, 1, 1)));
}

#[test]
fn pending_escape_never_reuses_vertical_first_fold_policy() {
  let options = GlideDetectorOptions {
    vertical_fill_threshold: 20.0,
    vertical_threshold: 100.0,
    ..GlideDetectorOptions::default()
  };
  let mut gesture = Gesture::with_options(options);
  gesture.move_by(stroke(0.0, 25.0));
  let unchanged = gesture.flick(stroke(0.0, 50.0));
  assert!(!unchanged.changed);
  assert_eq!(unchanged.pending, Some(GlideAction::Minimize));
}

#[test]
fn repeated_sideways_steps_wipe_vertical_drift() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(45.0, 0.0));
  for _ in 0..6 {
    gesture.flick(stroke(-50.0, -15.0));
  }
  assert_eq!(gesture.detector.region(), Some(region(2, 0, 1, 0, 2)));
  assert!(!gesture.flick(stroke(0.0, -20.0)).changed);
  assert_eq!(
    gesture.move_by(stroke(0.0, -30.0)).region,
    Some(region(2, 0, 1, 0, 1))
  );
}

#[test]
fn reset_clears_pending_region_and_porosity() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(50.0, 0.0));
  let reset = gesture.detector.reset();
  assert!(reset.changed);
  assert_eq!(reset.phase, GlidePhase::Ready);
  assert_eq!(gesture.detector.pending(), None);
  assert_eq!(gesture.detector.region(), None);
  assert_eq!(
    gesture.move_by(stroke(0.0, -50.0)).region,
    Some(region(2, 0, 2, 0, 2))
  );
}
