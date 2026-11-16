// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Audio brought to what the speech models hear: one channel at 16 kHz, as
//! 16-bit samples. Everything above what 16 kHz can carry is taken out
//! first, so it does not fold back into the voice. Dictation and meeting
//! notes bring their microphones and the computer's audio to it.

use std::f64::consts::PI;

use crate::runner::dictation::SAMPLE_RATE;

/// Where the low pass starts to cut, under half of 16 kHz with room for its
/// slope.
const CUTOFF_HZ: f64 = 7_000.0;

/// A source's samples, as it delivers them, turned into the speech model's.
pub struct ToSpeech {
  channels: usize,
  /// How far the input moves for each sample out, in input samples.
  step: f64,
  /// Where the next sample out falls after `previous`, in input samples.
  position: f64,
  previous: f32,
  low_pass: Option<[Biquad; 4]>,
}

impl ToSpeech {
  pub fn new(channels: u16, sample_rate: u32) -> Self {
    let rate = f64::from(sample_rate.max(1));
    Self {
      channels: usize::from(channels.max(1)),
      step: rate / f64::from(SAMPLE_RATE),
      position: 0.0,
      previous: 0.0,
      // An eighth-order Butterworth, as four sections.
      low_pass: (rate > f64::from(SAMPLE_RATE)).then(|| {
        [0.509_795_6, 0.601_344_9, 0.899_976_1, 2.562_915_4].map(|q| Biquad::low_pass(rate, q))
      }),
    }
  }

  /// Adds the samples of `frames`, channels interleaved, to `out`.
  pub fn push(&mut self, frames: &[f32], out: &mut Vec<i16>) {
    for frame in frames.chunks_exact(self.channels) {
      #[allow(clippy::cast_precision_loss)]
      let mut sample = frame.iter().sum::<f32>() / self.channels as f32;
      if let Some(sections) = &mut self.low_pass {
        for section in sections {
          sample = section.run(sample);
        }
      }
      while self.position <= 1.0 {
        #[allow(clippy::cast_possible_truncation)]
        let between = self.position as f32;
        out.push(to_i16(self.previous + (sample - self.previous) * between));
        self.position += self.step;
      }
      self.position -= 1.0;
      self.previous = sample;
    }
  }
}

#[allow(clippy::cast_possible_truncation)]
fn to_i16(sample: f32) -> i16 {
  (sample.clamp(-1.0, 1.0) * 32_767.0).round() as i16
}

/// One second-order section, in direct form II transposed.
struct Biquad {
  b: [f32; 3],
  a: [f32; 2],
  state: [f32; 2],
}

impl Biquad {
  fn low_pass(rate: f64, q: f64) -> Self {
    let omega = 2.0 * PI * CUTOFF_HZ / rate;
    let alpha = omega.sin() / (2.0 * q);
    let cos = omega.cos();
    let a0 = 1.0 + alpha;
    let b1 = (1.0 - cos) / a0;
    #[allow(clippy::cast_possible_truncation)]
    Self {
      b: [(b1 / 2.0) as f32, b1 as f32, (b1 / 2.0) as f32],
      a: [(-2.0 * cos / a0) as f32, ((1.0 - alpha) / a0) as f32],
      state: [0.0; 2],
    }
  }

  fn run(&mut self, x: f32) -> f32 {
    let y = self.b[0].mul_add(x, self.state[0]);
    self.state[0] = self.b[1].mul_add(x, -self.a[0] * y) + self.state[1];
    self.state[1] = self.b[2].mul_add(x, -self.a[1] * y);
    y
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn tone(hz: f64, rate: u32, seconds: f64) -> Vec<f32> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let count = (f64::from(rate) * seconds) as usize;
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    (0..count)
      .map(|n| (0.5 * (2.0 * PI * hz * n as f64 / f64::from(rate)).sin()) as f32)
      .collect()
  }

  fn peak(samples: &[i16]) -> f32 {
    samples[samples.len() / 2..]
      .iter()
      .map(|&sample| f32::from(sample).abs() / 32_767.0)
      .fold(0.0, f32::max)
  }

  #[test]
  fn keeps_the_voice_and_drops_what_would_fold_back() {
    let mut voice = Vec::new();
    ToSpeech::new(1, 48_000).push(&tone(1_000.0, 48_000, 1.0), &mut voice);
    assert!((15_990..=16_010).contains(&voice.len()), "{}", voice.len());
    assert!((peak(&voice) - 0.5).abs() < 0.03, "{}", peak(&voice));

    let mut whistle = Vec::new();
    ToSpeech::new(1, 48_000).push(&tone(12_000.0, 48_000, 1.0), &mut whistle);
    assert!(peak(&whistle) < 0.02, "{}", peak(&whistle));
  }

  #[test]
  fn mixes_channels_down() {
    let mut out = Vec::new();
    let stereo: Vec<f32> = tone(500.0, 44_100, 0.5)
      .into_iter()
      .flat_map(|sample| [sample, -sample])
      .collect();
    ToSpeech::new(2, 44_100).push(&stereo, &mut out);
    assert!(peak(&out) < 0.001);
  }
}
