// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system audio's channels on their way through their framers while it
//! is made to make way: each part turned down by its plan, and the parts put
//! back together.

use std::io::Write;

use super::super::super::framer::{Framer, HOP, SIZE};
use super::super::plan::Plan;
use super::SIDES;

/// The channels of every part on their way through their framers, one
/// framer to each, a part's two sides together.
pub(super) struct Sides {
  /// Samples of each side read from the track, which is how many go out.
  pub(super) read: usize,
  /// Samples of each side that came out of the framers, the first
  /// `SIZE - HOP` of which lie before the start.
  made: usize,
  split: Vec<Vec<f32>>,
  out: Vec<Vec<f32>>,
  bytes: Vec<u8>,
}

impl Sides {
  pub(super) fn new(channels: usize) -> Self {
    Self {
      read: 0,
      made: 0,
      split: vec![Vec::new(); channels],
      out: vec![Vec::new(); channels],
      bytes: Vec::new(),
    }
  }

  /// Feeds interleaved `samples` through `framers`, each part's turned down
  /// as its plan in `plans` says, counting them as read from the track when
  /// `from_track`.
  pub(super) fn feed(
    &mut self,
    samples: &[f32],
    framers: &mut [Framer],
    plans: &mut [Plan],
    from_track: bool,
  ) {
    let channels = self.split.len();
    for (channel, split) in self.split.iter_mut().enumerate() {
      split.clear();
      split.extend(samples.iter().skip(channel).step_by(channels));
    }
    if from_track {
      self.read += self.split[0].len();
    }
    let parts = framers.iter_mut().zip(&self.split).zip(&mut self.out);
    for (channel, ((framer, split), out)) in parts.enumerate() {
      let plan = &mut plans[channel / SIDES];
      framer.feed(split, Some(out), &mut |frame, spectrum| {
        plan.apply(frame, spectrum)
      });
    }
  }

  /// Writes what has come out of every channel since last time, each side
  /// the sum of the parts', leaving out what lies before the start and after
  /// the end.
  pub(super) fn write(&mut self, input: &mut impl Write) -> Result<(), String> {
    let latency = SIZE - HOP;
    let length = self.out.iter().map(Vec::len).min().unwrap_or(0);
    self.bytes.clear();
    for at in 0..length {
      let made = self.made + at;
      if made < latency || made - latency >= self.read {
        continue;
      }
      for side in 0..SIDES {
        let sum: f32 = self
          .out
          .iter()
          .skip(side)
          .step_by(SIDES)
          .map(|out| out[at])
          .sum();
        self.bytes.extend_from_slice(&sum.to_le_bytes());
      }
    }
    self.made += length;
    for out in &mut self.out {
      out.drain(..length);
    }
    input
      .write_all(&self.bytes)
      .map_err(|error| error.to_string())
  }
}
