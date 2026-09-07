// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::{
  folds::{self, GlideAction, GlideDetectorOptions},
  intent::{CornerRefinement, OpeningGate},
  regions::{self, GlideRegion},
  settling::{GlidePhase, RestGate},
  travel::TurnPointTracker,
};
use serde::Serialize;

mod lifecycle;

/// The detector's complete observable state after one input or timer event.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlideDetection {
  /// A rest completed and this result still accepts the next step.
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
  opening: OpeningGate,
  refinement: CornerRefinement,
  last_motion: f64,
  options: GlideDetectorOptions,
  horizontal: TurnPointTracker,
  vertical: TurnPointTracker,
  rest: RestGate,
  pending: Option<GlideAction>,
  region: Option<GlideRegion>,
  thirds: bool,
}
impl GlideDetector {
  pub fn new(options: GlideDetectorOptions) -> Self {
    Self {
      horizontal: TurnPointTracker::new(options.reversal_hysteresis),
      vertical: TurnPointTracker::new(options.reversal_hysteresis),
      rest: RestGate::new(options.motion_noise_floor, options.rest_ms),
      opening: OpeningGate::default(),
      refinement: CornerRefinement::new(options.reversal_hysteresis),
      last_motion: 0.0,
      options,
      pending: None,
      region: None,
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
      became_ready: became_ready && self.rest.phase() == GlidePhase::Ready,
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
  fn escape_pending(&mut self) -> Option<()> {
    if let Some(region) = self.region {
      let step = self.horizontal.step(self.options.horizontal_threshold);
      if step == 0 {
        return None;
      }
      self.region = Some(regions::step_columns(region, step));
    } else {
      let region = folds::fold_horizontal(
        self.horizontal.travel(),
        self.vertical.travel(),
        self.options,
        self.thirds,
      )?;
      self.region = Some(region);
    }
    self.pending = None;
    Some(())
  }
  fn transition(&mut self, timestamp: f64, force: bool) -> Option<()> {
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
      return Some(());
    }
    let horizontal = self.horizontal.travel();
    let vertical = self.vertical.travel();
    let fold = if let Some(region) = self.region {
      folds::step_ladder(region, horizontal, vertical, self.options)
    } else {
      folds::detect_first_fold(horizontal, vertical, self.options, self.thirds)
    };
    if let Some(fold) = fold {
      if self.region.is_none()
        && !force
        && !self.opening.accepts(
          timestamp,
          fold.region.is_some_and(|r| r.row_span == 1),
          self.options.opening_grace_ms,
        )
      {
        return None;
      }
      self.opening.reset();
      self
        .refinement
        .arm(fold.can_refine && fold.region.is_some_and(|r| r.col_span < r.grid_cols));
      self.pending = fold.pending;
      self.region = fold.region;
      Some(())
    } else {
      None
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
    if became_ready {
      // Start fresh when the hand has rested, so discarded travel
      // cannot immediately trigger the next step.
      self.rebase();
      self.refinement.arm(false);
    }
    self.thirds = thirds;
    if delta_x.abs() + delta_y.abs() >= self.options.motion_noise_floor {
      self.last_motion = timestamp;
    }
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
      let step =
        self
          .refinement
          .update(delta_x, delta_y, timestamp, self.options.vertical_threshold);
      if step != 0 {
        self.region = self.region.map(|region| regions::step_rows(region, step));
        self.rest.hold(timestamp);
      }
      // Other travel cannot buy further steps before stillness.
      self.rebase();
    } else if self.transition(timestamp, false).is_some() {
      self.rebase();
      self.rest.hold(timestamp);
    }
    self.detection(previous_region, previous_pending, became_ready)
  }
}

impl Default for GlideDetector {
  fn default() -> Self {
    Self::new(GlideDetectorOptions::default())
  }
}
