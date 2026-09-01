// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::{
  folds::{self, GlideAction, GlideDetectorOptions},
  regions::{self, GlideRegion},
  settling::{GlidePhase, RestGate},
  travel::TurnPointTracker,
};
use serde::Serialize;

/// The detector's complete observable state after one input or timer event.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlideDetection {
  pub became_ready: bool,
  pub changed: bool,
  pub pending: Option<GlideAction>,
  pub phase: GlidePhase,
  pub region: Option<GlideRegion>,
}
/// One normalized motion sample supplied by a platform input adapter.
#[derive(Clone, Copy, Debug)]
pub struct GlideSample {
  pub delta_x: f64,
  pub delta_y: f64,
  pub thirds: bool,
  pub timestamp: f64,
}
/// Platform-neutral gesture state machine.
///
/// Native adapters normalize their input into [`GlideSample`]. This type owns
/// every threshold, transition, and pending action decision.
pub struct GlideDetector {
  options: GlideDetectorOptions,
  horizontal: TurnPointTracker,
  vertical: TurnPointTracker,
  rest: RestGate,
  pending: Option<GlideAction>,
  region: Option<GlideRegion>,
  porous: bool,
  thirds: bool,
}
impl GlideDetector {
  pub fn new(options: GlideDetectorOptions) -> Self {
    Self {
      horizontal: TurnPointTracker::new(options.reversal_hysteresis),
      vertical: TurnPointTracker::new(options.reversal_hysteresis),
      rest: RestGate::new(options.motion_noise_floor, options.rest_ms),
      options,
      pending: None,
      region: None,
      porous: false,
      thirds: false,
    }
  }
  fn detection(
    &self,
    previous_region: Option<GlideRegion>,
    previous_pending: Option<GlideAction>,
    became_ready: bool,
  ) -> GlideDetection {
    GlideDetection {
      became_ready,
      changed: previous_region != self.region || previous_pending != self.pending,
      pending: self.pending,
      phase: self.rest.phase(),
      region: self.region,
    }
  }
  fn rebase(&mut self) {
    self.horizontal.rebase();
    self.vertical.rebase()
  }
  fn escape_pending(&mut self) -> bool {
    if let Some(region) = self.region {
      let step = self.horizontal.step(self.options.horizontal_threshold);
      if step == 0 {
        return false;
      }
      self.region = Some(regions::step_columns(region, step));
    } else {
      let Some(region) = folds::fold_horizontal(
        self.horizontal.travel(),
        self.vertical.travel(),
        self.options,
        self.thirds,
      ) else {
        return false;
      };
      self.region = Some(region);
    }
    self.pending = None;
    true
  }
  fn transition(&mut self) -> bool {
    if self.pending.is_some() {
      let step = self.vertical.step(self.options.vertical_threshold);
      if step == 0 {
        return self.escape_pending();
      }
      if step < 0 {
        self.pending = None;
      } else if self.region.is_none() {
        self.pending = None;
        self.region = Some(regions::bottom_row_region(if self.thirds { 3 } else { 2 }));
      }
      return true;
    }
    let fold = if let Some(region) = self.region {
      folds::step_ladder(
        region,
        self.horizontal.travel(),
        self.vertical.travel(),
        self.options,
      )
    } else {
      folds::detect_first_fold(
        self.horizontal.travel(),
        self.vertical.travel(),
        self.options,
        self.thirds,
      )
    };
    if let Some(fold) = fold {
      self.pending = fold.pending;
      self.porous = fold.porous;
      self.region = fold.region;
      true
    } else {
      false
    }
  }
  /// Applies one normalized adapter sample.
  pub fn update(&mut self, sample: GlideSample) -> GlideDetection {
    let GlideSample {
      delta_x,
      delta_y,
      thirds,
      timestamp,
    } = sample;
    let previous_region = self.region;
    let previous_pending = self.pending;
    let became_ready = self.rest.settle(timestamp);
    self.thirds = thirds;
    if let Some(region) = self.region {
      if thirds != (region.grid_cols == 3) {
        self.region = Some(regions::regrid_region(region, thirds));
        self.horizontal.rebase();
      }
    }
    self.horizontal.update(delta_x);
    self.vertical.update(delta_y);
    if self.rest.phase() == GlidePhase::Settling {
      self.rest.stir(timestamp, delta_x.abs() + delta_y.abs());
      self.horizontal.rebase();
      if !self.porous {
        self.vertical.rebase();
      } else {
        let step = self.vertical.step(self.options.vertical_threshold);
        if step != 0 {
          self.region = self.region.map(|region| regions::step_rows(region, step));
          self.porous = false;
          self.rebase();
          self.rest.hold(timestamp);
        }
      }
    } else if self.transition() {
      self.rebase();
      self.rest.hold(timestamp);
    }
    self.detection(previous_region, previous_pending, became_ready)
  }
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
    self.porous = false;
    self.horizontal.reset();
    self.vertical.reset();
    self.rest.reset();
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
    let became_ready = self.rest.settle(timestamp);
    if became_ready {
      self.porous = false;
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
}

impl Default for GlideDetector {
  fn default() -> Self {
    Self::new(GlideDetectorOptions::default())
  }
}
