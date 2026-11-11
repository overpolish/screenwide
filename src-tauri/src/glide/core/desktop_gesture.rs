// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use super::{desktops::Direction, stroke::StrokeGate, travel::TurnPointTracker};

/// How far a push travels sideways for each desktop it steps along.
const STEP: f64 = 36.0;
/// How far a push comes back to turn and step back the other way.
const TURN: f64 = 10.0;
const MOTION_NOISE_FLOOR: f64 = 2.0;
/// How long the selection stays on a desktop before a push still going the
/// same way moves it on, so carrying on reads as meaning to go further
/// rather than as overshoot.
const DWELL_MS: f64 = 300.0;

/// Steps the selection along the row of desktops. A push that keeps going
/// moves it on once it has stayed [`DWELL_MS`] on a desktop, and a turn
/// steps back at once. The desktop is only taken on release. OS adapters
/// normalize pointer, finger and wheel input before it reaches this policy.
pub(crate) struct DesktopGesture {
  travel: TurnPointTracker,
  /// Only says when the hand has stopped after a step, for the picker to
  /// show the selection settled: travel never waits on it.
  stroke: StrokeGate,
  /// When the selection last stepped, and which way.
  stepped: Option<(f64, i32)>,
}
impl Default for DesktopGesture {
  fn default() -> Self {
    Self {
      travel: TurnPointTracker::new(TURN),
      // A turn does not end the push here: travel back steps back by itself.
      stroke: StrokeGate::new(MOTION_NOISE_FLOOR, f64::INFINITY),
      stepped: None,
    }
  }
}
impl DesktopGesture {
  pub fn update(&mut self, x: f64, y: f64, time: f64) -> Option<Direction> {
    self.settle(time);
    self.stroke.stir(time, x, 0.0);
    if y.abs() > x.abs() {
      self.travel.rebase();
      return None;
    }
    self.travel.update(x);
    match self.travel.step(STEP) {
      0 => None,
      direction => {
        let carrying_on = self
          .stepped
          .is_some_and(|(at, last)| last == direction && time - at < DWELL_MS);
        if carrying_on {
          // Kept until the dwell is up; a stop clears it meanwhile.
          return None;
        }
        self.stepped = Some((time, direction));
        self.stroke.hold(time, (f64::from(direction), 0.0));
        self.travel.reset();
        Some(if direction < 0 {
          Direction::Previous
        } else {
          Direction::Next
        })
      }
    }
  }
  /// Whether the hand stopped just now after a step, for [`PUSH_GAP_MS`]. A
  /// stop also starts the next push from nothing: travel left over from the
  /// last cannot finish a step for it, and a fresh flick steps at once,
  /// without the dwell.
  ///
  /// [`PUSH_GAP_MS`]: super::stroke::PUSH_GAP_MS
  pub fn settle(&mut self, time: f64) -> bool {
    let stopped = self.stroke.settle(time);
    if stopped {
      self.travel.reset();
      self.stepped = None;
    }
    stopped
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::glide::core::stroke::PUSH_GAP_MS;

  /// A push of `samples` sideways moves of `x`, 8 ms apart from `start`.
  fn push(gesture: &mut DesktopGesture, start: f64, x: f64, samples: u32) -> Vec<Direction> {
    (0..samples)
      .filter_map(|sample| gesture.update(x, 0.0, start + f64::from(sample) * 8.0))
      .collect()
  }

  #[test]
  fn carrying_on_stays_on_each_desktop_for_the_dwell() {
    let mut gesture = DesktopGesture::default();
    // 280 ms of steady push: the first desktop is not left yet.
    assert_eq!(push(&mut gesture, 0.0, 10.0, 36), [Direction::Next]);
  }

  #[test]
  fn a_push_that_keeps_going_moves_on_once_each_desktop_has_held() {
    let mut gesture = DesktopGesture::default();
    // Steps at 24 ms, then once 300 ms have passed on each desktop.
    assert_eq!(push(&mut gesture, 0.0, 10.0, 80), [Direction::Next; 3]);
  }

  #[test]
  fn a_second_flick_steps_again_once_the_first_has_stopped() {
    let mut gesture = DesktopGesture::default();
    assert_eq!(push(&mut gesture, 0.0, 10.0, 6), [Direction::Next]);
    // The first flick ended at 40 ms; stopped long enough, it is over.
    assert!(!gesture.settle(40.0 + PUSH_GAP_MS - 1.0));
    assert!(gesture.settle(40.0 + PUSH_GAP_MS));
    assert_eq!(push(&mut gesture, 80.0, 10.0, 6), [Direction::Next]);
  }

  #[test]
  fn turning_back_steps_back_without_stopping() {
    let mut gesture = DesktopGesture::default();
    assert_eq!(push(&mut gesture, 0.0, 10.0, 6), [Direction::Next]);
    assert_eq!(push(&mut gesture, 48.0, -10.0, 6), [Direction::Previous]);
  }

  #[test]
  fn jitter_does_not_keep_a_push_going_and_vertical_motion_does_not_step() {
    let mut gesture = DesktopGesture::default();
    assert!(gesture.update(10.0, 50.0, 0.0).is_none());
    assert_eq!(push(&mut gesture, 0.0, 40.0, 1), [Direction::Next]);
    for time in [8.0, 16.0, 24.0] {
      assert!(gesture.update(0.5, 0.5, time).is_none());
    }
    assert!(gesture.settle(PUSH_GAP_MS));
  }
}
