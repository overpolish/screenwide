// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GlidePhase {
  Ready,
  Settling,
}
#[derive(Debug, Clone, Copy)]
pub struct RestGate {
  last_active: f64,
  phase: GlidePhase,
  pub motion_noise_floor: f64,
  pub rest_ms: f64,
}
impl RestGate {
  pub fn new(noise: f64, rest: f64) -> Self {
    Self {
      last_active: 0.0,
      phase: GlidePhase::Ready,
      motion_noise_floor: noise,
      rest_ms: rest,
    }
  }
  pub fn phase(&self) -> GlidePhase {
    self.phase
  }
  pub fn hold(&mut self, t: f64) {
    self.last_active = t;
    self.phase = GlidePhase::Settling;
  }
  pub fn remaining(&self, t: f64) -> f64 {
    if self.phase == GlidePhase::Ready {
      0.0
    } else {
      (self.rest_ms - (t - self.last_active)).max(0.0)
    }
  }
  pub fn reset(&mut self) {
    self.last_active = 0.0;
    self.phase = GlidePhase::Ready;
  }
  pub fn settle(&mut self, t: f64) -> bool {
    if self.phase == GlidePhase::Ready || self.remaining(t) > 0.0 {
      false
    } else {
      self.phase = GlidePhase::Ready;
      true
    }
  }
  pub fn stir(&mut self, t: f64, motion: f64) {
    if self.phase == GlidePhase::Settling && motion >= self.motion_noise_floor {
      self.last_active = t;
    }
  }
}
