// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// A three-part eased transition: where it came from, where it is going and
/// when it started (`animation_amount`, `+ruler.m:180-197`).
#[derive(Clone, Copy, Debug)]
pub(super) struct Animation {
  from: f64,
  pub(super) target: bool,
  started: Instant,
}

impl Default for Animation {
  fn default() -> Self {
    Self {
      from: 0.0,
      target: false,
      started: settled(),
    }
  }
}

impl Animation {
  pub(super) fn amount(&self, now: Instant) -> f64 {
    let elapsed = now.saturating_duration_since(self.started).as_secs_f64();
    let progress = (elapsed / ANIMATION_DURATION.as_secs_f64()).clamp(0.0, 1.0);
    let eased = ease(progress);
    let target = if self.target { 1.0 } else { 0.0 };
    self.from + (target - self.from) * eased
  }

  pub(super) fn running(&self, now: Instant) -> bool {
    now.saturating_duration_since(self.started) < ANIMATION_DURATION
  }

  /// `restart` replays from zero even when the target is unchanged, which is
  /// how a tolerance notice re-triggers for a new mode (`set_tolerance_visible`).
  pub(super) fn set(&mut self, target: bool, restart: bool, now: Instant) {
    if !restart && self.target == target {
      return;
    }
    self.from = if restart { 0.0 } else { self.amount(now) };
    self.target = target;
    self.started = now;
  }
}

pub(super) fn ease(progress: f64) -> f64 {
  1.0 - (1.0 - progress).powi(3)
}
