// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The room's noise measured where nobody speaks, then turned down band by
//! band inside speech as well. The enhancer leaves a frame it judges clear
//! of noise exactly as it was, which is most frames of a word, so the static
//! under the voice stays. With the noise's own spectrum known, each band of
//! each frame comes down by how much of it is noise.

use std::io::Read;

use realfft::num_complex::Complex32;

use super::super::analysis::SpeechMap;
use super::super::framer::{Framer, HOP, SIZE};
use super::super::silences::{speech_ranges, KEEP_AFTER_SPEECH_MS, KEEP_BEFORE_SPEECH_MS};

/// How many times the noise's power is taken off each band. Taking it off
/// twice brings down a band where voice and noise are about even, which is
/// where static shows through.
const OVER: f32 = 2.0;
/// The most a band comes down, 12 dB. Further, and what is left of the voice
/// in a noisy band starts to sound watery.
const FLOOR: f32 = 0.25;
/// Bands either side that a band's gain is averaged with, so neighbours do
/// not come down by very different amounts, which is what makes the warble.
const SPREAD: usize = 2;
/// A band coming back up moves this share of the way each frame, so a gain
/// that dips for one frame does not flutter.
const RISE: f32 = 1.0 / 3.0;
/// Frames of pause needed to trust the measure: half a second.
const FEWEST_FRAMES: usize = 47;
/// Samples read at a time while the noise is measured.
const CHUNK: usize = 1 << 16;

/// The noise's average power in each band of a frame.
pub(super) struct Profile(Vec<f32>);

impl Profile {
  /// The noise in the pauses of the `length` samples `reader` holds, at
  /// `rate`, where `speech` hears no speech and outside the margins Remove
  /// silences keeps around it. None when there is too little pause to tell
  /// the noise by, and the track is left as the enhancer made it.
  pub(super) fn measure(
    reader: &mut impl Read,
    length: u64,
    speech: &SpeechMap,
    rate: u32,
  ) -> Result<Option<Self>, String> {
    let per_ms = f64::from(rate) / 1_000.0;
    let spoken: Vec<(f64, f64)> = speech_ranges(speech, length as f64 / per_ms)
      .into_iter()
      .map(|(start, end)| {
        (
          (start - KEEP_BEFORE_SPEECH_MS) * per_ms,
          (end + KEEP_AFTER_SPEECH_MS) * per_ms,
        )
      })
      .collect();
    let mut framer = Framer::new();
    let mut power = vec![0.0_f64; SIZE / 2 + 1];
    let mut frames = 0;
    let mut next = 0;
    let mut on_frame = |frame: usize, spectrum: &mut [Complex32]| {
      let end = ((frame + 1) * HOP) as f64;
      let start = end - SIZE as f64;
      while spoken.get(next).is_some_and(|&(_, until)| until <= start) {
        next += 1;
      }
      let in_speech = spoken.get(next).is_some_and(|&(from, _)| from < end);
      // A frame reaching before the start is partly made up, and a frame of
      // digital silence is the recording not yet running, not the room.
      if start < 0.0 || in_speech || spectrum.iter().all(|bin| bin.norm_sqr() == 0.0) {
        return;
      }
      for (total, bin) in power.iter_mut().zip(spectrum.iter()) {
        *total += f64::from(bin.norm_sqr());
      }
      frames += 1;
    };
    let mut bytes = vec![0_u8; CHUNK * 4];
    let mut samples = Vec::with_capacity(CHUNK);
    loop {
      let read = super::fill(reader, &mut bytes)?;
      if read < 4 {
        break;
      }
      samples.clear();
      samples.extend(
        bytes[..read]
          .as_chunks::<4>()
          .0
          .iter()
          .map(|sample| f32::from_le_bytes(*sample)),
      );
      framer.feed(&samples, None, &mut on_frame);
    }
    if frames < FEWEST_FRAMES {
      return Ok(None);
    }
    Ok(Some(Self(
      power
        .into_iter()
        .map(|total| (total / frames as f64) as f32)
        .collect(),
    )))
  }
}

/// A track streamed through with its noise turned down as a profile says.
pub(super) struct Denoiser {
  noise: Vec<f32>,
  framer: Framer,
  /// Each band's gain as the noise alone would have it this frame.
  wanted: Vec<f32>,
  /// Each band's gain as applied, smoothed across bands and over time.
  gains: Vec<f32>,
  made: Vec<f32>,
  /// Samples fed from the track, which is how many go out.
  read: usize,
  /// Samples out of the framer, the first `SIZE - HOP` of which lie before
  /// the start.
  out: usize,
}

impl Denoiser {
  pub(super) fn new(profile: Profile) -> Self {
    Self {
      noise: profile.0,
      framer: Framer::new(),
      wanted: vec![1.0; SIZE / 2 + 1],
      gains: vec![1.0; SIZE / 2 + 1],
      made: Vec::new(),
      read: 0,
      out: 0,
    }
  }

  /// Turns the noise in `samples`, which follow those fed last, down,
  /// putting in `out` what is ready, which lags behind.
  pub(super) fn feed(&mut self, samples: &[f32], out: &mut Vec<f32>) {
    self.read += samples.len();
    self.run(samples, out);
  }

  /// Puts in `out` the rest of what was fed, so as many samples have come
  /// out as went in.
  pub(super) fn finish(&mut self, out: &mut Vec<f32>) {
    self.run(&[0.0; SIZE], out);
  }

  fn run(&mut self, samples: &[f32], out: &mut Vec<f32>) {
    let Self {
      noise,
      framer,
      wanted,
      gains,
      made,
      ..
    } = self;
    made.clear();
    framer.feed(samples, Some(made), &mut |_, spectrum| {
      turn_down(spectrum, noise, wanted, gains)
    });
    out.clear();
    let latency = SIZE - HOP;
    for (at, &sample) in made.iter().enumerate() {
      let position = self.out + at;
      if position >= latency && position - latency < self.read {
        out.push(sample);
      }
    }
    self.out += made.len();
  }
}

fn turn_down(spectrum: &mut [Complex32], noise: &[f32], wanted: &mut [f32], gains: &mut [f32]) {
  for ((wanted, bin), noise) in wanted.iter_mut().zip(spectrum.iter()).zip(noise) {
    let share = noise / bin.norm_sqr().max(f32::MIN_POSITIVE);
    *wanted = (1.0 - OVER * share).max(0.0).sqrt().max(FLOOR);
  }
  let last = wanted.len() - 1;
  for (band, gain) in gains.iter_mut().enumerate() {
    let near = &wanted[band.saturating_sub(SPREAD)..=(band + SPREAD).min(last)];
    let smooth = near.iter().sum::<f32>() / near.len() as f32;
    // Down at once, so the noise at a word's start is caught, and up gently.
    *gain = if smooth < *gain {
      smooth
    } else {
      *gain + (smooth - *gain) * RISE
    };
  }
  for (bin, gain) in spectrum.iter_mut().zip(gains.iter()) {
    *bin *= *gain;
  }
}
