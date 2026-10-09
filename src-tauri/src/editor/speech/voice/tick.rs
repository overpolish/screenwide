// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Ticks taken down. The lips parting on a word, a dry mouth opening between
//! words, or the crack of a popped "p" put a spike on the microphone a
//! millisecond or two long, reaching from the voice's range up to the top of
//! the band, 12 dB to 40 dB over what is around it above 2 kHz. A wet mouth
//! makes a few of them a few milliseconds apart. The voice behind them, a
//! vowel starting 2 ms to 5 ms later, has little up there, so a tick is told
//! by how far it rises above 2 kHz, and by how fast it is gone: the release
//! of a "t" or a "k" is a hiss 10 ms to 30 ms long, so it raises the level
//! around itself and never stands out from it.
//!
//! A deep, pressed voice puts a spike on every pulse of the vocal cords, each
//! a few milliseconds after the last. Those rise as far over the level
//! between them as a tick does, but never over each other, so a tick must
//! also stand over the loudest moment around it. Ticks close together are
//! judged as one, against what lies outside them all; and once the loudest
//! is found to be a tick, it no longer counts as what is around the next, so
//! a wet click's smaller ticks are not hidden by its first.
//!
//! Each tick found has the whole band turned down over just its length, to
//! the level around it, so a vowel's first cycle under its tail loses no
//! more than a few decibels for a millisecond or two, and a pulse of the
//! voice taken for a tick is barely touched. Auto volume's lift would
//! otherwise carry a spike left even 3 dB over its word to the limiter, well
//! above the word.

mod turn_down;

use super::biquad::Split;
use super::spectrum::median;

/// Samples whose peak is taken together: a third of a millisecond at 48 kHz.
/// A divisor of the runs the cleanup reads, so blocks fall in the same place
/// whichever run they are in.
const BLOCK: usize = 16;
/// Ticks are judged above this frequency.
const ABOVE_HZ: f64 = 2_000.0;
/// What is around a moment: the 20 ms either side of it...
const AROUND_MS: usize = 20;
/// ... leaving out the 2 ms beside it, where the tick itself is.
const BESIDE_MS: usize = 2;
/// A tick rises this far, 12 dB, over the median level around it...
const RISES_DB: f32 = 12.0;
/// ... and lasts while it stays this far, 6 dB, over it...
const HOLDS_DB: f32 = 6.0;
/// ... and stands this far, 4 dB, over the loudest moment around it, which
/// a pulse of the voice does not: the next is as loud, give or take the
/// rise and fall of a syllable.
const STANDS_DB: f32 = 4.0;
/// The longest a tick lasts; a burst longer is a consonant.
const LONGEST_MS: usize = 8;
/// Ticks this close together are judged as one...
const MERGE_MS: usize = 8;
/// ... while they do not go on longer than a wet click does.
const CLUSTER_MS: usize = 25;
/// Below this peak, -66 dB, a burst is in the noise and left alone.
const QUIETEST: f32 = 0.0005;
/// A tick is turned down to the level around it.
const LEAVE: f32 = 1.0;
/// Blocks on either side the turn-down eases over, so it is no click itself.
const EASE_BLOCKS: usize = 2;

/// Ticks close together: the blocks they span, which of the ticks found
/// they are, and the loudest of them.
struct Cluster {
  start: usize,
  end: usize,
  ticks: std::ops::Range<usize>,
  peak: f32,
}

pub(super) struct Ticks {
  rate: u32,
  split: Split,
  high: Vec<f32>,
  /// Each block's peak above the cutoff, in decibels, and over the whole
  /// band, as it is.
  high_peaks: Vec<f32>,
  peaks: Vec<f32>,
  /// How far each block's high peak rises over the median around it.
  rises: Vec<f32>,
  /// The high peaks with each tick found so far brought down to the level
  /// around it, which the next is judged against.
  judged: Vec<f32>,
  window: Vec<f32>,
  gain: Vec<f32>,
  ticks: Vec<(usize, usize)>,
  clusters: Vec<Cluster>,
  order: Vec<usize>,
  taken: Vec<bool>,
}

impl Ticks {
  pub(super) fn new(rate: u32) -> Self {
    Self {
      rate,
      split: Split::new(rate, ABOVE_HZ),
      high: Vec::new(),
      high_peaks: Vec::new(),
      peaks: Vec::new(),
      rises: Vec::new(),
      judged: Vec::new(),
      window: Vec::new(),
      gain: Vec::new(),
      ticks: Vec::new(),
      clusters: Vec::new(),
      order: Vec::new(),
      taken: Vec::new(),
    }
  }

  fn blocks(&self, ms: usize) -> usize {
    (ms * self.rate as usize / 1_000).div_ceil(BLOCK)
  }

  /// The blocks around `start..end`, leaving out those beside it.
  fn around(&self, start: usize, end: usize) -> [std::ops::Range<usize>; 2] {
    let (reach, beside) = (self.blocks(AROUND_MS), self.blocks(BESIDE_MS));
    [start - reach..start - beside, end + beside..end + reach]
  }

