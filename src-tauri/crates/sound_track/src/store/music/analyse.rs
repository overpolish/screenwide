// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding a sound's beat in its samples: where the spectrum rises from one
//! moment to the next, notes and drums start (spectral flux). Those starts
//! repeat at the beat, so the tempo is the period at which the flux best
//! matches itself, leaning towards common tempos around 120 BPM, and
//! refined, with where the first beat falls, to the grid the flux lines up
//! with best across the whole sound. A sound whose flux barely favours its
//! grid over any other phase has no steady beat to blend on: rubato,
//! ambient, speech.

use std::f64::consts::TAU;

use realfft::RealFftPlanner;

use super::{Beat, Music};

/// Samples per frame of the flux, and between frames: 64 ms every 10 ms at
/// the rate the library reads sounds.
const FLUX_FRAME: usize = 512;
const FLUX_HOP: usize = 80;
/// The slowest and fastest tempos looked for, in BPM.
const SLOWEST_BPM: f64 = 60.0;
const FASTEST_BPM: f64 = 200.0;
/// The tempo the search leans towards, in BPM, and how far either side it
/// leans, in octaves.
const LIKELY_BPM: f64 = 120.0;
const LIKELY_OCTAVES: f64 = 1.0;
/// How much more the flux must favour the beat's phase than the average
/// phase for the beat to be steady.
const STEADY: f64 = 2.5;

/// The beat of the mono `samples` at `rate`.
pub(crate) fn analyse(samples: &[f32], rate: usize) -> Music {
  Music {
    beat: beat(&flux(samples), rate),
    vocals: None,
  }
}

/// How much the spectrum rises into each frame, its local average taken off
/// so only the starts stand out.
fn flux(samples: &[f32]) -> Vec<f64> {
  let mut planner = RealFftPlanner::<f64>::new();
  let fft = planner.plan_fft_forward(FLUX_FRAME);
  let window = hann(FLUX_FRAME);
  let mut input = fft.make_input_vec();
  let mut spectrum = fft.make_output_vec();
  let mut previous = vec![0.0_f64; spectrum.len()];
  let mut flux = Vec::with_capacity(samples.len() / FLUX_HOP);
  let mut start = 0;
  while start + FLUX_FRAME <= samples.len() {
    for (index, slot) in input.iter_mut().enumerate() {
      *slot = f64::from(samples[start + index]) * window[index];
    }
    fft
      .process(&mut input, &mut spectrum)
      .expect("the buffers are the plan's own sizes");
    let mut rise = 0.0;
    for (bin, value) in spectrum.iter().enumerate() {
      let level = (1.0 + 100.0 * value.norm()).ln();
      rise += (level - previous[bin]).max(0.0);
      previous[bin] = level;
    }
    // The first frame has nothing before it to rise from.
    flux.push(if start == 0 { 0.0 } else { rise });
    start += FLUX_HOP;
  }
  let half = 8;
  (0..flux.len())
    .map(|frame| {
      let around = &flux[frame.saturating_sub(half)..(frame + half).min(flux.len())];
      #[expect(clippy::cast_precision_loss, reason = "a count of frames")]
      let mean = around.iter().sum::<f64>() / around.len() as f64;
      (flux[frame] - mean).max(0.0)
    })
    .collect()
}

/// The flux at a frame between frames.
fn at(flux: &[f64], frame: f64) -> f64 {
  #[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a frame index"
  )]
  let below = frame.floor() as usize;
  let share = frame - frame.floor();
  match (flux.get(below), flux.get(below + 1)) {
    (Some(low), Some(high)) => low * (1.0 - share) + high * share,
    (Some(low), None) => *low,
    _ => 0.0,
  }
}

/// How well beats every `period` frames from `phase` line up with `flux`:
/// the mean flux on them.
fn on_grid(flux: &[f64], period: f64, phase: f64) -> f64 {
  #[expect(clippy::cast_precision_loss, reason = "a count of frames")]
  let end = flux.len() as f64;
  let (mut sum, mut beats) = (0.0, 0.0);
  let mut frame = phase;
  while frame < end - 1.0 {
    sum += at(flux, frame);
    beats += 1.0;
    frame += period;
  }
  if beats > 0.0 {
    sum / beats
  } else {
    0.0
  }
}

/// The steady beat in `flux`, frames `FLUX_HOP` samples apart at `rate`, if
/// it has one.
fn beat(flux: &[f64], rate: usize) -> Option<Beat> {
  #[expect(clippy::cast_precision_loss, reason = "rates and sizes")]
  let frames_per_second = rate as f64 / FLUX_HOP as f64;
  let lag_of = |bpm: f64| 60.0 * frames_per_second / bpm;
  #[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "lags in frames"
  )]
  let (shortest, longest) = (
    lag_of(FASTEST_BPM) as usize,
    lag_of(SLOWEST_BPM).ceil() as usize,
  );
  if flux.len() < longest * 8 {
    return None;
  }
  // The tempo the flux repeats at, leaning towards the likely tempos.
  let correlation = |lag: usize| -> f64 { flux.iter().zip(&flux[lag..]).map(|(a, b)| a * b).sum() };
  #[expect(clippy::cast_precision_loss, reason = "lags in frames")]
  let leaning = |lag: usize| {
    let bpm = 60.0 * frames_per_second / lag as f64;
    (-0.5 * ((bpm / LIKELY_BPM).log2() / LIKELY_OCTAVES).powi(2)).exp()
  };
  let best = (shortest..=longest)
    .max_by(|a, b| (correlation(*a) * leaning(*a)).total_cmp(&(correlation(*b) * leaning(*b))))?;
  // The period and phase, to a hundredth of a frame, the grid lines up
  // with best over the whole sound.
  #[expect(clippy::cast_precision_loss, reason = "lags in frames")]
  let rough = best as f64;
  let mut found = (rough, 0.0, f64::MIN);
  for step in -100..=100 {
    let period = rough * (1.0 + f64::from(step) * 0.000_3);
    let mut phase = 0.0;
    while phase < period {
      let fit = on_grid(flux, period, phase);
      if fit > found.2 {
        found = (period, phase, fit);
      }
      phase += 0.25;
    }
  }
  let (period, phase, fit) = found;
  // Steady where the grid's phase stands out from the average one.
  let phases = 16;
  let average = (0..phases)
    .map(|index| on_grid(flux, period, period * f64::from(index) / f64::from(phases)))
    .sum::<f64>()
    / f64::from(phases);
  // A frame's flux peaks as an onset passes the steepest rise of its
  // window, three quarters of the way into it.
  #[expect(clippy::cast_precision_loss, reason = "rates and sizes")]
  let lag_ms = (FLUX_FRAME * 3 / 4) as f64 * 1_000.0 / rate as f64;
  let period_ms = period * 1_000.0 / frames_per_second;
  (average > 0.0 && fit / average >= STEADY).then(|| Beat {
    period_ms,
    first_ms: (phase * 1_000.0 / frames_per_second + lag_ms) % period_ms,
  })
}

fn hann(size: usize) -> Vec<f64> {
  #[expect(clippy::cast_precision_loss, reason = "a window's shape")]
  (0..size)
    .map(|index| 0.5 - 0.5 * (TAU * index as f64 / size as f64).cos())
    .collect()
}

#[cfg(test)]
mod tests;
