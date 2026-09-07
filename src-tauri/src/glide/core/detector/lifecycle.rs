// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl GlideDetector {
  pub fn pending(&self) -> Option<GlideAction> {
    self.pending
  }
  pub fn phase(&self) -> GlidePhase {
    self.rest.phase()
  }
  pub fn region(&self) -> Option<GlideRegion> {
    self.region
  }
  pub fn options(&self) -> GlideDetectorOptions {
    self.options
  }
  pub fn reset(&mut self) -> GlideDetection {
    let changed = self.region.is_some() || self.pending.is_some();
    self.region = None;
    self.pending = None;
    self.opening.reset();
    self.refinement.arm(false);
    self.horizontal.reset();
    self.vertical.reset();
    self.rest.reset();
    self.last_motion = 0.0;
    GlideDetection {
      became_ready: false,
      changed,
      pending: None,
      phase: GlidePhase::Ready,
      region: None,
    }
  }
  pub fn settle(&mut self, timestamp: f64) -> GlideDetection {
    let previous_region = self.region;
    let previous_pending = self.pending;
    if self.opening.pending() && self.transition(timestamp, false).is_some() {
      self.rebase();
      self.rest.hold(self.last_motion);
    }
    let became_ready = self.rest.settle(timestamp);
    if became_ready {
      self.refinement.arm(false);
      self.rebase();
    }
    self.detection(previous_region, previous_pending, became_ready)
  }
  pub fn set_thirds(&mut self, thirds: bool) -> GlideDetection {
    let previous_region = self.region;
    let previous_pending = self.pending;
    self.thirds = thirds;
    if let Some(region) = self.region {
      if region.grid_cols != (if thirds { 3 } else { 2 }) {
        self.region = Some(regions::regrid_region(region, thirds));
        self.horizontal.rebase();
      }
    }
    self.detection(previous_region, previous_pending, false)
  }
  pub fn rest_remaining(&self, timestamp: f64) -> f64 {
    self.rest.remaining(timestamp)
  }
  /// A short flick can lift before the opening grace expires. Commit its last
  /// direction on release, while cancellation leaves it unapplied.
  pub fn finish_opening(&mut self, timestamp: f64) -> GlideDetection {
    let previous_region = self.region;
    let previous_pending = self.pending;
    if self.opening.pending() && self.transition(timestamp, true).is_some() {
      self.rebase();
      self.rest.hold(self.last_motion);
    }
    self.detection(previous_region, previous_pending, false)
  }
}