  /// The median of `values` around block `block`.
  fn middle(&mut self, values: &[f32], block: usize) -> f32 {
    let [before, after] = self.around(block, block + 1);
    self.window.clear();
    self.window.extend_from_slice(&values[before]);
    self.window.extend_from_slice(&values[after]);
    median(&mut self.window)
  }

  /// Turns down the ticks in `samples`. A run handed over must start a
  /// multiple of `BLOCK` samples into the track.
  pub(super) fn apply(&mut self, samples: &mut [f32]) {
    self.split.high(samples, &mut self.high);
    let peak = |block: &[f32; BLOCK]| block.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()));
    self.high_peaks.clear();
    self.high_peaks.extend(
      self
        .high
        .as_chunks::<BLOCK>()
        .0
        .iter()
        .map(|block| decibels(peak(block))),
    );
    self.peaks.clear();
    self
      .peaks
      .extend(samples.as_chunks::<BLOCK>().0.iter().map(peak));
    let count = self.peaks.len();
    let reach = self.blocks(AROUND_MS);
    let (first, last) = (reach, count.saturating_sub(reach));
    let high_peaks = std::mem::take(&mut self.high_peaks);
    self.rises.clear();
    self.rises.resize(count, f32::NEG_INFINITY);
    for block in first..last {
      if high_peaks[block] > decibels(QUIETEST) {
        self.rises[block] = high_peaks[block] - self.middle(&high_peaks, block);
      }
    }
    self.find(first, last);
    self.gather(&high_peaks);
    self.judge(&high_peaks);
    self.high_peaks = high_peaks;
    if self.taken.iter().any(|&taken| taken) {
      self.turn_down(samples, count);
    }
  }

  /// Blocks rising `RISES_DB` over their surroundings, each stretched to the
  /// blocks beside it that still rise `HOLDS_DB`, while short enough.
  fn find(&mut self, first: usize, last: usize) {
    let longest = self.blocks(LONGEST_MS);
    self.ticks.clear();
    let mut block = first;
    while block < last {
      if self.rises[block] <= RISES_DB {
        block += 1;
        continue;
      }
      let mut start = block;
      while start > first && self.rises[start - 1] > HOLDS_DB {
        start -= 1;
      }
      let mut end = block + 1;
      while end < last && self.rises[end] > HOLDS_DB {
        end += 1;
      }
      if end - start <= longest {
        self.ticks.push((start, end));
      }
      block = end;
    }
  }

  /// Ticks within `MERGE_MS` of each other gathered into clusters, those
  /// too long or too near the run's end dropped.
  fn gather(&mut self, high_peaks: &[f32]) {
    let (merge, longest, reach) = (
      self.blocks(MERGE_MS),
      self.blocks(CLUSTER_MS),
      self.blocks(AROUND_MS),
    );
    self.clusters.clear();
    for (index, &(start, end)) in self.ticks.iter().enumerate() {
      let peak = high_peaks[start..end]
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
      match self.clusters.last_mut() {
        Some(last) if start <= last.end + merge => {
          last.end = end;
          last.ticks.end = index + 1;
          last.peak = last.peak.max(peak);
        }
        _ => self.clusters.push(Cluster {
          start,
          end,
          ticks: index..index + 1,
          peak,
        }),
      }
    }
    self.clusters.retain(|cluster| {
      cluster.end - cluster.start <= longest && cluster.end + reach <= high_peaks.len()
    });
  }

  /// Marks the ticks of each cluster standing `STANDS_DB` over the loudest
  /// moment around it as taken, loudest cluster first, each taken one
  /// brought down to the level around it before the next is judged.
  fn judge(&mut self, high_peaks: &[f32]) {
    self.taken.clear();
    self.taken.resize(self.ticks.len(), false);
    self.judged.clear();
    self.judged.extend_from_slice(high_peaks);
    self.order.clear();
    self.order.extend(0..self.clusters.len());
    let clusters = std::mem::take(&mut self.clusters);
    self
      .order
      .sort_unstable_by(|&a, &b| clusters[b].peak.total_cmp(&clusters[a].peak));
    for &index in &self.order {
      let cluster = &clusters[index];
      let [before, after] = self.around(cluster.start, cluster.end);
      let loudest = before
        .chain(after)
        .map(|block| self.judged[block])
        .fold(f32::NEG_INFINITY, f32::max);
      if cluster.peak - loudest <= STANDS_DB {
        continue;
      }
      for tick in cluster.ticks.clone() {
        self.taken[tick] = true;
        let (start, end) = self.ticks[tick];
        for ((judged, &peak), &rise) in self.judged[start..end]
          .iter_mut()
          .zip(&high_peaks[start..end])
          .zip(&self.rises[start..end])
        {
          *judged = peak - rise.max(0.0);
        }
      }
    }
    self.clusters = clusters;
  }
}

fn decibels(amplitude: f32) -> f32 {
  20.0 * (amplitude + 1e-9).log10()
}
