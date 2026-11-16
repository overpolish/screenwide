// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The features the voice model hears: Kaldi's 80-band log mel filterbank, as
//! sherpa-onnx computes it for its speaker models (kaldi-native-fbank, no
//! dither, frames centred on each 10 ms hop with the edges reflected), with
//! each band's mean over the stretch taken away, as the model's
//! `feature_normalize_type = global-mean` asks.

use std::f32::consts::PI;
use std::sync::Arc;

use realfft::{RealFftPlanner, RealToComplex};

const SAMPLE_RATE: f32 = 16_000.0;
const FRAME_LENGTH: usize = 400;
const FRAME_SHIFT: usize = 160;
const FFT_LENGTH: usize = 512;
const BANDS: usize = 80;
const LOW_HZ: f32 = 20.0;
/// sherpa-onnx's `high_freq = -400`: 400 Hz below Nyquist.
const HIGH_HZ: f32 = SAMPLE_RATE / 2.0 - 400.0;
const PREEMPHASIS: f32 = 0.97;

pub const DIMENSIONS: usize = BANDS;

pub struct Fbank {
  fft: Arc<dyn RealToComplex<f32>>,
  window: Vec<f32>,
  /// Each band's weight over the FFT bins below Nyquist, as `(first bin,
  /// weights)`.
  bands: Vec<(usize, Vec<f32>)>,
}

fn mel(hz: f32) -> f32 {
  1127.0 * (1.0 + hz / 700.0).ln()
}

impl Fbank {
  pub fn new() -> Self {
    // Povey's window: a Hann window raised to 0.85.
    let window = (0..FRAME_LENGTH)
      .map(|i| {
        let hann = 0.5 - 0.5 * (2.0 * PI * i as f32 / (FRAME_LENGTH - 1) as f32).cos();
        hann.powf(0.85)
      })
      .collect();
    let bin_hz = SAMPLE_RATE / FFT_LENGTH as f32;
    let (low, high) = (mel(LOW_HZ), mel(HIGH_HZ));
    let delta = (high - low) / (BANDS + 1) as f32;
    let bands = (0..BANDS)
      .map(|band| {
        let left = low + band as f32 * delta;
        let centre = left + delta;
        let right = centre + delta;
        let weights: Vec<(usize, f32)> = (0..FFT_LENGTH / 2)
          .filter_map(|bin| {
            let m = mel(bin_hz * bin as f32);
            (m > left && m < right).then(|| {
              let weight = if m <= centre {
                (m - left) / (centre - left)
              } else {
                (right - m) / (right - centre)
              };
              (bin, weight)
            })
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
      fft: RealFftPlanner::new().plan_fft_forward(FFT_LENGTH),
      window,
      bands,
    }
  }

  /// The features of `samples`, 16 kHz in [-1, 1], one row of
  /// `DIMENSIONS` per 10 ms, row-major, with each band's mean removed.
  pub fn features(&self, samples: &[f32]) -> Vec<f32> {
    let frames = (samples.len() + FRAME_SHIFT / 2) / FRAME_SHIFT;
    let mut out = vec![0.0f32; frames * BANDS];
    if samples.is_empty() {
      return out;
    }
    let length = samples.len() as isize;
    let mut buffer = self.fft.make_input_vec();
    let mut spectrum = self.fft.make_output_vec();
    let mut scratch = self.fft.make_scratch_vec();
    let mut power = vec![0.0f32; FFT_LENGTH / 2 + 1];
    for (frame, row) in out.as_chunks_mut::<BANDS>().0.iter_mut().enumerate() {
      // Centred on the hop, the edges reflected (snip_edges = false).
      let start = (frame * FRAME_SHIFT + FRAME_SHIFT / 2) as isize - (FRAME_LENGTH / 2) as isize;
      buffer.fill(0.0);
      for (i, slot) in buffer[..FRAME_LENGTH].iter_mut().enumerate() {
        let mut at = start + i as isize;
        while at < 0 || at >= length {
          at = if at < 0 { -at - 1 } else { 2 * length - 1 - at };
        }
        *slot = samples[at as usize];
      }
      let frame_samples = &mut buffer[..FRAME_LENGTH];
      let mean = frame_samples.iter().sum::<f32>() / FRAME_LENGTH as f32;
      frame_samples.iter_mut().for_each(|sample| *sample -= mean);
      for i in (1..FRAME_LENGTH).rev() {
        frame_samples[i] -= PREEMPHASIS * frame_samples[i - 1];
      }
      frame_samples[0] -= PREEMPHASIS * frame_samples[0];
      frame_samples
        .iter_mut()
        .zip(&self.window)
        .for_each(|(sample, weight)| *sample *= weight);
      if self
        .fft
        .process_with_scratch(&mut buffer, &mut spectrum, &mut scratch)
        .is_err()
      {
        continue;
      }
      power
        .iter_mut()
        .zip(&spectrum)
        .for_each(|(slot, bin)| *slot = bin.norm_sqr());
      for (value, (first, weights)) in row.iter_mut().zip(&self.bands) {
        let energy: f32 = weights
          .iter()
          .zip(&power[*first..])
          .map(|(weight, bin)| weight * bin)
          .sum();
        *value = energy.max(f32::EPSILON).ln();
      }
    }
    for band in 0..BANDS {
      let mean = out.iter().skip(band).step_by(BANDS).sum::<f32>() / frames as f32;
      out
        .iter_mut()
        .skip(band)
        .step_by(BANDS)
        .for_each(|value| *value -= mean);
    }
    out
  }
}
