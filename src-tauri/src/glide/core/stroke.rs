// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The gate between steps that only take effect on release: picking a monitor
//! or a desktop. Nothing there changes as the hand moves, so nothing is
//! gained by waiting for it to rest; one push still makes at most one step,
//! so a long drag does not run through every monitor. A push ends when the
//! hand stops for a moment or turns back.

use super::settling::GlidePhase;

/// How long without motion ends a push: a flick lifted or stopped, two of
/// the platforms' 16 ms polls.
pub const PUSH_GAP_MS: f64 = 32.0;

#[derive(Debug, Clone, Copy)]
pub struct StrokeGate {
  /// The last step's heading, while the push that made it goes on.
  pushing: Option<(f64, f64)>,
  last_motion: f64,
  /// How far the push has gone along its heading since the step, and the
  /// furthest it got.
  along: f64,
  furthest: f64,
  motion_noise_floor: f64,
  turn: f64,
}

impl StrokeGate {
  /// `turn` is how far back along its heading a push must come to have
  /// turned.
  pub fn new(motion_noise_floor: f64, turn: f64) -> Self {
    Self {
      pushing: None,
      last_motion: 0.0,
      along: 0.0,
      furthest: 0.0,
      motion_noise_floor,
      turn,
    }
  }

  pub fn phase(&self) -> GlidePhase {
    if self.pushing.is_some() {
      GlidePhase::Settling
    } else {
      GlidePhase::Ready
    }
  }

  pub fn reset(&mut self) {
    self.pushing = None;
  }

  /// A step was made heading `(x, y)`: the push that made it has to end
  /// before the next step.
  pub fn hold(&mut self, t: f64, (x, y): (f64, f64)) {
    let length = x.hypot(y);
    if length == 0.0 {
      return;
    }
    self.pushing = Some((x / length, y / length));
    self.last_motion = t;
    self.along = 0.0;
    self.furthest = 0.0;
  }

  /// One sample of the push. Answers whether it turned back, which ends it.
  pub fn stir(&mut self, t: f64, dx: f64, dy: f64) -> bool {
    let Some((x, y)) = self.pushing else {
      return false;
    };
    if dx.abs() + dy.abs() >= self.motion_noise_floor {
      self.last_motion = t;
    }
    self.along += dx * x + dy * y;
    self.furthest = self.furthest.max(self.along);
    if self.furthest - self.along > self.turn {
      self.pushing = None;
      return true;
    }
    false
  }

  /// Ends the push once the hand has stopped for [`PUSH_GAP_MS`]. Answers
  /// whether it ended just now.
  pub fn settle(&mut self, t: f64) -> bool {
    if self.pushing.is_some() && t - self.last_motion >= PUSH_GAP_MS {
      self.pushing = None;
      return true;
    }
    false
  }
}
