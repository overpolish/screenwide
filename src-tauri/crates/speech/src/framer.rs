// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A track's short-time spectrum taken as it streams past, frame after
//! frame, and, where frames are changed, put back together into sound. Every
//! track is cut into the same frames, so frame `n` of the microphone lies
//! over frame `n` of the system audio. The system audio making way for the
//! voice works in these frames, and so does Reduce noise's noise profile.

use std::sync::Arc;

use realfft::num_complex::Complex32;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};

/// Frames of 43 ms, one every 10.7 ms at 48 kHz: fine enough in frequency to
/// follow the voice's bands, quick enough to follow its syllables.
pub(super) const SIZE: usize = 2_048;
pub(super) const HOP: usize = 512;

pub(super) struct Framer {
  /// The square root of a Hann window, used both ways, so the two together
  /// make a Hann window, which a quarter apart sums to a constant.
  window: Vec<f32>,
  forward: Arc<dyn RealToComplex<f32>>,
  inverse: Arc<dyn ComplexToReal<f32>>,
  /// The last `SIZE` samples fed, the earliest first.
  input: Vec<f32>,
  /// Samples fed since the last frame.
  waiting: usize,
  /// What the frames so far add up to over the last `SIZE` samples.
  sum: Vec<f32>,
  frame: Vec<f32>,
  spectrum: Vec<Complex32>,
  next: usize,
}

impl Framer {
  pub(super) fn new() -> Self {
    let mut planner = RealFftPlanner::<f32>::new();
    Self {
      window: (0..SIZE)
        .map(|at| {
          let hann = 0.5 - 0.5 * (std::f64::consts::TAU * at as f64 / SIZE as f64).cos();
          hann.sqrt() as f32
        })
        .collect(),
      forward: planner.plan_fft_forward(SIZE),
      inverse: planner.plan_fft_inverse(SIZE),
      input: vec![0.0; SIZE],
      waiting: 0,
      sum: vec![0.0; SIZE],
      frame: vec![0.0; SIZE],
      spectrum: vec![Complex32::default(); SIZE / 2 + 1],
      next: 0,
    }
  }

  /// Where frame `frame`'s middle lies, in samples from the start: frame `n`
  /// covers the `SIZE` samples up to `(n + 1) * HOP`.
  pub(super) fn middle(frame: usize) -> f64 {
    (frame + 1) as f64 * HOP as f64 - SIZE as f64 / 2.0
  }

  /// Feeds `samples`, handing each frame they complete to `on_frame` by its
  /// number. With `out`, the frames as `on_frame` leaves them are put back
  /// together and pushed there, `SIZE - HOP` samples behind what is fed.
  pub(super) fn feed(
    &mut self,
    samples: &[f32],
    mut out: Option<&mut Vec<f32>>,
    on_frame: &mut dyn FnMut(usize, &mut [Complex32]),
  ) {
    for &sample in samples {
      self.input[SIZE - HOP + self.waiting] = sample;
      self.waiting += 1;
      if self.waiting < HOP {
        continue;
      }
      self.waiting = 0;
      for ((frame, input), weight) in self.frame.iter_mut().zip(&self.input).zip(&self.window) {
        *frame = input * weight;
      }
      // Sizes match by construction, so the transforms cannot fail.
      let _ = self.forward.process(&mut self.frame, &mut self.spectrum);
      on_frame(self.next, &mut self.spectrum);
      self.next += 1;
      self.input.copy_within(HOP.., 0);
      let Some(out) = out.as_deref_mut() else {
        continue;
      };
      self.spectrum[0].im = 0.0;
      let last = self.spectrum.len() - 1;
      self.spectrum[last].im = 0.0;
      let _ = self.inverse.process(&mut self.spectrum, &mut self.frame);
      // Hann windows a quarter apart sum to two, and the inverse transform
      // is unscaled.
      let scale = 1.0 / (2.0 * SIZE as f32);
      for ((sum, value), weight) in self.sum.iter_mut().zip(&self.frame).zip(&self.window) {
        *sum += value * weight * scale;
      }
      out.extend_from_slice(&self.sum[..HOP]);
      self.sum.copy_within(HOP.., 0);
      self.sum[SIZE - HOP..].fill(0.0);
    }
  }
}
