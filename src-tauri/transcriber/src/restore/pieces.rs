// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The audio kept for restoring, and the pieces it goes through the model
//! in. The model hears about ten seconds at a time; each piece is cut at the
//! quietest moment near where it would end, so the hard join between two
//! pieces lies where there is next to nothing to hear. The pieces depend
//! only on the audio and the stretches asked for, so a job taken up again
//! cuts them in the same places.

use std::ops::Range;

use screenwide_transcriber::SAMPLE_RATE;

const RATE: usize = SAMPLE_RATE as usize;
/// How long a piece is meant to be.
const PIECE: usize = 10 * RATE;
/// How far either side of where a piece would end its quietest moment is
/// looked for.
const SEARCH: usize = RATE / 2;
/// The level is compared over blocks of 10 ms.
const BLOCK: usize = RATE / 100;

/// One piece: its samples, and the stretch it lies in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Piece {
  pub(super) samples: Range<usize>,
  pub(super) stretch: Range<usize>,
}

/// The stretches of `samples` to restore, sorted, apart and inside the
/// audio whatever was asked.
pub(super) struct Kept<'a> {
  samples: &'a [f32],
  stretches: Vec<Range<usize>>,
}

impl<'a> Kept<'a> {
  pub(super) fn new(samples: &'a [f32], asked: &[[u64; 2]]) -> Self {
    let total = samples.len();
    let mut stretches: Vec<Range<usize>> = Vec::with_capacity(asked.len());
    for &[start, end] in asked {
      let start = to_usize(start).max(stretches.last().map_or(0, |last| last.end));
      let end = to_usize(end).min(total);
      if end > start {
        stretches.push(start..end);
      }
    }
    Self { samples, stretches }
  }

  pub(super) fn total(&self) -> usize {
    self.stretches.iter().map(ExactSizeIterator::len).sum()
  }

  pub(super) fn pieces(&self) -> Vec<Piece> {
    let mut pieces = Vec::new();
    for stretch in &self.stretches {
      let mut start = stretch.start;
      // The last piece takes up to half a piece more rather than leaving a
      // sliver too short to be heard well.
      while stretch.end - start > PIECE + PIECE / 2 {
        let end = self.quietest(start + PIECE);
        pieces.push(Piece {
          samples: start..end,
          stretch: stretch.clone(),
        });
        start = end;
      }
      pieces.push(Piece {
        samples: start..stretch.end,
        stretch: stretch.clone(),
      });
    }
    pieces
  }

  /// The middle of the quietest block within [`SEARCH`] of `around`.
  fn quietest(&self, around: usize) -> usize {
    let from = around - SEARCH;
    let energy = |block: usize| -> f32 {
      self.samples[from + block * BLOCK..from + (block + 1) * BLOCK]
        .iter()
        .map(|sample| sample * sample)
        .sum()
    };
    let quietest = (0..2 * SEARCH / BLOCK)
      .min_by(|&a, &b| energy(a).total_cmp(&energy(b)))
      .unwrap_or(0);
    from + quietest * BLOCK + BLOCK / 2
  }

  /// Up to `count` kept samples before `at`, in order.
  pub(super) fn before(&self, at: usize, count: usize) -> Vec<f32> {
    let mut taken: Vec<&[f32]> = Vec::new();
    let mut left = count;
    for stretch in self.stretches.iter().rev() {
      if left == 0 {
        break;
      }
      let end = stretch.end.min(at);
      if end <= stretch.start {
        continue;
      }
      let start = end.saturating_sub(left).max(stretch.start);
      taken.push(&self.samples[start..end]);
      left -= end - start;
    }
    taken.into_iter().rev().flatten().copied().collect()
  }

  /// Up to `count` kept samples from `at` on, in order.
  pub(super) fn after(&self, at: usize, count: usize) -> Vec<f32> {
    let mut taken = Vec::with_capacity(count);
    for stretch in &self.stretches {
      if taken.len() == count {
        break;
      }
      let start = stretch.start.max(at);
      if start >= stretch.end {
        continue;
      }
      let end = stretch.end.min(start + count - taken.len());
      taken.extend_from_slice(&self.samples[start..end]);
    }
    taken
  }

  pub(super) fn samples(&self, range: Range<usize>) -> &[f32] {
    &self.samples[range]
  }
}

fn to_usize(samples: u64) -> usize {
  usize::try_from(samples).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests;
