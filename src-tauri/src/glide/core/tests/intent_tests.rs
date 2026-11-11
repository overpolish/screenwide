// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use super::{region, stroke, Gesture};
use crate::glide::core::{
  GlideDetectorOptions, GlideEffects, GlidePhase, GlideRuntime, GlideSample,
};

fn ready(effects: GlideEffects) -> bool {
  effects.detection.phase == GlidePhase::Ready
}

#[test]
fn staggered_diagonals_choose_corners_in_all_four_directions() {
  for x in [-1.0, 1.0] {
    for y in [-1.0, 1.0] {
      let mut gesture = Gesture::with_options(GlideDetectorOptions::default());
      assert!(!gesture.move_by(stroke(x * 40.0, y * 24.0)).changed);
      gesture.advance(20.0);
      let corner = gesture.move_by(stroke(x * 22.0, y * 12.0));
      assert_eq!(
        corner.region,
        Some(region(2, u32::from(x > 0.0), 1, u32::from(y > 0.0), 1))
      );
    }
  }
}

#[test]
fn vertical_leading_diagonals_do_not_prematurely_fill_or_minimize() {
  for x in [-1.0, 1.0] {
    for y in [-1.0, 1.0] {
      let mut gesture = Gesture::with_options(GlideDetectorOptions::default());
      assert!(!gesture.move_by(stroke(x * 24.0, y * 40.0)).changed);
      gesture.advance(20.0);
      let corner = gesture.move_by(stroke(x * 12.0, y * 22.0));
      assert_eq!(
        corner.region,
        Some(region(2, u32::from(x > 0.0), 1, u32::from(y > 0.0), 1))
      );
      assert!(corner.pending.is_none());
    }
  }
}

#[test]
fn timer_commits_a_straight_opening_without_needing_more_motion() {
  let mut runtime = GlideRuntime::default();
  assert!(!runtime.update(sample(0.0, 45.0, 0.0)).reveal);
  assert!(runtime.settle(29.0).move_to.is_none());
  let half = runtime.settle(30.0);
  assert_eq!(half.move_to, Some(region(2, 1, 1, 0, 2)));
  assert!(half.reveal);
  assert!(!ready(half));
  assert!(!ready(runtime.settle(31.0)));
  assert!(ready(runtime.settle(32.0)));
}

#[test]
fn a_short_flick_commits_on_lift_without_waiting_for_the_timer() {
  let mut runtime = GlideRuntime::default();
  runtime.update(sample(0.0, -45.0, 0.0));
  let half = runtime.finish_opening(10.0);
  assert_eq!(half.move_to, Some(region(2, 0, 1, 0, 2)));
  assert!(runtime.finish_opening(11.0).move_to.is_none());
  let mut cancelled = GlideRuntime::default();
  cancelled.update(sample(0.0, 0.0, 45.0));
  assert!(!cancelled.should_minimize(true));
  // Cancellation skips finish_opening in the adapters.
  assert!(!cancelled.should_minimize(false));
}

#[test]
fn accumulated_sideways_drift_does_not_refine_but_a_slow_turn_can() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(45.0, 0.0));
  for _ in 0..30 {
    gesture.advance(10.0);
    assert!(!gesture.move_by(stroke(10.0, -2.0)).changed);
  }
  assert_eq!(gesture.detector.region(), Some(region(2, 1, 1, 0, 2)));
  // Even a slow deliberate turn can accumulate distance after the heading
  // changes; there is no minimum swipe speed imposed by the heading window.
  for _ in 0..30 {
    gesture.advance(10.0);
    gesture.move_by(stroke(0.0, 2.0));
  }
  assert_eq!(gesture.detector.region(), Some(region(2, 1, 1, 1, 1)));
}

#[test]
fn ending_the_push_closes_the_refinement_and_clears_discarded_travel() {
  let mut gesture = Gesture::new();
  gesture.move_by(stroke(45.0, 0.0));
  gesture.advance(10.0);
  gesture.move_by(stroke(0.0, -30.0));
  assert!(gesture.stop());
  assert!(!gesture.move_by(stroke(0.0, -6.0)).changed);
}

#[test]
fn refinement_is_available_for_halves_and_thirds_but_not_full_screen_or_minimize() {
  for thirds in [false, true] {
    for x in [-1.0, 1.0] {
      for y in [-1.0, 1.0] {
        let mut runtime = GlideRuntime::default();
        runtime.update(GlideSample {
          thirds,
          ..sample(0.0, x * 45.0, 0.0)
        });
        runtime.settle(30.0);
        let corner = runtime.update(GlideSample {
          thirds,
          ..sample(31.0, 0.0, y * 45.0)
        });
        assert_eq!(corner.move_to.unwrap().row_span, 1);
        // Back the way it came: the push ends, and nothing moves.
        let opposite = runtime.update(GlideSample {
          thirds,
          ..sample(40.0, 0.0, -y * 100.0)
        });
        assert!(opposite.move_to.is_none());
        assert!(ready(opposite));
      }
    }
  }
  for y in [-1.0, 1.0] {
    let mut runtime = GlideRuntime::default();
    runtime.update(sample(0.0, 0.0, y * 45.0));
    runtime.settle(30.0);
    let sideways = runtime.update(sample(31.0, 80.0, 0.0));
    assert!(sideways.move_to.is_none());
  }
}

fn sample(timestamp: f64, delta_x: f64, delta_y: f64) -> GlideSample {
  GlideSample {
    timestamp,
    delta_x,
    delta_y,
    thirds: false,
  }
}

#[test]
fn recorded_diagonals_and_turns_reach_the_intended_regions_without_cascading() {
  // Movement-only traces from the investigation; no window identities or positions.
  let gestures: Vec<Vec<[f64; 3]>> =
    serde_json::from_str(include_str!("fixtures/corner-movements.json")).unwrap();
  let expected = [
    region(2, 0, 1, 0, 1),
    region(2, 1, 1, 0, 1),
    region(2, 0, 2, 0, 2),
    region(2, 1, 1, 0, 1),
    region(2, 0, 1, 0, 1),
    region(2, 1, 1, 0, 1),
    region(2, 0, 1, 0, 1),
    region(2, 1, 1, 0, 1),
    region(2, 0, 1, 1, 1),
    region(2, 1, 1, 1, 1),
  ];
  for (index, (samples, expected)) in gestures.iter().zip(expected).enumerate() {
    let mut runtime = GlideRuntime::default();
    let mut timer = 0.0;
    let mut moves = Vec::new();
    for &[time, dx, dy] in samples {
      while timer < time {
        if let Some(region) = runtime.settle(timer).move_to {
          moves.push(region);
        }
        timer += 16.0;
      }
      if let Some(region) = runtime.update(sample(time, dx, dy)).move_to {
        moves.push(region);
      }
    }
    if let Some(region) = runtime.finish_opening(samples.last().unwrap()[0]).move_to {
      moves.push(region);
    }
    assert_eq!(
      moves.last(),
      Some(&expected),
      "gesture {}: {moves:?}",
      index + 1
    );
    assert!(
      moves.len() <= 2,
      "gesture {} cascaded: {moves:?}",
      index + 1
    );
  }
}
