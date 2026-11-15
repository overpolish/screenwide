// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A table's columns: what the pieces of its rows cover together, parted by
//! gutters that none of them covers, or only a rare piece over two columns.

use crate::pieces::Piece;

/// How much of each piece's edges, in its heights, is left out of what it
/// covers: readers' boxes reach a little past the ink.
const EDGE_SLACK: f64 = 0.1;
/// The share of rows whose pieces may cross a gutter: a heading over two
/// columns, or a line of text just past the table.
const SPANNING: f64 = 0.2;
/// How far into the next column, in a piece's heights, it must reach to
/// span it.
const SPAN_REACH: f64 = 0.25;
/// The least gap between two words, as a share of their line's height,
/// that can part two cells: where it lies on a gutter the rest of a table
/// shows. A small face's spaces come near it.
pub(crate) const GUTTER_GAP: f64 = 0.28;
const BINS: usize = 1000;

pub(crate) struct Columns {
  left: f64,
  right: f64,
  aspect: f64,
  /// Each column's first and last bin across the table.
  bins: Vec<(usize, usize)>,
}

impl Columns {
  /// The columns that `rows`, each a row's pieces, share; none where they
  /// share fewer than two. A piece covers its words, and the gaps between
  /// them only where they are narrower than a gap that could part two cells
  /// (`GUTTER_GAP`), so a row the reader gave as one line, a tight table's,
  /// leaves the gutters the other rows show.
  pub(crate) fn of<'a>(
    rows: impl Iterator<Item = &'a [Piece]> + Clone,
    aspect: f64,
  ) -> Option<Self> {
    let mut columns = Columns {
      left: f64::MAX,
      right: f64::MIN,
      aspect,
      bins: Vec::new(),
    };
    for piece in rows.clone().flatten() {
      let (from, to) = columns.span(piece);
      columns.left = columns.left.min(from);
      columns.right = columns.right.max(to);
    }
    if columns.right <= columns.left {
      return None;
    }
    let mut cover = vec![0_usize; BINS];
    let mut count = 0_u32;
    for row in rows {
      count += 1;
      let mut covered = vec![false; BINS];
      for piece in row {
        for (from, to) in columns.stretches(piece) {
          covered[columns.bin(from)..=columns.bin(to)].fill(true);
        }
      }
      cover
        .iter_mut()
        .zip(covered)
        .for_each(|(count, covered)| *count += usize::from(covered));
    }
    let allowed = (SPANNING * f64::from(count)).floor() as usize;
    for (index, count) in cover.into_iter().enumerate() {
      if count <= allowed {
        continue;
      }
      match columns.bins.last_mut() {
        Some(column) if column.1 + 1 == index => column.1 = index,
        _ => columns.bins.push((index, index)),
      }
    }
    (columns.bins.len() >= 2).then_some(columns)
  }

  pub(crate) fn len(&self) -> usize {
    self.bins.len()
  }

  /// The first column `piece` is in and how many it spans: the columns it
  /// reaches well into, or the one it reaches furthest into, or failing
  /// that the nearest. A heading wider than its column's cells, reaching
  /// into the gutter beside it, stays in its column; one over two columns
  /// spans them.
  pub(crate) fn place(&self, piece: &Piece) -> (usize, usize) {
    let (from, to) = self.span(piece);
    let reach = SPAN_REACH * piece.bounds.height * self.aspect;
    let overlaps: Vec<f64> = self
      .bins
      .iter()
      .map(|column| to.min(self.x(column.1 + 1)) - from.max(self.x(column.0)))
      .collect();
    let deepest = (0..overlaps.len())
      .max_by(|&one, &other| overlaps[one].total_cmp(&overlaps[other]))
      .unwrap_or(0);
    let reached = |index: &usize| *index == deepest || overlaps[*index] >= reach;
    let first = (0..overlaps.len()).find(reached).unwrap_or(deepest);
    let last = (0..overlaps.len()).rev().find(reached).unwrap_or(deepest);
    (first, last - first + 1)
  }

  /// Whether `row` sets two or more pieces each in a column of its own, as a
  /// table's rows do and a line of text or a list does not.
  pub(crate) fn fits(&self, row: &[Piece]) -> bool {
    let mut taken = Vec::new();
    for piece in row {
      let (column, span) = self.place(piece);
      if span == 1 && !taken.contains(&column) {
        taken.push(column);
      }
    }
    taken.len() >= 2
  }

  /// Whether `row` parts two pieces at the gutter after column `index`: one
  /// ending in that column and another starting in the next.
  pub(crate) fn parts(&self, row: &[Piece], index: usize) -> bool {
    let placed: Vec<(usize, usize)> = row.iter().map(|piece| self.place(piece)).collect();
    placed
      .iter()
      .any(|&(first, span)| first + span - 1 == index)
      && placed.iter().any(|&(first, _)| first == index + 1)
  }

  /// Column `index` and the next as one.
  pub(crate) fn merge(&mut self, index: usize) {
    self.bins[index].1 = self.bins[index + 1].1;
    self.bins.remove(index + 1);
  }

  /// Each gutter's start and end across the picture.
  pub(crate) fn gutters(&self) -> Vec<(f64, f64)> {
    self
      .bins
      .windows(2)
      .map(|pair| (self.x(pair[0].1 + 1), self.x(pair[1].0)))
      .collect()
  }

  fn span(&self, piece: &Piece) -> (f64, f64) {
    let inset = (EDGE_SLACK * piece.bounds.height * self.aspect).min(piece.bounds.width / 4.0);
    (
      piece.bounds.x + inset,
      piece.bounds.x + piece.bounds.width - inset,
    )
  }

  /// The stretches of `piece` that it covers: its words, joined across
  /// gaps too narrow to part two cells, each a little in from its edges.
  fn stretches(&self, piece: &Piece) -> Vec<(f64, f64)> {
    let least = GUTTER_GAP * piece.bounds.height * self.aspect;
    let inset = EDGE_SLACK * piece.bounds.height * self.aspect;
    let mut stretches: Vec<(f64, f64)> = Vec::new();
    for word in &piece.words {
      match stretches.last_mut() {
        Some(last) if word.from - last.1 < least => last.1 = last.1.max(word.to),
        _ => stretches.push((word.from, word.to)),
      }
    }
    stretches
      .into_iter()
      .map(|(from, to)| {
        let inset = inset.min((to - from) / 4.0);
        (from + inset, to - inset)
      })
      .collect()
  }

  fn bin(&self, x: f64) -> usize {
    (((x - self.left) / (self.right - self.left)) * BINS as f64).clamp(0.0, BINS as f64 - 1.0)
      as usize
  }

  fn x(&self, bin: usize) -> f64 {
    self.left + (self.right - self.left) * bin as f64 / BINS as f64
  }
}
