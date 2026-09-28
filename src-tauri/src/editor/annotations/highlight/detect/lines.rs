// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rows and columns of ink: which rows make one line, and where along a line
//! its words and runs of text begin and end.

use super::ink::{Cell, Mask};
use super::{RUN_GAP, WORD_GAP};

/// A run of rows or columns, `[from, to)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Run {
  pub(super) from: usize,
  pub(super) to: usize,
}

impl Run {
  pub(super) fn len(self) -> usize {
    self.to - self.from
  }

  /// How far `at` lies outside the run; zero inside it.
  fn distance(self, at: f64) -> f64 {
    if at < self.from as f64 {
      self.from as f64 - at
    } else if at >= self.to as f64 {
      at - self.to as f64 + 1.0
    } else {
      0.0
    }
  }
}

/// Joins what a row profile splits but a reader does not: rows a pixel apart,
/// and the dots, accents and descenders that stand clear of their line.
pub(super) fn merge_lines(runs: Vec<Run>) -> Vec<Run> {
  let mut joined: Vec<Run> = Vec::with_capacity(runs.len());
  for run in runs {
    match joined.last_mut() {
      Some(last) if run.from <= last.to + 1 => last.to = run.to,
      _ => joined.push(run),
    }
  }
  loop {
    let mut merged = false;
    let mut index = 0;
    while index < joined.len() {
      let run = joined[index];
      let neighbour = |other: usize| {
        let other = joined[other];
        let gap = if other.from >= run.to {
          other.from - run.to
        } else {
          run.from.saturating_sub(other.to)
        };
        (gap, other.len())
      };
      let candidates = [
        index.checked_sub(1).map(|other| (other, neighbour(other))),
        (index + 1 < joined.len()).then(|| (index + 1, neighbour(index + 1))),
      ];
      let best = candidates
        .into_iter()
        .flatten()
        .filter(|(_, (gap, height))| {
          (run.len() as f64) < *height as f64 * 0.45 && (*gap as f64) <= *height as f64 * 0.6
        })
        .min_by_key(|(_, (gap, _))| *gap);
      if let Some((other, _)) = best {
        let (low, high) = (index.min(other), index.max(other));
        joined[low] = Run {
          from: joined[low].from,
          to: joined[high].to,
        };
        joined.remove(high);
        merged = true;
      } else {
        index += 1;
      }
    }
    if !merged {
      return joined;
    }
  }
}

/// The line `at` falls on, or the nearest one within reach of it.
pub(super) fn line_at(lines: &[Run], at: f64, reach: f64) -> Option<usize> {
  lines
    .iter()
    .enumerate()
    .map(|(index, line)| (index, line.distance(at)))
    .filter(|(index, distance)| {
      let line = lines[*index];
      *distance <= (line.len() as f64).max(reach * 0.5) * 0.75
    })
    .min_by(|a, b| a.1.total_cmp(&b.1))
    .map(|(index, _)| index)
}

/// Which columns of one line carry text, over the stretch the line is read
/// along, and which are a picture or a pane that ends it.
pub(super) struct Columns {
  from: usize,
  cells: Vec<Cell>,
}

impl Columns {
  /// A column is solid where most of the line's rows are: a picture or a pane
  /// fills them all, where an underline or a rule crosses only a row or two.
  pub(super) fn read(mask: &Mask, line: Run, along: Run) -> Self {
    let mut glyph = vec![false; along.len()];
    let mut solid_rows = vec![0_usize; along.len()];
    for y in line.from..line.to {
      for (index, cell) in mask.row(y, along).enumerate() {
        match cell {
          Cell::Glyph => glyph[index] = true,
          Cell::Solid => solid_rows[index] += 1,
          Cell::Page => {}
        }
      }
    }
    Self {
      from: along.from,
      cells: glyph
        .into_iter()
        .zip(solid_rows)
        .map(|(glyph, solid_rows)| {
          if solid_rows * 2 >= line.len().max(1) {
            Cell::Solid
          } else if glyph {
            Cell::Glyph
          } else {
            Cell::Page
          }
        })
        .collect(),
    }
  }

