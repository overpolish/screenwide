// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How far the system audio makes way in each frame, frequency by frequency.
//! All of it comes down by `WHOLE_DB` while the voice speaks; and in each
//! band where the voice has energy at that moment, measured against its own
//! usual level there, it comes down by up to `UNMASK_DB` more. Bands the
//! voice barely reaches, and the bass and treble around it, keep playing,
//! so music under speech still sounds full while the voice sits on top.

use realfft::num_complex::Complex32;

use super::super::framer::SIZE;

/// All of the system audio comes down this far while the voice speaks.
const WHOLE_DB: f32 = 6.0;
/// The most a band the voice is using comes down on top of that.
const UNMASK_DB: f32 = 6.0;
/// The voice's bands: sixteen from 150 Hz to 6 kHz, a third of an octave
/// apart, give or take.
const BANDS: usize = 16;
const LOWEST_HZ: f32 = 150.0;
const HIGHEST_HZ: f32 = 6_000.0;
/// A band as loud as the voice usually is in it makes way fully, one this
/// far under, 20 dB, not at all.
const PRESENCE_DB: f32 = 20.0;
/// A band within this, 10 dB, of the voice's strongest makes way fully; one
/// 30 dB under it, where the voice has little to be heard by, not at all.
const STRONG_DB: (f32, f32) = (10.0, 30.0);
/// How quickly a band makes way and comes back, in frames of 10.7 ms.
const DOWN_FRAMES: f32 = 2.0;
const UP_FRAMES: f32 = 15.0;

/// The bins a band spans.
fn band_edges(rate: u32) -> [(usize, usize); BANDS] {
  let bin = |hz: f32| ((hz / rate as f32) * SIZE as f32).round() as usize;
  let ratio = (HIGHEST_HZ / LOWEST_HZ).powf(1.0 / BANDS as f32);
  std::array::from_fn(|band| {
    let low = LOWEST_HZ * ratio.powi(band as i32);
    (bin(low), bin(low * ratio).max(bin(low) + 1))
  })
}

/// The voice's level in each band, frame by frame, as the microphone gives
/// it, in decibels.
#[derive(Default)]
pub(super) struct VoiceBands {
  levels: Vec<[f32; BANDS]>,
}

impl VoiceBands {
  pub(super) fn add(&mut self, spectrum: &[Complex32], rate: u32) {
    self.levels.push(band_edges(rate).map(|(from, to)| {
      let power = spectrum[from..to]
        .iter()
        .map(Complex32::norm_sqr)
        .sum::<f32>()
        / (to - from) as f32;
      10.0 * (power + 1e-12).log10()
    }));
  }
}

/// How far each band makes way in each frame, in decibels, and how far all
/// of the audio does.
pub(super) struct Plan {
  whole: Vec<f32>,
  bands: Vec<[f32; BANDS]>,
  bins: Vec<Option<(usize, usize, f32)>>,
  /// Each bin's gain in the frame last asked for, and which frame that was.
  gains: Vec<f32>,
  gains_frame: Option<usize>,
}

impl Plan {
  /// The plan for `gate.len()` frames of system audio, which make way as far
  /// as `gate` says, against what the microphone gave in `voice`.
  pub(super) fn new(voice: VoiceBands, gate: Vec<f32>, rate: u32) -> Self {
    let usual = usual_levels(&voice.levels, &gate);
    let strongest = usual.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    // With no speech heard, nothing is unmasked.
    let strength = usual.map(|level| {
      if !strongest.is_finite() {
        return 0.0;
      }
      ((strongest - level - STRONG_DB.1) / (STRONG_DB.0 - STRONG_DB.1)).clamp(0.0, 1.0)
    });
    let mut held = [0.0_f32; BANDS];
    let bands = gate
      .iter()
      .enumerate()
      .map(|(frame, &open)| {
        let levels = voice
          .levels
          .get(frame)
          .copied()
          .unwrap_or([f32::NEG_INFINITY; BANDS]);
        for band in 0..BANDS {
          let presence = ((levels[band] - usual[band] + PRESENCE_DB) / PRESENCE_DB).clamp(0.0, 1.0);
          let target = UNMASK_DB * presence * strength[band] * open;
          let frames = if target > held[band] {
            DOWN_FRAMES
          } else {
            UP_FRAMES
          };
          held[band] += (target - held[band]) / frames;
        }
        held
      })
      .collect();
    let bins = bin_bands(rate);
    Self {
      whole: gate.iter().map(|open| WHOLE_DB * open).collect(),
      bands,
      gains: vec![1.0; bins.len()],
      gains_frame: None,
      bins,
    }
  }

  /// Turns frame `frame` of the system audio down as planned.
  pub(super) fn apply(&mut self, frame: usize, spectrum: &mut [Complex32]) {
    if self.gains_frame != Some(frame) {
      self.fill_gains(frame);
    }
    for (value, gain) in spectrum.iter_mut().zip(&self.gains) {
      *value *= *gain;
    }
  }

  fn fill_gains(&mut self, frame: usize) {
    let whole = self.whole.get(frame).copied().unwrap_or(0.0);
    let bands = self.bands.get(frame).copied().unwrap_or([0.0; BANDS]);
    for (gain, bin) in self.gains.iter_mut().zip(&self.bins) {
      let unmask = bin.map_or(0.0, |(low, high, along)| {
        bands[low] + (bands[high] - bands[low]) * along
      });
      *gain = 10_f32.powf(-(whole + unmask) / 20.0);
    }
    self.gains_frame = Some(frame);
  }
}

/// The voice's usual level in each band: the loud end of what it reaches
/// while it speaks, a tenth of its speech frames louder.
fn usual_levels(levels: &[[f32; BANDS]], gate: &[f32]) -> [f32; BANDS] {
  let speaking: Vec<&[f32; BANDS]> = levels
    .iter()
    .zip(gate)
    .filter(|(_, &open)| open > 0.9)
    .map(|(levels, _)| levels)
    .collect();
  std::array::from_fn(|band| {
    let mut values: Vec<f32> = speaking.iter().map(|levels| levels[band]).collect();
    if values.is_empty() {
      return f32::INFINITY;
    }
    let loud = (values.len() * 9 / 10).min(values.len() - 1);
    *values.select_nth_unstable_by(loud, f32::total_cmp).1
  })
}

/// Each bin's place among the bands' middles.
fn bin_bands(rate: u32) -> Vec<Option<(usize, usize, f32)>> {
  let middles = band_edges(rate).map(|(from, to)| (from + to) as f32 / 2.0);
  (0..=SIZE / 2)
    .map(|bin| {
      let bin = bin as f32;
      if bin < middles[0] - 1.0 || bin > middles[BANDS - 1] + 1.0 {
        return None;
      }
      let high = middles
        .iter()
        .position(|&middle| middle >= bin)
        .unwrap_or(BANDS - 1);
      let low = high.saturating_sub(1);
      let along = if high == low {
        0.0
      } else {
        ((bin - middles[low]) / (middles[high] - middles[low])).clamp(0.0, 1.0)
      };
      Some((low, high, along))
    })
    .collect()
}
