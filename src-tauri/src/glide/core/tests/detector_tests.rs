// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{region, stroke, Gesture, Stroke};
use crate::glide::core::{GlideAction, GlideDetectorOptions, GlidePhase};

#[test]
fn opening_folds_match_halves_and_thirds_policy() {
  for (delta_x, thirds, expected) in [
    (-45.0, false, region(2, 0, 1, 0, 2)),
    (45.0, false, region(2, 1, 1, 0, 2)),
    (-45.0, true, region(3, 0, 1, 0, 2)),
    (45.0, true, region(3, 2, 1, 0, 2)),
  ] {
    let mut gesture = Gesture::new();
    let result = gesture.move_by(Stroke {
      delta_x,
      thirds,
      ..Stroke::default()
    });
    assert_eq!(result.region, Some(expected));
  }

  let mut halves = Gesture::new();
  assert_eq!(
    halves.move_by(stroke(0.0, -45.0)).region,
    Some(region(2, 0, 2, 0, 2))
  );
  let mut thirds = Gesture::new();
  assert_eq!(
    thirds
      .move_by(Stroke {
        delta_y: -45.0,
        thirds: true,
        ..Stroke::default()
      })
      .region,
    Some(region(3, 1, 1, 0, 2))
  );
}

#[test]
fn first_fold_thresholds_and_dominance_are_inclusive() {
  let mut gesture = Gesture::new();
  assert!(!gesture.move_by(stroke(35.0, 2.0)).changed);
  assert_eq!(
    gesture.move_by(stroke(1.0, 0.0)).region,
    Some(region(2, 1, 1, 0, 2))
  );

  let mut down_wins = Gesture::new();
  let result = down_wins.move_by(stroke(45.0, 100.0));
  assert_eq!(result.pending, Some(GlideAction::Minimize));
  assert_eq!(result.region, None);
}

#[test]
fn diagonal_cone_lands_directly_on_corners() {
  for (delta_x, delta_y, expected) in [
    (45.0, -45.0, region(2, 1, 1, 0, 1)),
    (-45.0, -45.0, region(2, 0, 1, 0, 1)),
    (45.0, 45.0, region(2, 1, 1, 1, 1)),
    (-45.0, 45.0, region(2, 0, 1, 1, 1)),
    (100.0, -60.0, region(2, 1, 1, 0, 1)),
  ] {
    let mut gesture = Gesture::new();
    let result = gesture.move_by(stroke(delta_x, delta_y));
    assert_eq!(result.region, Some(expected));
    assert_eq!(result.pending, None);
  }

  let mut outside_cone = Gesture::new();
  assert_eq!(
    outside_cone.move_by(stroke(100.0, -44.0)).region,
    Some(region(2, 1, 1, 0, 2))
  );
}

#[test]
fn reversal_is_measured_from_the_turn_point() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(45.0, 0.0));
  gesture.rest();
  gesture.move_by(stroke(30.0, 0.0));
  assert!(!gesture.move_by(stroke(-12.0, 0.0)).changed);
  assert!(!gesture.move_by(stroke(-12.0, 0.0)).changed);
  assert_eq!(
    gesture.move_by(stroke(-12.0, 0.0)).region,
    Some(region(2, 0, 1, 0, 2))
  );
}

#[test]
fn direction_changes_fold_without_waiting_for_rest() {
  let mut gesture = Gesture::new();
  assert_eq!(
    gesture.move_by(stroke(45.0, -45.0)).region,
    Some(region(2, 1, 1, 0, 1))
  );

  // Down reverses the diagonal's vertical component, then left changes axis
  // from that row fold. Neither transition needs a timer boundary.
  assert_eq!(
    gesture.move_by(stroke(0.0, 45.0)).region,
    Some(region(2, 1, 1, 0, 2))
  );
  assert_eq!(
    gesture.move_by(stroke(-36.0, 0.0)).region,
    Some(region(2, 0, 1, 0, 2))
  );
}

#[test]
fn orthogonal_change_uses_the_normal_threshold() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(0.0, -45.0));

  assert!(!gesture.move_by(stroke(35.0, 0.0)).changed);
  assert_eq!(
    gesture.move_by(stroke(1.0, 0.0)).region,
    Some(region(2, 1, 1, 0, 2))
  );
}

#[test]
fn decisive_reversal_keeps_the_normal_threshold() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(45.0, 0.0));

  assert_eq!(
    gesture.move_by(stroke(-36.0, 0.0)).region,
    Some(region(2, 0, 1, 0, 2))
  );
}

