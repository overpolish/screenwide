// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
pub fn axis_step(travel: f64, size: f64) -> i32 {
  if travel.abs() < size {
    0
  } else if travel < 0.0 {
    -1
  } else {
    1
  }
}
#[derive(Debug, Clone, Copy)]
pub struct TurnPointTracker {
  direction: i32,
  extremum: f64,
  origin: f64,
  position: f64,
  hysteresis: f64,
}
impl TurnPointTracker {
  pub fn new(hysteresis: f64) -> Self {
    Self {
      direction: 0,
      extremum: 0.0,
      origin: 0.0,
      position: 0.0,
      hysteresis,
    }
  }
  pub fn travel(&self) -> f64 {
    self.position - self.origin
  }
  pub fn rebase(&mut self) {
    self.direction = 0;
    self.extremum = self.position;
    self.origin = self.position;
  }
  pub fn reset(&mut self) {
    self.position = 0.0;
    self.rebase();
  }
  pub fn step(&self, size: f64) -> i32 {
    axis_step(self.travel(), size)
  }
  pub fn update(&mut self, delta: f64) {
    self.position += delta;
    let offset = self.position - self.extremum;
    if self.direction == 0 || offset * self.direction as f64 > 0.0 {
      if offset != 0.0 {
        self.direction = offset.signum() as i32;
      }
      self.extremum = self.position;
      return;
    }
    if offset.abs() <= self.hysteresis {
      return;
    }
    self.direction = -self.direction;
    self.origin = self.extremum;
    self.extremum = self.position;
  }
}
