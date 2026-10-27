// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Measuring each sampled frame against the ones before it.

use std::collections::VecDeque;

/// The size each frame is measured at, in pixels. Small enough that a
/// recording's worth costs nothing to measure, large enough that a dialog or
/// a line of text still covers whole cells: at 128 wide a 3600-wide screen
/// is 28 of its pixels to a cell.
pub const SAMPLE_SIZE: (usize, usize) = (128, 80);

/// How far a cell's grey must move, out of 255, to count as changed: above
/// what scaling and compression leave on a still screen.
const CHANGE_LEVEL: u8 = 12;

/// How many frames back novelty looks, and how many of the newest it leaves
/// out, at ten a second: a frame unlike everything from three seconds to
/// half a second ago is new, where one that only moves or comes back to
/// what was there is not.
const NOVELTY_BACK: usize = 30;
const NOVELTY_SKIP: usize = 5;

/// What one sampled moment of the picture did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PictureReading {
  /// When, in milliseconds of the recording.
  pub ms: u64,
  /// The share of the picture, 0 to 1, that changed since the frame before.
  pub changed: f32,
  /// The share of the picture, 0 to 1, unlike the closest of the frames
  /// from the seconds before: low for motion and for a return to an earlier
  /// screen, high for a screen not seen just before.
  pub novelty: f32,
}

/// What a stretch of readings says the picture was doing, judged against
/// the recording's own level (`states`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureState {
  /// Nothing changed, or little against what usually does: a still screen,
  /// a caret, a game's menu, loading or death screen.
  Still,
  /// Something moved: typing, scrolling, a spinner the size of a dialog,
  /// play in a game.
  Moving,
  /// Much of the picture is new, and newer than the recording usually is:
  /// another page, another app, a sheet, a game's next level.
  New,
}

/// Measures frames in order, keeping the last few seconds to compare with.
pub struct Measure {
  recent: VecDeque<Vec<u8>>,
  ms_per_frame: f64,
  count: u64,
}

impl Measure {
  pub fn new(frames_per_second: f64) -> Self {
    Self {
      recent: VecDeque::with_capacity(NOVELTY_BACK + 1),
      ms_per_frame: 1000.0 / frames_per_second,
      count: 0,
    }
  }

  /// The reading for the next frame, `SAMPLE_SIZE` grey bytes row by row.
  pub fn next(&mut self, frame: Vec<u8>) -> PictureReading {
    let changed = self.recent.back().map_or(0.0, |last| share(last, &frame));
    let older = self.recent.len().saturating_sub(NOVELTY_SKIP);
    let novelty = if older == 0 {
      changed
    } else {
      self
        .recent
        .iter()
        .take(older)
        .map(|earlier| share(earlier, &frame))
        .fold(1.0, f32::min)
    };
    let ms = (self.count as f64 * self.ms_per_frame).round() as u64;
    self.count += 1;
    if self.recent.len() == NOVELTY_BACK {
      self.recent.pop_front();
    }
    self.recent.push_back(frame);
    PictureReading {
      ms,
      changed,
      novelty,
    }
  }
}

/// The share of cells that differ between two frames.
fn share(first: &[u8], second: &[u8]) -> f32 {
  let differing = first
    .iter()
    .zip(second)
    .filter(|(a, b)| a.abs_diff(**b) > CHANGE_LEVEL)
    .count();
  differing as f32 / first.len().max(1) as f32
}
