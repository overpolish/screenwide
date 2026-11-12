// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The spectrum Mel-Band RoFormer hears and answers in: the short-time
//! Fourier transform its training code takes with PyTorch's `torch.stft`
//! (frames of 2048 samples every 441, a periodic Hann window, centred with
//! the edges reflected, unnormalised) and its inverse. The model's own
//! export leaves both out, as they are outside the network.

use std::sync::Arc;

use realfft::num_complex::Complex32;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};

use super::{CHUNK, SIDES};

const N_FFT: usize = 2048;
const HOP: usize = 441;
const HALF: usize = N_FFT / 2;
/// Frequencies in each frame.
pub(super) const BINS: usize = HALF + 1;
/// Frames in a chunk.
pub(super) const FRAMES: usize = CHUNK / HOP + 1;
/// Samples the frames of a chunk span, centred on it.
const SPAN: usize = N_FFT + HOP * (FRAMES - 1);

pub(super) struct Spectrum {
  forward: Arc<dyn RealToComplex<f32>>,
  inverse: Arc<dyn ComplexToReal<f32>>,
  window: Vec<f32>,
  /// How much the windows overlapping each sample add up to, squared, which
  /// the inverse divides by.
  overlap: Vec<f32>,
  frame: Vec<f32>,
  bins: Vec<Complex32>,
  scratch: Vec<Complex32>,
}

impl Spectrum {
  pub(super) fn new() -> Self {
    let mut planner = RealFftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(N_FFT);
    let inverse = planner.plan_fft_inverse(N_FFT);
    #[expect(clippy::cast_precision_loss, reason = "a window's shape")]
    let window: Vec<f32> = (0..N_FFT)
      .map(|n| 0.5 - 0.5 * (std::f32::consts::TAU * n as f32 / N_FFT as f32).cos())
      .collect();
    let mut overlap = vec![0.0_f32; SPAN];
    for frame in 0..FRAMES {
      for (offset, weight) in window.iter().enumerate() {
        overlap[frame * HOP + offset] += weight * weight;
      }
    }
    let scratch =
      vec![Complex32::default(); forward.get_scratch_len().max(inverse.get_scratch_len())];
    Self {
      frame: forward.make_input_vec(),
      bins: forward.make_output_vec(),
      forward,
      inverse,
      window,
      overlap,
      scratch,
    }
  }

  /// The spectrum of one planar chunk as the model takes it: the left
  /// side's real and imaginary parts, then the right's, each frequency by
  /// frame.
  pub(super) fn analyse(&mut self, planar: &[f32]) -> Vec<f32> {
    let mut spectrum = vec![0.0_f32; 2 * SIDES * BINS * FRAMES];
    for (side, samples) in planar.as_chunks::<CHUNK>().0.iter().enumerate() {
      for frame in 0..FRAMES {
        for (offset, slot) in self.frame.iter_mut().enumerate() {
          *slot = samples[reflected(frame * HOP + offset)] * self.window[offset];
        }
        self
          .forward
          .process_with_scratch(&mut self.frame, &mut self.bins, &mut self.scratch)
          .expect("the buffers are the plan's own sizes");
        for (bin, value) in self.bins.iter().enumerate() {
          spectrum[((2 * side) * BINS + bin) * FRAMES + frame] = value.re;
          spectrum[((2 * side + 1) * BINS + bin) * FRAMES + frame] = value.im;
        }
      }
    }
    spectrum
  }

  /// The planar chunk `spectrum`, laid out as [`Self::analyse`] gives it,
  /// is the spectrum of.
  pub(super) fn synthesise(&mut self, spectrum: &[f32]) -> Vec<f32> {
    let mut planar = vec![0.0_f32; SIDES * CHUNK];
    let mut summed = vec![0.0_f32; SPAN];
    #[expect(clippy::cast_precision_loss, reason = "the transform's length")]
    let scale = 1.0 / N_FFT as f32;
    for side in 0..SIDES {
      summed.fill(0.0);
      for frame in 0..FRAMES {
        for (bin, value) in self.bins.iter_mut().enumerate() {
          *value = Complex32::new(
            spectrum[((2 * side) * BINS + bin) * FRAMES + frame],
            spectrum[((2 * side + 1) * BINS + bin) * FRAMES + frame],
          );
        }
        // A real signal's spectrum has no imaginary part at either end.
        self.bins[0].im = 0.0;
        self.bins[BINS - 1].im = 0.0;
        self
          .inverse
          .process_with_scratch(&mut self.bins, &mut self.frame, &mut self.scratch)
          .expect("the buffers are the plan's own sizes");
        for (offset, sample) in self.frame.iter().enumerate() {
          summed[frame * HOP + offset] += sample * scale * self.window[offset];
        }
      }
      for (index, sample) in planar[side * CHUNK..(side + 1) * CHUNK]
        .iter_mut()
        .enumerate()
      {
        *sample = summed[index + HALF] / self.overlap[index + HALF].max(1e-8);
      }
    }
    planar
  }
}

/// The chunk's sample at `index` of its centred span, half a frame before
/// it and after it reflected back in.
fn reflected(index: usize) -> usize {
  let at = index.abs_diff(HALF);
  if index < HALF {
    at
  } else if at >= CHUNK {
    2 * (CHUNK - 1) - at
  } else {
    at
  }
}
