// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether rows laid out in columns are a table. Code, lists and text set in
//! columns all make rows and columns of a kind; a table fills its columns
//! cell by cell, and some column of it holds a few words a cell.

use crate::columns::Columns;
use crate::pieces::Piece;
use crate::Table;

/// The least share of a table's cells, in percent, that rows of several
/// pieces fill.
const FILLED: usize = 65;
/// Words a cell, in every column's usual cell, past which they are columns
/// of text rather than a table.
const PROSE_WORDS: usize = 4;

/// Whether the rows of several pieces fill most of the columns, mostly set
/// their pieces in columns of their own, and seldom cross a gutter, as a
/// table's rows do. Code, which a monospaced face spaces as widely as a
/// table's cells, is cut into pieces at its spaces and shares gutters by
/// chance, but its lines fill few of them and run across the rest.
pub(crate) fn is_tabular(rows: &[&[Piece]], columns: &Columns) -> bool {
  let fitting = rows.iter().filter(|row| columns.fits(row)).count();
  let pieces = rows.iter().map(|row| row.len()).sum::<usize>();
  let placed: Vec<Vec<(usize, usize)>> = rows
    .iter()
    .map(|row| row.iter().map(|piece| columns.place(piece)).collect())
    .collect();
  let spanning = placed
    .iter()
    .flatten()
    .filter(|(_, span)| *span > 1)
    .count();
  let filled: usize = placed
    .iter()
    .map(|row| {
      (0..columns.len())
        .filter(|column| {
          row
            .iter()
            .any(|(first, span)| (*first..first + span).contains(column))
        })
        .count()
    })
    .sum();
  fitting * 10 >= rows.len() * 7
    && spanning * 5 <= pieces
    && filled * 100 >= rows.len() * columns.len() * FILLED
}

/// Whether every column's usual cell is a sentence's worth of words.
pub(crate) fn is_prose(table: &Table) -> bool {
  (0..table.columns).all(|column| {
    let mut words: Vec<usize> = table
      .rows
      .iter()
      .flat_map(|row| row.iter().filter(|cell| cell.column == column))
      .map(|cell| cell.text.split_whitespace().count())
      .collect();
    if words.is_empty() {
      return true;
    }
    words.sort_unstable();
    words[words.len() / 2] >= PROSE_WORDS
  })
}
