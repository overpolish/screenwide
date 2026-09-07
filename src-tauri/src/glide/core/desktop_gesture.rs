// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{desktops::Direction, settling::RestGate, travel::TurnPointTracker};

/// One horizontal selection stroke per quiet period. OS adapters
/// normalize pointer/finger/wheel input before it reaches this policy.
pub(crate) struct DesktopGesture {
  travel: TurnPointTracker,
  rest: RestGate,
  moving: bool,
}
impl Default for DesktopGesture {
  fn default() -> Self {
    Self {
      travel: TurnPointTracker::new(10.0),
      rest: RestGate::new(2.0, 100.0),
      moving: false,
    }
  }
}
impl DesktopGesture {
  pub fn update(&mut self, x: f64, y: f64, time: f64) -> Option<Direction> {
    self.rest.stir(time, x.abs() + y.abs());
    if self.moving || self.rest.remaining(time) > 0.0 {
      return None;
    }
    if y.abs() > x.abs() {
      self.travel.rebase();
      return None;
    }
    self.travel.update(x);
    match self.travel.step(36.0) {
      0 => None,
      direction => {
        self.moving = true;
        self.rest.hold(time);
        self.travel.reset();
        Some(if direction < 0 {
          Direction::Previous
        } else {
          Direction::Next
        })
      }
    }
  }
  pub fn completed(&mut self, time: f64) {
    self.moving = false;
    self.rest.hold(time);
    self.travel.reset();
  }
  pub fn settle(&mut self, time: f64) -> bool {
    !self.moving && self.rest.settle(time)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn long_drag_cannot_chain_selection_steps_until_stillness() {
    let mut gesture = DesktopGesture::default();
    assert!(matches!(
      gesture.update(40.0, 0.0, 0.0),
      Some(Direction::Next)
    ));
    assert!(gesture.update(400.0, 0.0, 500.0).is_none());
    assert!(!gesture.settle(1000.0));
    gesture.completed(1000.0);
    assert!(gesture.update(40.0, 0.0, 1090.0).is_none());
    assert!(!gesture.settle(1150.0));
    assert!(gesture.settle(1190.0));
    assert!(matches!(
      gesture.update(-40.0, 0.0, 1200.0),
      Some(Direction::Previous)
    ));
  }
  #[test]
  fn jitter_does_not_postpone_readiness_and_vertical_motion_does_not_commit() {
    let mut gesture = DesktopGesture::default();
    assert!(gesture.update(10.0, 50.0, 0.0).is_none());
    gesture.completed(0.0);
    assert!(gesture.update(0.5, 0.5, 90.0).is_none());
    assert!(gesture.settle(100.0));
  }
}
