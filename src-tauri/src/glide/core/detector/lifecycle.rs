// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use super::*;

impl GlideDetector {
  pub fn begin_monitor_navigation(&mut self) {
    self.reset();
    self.monitor_navigation = true;
  }
  pub fn monitor_navigation_active(&self) -> bool {
    self.monitor_navigation
  }
  pub fn monitor_step(&self) -> Option<(i8, i8)> {
    self.monitor_step
  }
  pub fn pending(&self) -> Option<GlideAction> {
    self.pending
  }
  pub fn phase(&self) -> GlidePhase {
    self.stroke.phase()
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
    self.monitor_navigation = false;
    self.monitor_step = None;
    self.opening.reset();
    self.refinement.arm(false);
    self.horizontal.reset();
    self.vertical.reset();
    self.stroke.reset();
    self.last_motion = 0.0;
    GlideDetection {
      changed,
      pending: None,
      phase: GlidePhase::Ready,
      region: None,
    }
  }
  pub fn settle(&mut self, timestamp: f64) -> GlideDetection {
    self.monitor_step = None;
    let previous_region = self.region;
    let previous_pending = self.pending;
    if self.opening.pending() {
      self.open(timestamp, false);
    }
    if self.stroke.settle(timestamp) {
      self.push_ended();
    }
    self.detection(previous_region, previous_pending)
  }
  pub fn set_thirds(&mut self, thirds: bool) -> GlideDetection {
    self.monitor_step = None;
    let previous_region = self.region;
    let previous_pending = self.pending;
    self.thirds = thirds;
    if let Some(region) = self.region {
      if region.grid_cols != (if thirds { 3 } else { 2 }) {
        self.region = Some(regions::regrid_region(region, thirds));
        self.horizontal.rebase();
      }
    }
    self.detection(previous_region, previous_pending)
  }
  /// A short flick can lift before the opening grace expires. Commit its last
  /// direction on release, while cancellation leaves it unapplied.
  pub fn finish_opening(&mut self, timestamp: f64) -> GlideDetection {
    self.monitor_step = None;
    let previous_region = self.region;
    let previous_pending = self.pending;
    if self.opening.pending() {
      self.open(timestamp, true);
    }
    self.detection(previous_region, previous_pending)
  }

  /// Takes the opening fold held back by its grace, as a push of its own.
  fn open(&mut self, timestamp: f64, force: bool) {
    let heading = self.heading();
    if self.transition(timestamp, force).is_some() {
      self.rebase();
      self.stroke.hold(self.last_motion, heading);
    }
  }
}
