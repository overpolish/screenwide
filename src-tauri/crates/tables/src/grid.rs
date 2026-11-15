// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pieces laid out as a table: rows, columns and the cells where they meet.
//!
//! Pieces side by side are a row; a cell's text that wraps onto a second
//! line still sits beside the rest of its row. Rows close together, among
//! them two or more of several pieces, may be a table: its rows are those
//! from the first to the last that set their pieces in its columns
//! (`columns`), with what sits tight against them above or below, such as a
//! heading over two columns. Text set in columns, a page's or a newspaper's,
//! code and lists make rows and columns too; a table is told from them by
//! how it fills its columns (`verdict`).

use crate::columns::{Columns, GUTTER_GAP};
use crate::pieces::Piece;
use crate::verdict::{is_prose, is_tabular};
use crate::{Cell, Table};

/// How much of a piece's height must be level with a row for it to be in
/// the row: enough that the next line down is not, and a cell's second line,
/// set level with the middle of its row, is.
const ROW_OVERLAP: f64 = 0.3;
/// How far apart rows of a table can be, in the pieces' usual heights.
const ROW_GAP: f64 = 1.5;
/// How far apart rows of a table can be, in the usual gaps between rows of
/// several pieces, where that is further than `ROW_GAP`.
const SPACING_GAP: f64 = 2.0;
/// How close a row of one piece above or below a table must be, in the
/// pieces' usual heights, to be part of it: a heading over its columns is,
/// a title or a paragraph is not.
const EDGE_GAP: f64 = 0.6;
/// How close a row must be to the one above, in the pieces' usual heights,
/// to hold its cells' wrapped lines rather than be a row of its own.
const WRAP_GAP: f64 = 0.35;

#[derive(Clone)]
struct Row {
  pieces: Vec<Piece>,
  top: f64,
  bottom: f64,
}

pub(crate) fn tables(mut pieces: Vec<Piece>, aspect: f64) -> Vec<Table> {
  if pieces.len() < 4 {
    return Vec::new();
  }
  let mut heights: Vec<f64> = pieces.iter().map(|piece| piece.bounds.height).collect();
  heights.sort_by(f64::total_cmp);
  let unit = heights[heights.len() / 2];
  pieces.sort_by(|one, other| centre(one).total_cmp(&centre(other)));
  let rows = rows(pieces);
  let gap = |above: &Row, below: &Row| (below.top - above.bottom) / unit;
  // A table pads its cells evenly, so a row whose cell holds several lines,
  // with the others centred beside them, stands further from its neighbours
  // than one-line rows do: as far as its padding and half its extra lines.
  let mut spacing: Vec<f64> = rows
    .windows(2)
    .filter(|pair| pair[0].pieces.len() > 1 && pair[1].pieces.len() > 1)
    .map(|pair| gap(&pair[0], &pair[1]))
    .collect();
  spacing.sort_by(f64::total_cmp);
  let row_gap = spacing
    .get(spacing.len() / 2)
    .map_or(ROW_GAP, |usual| ROW_GAP.max(SPACING_GAP * usual));
  let mut tables = Vec::new();
  let mut start = 0;
  for end in 1..=rows.len() {
    if end == rows.len() || gap(&rows[end - 1], &rows[end]) > row_gap {
      tables.extend(table(&rows[start..end], unit, aspect));
      start = end;
    }
  }
  tables
}

fn centre(piece: &Piece) -> f64 {
  piece.bounds.y + piece.bounds.height / 2.0
}

fn rows(pieces: Vec<Piece>) -> Vec<Row> {
  let mut rows: Vec<Row> = Vec::new();
  for piece in pieces {
    let (top, bottom) = (piece.bounds.y, piece.bounds.y + piece.bounds.height);
    match rows.last_mut() {
      Some(row)
        if row.bottom.min(bottom) - row.top.max(top) >= ROW_OVERLAP * piece.bounds.height =>
      {
        row.top = row.top.min(top);
        row.bottom = row.bottom.max(bottom);
        row.pieces.push(piece);
      }
      _ => rows.push(Row {
        pieces: vec![piece],
        top,
        bottom,
      }),
    }
  }
  rows
}

/// The pieces of `rows` that have several, a row at a time.
fn split(rows: &[Row]) -> Vec<&[Piece]> {
  rows
    .iter()
    .filter(|row| row.pieces.len() > 1)
    .map(|row| row.pieces.as_slice())
    .collect()
}

