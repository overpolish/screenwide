// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use super::*;

impl GlideDetector {
  /// One sample while picking a monitor. A step is ready again as soon as
  /// the push that made it ends, by the hand stopping or turning back, as
  /// when arranging a window.
  pub(super) fn update_monitor(&mut self, sample: GlideSample) -> GlideDetection {
    let GlideSample {
      delta_x,
      delta_y,
      thirds,
      timestamp,
    } = sample;
    self.monitor_step = None;
    self.thirds = thirds;
    let stopped = self.stroke.settle(timestamp);
    if self.stroke.phase() == GlidePhase::Settling {
      // Travel within the push that made the last step buys no other; a
      // turn back ends the push, and travel from the turn counts.
      self.stroke.stir(timestamp, delta_x, delta_y);
      self.rebase();
      return self.detection(self.region, self.pending);
    }
    if stopped {
      self.rebase();
    }
    self.horizontal.update(delta_x);
    self.vertical.update(delta_y);
    if self.transition(timestamp, false).is_some() {
      self.rebase();
      if let Some((x, y)) = self.monitor_step {
        self.stroke.hold(timestamp, (f64::from(x), f64::from(y)));
      }
    }
    self.detection(self.region, self.pending)
  }
}
