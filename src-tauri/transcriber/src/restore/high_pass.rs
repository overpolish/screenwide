// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// A second-order high-pass filter, after the Audio EQ Cookbook, with a
/// Butterworth response, clamped to full scale as torchaudio's is.
pub(super) struct HighPass {
  b: [f64; 3],
  a: [f64; 2],
  x: [f64; 2],
  y: [f64; 2],
}

impl HighPass {
  pub(super) fn new(rate: f64, frequency: f64) -> Self {
    let w0 = std::f64::consts::TAU * frequency / rate;
    let alpha = w0.sin() / (2.0 * std::f64::consts::FRAC_1_SQRT_2);
    let cos = w0.cos();
    let a0 = 1.0 + alpha;
    Self {
      b: [
        (1.0 + cos) / 2.0 / a0,
        -(1.0 + cos) / a0,
        (1.0 + cos) / 2.0 / a0,
      ],
      a: [-2.0 * cos / a0, (1.0 - alpha) / a0],
      x: [0.0; 2],
      y: [0.0; 2],
    }
  }

  pub(super) fn process(&mut self, sample: f32) -> f32 {
    let x = f64::from(sample);
    let y = self.b[0] * x + self.b[1] * self.x[0] + self.b[2] * self.x[1]
      - self.a[0] * self.y[0]
      - self.a[1] * self.y[1];
    self.x = [x, self.x[0]];
    self.y = [y, self.y[0]];
    y.clamp(-1.0, 1.0) as f32
  }
}
