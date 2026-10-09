// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ticks found turned down, each to the level around it.

use super::{Ticks, BLOCK, EASE_BLOCKS, LEAVE};

impl Ticks {
  /// Turns the ticks marked as taken down in `samples`, which hold `count`
  /// whole blocks.
  pub(super) fn turn_down(&mut self, samples: &mut [f32], count: usize) {
    self.gain.clear();
    self.gain.resize(count, 1.0);
    let peaks = std::mem::take(&mut self.peaks);
    for index in 0..self.ticks.len() {
      if !self.taken[index] {
        continue;
      }
      let (start, end) = self.ticks[index];
      for block in start..end {
        let target = self.middle(&peaks, block) * LEAVE;
        self.gain[block] = self.gain[block].min(target / peaks[block].max(f32::MIN_POSITIVE));
      }
      for step in 1..=EASE_BLOCKS {
        let eased = step as f32 / (EASE_BLOCKS + 1) as f32;
        let edge = self.gain[start];
        self.gain[start - step] = self.gain[start - step].min(edge + (1.0 - edge) * eased);
        let edge = self.gain[end - 1];
        self.gain[end - 1 + step] = self.gain[end - 1 + step].min(edge + (1.0 - edge) * eased);
      }
    }
    self.peaks = peaks;
    // A block's gain holds over all of it, and where the next block's is
    // lower, runs down to it from the block's middle, so no tick peeks out
    // at an edge and no turn-down is a step.
    let half = BLOCK / 2;
    for (at, sample) in samples[..count * BLOCK].iter_mut().enumerate() {
      let (block, offset) = (at / BLOCK, at % BLOCK);
      let (from, to, along) = if offset < half {
        (
          block.saturating_sub(1),
          block,
          (offset + half) as f32 / BLOCK as f32,
        )
      } else {
        (
          block,
          (block + 1).min(count - 1),
          (offset - half) as f32 / BLOCK as f32,
        )
      };
      let gain = self.gain[from] + (self.gain[to] - self.gain[from]) * along;
      *sample *= gain.min(self.gain[block]);
    }
  }
}
