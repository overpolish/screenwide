// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The input the restoration model was trained on: Kaldi-style log mel
//! filter banks of 16 kHz audio, normalised per band and paired frame by
//! frame, as the SeamlessM4T feature extractor in Hugging Face's
//! `transformers` computes them. Every step follows that extractor, since
//! the model hears any difference as a different voice.

use std::f64::consts::PI;
use std::sync::Arc;

use realfft::{RealFftPlanner, RealToComplex};

const FRAME: usize = 400;
const HOP: usize = 160;
const FFT: usize = 512;
const BINS: usize = FFT / 2 + 1;
const MELS: usize = 80;
/// Consecutive frames joined into one, doubling each frame's width.
const STRIDE: usize = 2;
pub(super) const WIDTH: usize = MELS * STRIDE;
const PREEMPHASIS: f32 = 0.97;
/// The smallest band energy taken a logarithm of, as the extractor's.
const MEL_FLOOR: f32 = f32::EPSILON;
/// The extractor scales samples to 16-bit integers first, as Kaldi expects.
const SCALE: f32 = 32_768.0;

/// Each frame is worked in double precision, as the extractor does: the top
/// bands of a quiet frame hold little more than rounding, and single
/// precision moves them by a tenth of a standard deviation.
pub(super) struct Fbank {
  fft: Arc<dyn RealToComplex<f64>>,
  window: Vec<f64>,
  /// For each mel band, its first FFT bin and the weights from there.
  filters: Vec<(usize, Vec<f64>)>,
}

/// Kaldi's mel scale.
fn mel(hertz: f64) -> f64 {
  1127.0 * (1.0 + hertz / 700.0).ln()
}

impl Fbank {
  pub(super) fn new(rate: u32) -> Self {
    // The Povey window: a symmetric Hann window raised to 0.85.
    let window = (0..FRAME)
      .map(|at| {
        let hann = 0.5 - 0.5 * (2.0 * PI * at as f64 / (FRAME - 1) as f64).cos();
        hann.powf(0.85)
      })
      .collect();
    // Triangles spaced evenly in mel from 20 Hz to half the rate, laid over
    // the FFT bins by each bin's own mel value.
    let (low, high) = (mel(20.0), mel(f64::from(rate) / 2.0));
    let edges: Vec<f64> = (0..MELS + 2)
      .map(|index| low + (high - low) * index as f64 / (MELS + 1) as f64)
      .collect();
    let bin_width = f64::from(rate) / FFT as f64;
    let bin_mels: Vec<f64> = (0..BINS).map(|bin| mel(bin as f64 * bin_width)).collect();
    let filters = (0..MELS)
      .map(|band| {
        let (left, centre, right) = (edges[band], edges[band + 1], edges[band + 2]);
        let weights: Vec<(usize, f64)> = bin_mels
          .iter()
          .enumerate()
          .filter_map(|(bin, &at)| {
            let up = (at - left) / (centre - left);
            let down = (right - at) / (right - centre);
            let weight = up.min(down).max(0.0);
            (weight > 0.0).then_some((bin, weight))
          })
          .collect();
        let first = weights.first().map_or(0, |&(bin, _)| bin);
        (
          first,
          weights.into_iter().map(|(_, weight)| weight).collect(),
        )
      })
      .collect();
    Self {
      fft: RealFftPlanner::<f64>::new().plan_fft_forward(FFT),
      window,
      filters,
    }
  }

  /// The paired, normalised features of `samples`, row after row of
  /// [`WIDTH`] values, and how many rows there are.
  pub(super) fn features(&self, samples: &[f32]) -> (Vec<f32>, usize) {
    let frames = if samples.len() >= FRAME {
      1 + (samples.len() - FRAME) / HOP
    } else {
      0
    };
    let mut bands = vec![0.0_f32; frames * MELS];
    let mut buffer = vec![0.0_f64; FFT];
    let mut spectrum = self.fft.make_output_vec();
    let preemphasis = f64::from(PREEMPHASIS);
    for frame in 0..frames {
      let raw = &samples[frame * HOP..frame * HOP + FRAME];
      let scaled = |sample: f32| f64::from(sample * SCALE);
      let mean = raw.iter().map(|&sample| scaled(sample)).sum::<f64>() / FRAME as f64;
      let mut previous = scaled(raw[0]) - mean;
      buffer[0] = previous * (1.0 - preemphasis) * self.window[0];
      for (at, &sample) in raw.iter().enumerate().skip(1) {
        let centred = scaled(sample) - mean;
        buffer[at] = (centred - preemphasis * previous) * self.window[at];
        previous = centred;
      }
      buffer[FRAME..].fill(0.0);
      // Sizes match by construction, so the transform cannot fail.
      let _ = self.fft.process(&mut buffer, &mut spectrum);
      let row = &mut bands[frame * MELS..(frame + 1) * MELS];
      for (value, (first, weights)) in row.iter_mut().zip(&self.filters) {
        let energy: f64 = weights
          .iter()
          .zip(&spectrum[*first..])
          .map(|(weight, bin)| weight * bin.norm_sqr())
          .sum();
        *value = energy.max(f64::from(MEL_FLOOR)).ln() as f32;
      }
    }
    normalise(&mut bands, frames);
    let rows = frames / STRIDE;
    bands.truncate(rows * WIDTH);
    (bands, rows)
  }
}

/// Each band brought to a mean of zero and a variance of one over the
/// frames, the variance taken as an unbiased estimate as the extractor does.
fn normalise(bands: &mut [f32], frames: usize) {
  if frames < 2 {
    return;
  }
  for band in 0..MELS {
    let values = || {
      bands
        .iter()
        .skip(band)
        .step_by(MELS)
        .map(|&value| f64::from(value))
    };
    let mean = values().sum::<f64>() / frames as f64;
    let variance = values().map(|value| (value - mean).powi(2)).sum::<f64>() / (frames - 1) as f64;
    let scale = 1.0 / (variance + 1e-7).sqrt();
    for value in bands.iter_mut().skip(band).step_by(MELS) {
      *value = ((f64::from(*value) - mean) * scale) as f32;
    }
  }
}

#[cfg(test)]
mod tests;
