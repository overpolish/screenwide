// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared session orchestration around the gesture detector. Platform adapters
//! apply the returned effects without owning any transition policy.

use super::{
  GlideAction, GlideDetection, GlideDetector, GlideDetectorOptions, GlideRegion, GlideSample,
};

/// One detector result plus the native work it requires.
///
/// This is the extension point for platform adapters: input implementations
/// provide normalized samples and translate these effects into OS operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlideEffects {
  pub detection: GlideDetection,
  pub move_to: Option<GlideRegion>,
  pub ready: bool,
  pub reveal: bool,
}

pub struct GlideRuntime {
  detector: GlideDetector,
  moved: Option<GlideRegion>,
  reveal_requested: bool,
}

impl GlideRuntime {
  pub fn new(options: GlideDetectorOptions) -> Self {
    Self {
      detector: GlideDetector::new(options),
      moved: None,
      reveal_requested: false,
    }
  }

  pub fn update(&mut self, sample: GlideSample) -> GlideEffects {
    let detection = self.detector.update(sample);
    self.effects(detection)
  }

  pub fn set_thirds(&mut self, thirds: bool) -> GlideEffects {
    let detection = self.detector.set_thirds(thirds);
    self.effects(detection)
  }

  pub fn settle(&mut self, timestamp: f64) -> GlideEffects {
    let detection = self.detector.settle(timestamp);
    self.effects(detection)
  }

  /// Whether this session's lift commits the action it has armed.
  pub fn should_minimize(&self, cancelled: bool) -> bool {
    !cancelled && self.detector.pending() == Some(GlideAction::Minimize)
  }

  fn effects(&mut self, detection: GlideDetection) -> GlideEffects {
    let reveal =
      !self.reveal_requested && (detection.region.is_some() || detection.pending.is_some());
    if reveal {
      self.reveal_requested = true;
    }

    let move_to = if detection.pending.is_none()
      && detection.region.is_some()
      && detection.region != self.moved
    {
      self.moved = detection.region;
      detection.region
    } else {
      None
    };

    GlideEffects {
      detection,
      move_to,
      ready: detection.became_ready,
      reveal,
    }
  }
}

impl Default for GlideRuntime {
  fn default() -> Self {
    Self::new(GlideDetectorOptions::default())
  }
}
