// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The tables in text read off a picture, with their rows, columns and
//! cells, ruled or not, and the clipboard formats that keep them tables.
//!
//! A reader gives lines, each with its words' boxes, but no say on how they
//! are laid out. The lines are cut into the pieces a cell holds where their
//! words are far apart (`pieces`), using the picture where the reader's boxes
//! do not say where the words are (`ink`); pieces level with each other are
//! rows, and rows that share gutters are a table (`grid`).
//!
//! Its own crate as it needs nothing of the app: the reader's lines and the
//! picture in, tables out.

mod clipboard;
mod columns;
mod grid;
mod ink;
mod pieces;
#[cfg(test)]
mod tests;
mod verdict;

pub use clipboard::{html, text, Block};
pub use ink::Picture;

/// Where something is, as shares of the picture from its top left.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

/// A character of a line, by its UTF-16 offsets into the line's text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
  pub start: usize,
  pub end: usize,
  pub bounds: Rect,
}

/// A line as a reader gave it.
#[derive(Clone, Copy, Debug)]
pub struct Line<'a> {
  pub text: &'a str,
  pub bounds: Rect,
  pub glyphs: &'a [Glyph],
}

/// Text of a line, by its index among the lines and its UTF-16 offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
  pub line: usize,
  pub start: usize,
  pub end: usize,
}

/// A cell: its text, the column it starts in, how many it spans, and where
/// its text is.
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
  pub text: String,
  pub column: usize,
  pub span: usize,
  /// Where its text is among the lines.
  pub sources: Vec<Source>,
  /// Where its text is in the picture.
  pub bounds: Rect,
}

/// A table: its rows from the top, each with the cells it has, in column
/// order; a column a row has nothing in has no cell.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
  pub columns: usize,
  pub rows: Vec<Vec<Cell>>,
}

impl Rect {
  /// The least rect holding both.
  pub(crate) fn union(self, other: Rect) -> Rect {
    let (left, top) = (self.x.min(other.x), self.y.min(other.y));
    let right = (self.x + self.width).max(other.x + other.width);
    let bottom = (self.y + self.height).max(other.y + other.height);
    Rect {
      x: left,
      y: top,
      width: right - left,
      height: bottom - top,
    }
  }
}

impl Table {
  /// Where its cells' text is in the picture, all of it.
  pub fn bounds(&self) -> Option<Rect> {
    self
      .rows
      .iter()
      .flatten()
      .map(|cell| cell.bounds)
      .reduce(Rect::union)
  }
}

/// The tables among `lines`, read off `picture`, from the top.
pub fn find(lines: &[Line], picture: Picture) -> Vec<Table> {
  if picture.width == 0 || picture.height == 0 {
    return Vec::new();
  }
  let aspect = f64::from(picture.height) / f64::from(picture.width);
  grid::tables(pieces::pieces(lines, picture, aspect), aspect)
}
