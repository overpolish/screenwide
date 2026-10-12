// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Plosives softened, in the spectrum. The breath of a "p" or a "b" hitting
//! the microphone is a thump below 200 Hz, up to 120 ms long, far above the
//! low end around it and louder than the voice above it at that moment. A
//! vowel, however deep the voice, keeps most of its energy above 200 Hz, so
//! it is never one. Each thump found has only its frequencies under 250 Hz
//! brought back to the level around them; the consonant itself stays.

use super::declick::decibels;
use super::spectrum::{median, Spectra, Stft};

/// Frames of 43 ms, one every 5.3 ms: fine enough in frequency to tell a
/// thump from a deep voice.
const FRAME: usize = 2_048;
const HOP: usize = 256;
/// The thump's band, and the voice's band it is weighed against.
const THUMP_HZ: (f32, f32) = (20.0, 200.0);
const VOICE_HZ: (f32, f32) = (200.0, 2_000.0);
/// The low end around a moment: the 150 ms either side of it, leaving out
/// the 25 ms beside it, where the thump itself is.
const AROUND_MS: usize = 150;
const BESIDE_MS: usize = 25;
/// How far a thump rises above the low end around it, 15 dB...
const RISES: f32 = 31.6;
/// ... and over the voice above it, 6 dB.
const OVER_VOICE: f32 = 3.98;
const LONGEST_MS: usize = 120;
/// A thump's core: what lies within 20 dB of its peak.
const CORE: f32 = 0.01;
const MERGE_FRAMES: usize = 2;
/// Frequencies under this are brought down, to 3 dB over the level around.
const SOFTENED_BELOW_HZ: f32 = 250.0;
const LEAVE_DB: f32 = 3.0;

pub(super) struct Deplosive {
  rate: u32,
  stft: Stft,
  spectra: Spectra,
  change: Spectra,
  changed: Vec<bool>,
  thump: Vec<f32>,
  voice: Vec<f32>,
  window: Vec<f32>,
  events: Vec<(usize, usize)>,
}

impl Deplosive {
  pub(super) fn new(rate: u32) -> Self {
    Self {
      rate,
      stft: Stft::new(FRAME, HOP),
      spectra: Spectra::default(),
      change: Spectra::default(),
      changed: Vec::new(),
      thump: Vec::new(),
      voice: Vec::new(),
      window: Vec::new(),
      events: Vec::new(),
    }
  }

  fn frames(&self, ms: usize) -> usize {
    (ms * self.rate as usize / 1_000).div_ceil(self.stft.hop())
  }

  fn bin_at(&self, hz: f32) -> usize {
    ((hz / self.rate as f32) * FRAME as f32).round() as usize
  }

  /// The median of `values` over the frames around `frame`, leaving out
  /// those beside it.
  fn around(&mut self, values: &[f32], frame: usize) -> f32 {
    let (reach, beside) = (self.frames(AROUND_MS), self.frames(BESIDE_MS));
    self.window.clear();
    let before = frame.saturating_sub(reach)..frame.saturating_sub(beside);
    let after = (frame + beside + 1).min(values.len())..(frame + reach + 1).min(values.len());
    self
      .window
      .extend(before.chain(after).map(|other| values[other]));
    if self.window.is_empty() {
      return values[frame];
    }
    median(&mut self.window)
  }

  /// Softens the plosives in `samples`. A run handed over must start a
  /// multiple of `HOP` samples into the track.
  pub(super) fn apply(&mut self, samples: &mut [f32]) {
    self.stft.analyze(samples, &mut self.spectra);
    let (bins, frames) = (self.spectra.bins, self.spectra.frames);
    let thump = (self.bin_at(THUMP_HZ.0), self.bin_at(THUMP_HZ.1));
    let voice = (self.bin_at(VOICE_HZ.0), self.bin_at(VOICE_HZ.1));
    self.thump.clear();
    self.voice.clear();
    for frame in 0..frames {
      let values = self.spectra.frame(frame);
      self.thump.push(
        values[thump.0..thump.1]
          .iter()
          .map(|value| value.norm_sqr())
          .sum(),
      );
      self.voice.push(
        values[voice.0..voice.1]
          .iter()
          .map(|value| value.norm_sqr())
          .sum(),
      );
    }
    let thumps = std::mem::take(&mut self.thump);
    self.events.clear();
    let mut start = None;
    for frame in 0..=frames {
      let rises = frame < frames && {
        let around = self.around(&thumps, frame);
        thumps[frame] > around * RISES && thumps[frame] > self.voice[frame] * OVER_VOICE
      };
      match (start, rises) {
        (None, true) => start = Some(frame),
        (Some(from), false) => {
          start = None;
          match self.events.last_mut() {
            Some(last) if from - last.1 <= MERGE_FRAMES => last.1 = frame,
            _ => self.events.push((from, frame)),
          }
        }
        _ => {}
      }
    }
    self.thump = thumps;
    self.change.clear_like(&self.spectra);
    self.changed.clear();
    self.changed.resize(frames, false);
    let edge = FRAME / HOP + 1;
    let softened = self.bin_at(SOFTENED_BELOW_HZ).min(bins);
    let longest = self.frames(LONGEST_MS);
    let thumps = &self.thump;
    // How long a thump lasts is judged by its core, within 20 dB of its peak:
    // in a quiet room its ringing can trail on far longer.
    let core = |&(start, end): &(usize, usize)| {
      let peak = thumps[start..end].iter().copied().fold(0.0, f32::max);
      thumps[start..end]
        .iter()
        .filter(|&&thump| thump >= peak * CORE)
        .count()
    };
    self
      .events
      .retain(|event| core(event) <= longest && event.0 >= edge && event.1 + edge <= frames);
    if self.events.is_empty() {
      return;
    }
    // Each softened frequency's power, frame by frame, for the level around;
    // from the very bottom, where much of a thump's push of air sits.
    let columns: Vec<Vec<f32>> = (0..softened)
      .map(|bin| {
        (0..frames)
          .map(|frame| self.spectra.frame(frame)[bin].norm_sqr())
          .collect()
      })
      .collect();
    for index in 0..self.events.len() {
      let (start, end) = self.events[index];
      for frame in start - 1..end + 1 {
        self.changed[frame] = true;
        for (bin, column) in columns.iter().enumerate() {
          let around = decibels(self.around(column, frame));
          let value = self.spectra.frame(frame)[bin];
          let excess = decibels(value.norm_sqr()) - around - LEAVE_DB;
          if excess > 0.0 {
            self.change.frame_mut(frame)[bin] = value * (10_f32.powf(-excess / 20.0) - 1.0);
          }
        }
      }
    }
    self.stft.add(&self.change, &self.changed, samples);
  }
}
