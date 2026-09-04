// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Identifies one-finger native trackpad pointer episodes from HID contacts.

const MOTION_THRESHOLD: f64 = 1.0;

#[derive(Default)]
pub(super) struct PointerEpisode {
  active: bool,
  previous: Option<(f64, f64)>,
}

impl PointerEpisode {
  pub fn update(&mut self, count: usize, point: Option<(f64, f64)>) {
    if count != 1 {
      self.active = false;
      self.previous = None;
      return;
    }
    if let (Some(previous), Some(current)) = (self.previous, point) {
      let travel = (current.0 - previous.0).abs() + (current.1 - previous.1).abs();
      self.active |= travel >= MOTION_THRESHOLD;
    }
    self.previous = point;
  }

  pub const fn active(&self) -> bool {
    self.active
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn movement_latches_until_the_one_finger_episode_ends() {
    let mut episode = PointerEpisode::default();
    episode.update(1, Some((500.0, 500.0)));
    assert!(!episode.active());
    episode.update(1, Some((502.0, 500.0)));
    assert!(episode.active());
    episode.update(1, Some((502.0, 500.0)));
    assert!(episode.active());
    episode.update(0, None);
    assert!(!episode.active());
  }
}
