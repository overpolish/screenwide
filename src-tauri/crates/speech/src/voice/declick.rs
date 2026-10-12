// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Mouth clicks taken out, in the spectrum. A click is a moment, up to 25 ms
//! long, where most frequencies between 1.5 kHz and 16 kHz jump far above
//! the level around them. Each one found has only the frequencies it raised
//! brought back down, so the voice beneath a click is kept. Wet clicks, a
//! few sharp ticks and a smack together, are one event while they stay
//! short.
//!
//! A consonant can look like a click, so a burst is left alone when it is
//! where something starts (more sound straight after it than before), when
//! it does not stand out from both sides, or when a vowel follows it within
//! 25 ms, as one follows the release of a "t" or a "k".

use realfft::num_complex::Complex32;

use super::spectrum::{median, Spectra, Stft};

/// Frames of 5.3 ms, one every 1.3 ms.
const FRAME: usize = 256;
const HOP: usize = 64;
/// Frames whose power is averaged into one block of the level around.
const BLOCK: usize = 4;
/// Blocks either side whose median is the level around a block, 43 ms...
const SPREAD: usize = 8;
/// ... leaving out the block either side, where the click itself spreads.
const BESIDE: usize = 1;
/// Clicks are looked for between these frequencies: above the voice's body,
/// and below where a compressed recording has nothing left.
const ABOVE_HZ: f32 = 1_500.0;
const BELOW_HZ: f32 = 16_000.0;
/// Whether a burst stands out, and from what, is judged above 3 kHz, where
/// a held vowel has little of its own to hide a click in.
const JUDGED_ABOVE_HZ: f32 = 3_000.0;
/// How far above the level around a frequency has to jump, 12 dB...
const JUMP_DB: f32 = 12.0;
/// ... in at least this share of the frequencies looked at.
const SHARE: f32 = 0.3;
const LONGEST_MS: usize = 25;
/// Bursts this many frames apart or closer are one event.
const MERGE_FRAMES: usize = 2;
/// How far a click stands out from the 40 ms on either side, 12 dB.
const STANDS: f32 = 15.85;
const SIDE_MS: usize = 40;
/// Followed by this much more than came before it, 6 dB, within 8 ms, it
/// starts something rather than interrupting it...
const STARTS: f32 = 3.98;
const STARTS_MS: usize = 8;
/// ... unless what follows is 20 dB under it.
const TRAILS: f32 = 0.01;
/// The voice, between 150 Hz and 1 kHz, rising this much, 10 dB, within
/// 25 ms of a burst, against the 60 ms before it, means a vowel follows.
const VOWEL_RISES: f32 = 10.0;
const VOWEL_WITHIN_MS: usize = 25;
const VOWEL_BEFORE_MS: usize = 60;
/// What is brought down is brought to 3 dB over the level around it.
const LEAVE_DB: f32 = 3.0;
/// A burst that reaches full scale is the recording clipping, not a click.
const CLIPPED: f32 = 0.99;

pub(super) struct Declicker {
  rate: u32,
  stft: Stft,
  spectra: Spectra,
  change: Spectra,
  changed: Vec<bool>,
  /// Each frame's power in decibels, frame by frame.
  levels: Vec<f32>,
  /// Each block's summed power, and the level around it in decibels.
  blocks: Vec<f32>,
  around: Vec<f32>,
  window: Vec<f32>,
  /// Each frame's power where a burst is judged, and in the voice.
  high: Vec<f32>,
  voice: Vec<f32>,
  events: Vec<(usize, usize)>,
}

impl Declicker {
  pub(super) fn new(rate: u32) -> Self {
    Self {
      rate,
      stft: Stft::new(FRAME, HOP),
      spectra: Spectra::default(),
      change: Spectra::default(),
      changed: Vec::new(),
      levels: Vec::new(),
      blocks: Vec::new(),
      around: Vec::new(),
      window: Vec::new(),
      high: Vec::new(),
      voice: Vec::new(),
      events: Vec::new(),
    }
  }