/// The rows of `run`, rows close together, that are a table, as one.
fn table(run: &[Row], unit: f64, aspect: f64) -> Option<Table> {
  let columns = layout(run, aspect)?;
  let mut first = run.iter().position(|row| columns.fits(&row.pieces))?;
  let mut last = run.iter().rposition(|row| columns.fits(&row.pieces))?;
  let tight = |above: &Row, below: &Row| (below.top - above.bottom) / unit <= EDGE_GAP;
  while first > 0 && run[first - 1].pieces.len() == 1 && tight(&run[first - 1], &run[first]) {
    first -= 1;
  }
  while last + 1 < run.len() && run[last + 1].pieces.len() == 1 && tight(&run[last], &run[last + 1])
  {
    last += 1;
  }
  let mut rows = run[first..=last].to_vec();
  let mut columns = layout(&rows, aspect)?;
  if cut_at_gutters(&mut rows, &columns, aspect) {
    columns = layout(&rows, aspect)?;
  }
  if !is_tabular(&split(&rows), &columns) {
    return None;
  }
  let mut table_rows: Vec<(Vec<Cell>, f64)> = Vec::new();
  for row in &rows {
    let cells = cells(row, &columns, unit);
    match table_rows.last_mut() {
      Some((above, bottom)) if is_wrapped(&cells, above, (row.top - *bottom) / unit) => {
        for cell in cells {
          let other = above
            .iter_mut()
            .find(|other| other.column == cell.column)
            .expect("checked");
          other.text.push(' ');
          other.text.push_str(&cell.text);
          other.sources.extend(cell.sources);
          other.bounds = other.bounds.union(cell.bounds);
        }
        *bottom = row.bottom;
      }
      _ => table_rows.push((cells, row.bottom)),
    }
  }
  let table = Table {
    columns: columns.len(),
    rows: table_rows.into_iter().map(|(cells, _)| cells).collect(),
  };
  (!is_prose(&table)).then_some(table)
}

/// The columns the rows of several pieces among `rows` share. A column whose
/// heading sits at one side and its cells at the other, as a column of
/// numbers set to its end under a heading at its start does, covers two
/// stretches with a gutter between, and no row has a piece in both: they
/// are one column.
fn layout(rows: &[Row], aspect: f64) -> Option<Columns> {
  let pieces = split(rows);
  if pieces.len() < 2 {
    return None;
  }
  let mut columns = Columns::of(pieces.iter().copied(), aspect)?;
  while let Some(index) =
    (0..columns.len() - 1).find(|&index| rows.iter().all(|row| !columns.parts(&row.pieces, index)))
  {
    columns.merge(index);
  }
  (columns.len() >= 2).then_some(columns)
}

/// Cuts each piece across a gutter where two of its words are apart there,
/// as a tight table's cells, which the reader gave as one line, are; whether
/// any was cut.
fn cut_at_gutters(rows: &mut [Row], columns: &Columns, aspect: f64) -> bool {
  let gutters = columns.gutters();
  let mut cut = false;
  for row in rows {
    let mut pieces = Vec::with_capacity(row.pieces.len());
    for piece in row.pieces.drain(..) {
      let least = GUTTER_GAP * piece.bounds.height * aspect;
      let mut rest = piece;
      while let Some(index) = (1..rest.words.len()).find(|&index| {
        let (before, after) = (rest.words[index - 1].to, rest.words[index].from);
        after - before >= least
          && gutters
            .iter()
            .any(|&(from, to)| before < to && from < after)
      }) {
        let (head, tail) = rest.split_at(index);
        pieces.push(head);
        rest = tail;
        cut = true;
      }
      pieces.push(rest);
    }
    row.pieces = pieces;
  }
  cut
}

/// The row's pieces in cells, a cell's lines in reading order: line by line,
/// then along the line.
fn cells(row: &Row, columns: &Columns, unit: f64) -> Vec<Cell> {
  let mut pieces: Vec<&Piece> = row.pieces.iter().collect();
  let line = |piece: &Piece| (centre(piece) / (unit / 2.0)).round() as i64;
  pieces.sort_by(|one, other| {
    line(one)
      .cmp(&line(other))
      .then(one.bounds.x.total_cmp(&other.bounds.x))
  });
  let mut cells: Vec<Cell> = Vec::new();
  for piece in pieces {
    let (column, span) = columns.place(piece);
    match cells.iter_mut().find(|cell| cell.column == column) {
      Some(cell) => {
        cell.text.push(' ');
        cell.text.push_str(&piece.text);
        cell.span = cell.span.max(span);
        cell.sources.extend(piece.sources.iter().copied());
        cell.bounds = cell.bounds.union(piece.bounds);
      }
      None => cells.push(Cell {
        text: piece.text.clone(),
        column,
        span,
        sources: piece.sources.clone(),
        bounds: piece.bounds,
      }),
    }
  }
  cells.sort_by_key(|cell| cell.column);
  cells
}

/// Whether `cells`, `gap` heights below `above`, are the wrapped lines of
/// some of its cells: tight under it, in no more than half its columns, and
/// only in columns it has text in. A row with only a cell or two missing is
/// a row of its own.
fn is_wrapped(cells: &[Cell], above: &[Cell], gap: f64) -> bool {
  gap <= WRAP_GAP
    && cells.len() * 2 <= above.len()
    && cells
      .iter()
      .all(|cell| above.iter().any(|other| other.column == cell.column))
}