#[test]
fn same_direction_still_needs_a_quiet_beat() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(45.0, -45.0));
  gesture.move_by(stroke(0.0, 45.0));

  // Continuing down is one stroke, so it cannot cascade to the bottom row.
  assert!(!gesture.move_by(stroke(0.0, 80.0)).changed);
  assert_eq!(gesture.detector.region(), Some(region(2, 1, 1, 0, 2)));

  assert_eq!(
    gesture.flick(stroke(0.0, 45.0)).region,
    Some(region(2, 1, 1, 1, 1))
  );
  assert_eq!(
    gesture.flick(stroke(0.0, 45.0)).pending,
    Some(GlideAction::Minimize)
  );
}

#[test]
fn settling_discards_motion_until_the_hand_is_quiet() {
  let options = GlideDetectorOptions {
    rest_ms: 120.0,
    ..GlideDetectorOptions::default()
  };
  let mut gesture = Gesture::with_options(options);
  gesture.move_by(stroke(0.0, 45.0));

  for _ in 0..4 {
    gesture.advance(100.0);
    assert_eq!(
      gesture.move_by(stroke(0.0, 60.0)).phase,
      GlidePhase::Settling
    );
  }
  assert_eq!(gesture.detector.pending(), Some(GlideAction::Minimize));
  gesture.advance(120.0);
  let result = gesture.move_by(stroke(0.0, 60.0));
  assert!(result.became_ready);
  assert_eq!(result.region, Some(region(2, 0, 2, 1, 1)));
}

#[test]
fn sub_noise_jitter_counts_as_rest() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(0.0, 45.0));
  assert_eq!(gesture.detector.phase(), GlidePhase::Settling);
  gesture.advance(40.0);
  assert_eq!(
    gesture.move_by(stroke(1.0, 0.0)).phase,
    GlidePhase::Settling
  );
  assert_eq!(gesture.detector.rest_remaining(40.0), 20.0);
  gesture.advance(20.0);
  assert!(gesture.settle());
}

#[test]
fn readiness_is_reported_once_whether_timer_or_sample_wins() {
  let mut timer = Gesture::new();
  timer.move_by(stroke(45.0, 0.0));
  timer.advance(60.0);
  assert!(timer.settle());
  assert!(!timer.settle());
  assert!(!timer.move_by(stroke(-50.0, 0.0)).became_ready);

  let mut sample = Gesture::new();
  sample.move_by(stroke(45.0, 0.0));
  sample.advance(60.0);
  assert!(sample.move_by(stroke(-50.0, 0.0)).became_ready);
  assert!(!sample.settle());
}

#[test]
fn opening_sideways_settle_accepts_each_new_direction_once() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(50.0, 0.0));
  let corner = gesture.move_by(stroke(0.0, -50.0));
  assert!(corner.changed);
  assert_eq!(corner.region, Some(region(2, 1, 1, 0, 1)));

  assert!(!gesture.move_by(stroke(0.0, -50.0)).changed);
  assert_eq!(
    gesture.move_by(stroke(-60.0, 0.0)).region,
    Some(region(2, 0, 2, 0, 1))
  );
}

#[test]
fn column_step_settle_accepts_each_new_direction_once() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(-50.0, 0.0));
  gesture.rest();
  let stepped = gesture.move_by(stroke(50.0, 0.0));
  assert_eq!(stepped.region, Some(region(2, 1, 1, 0, 2)));

  let corner = gesture.move_by(stroke(0.0, -50.0));
  assert!(corner.changed);
  assert_eq!(corner.region, Some(region(2, 1, 1, 0, 1)));

  assert!(!gesture.move_by(stroke(0.0, -50.0)).changed);
  assert_eq!(
    gesture.move_by(stroke(-60.0, 0.0)).region,
    Some(region(2, 0, 2, 0, 1))
  );
}

#[test]
fn porous_conversion_restarts_rest_and_emits_one_ready_tick() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(50.0, 0.0));
  gesture.advance(50.0);
  assert!(!gesture.move_by(stroke(0.0, -50.0)).became_ready);
  gesture.advance(50.0);
  assert!(!gesture.settle());
  gesture.advance(10.0);
  assert!(gesture.settle());
  assert!(!gesture.settle());
}

#[test]
fn rest_closes_porosity_before_the_next_flick() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(50.0, 0.0));
  gesture.rest();
  assert!(!gesture.move_by(stroke(0.0, -20.0)).changed);
  assert_eq!(
    gesture.move_by(stroke(0.0, -30.0)).region,
    Some(region(2, 1, 1, 0, 1))
  );
}
