// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Opening direction collection and one deliberate sideways-to-corner turn.

use std::collections::VecDeque;

use super::travel::TurnPointTracker;

#[derive(Default)]
pub(super) struct OpeningGate {
  since: Option<f64>,
}

impl OpeningGate {
  pub fn accepts(&mut self, timestamp: f64, corner: bool, grace_ms: f64) -> bool {
    corner || timestamp - *self.since.get_or_insert(timestamp) >= grace_ms
  }

  pub fn pending(&self) -> bool {
    self.since.is_some()
  }

  pub fn reset(&mut self) {
    self.since = None;
  }
}

const HEADING_WINDOW_MS: f64 = 60.0;
const VERTICAL_DOMINANCE: f64 = 1.5;

pub(super) struct CornerRefinement {
  armed: bool,
  recent: VecDeque<(f64, f64, f64)>,
  vertical: TurnPointTracker,
}

impl CornerRefinement {
  pub fn new(hysteresis: f64) -> Self {
    Self {
      armed: false,
      recent: VecDeque::new(),
      vertical: TurnPointTracker::new(hysteresis),
    }
  }

  pub fn arm(&mut self, armed: bool) {
    self.armed = armed;
    self.recent.clear();
    self.vertical.reset();
  }

  pub fn update(&mut self, dx: f64, dy: f64, timestamp: f64, threshold: f64) -> i32 {
    if !self.armed {
      return 0;
    }
    self.recent.push_back((timestamp, dx, dy));
    while self
      .recent
      .front()
      .is_some_and(|sample| timestamp - sample.0 > HEADING_WINDOW_MS)
    {
      self.recent.pop_front();
    }
    let (horizontal, vertical) = self.recent.iter().fold((0.0, 0.0), |sum, sample| {
      (sum.0 + sample.1.abs(), sum.1 + sample.2)
    });
    // Long sideways travel with small vertical drift never earns a corner.
    // Distance only accumulates while the recent heading is clearly vertical.
    if vertical.abs() <= horizontal * VERTICAL_DOMINANCE {
      self.vertical.reset();
      return 0;
    }
    self.vertical.update(dy);
    let step = self.vertical.step(threshold);
    if step != 0 {
      self.arm(false);
    }
    step
  }
}
