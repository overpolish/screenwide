// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Identifies the one-finger contact episode behind trackpad pointer motion.

const MOTION_THRESHOLD: f32 = 0.001;

#[derive(Default)]
pub(super) struct PointerEpisode {
  active: bool,
  previous: Option<(f32, f32)>,
}

impl PointerEpisode {
  pub fn update(&mut self, count: usize, centroid: Option<(f32, f32)>) {
    if count != 1 {
      self.active = false;
      self.previous = None;
      return;
    }
    if let (Some(previous), Some(current)) = (self.previous, centroid) {
      let travel = (current.0 - previous.0).abs() + (current.1 - previous.1).abs();
      self.active |= travel >= MOTION_THRESHOLD;
    }
    self.previous = centroid;
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
    episode.update(1, Some((0.5, 0.5)));
    assert!(!episode.active());
    episode.update(1, Some((0.502, 0.5)));
    assert!(episode.active());
    episode.update(1, Some((0.502, 0.5)));
    assert!(episode.active());
    episode.update(0, None);
    assert!(!episode.active());
  }
}
