// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use super::{region, stroke, Gesture, Stroke};
use crate::glide::core::{GlideAction, GlidePhase};

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
  gesture.stop();
  gesture.move_by(stroke(30.0, 0.0));
  assert!(!gesture.move_by(stroke(-12.0, 0.0)).changed);
  assert!(!gesture.move_by(stroke(-12.0, 0.0)).changed);
  assert_eq!(
    gesture.move_by(stroke(-12.0, 0.0)).region,
    Some(region(2, 0, 1, 0, 2))
  );
}

#[test]
fn a_push_makes_one_step_however_far_it_goes() {
  let mut gesture = Gesture::new();
  let first = gesture.move_by(stroke(45.0, 0.0));
  for _ in 0..20 {
    gesture.advance(16.0);
    let pushed = gesture.move_by(stroke(30.0, 0.0));
    assert!(!pushed.changed);
    assert_eq!(pushed.phase, GlidePhase::Settling);
  }
  assert_eq!(gesture.detector.region(), first.region);
}

#[test]
fn turning_back_ends_the_push_without_a_pause() {
  let mut gesture = Gesture::new();
  let first = gesture.move_by(stroke(45.0, 0.0));
  let turned = gesture.move_by(stroke(-50.0, 0.0));
  assert!(!turned.changed);
  assert_eq!(turned.phase, GlidePhase::Ready);
  assert_eq!(turned.region, first.region);
  assert!(gesture.move_by(stroke(-50.0, 0.0)).changed);
}

#[test]
fn sideways_travel_does_not_end_the_push_but_stopping_does() {
  let mut gesture = Gesture::new();
  let first = gesture.move_by(stroke(0.0, -45.0));
  let sideways = gesture.move_by(stroke(50.0, 0.0));
  assert!(!sideways.changed);
  assert_eq!(sideways.phase, GlidePhase::Settling);
  assert_eq!(sideways.region, first.region);
  gesture.advance(31.0);
  assert!(!gesture.settle());
  gesture.advance(1.0);
  assert!(gesture.settle());
  assert!(gesture.move_by(stroke(50.0, 0.0)).changed);
}

#[test]
fn discarded_motion_cannot_spend_the_next_ready_step() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(45.0, 0.0));
  gesture.move_by(stroke(100.0, -100.0));
  assert!(gesture.stop());
  assert!(!gesture.move_by(stroke(0.0, -35.0)).changed);
  assert_eq!(
    gesture.move_by(stroke(0.0, -1.0)).region,
    Some(region(2, 1, 1, 0, 1))
  );
}

#[test]
fn a_held_minimize_waits_for_the_push_to_end() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(0.0, 45.0));
  for _ in 0..10 {
    gesture.advance(16.0);
    assert_eq!(
      gesture.move_by(stroke(0.0, 60.0)).phase,
      GlidePhase::Settling
    );
  }
  assert_eq!(gesture.detector.pending(), Some(GlideAction::Minimize));
  assert!(gesture.stop());
  let result = gesture.move_by(stroke(0.0, 60.0));
  assert_eq!(result.region, Some(region(2, 0, 2, 1, 1)));
}

#[test]
fn sub_noise_jitter_does_not_keep_the_push_going() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(0.0, 45.0));
  assert_eq!(gesture.detector.phase(), GlidePhase::Settling);
  gesture.advance(20.0);
  assert_eq!(
    gesture.move_by(stroke(1.0, 0.0)).phase,
    GlidePhase::Settling
  );
  gesture.advance(12.0);
  assert!(gesture.settle());
}

#[test]
fn a_sample_after_the_pause_ends_the_push_without_the_timer() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(45.0, 0.0));
  gesture.advance(32.0);
  assert_eq!(gesture.move_by(stroke(-5.0, 0.0)).phase, GlidePhase::Ready);
}

#[test]
fn the_next_push_steps_at_once_and_is_held_in_turn() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(45.0, 0.0));
  gesture.advance(32.0);
  let next = gesture.move_by(stroke(-50.0, 0.0));
  assert!(next.changed);
  assert_eq!(next.phase, GlidePhase::Settling);
  assert!(gesture.stop());
}

#[test]
fn one_vertical_turn_refines_a_half_within_its_push() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(50.0, 0.0));
  gesture.advance(10.0);
  let curve = gesture.move_by(stroke(0.0, -50.0));
  assert_eq!(curve.region, Some(region(2, 1, 1, 0, 1)));
  assert_eq!(curve.phase, GlidePhase::Settling);
  gesture.advance(10.0);
  assert!(!gesture.move_by(stroke(0.0, -100.0)).changed);
  gesture.advance(31.0);
  assert!(!gesture.settle());
  gesture.advance(1.0);
  assert!(gesture.settle());
}

#[test]
fn a_half_ended_by_a_pause_steps_on_rather_than_refining() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(50.0, 0.0));
  assert!(gesture.stop());
  let next = gesture.move_by(stroke(0.0, -50.0));
  assert!(next.changed);
  assert_eq!(next.phase, GlidePhase::Settling);
}
