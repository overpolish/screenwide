// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Second-order filters after the Audio EQ Cookbook, which the cleanup
//! shapes the voice with and splits it by.

use std::f64::consts::TAU;

/// The quality of each section of a fourth-order Butterworth filter.
const BUTTERWORTH_Q: [f64; 2] = [0.541_196_1, 1.306_563];

#[derive(Clone, Copy)]
pub(super) struct Biquad {
  b0: f64,
  b1: f64,
  b2: f64,
  a1: f64,
  a2: f64,
  z1: f64,
  z2: f64,
}

impl Biquad {
  fn new([b0, b1, b2]: [f64; 3], [a0, a1, a2]: [f64; 3]) -> Self {
    Self {
      b0: b0 / a0,
      b1: b1 / a0,
      b2: b2 / a0,
      a1: a1 / a0,
      a2: a2 / a0,
      z1: 0.0,
      z2: 0.0,
    }
  }

  pub(super) fn high_pass(rate: u32, frequency: f64, q: f64) -> Self {
    let omega = TAU * frequency / f64::from(rate);
    let (alpha, cos) = (omega.sin() / (2.0 * q), omega.cos());
    Self::new(
      [(1.0 + cos) / 2.0, -(1.0 + cos), (1.0 + cos) / 2.0],
      [1.0 + alpha, -2.0 * cos, 1.0 - alpha],
    )
  }

  /// A bell around `frequency` raised or lowered by `gain_db`.
  pub(super) fn peaking(rate: u32, frequency: f64, q: f64, gain_db: f64) -> Self {
    let omega = TAU * frequency / f64::from(rate);
    let (alpha, cos) = (omega.sin() / (2.0 * q), omega.cos());
    let amplitude = 10_f64.powf(gain_db / 40.0);
    Self::new(
      [1.0 + alpha * amplitude, -2.0 * cos, 1.0 - alpha * amplitude],
      [1.0 + alpha / amplitude, -2.0 * cos, 1.0 - alpha / amplitude],
    )
  }

  pub(super) fn process(&mut self, x: f64) -> f64 {
    let y = self.b0 * x + self.z1;
    self.z1 = self.b1 * x - self.a1 * y + self.z2;
    self.z2 = self.b2 * x - self.a2 * y;
    y
  }

  fn reset(&mut self) {
    self.z1 = 0.0;
    self.z2 = 0.0;
  }
}

/// The Butterworth high-pass the tick finder and de-esser split the voice
/// with, run forward and then backward over each run so it shifts nothing in
/// time. The split is `x = (x - h) + h`, and with no shift `x - h` holds only
/// what lies below the cutoff, so turning `h` down touches nothing else.
pub(super) struct Split([Biquad; 2]);

impl Split {
  pub(super) fn new(rate: u32, frequency: f64) -> Self {
    Self(BUTTERWORTH_Q.map(|q| Biquad::high_pass(rate, frequency, q)))
  }

  /// Writes what lies above the cutoff in `input` into `output`.
  pub(super) fn high(&mut self, input: &[f32], output: &mut Vec<f32>) {
    let [first, second] = &mut self.0;
    first.reset();
    second.reset();
    output.clear();
    output.extend(
      input
        .iter()
        .map(|&sample| second.process(first.process(f64::from(sample))) as f32),
    );
    first.reset();
    second.reset();
    for sample in output.iter_mut().rev() {
      *sample = second.process(first.process(f64::from(*sample))) as f32;
    }
  }
}

/// The voice's tone set right, gently: the rumble below 80 Hz taken off,
/// which no voice reaches, and the boxiness around 300 Hz of a microphone
/// close to a desk or a wall eased by 2 dB.
pub(super) struct Tone {
  rumble: Biquad,
  boxiness: Biquad,
}

impl Tone {
  pub(super) fn new(rate: u32) -> Self {
    Self {
      rumble: Biquad::high_pass(rate, 80.0, std::f64::consts::FRAC_1_SQRT_2),
      boxiness: Biquad::peaking(rate, 300.0, 1.0, -2.0),
    }
  }

  /// Shapes `samples`, which carry on from the last run.
  pub(super) fn apply(&mut self, samples: &mut [f32]) {
    for sample in samples {
      *sample = self
        .boxiness
        .process(self.rumble.process(f64::from(*sample))) as f32;
    }
  }
}

/// The weighting ITU-R BS.1770 measures loudness through, at 48 kHz: a lift
/// of 4 dB above 1.5 kHz, where the ear is keenest, and a cut below 40 Hz,
/// where it hardly hears at all.
pub(super) struct Loudness([Biquad; 2]);

impl Loudness {
  pub(super) fn new() -> Self {
    Self([
      Biquad::new(
        [
          1.535_124_859_586_97,
          -2.691_696_189_406_38,
          1.198_392_810_852_85,
        ],
        [1.0, -1.690_659_293_182_41, 0.732_480_774_215_85],
      ),
      Biquad::new(
        [1.0, -2.0, 1.0],
        [1.0, -1.990_047_454_833_98, 0.990_072_250_366_21],
      ),
    ])
  }

  /// Writes `input` weighted into `output`, carrying on from the last run.
  pub(super) fn weigh(&mut self, input: &[f32], output: &mut Vec<f32>) {
    let [first, second] = &mut self.0;
    output.clear();
    output.extend(
      input
        .iter()
        .map(|&sample| second.process(first.process(f64::from(sample))) as f32),
    );
  }
}