  /// Takes the clicks out of `samples`. A run handed over must start a
  /// multiple of `FRAME` samples into the track, so a click is measured the
  /// same way whichever run it falls in.
  pub(super) fn apply(&mut self, samples: &mut [f32]) {
    self.stft.analyze(samples, &mut self.spectra);
    self.measure();
    self.change.clear_like(&self.spectra);
    self.changed.clear();
    self.changed.resize(self.spectra.frames, false);
    self.events.clear();
    let mut start = None;
    for frame in 0..=self.spectra.frames {
      let jumped = frame < self.spectra.frames && self.jumped(frame);
      match (start, jumped) {
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
    for index in 0..self.events.len() {
      let (from, to) = self.events[index];
      self.repair(from, to, samples);
    }
    self.stft.add(&self.change, &self.changed, samples);
  }

  fn frames(&self, ms: usize) -> usize {
    (ms * self.rate as usize / 1_000).div_ceil(HOP)
  }

  fn bin_at(&self, hz: f32) -> usize {
    ((hz / self.rate as f32) * FRAME as f32).round() as usize
  }

  /// Each frame's level, each block's level around it, and each frame's
  /// power in the bands the checks listen to.
  fn measure(&mut self) {
    let (bins, frames) = (self.spectra.bins, self.spectra.frames);
    let (judged, below) = (
      self.bin_at(JUDGED_ABOVE_HZ),
      self.bin_at(BELOW_HZ).min(bins),
    );
    let (voice_from, voice_to) = (self.bin_at(150.0), self.bin_at(1_000.0));
    self.levels.clear();
    self.high.clear();
    self.voice.clear();
    for frame in 0..frames {
      let values = self.spectra.frame(frame);
      self
        .levels
        .extend(values.iter().map(|value| decibels(value.norm_sqr())));
      self.high.push(mean_power(&values[judged..below]));
      self.voice.push(mean_power(&values[voice_from..voice_to]));
    }
    let count = frames.div_ceil(BLOCK);
    self.blocks.clear();
    self.blocks.resize(count * bins, 0.0);
    for frame in 0..frames {
      let block = frame / BLOCK;
      for (bin, value) in self.spectra.frame(frame).iter().enumerate() {
        self.blocks[block * bins + bin] += value.norm_sqr();
      }
    }
    self.around.clear();
    self.around.resize(count * bins, decibels(0.0));
    for block in 0..count {
      let before = block.saturating_sub(SPREAD)..block.saturating_sub(BESIDE);
      let after = (block + BESIDE + 1).min(count)..(block + SPREAD + 1).min(count);
      for bin in 0..bins {
        self.window.clear();
        self.window.extend(
          before
            .clone()
            .chain(after.clone())
            .map(|other| self.blocks[other * bins + bin]),
        );
        if !self.window.is_empty() {
          self.around[block * bins + bin] = decibels(median(&mut self.window) / BLOCK as f32);
        }
      }
    }
  }

  /// The level around bin `bin` of frame `frame`, in decibels.
  fn around(&self, frame: usize, bin: usize) -> f32 {
    self.around[(frame / BLOCK) * self.spectra.bins + bin]
  }

  /// Whether most frequencies looked at jump in frame `frame`.
  fn jumped(&self, frame: usize) -> bool {
    let bins = self.spectra.bins;
    let (above, below) = (self.bin_at(ABOVE_HZ), self.bin_at(BELOW_HZ).min(bins));
    let jumps = (above..below)
      .filter(|&bin| self.levels[frame * bins + bin] - self.around(frame, bin) > JUMP_DB)
      .count();
    jumps as f32 > SHARE * (below - above) as f32
  }

  fn middle(&mut self, range: std::ops::Range<usize>) -> f32 {
    self.window.clear();
    self.window.extend_from_slice(&self.high[range]);
    median(&mut self.window)
  }

  /// Brings the frequencies the burst over frames `start..end` raised back
  /// down, if it is a click.
  fn repair(&mut self, start: usize, end: usize, samples: &[f32]) {
    let side = self.frames(SIDE_MS);
    let vowel_before = self.frames(VOWEL_BEFORE_MS);
    let reach = FRAME / HOP + 1 + side.max(vowel_before);
    if end - start > self.frames(LONGEST_MS) || start < reach || end + reach > self.spectra.frames {
      return;
    }
    let before = self.middle(start - side..start - 1);
    let after = self.middle(end + 1..end + side);
    let near = self.middle(end..end + self.frames(STARTS_MS));
    let peak = self.high[start..end].iter().copied().fold(0.0, f32::max);
    if peak < before.max(after) * STANDS || (near > before * STARTS && near > peak * TRAILS) {
      return;
    }
    let voice_before = self.voice[start - vowel_before..start - 2]
      .iter()
      .sum::<f32>()
      / (vowel_before - 2) as f32;
    let voice_after = self.voice[end..end + self.frames(VOWEL_WITHIN_MS)]
      .iter()
      .copied()
      .fold(0.0, f32::max);
    if voice_after > voice_before * VOWEL_RISES + f32::MIN_POSITIVE {
      return;
    }
    let (from, to) = (start - 1, end + 1);
    let first = self.stft.span(from).0.max(0) as usize;
    let last = (self.stft.span(to - 1).1 as usize).min(samples.len());
    if samples[first..last]
      .iter()
      .any(|sample| sample.abs() >= CLIPPED)
    {
      return;
    }
    let (lowest, bins) = (self.bin_at(ABOVE_HZ / 2.0), self.spectra.bins);
    for frame in from..to {
      self.changed[frame] = true;
      for bin in lowest..bins {
        let excess = self.levels[frame * bins + bin] - self.around(frame, bin) - LEAVE_DB;
        if excess > 0.0 {
          let value = self.spectra.frame(frame)[bin];
          self.change.frame_mut(frame)[bin] = value * (10_f32.powf(-excess / 20.0) - 1.0);
        }
      }
    }
  }
}

pub(super) fn decibels(power: f32) -> f32 {
  10.0 * (power + 1e-18).log10()
}

pub(super) fn mean_power(values: &[Complex32]) -> f32 {
  values.iter().map(Complex32::norm_sqr).sum::<f32>() / values.len().max(1) as f32
}