  fn cell(&self, x: usize) -> Cell {
    x.checked_sub(self.from)
      .and_then(|index| self.cells.get(index).copied())
      .unwrap_or(Cell::Page)
  }

  fn at(&self, x: usize) -> bool {
    self.cell(x) == Cell::Glyph
  }

  fn blocked(&self, x: usize) -> bool {
    self.cell(x) == Cell::Solid
  }

  pub(super) fn clamp(&self, x: f64) -> usize {
    let last = self.from + self.cells.len().saturating_sub(1);
    (x.max(0.0).round() as usize).clamp(self.from, last.max(self.from))
  }

  /// The ink column nearest `x`, within `within` of it.
  pub(super) fn nearest(&self, x: f64, within: f64) -> Option<usize> {
    let centre = self.clamp(x);
    let reach = within.ceil() as usize;
    (0..=reach).find_map(|step| {
      [centre.checked_sub(step), Some(centre + step)]
        .into_iter()
        .flatten()
        .find(|&column| self.at(column))
    })
  }

  /// The first ink column of the run `from` is in, where a gap of `gap` or
  /// more ends it - but only once the walk is left of `through`: a gap inside
  /// the stretch a drag covered is one it chose to cross, as a table row's
  /// cells are when a selection spans them.
  pub(super) fn run_left(&self, from: usize, gap: f64, through: usize) -> Option<usize> {
    if !self.at(from) {
      return None;
    }
    let mut left = from;
    let mut column = from;
    while column > self.from {
      column -= 1;
      if self.blocked(column) {
        break;
      }
      if self.at(column) {
        left = column;
      } else if column < through && (left - column) as f64 >= gap {
        break;
      }
    }
    Some(left)
  }

  /// The last ink column of the run `from` is in, crossing any gap until the
  /// walk is right of `through`.
  pub(super) fn run_right(&self, from: usize, gap: f64, through: usize) -> Option<usize> {
    if !self.at(from) {
      return None;
    }
    let end = self.from + self.cells.len();
    let mut right = from;
    let mut column = from;
    while column + 1 < end {
      column += 1;
      if self.blocked(column) {
        break;
      }
      if self.at(column) {
        right = column;
      } else if column > through && (column - right) as f64 >= gap {
        break;
      }
    }
    Some(right)
  }

  /// Where a selection starting at `x` begins: the start of the word under
  /// it, or where it starts in a gap, the start of the next word along before
  /// `until`. Nothing there leaves it where it was pressed.
  pub(super) fn snap_start(&self, x: f64, until: f64, run_gap: f64) -> usize {
    let column = self.clamp(x);
    let word_gap = self.word_gap(run_gap);
    if let Some(inside) = [column, column.saturating_sub(1), column + 1]
      .into_iter()
      .find(|&column| self.at(column))
    {
      return self.run_left(inside, word_gap, inside).unwrap_or(inside);
    }
    let limit = self.clamp(until).max(column);
    (column..=limit)
      .take_while(|&next| !self.blocked(next))
      .find(|&next| self.at(next))
      .unwrap_or(column)
  }

  /// Where a selection ending at `x` ends: the end of the word under it, or
  /// where it ends in a gap, the end of the word before it back to `until`.
  pub(super) fn snap_end(&self, x: f64, until: f64, run_gap: f64) -> usize {
    let column = self.clamp(x);
    let word_gap = self.word_gap(run_gap);
    if let Some(inside) = [column, column + 1, column.saturating_sub(1)]
      .into_iter()
      .find(|&column| self.at(column))
    {
      return self.run_right(inside, word_gap, inside).unwrap_or(inside);
    }
    let limit = self.clamp(until).min(column);
    (limit..=column)
      .rev()
      .take_while(|&previous| !self.blocked(previous))
      .find(|&previous| self.at(previous))
      .unwrap_or(column)
  }

  /// A word space, from the run gap it is a fixed share of.
  fn word_gap(&self, run_gap: f64) -> f64 {
    (run_gap * WORD_GAP / RUN_GAP).max(2.0)
  }
}
