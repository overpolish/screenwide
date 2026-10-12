// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which frame of the cursor and keyboard grids an exported frame shows.
//! Both are evaluated once per sixtieth of a second of the source ahead of
//! the export, and each exported frame picks the one current at its time.

use super::macos::native::CURSOR_FRAME_RATE;

/// How far before a grid timestamp a frame's time may land and still select
/// it. Times arrive in whole microseconds, mapped through the edit, so a
/// frame exactly on a grid timestamp - a third of a microsecond past a whole
/// one - can arrive just short of it; without the slack every third frame
/// would repeat the one before and the overlay would move in a stutter.
const GRID_SLACK_US: u64 = 2;

/// The grid frame current `source_us` into the source: frame `n` becomes
/// current the moment its own timestamp is reached.
pub(super) fn grid_index(source_us: u64) -> usize {
  ((source_us + GRID_SLACK_US) * CURSOR_FRAME_RATE / 1_000_000) as usize
}

/// Frame `index` of a grid, held at its last frame past its end.
pub(super) fn grid_frame<T: Copy>(grid: &[T], index: usize) -> Option<T> {
  grid.get(index.min(grid.len().checked_sub(1)?)).copied()
}

#[cfg(test)]
mod tests;
