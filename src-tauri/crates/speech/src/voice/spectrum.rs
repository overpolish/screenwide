// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The short-time spectrum the de-clicker and de-plosive work in: a run of
//! samples cut into overlapping frames, each windowed and turned into its
//! frequencies. What a stage changes in the spectrum is turned back into
//! sound and added to the samples, so a frame left alone adds exactly
//! nothing and the voice around a repair comes back bit for bit.

use std::sync::Arc;

use realfft::num_complex::Complex32;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};

/// The frames of a run, one after another, each `bins` frequencies long.
#[derive(Default)]
pub(super) struct Spectra {
  pub bins: usize,
  pub frames: usize,
  pub values: Vec<Complex32>,
}

impl Spectra {
  pub(super) fn frame(&self, frame: usize) -> &[Complex32] {
    &self.values[frame * self.bins..(frame + 1) * self.bins]
  }

  pub(super) fn frame_mut(&mut self, frame: usize) -> &mut [Complex32] {
    &mut self.values[frame * self.bins..(frame + 1) * self.bins]
  }

  /// Empty frames, as many as and as long as `like`'s.
  pub(super) fn clear_like(&mut self, like: &Spectra) {
    self.bins = like.bins;
    self.frames = like.frames;
    self.values.clear();
    self.values.resize(like.values.len(), Complex32::default());
  }
}

pub(super) struct Stft {
  size: usize,
  hop: usize,
  /// The square root of a Hann window, used both ways, so the two together
  /// make a Hann window, which overlapping evenly sums to a constant.
  window: Vec<f32>,
  forward: Arc<dyn RealToComplex<f32>>,
  inverse: Arc<dyn ComplexToReal<f32>>,
  frame: Vec<f32>,
  spectrum: Vec<Complex32>,
}

impl Stft {
  /// Frames `size` samples long, one every `hop`, which must divide half of
  /// `size` so the windows overlap evenly.
  pub(super) fn new(size: usize, hop: usize) -> Self {
    debug_assert_eq!(size / 2 % hop, 0);
    let mut planner = RealFftPlanner::<f32>::new();
    let window = (0..size)
      .map(|at| {
        let hann = 0.5 - 0.5 * (std::f64::consts::TAU * at as f64 / size as f64).cos();
        hann.sqrt() as f32
      })
      .collect();
    Self {
      size,
      hop,
      window,
      forward: planner.plan_fft_forward(size),
      inverse: planner.plan_fft_inverse(size),
      frame: vec![0.0; size],
      spectrum: vec![Complex32::default(); size / 2 + 1],
    }
  }

  pub(super) fn hop(&self) -> usize {
    self.hop
  }

  /// The samples frame `frame` covers: from `frame * hop - size`, so the
  /// first frames lead into the run and the last trail out of it.
  pub(super) fn span(&self, frame: usize) -> (isize, isize) {
    let end = (frame * self.hop) as isize;
    (end - self.size as isize, end)
  }

  /// The spectrum of `samples`, taken as silence before and after them.
  pub(super) fn analyze(&mut self, samples: &[f32], out: &mut Spectra) {
    out.bins = self.size / 2 + 1;
    out.frames = (samples.len() + self.size) / self.hop + 1;
    out.values.clear();
    for frame in 0..out.frames {
      let (start, _) = self.span(frame);
      for (at, (value, weight)) in self.frame.iter_mut().zip(&self.window).enumerate() {
        let sample = usize::try_from(start + at as isize)
          .ok()
          .and_then(|index| samples.get(index));
        *value = sample.map_or(0.0, |sample| sample * weight);
      }
      // Sizes match by construction, so the transform cannot fail.
      let _ = self.forward.process(&mut self.frame, &mut self.spectrum);
      out.values.extend_from_slice(&self.spectrum);
    }
  }

  /// Adds to `samples` the sound of `change`, a spectrum like the one
  /// [`Self::analyze`] gave, frame by frame; only the frames `changed` marks
  /// are worked out.
  pub(super) fn add(&mut self, change: &Spectra, changed: &[bool], samples: &mut [f32]) {
    // Hann windows a hop apart sum to `size / (2 * hop)`, and the inverse
    // transform is unscaled.
    let scale = 2.0 * self.hop as f32 / (self.size * self.size) as f32;
    for frame in (0..change.frames).filter(|&frame| changed[frame]) {
      self.spectrum.copy_from_slice(change.frame(frame));
      // The ends of a real signal's spectrum are real.
      self.spectrum[0].im = 0.0;
      let last = self.spectrum.len() - 1;
      self.spectrum[last].im = 0.0;
      let _ = self.inverse.process(&mut self.spectrum, &mut self.frame);
      let (start, _) = self.span(frame);
      for (at, (value, weight)) in self.frame.iter().zip(&self.window).enumerate() {
        if let Some(sample) = usize::try_from(start + at as isize)
          .ok()
          .and_then(|index| samples.get_mut(index))
        {
          *sample += value * weight * scale;
        }
      }
    }
  }
}

/// The middle of `values`, which it reorders.
pub(super) fn median(values: &mut [f32]) -> f32 {
  let middle = values.len() / 2;
  *values.select_nth_unstable_by(middle, f32::total_cmp).1
}
