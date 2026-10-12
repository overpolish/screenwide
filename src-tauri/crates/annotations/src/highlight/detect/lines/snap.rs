// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a selection's ends land along their lines: on a word's edge when let
//! go near one, cutting the word where they were let go further inside it.

use super::super::{EDGE_SNAP, RUN_GAP, WORD_GAP};
use super::Columns;

/// Where one end of a selection lands on its line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) struct Snap {
  /// The column the end is at.
  pub(in super::super) at: usize,
  /// Whether the end cuts through a word rather than sitting at a word's edge.
  /// A cut end is held where it was let go, with no reach past it onto the
  /// letters beyond.
  pub(in super::super) cut: bool,
  /// An ink column of the word the end is in, which a walk along the line
  /// starts from: `at` itself may fall between two letters of a cut word.
  pub(in super::super) ink: usize,
}

impl Snap {
  /// An end on the edge of the ink at `at`.
  pub(in super::super) fn edge(at: usize) -> Self {
    Self {
      at,
      cut: false,
      ink: at,
    }
  }
}

impl Columns {
  /// Where a selection starting at `x` begins. Inside a word it begins there,
  /// or at the word's start when that is within reach; in a gap, at the start
  /// of the next word along before `until`. Nothing there leaves it where it
  /// was pressed.
  pub(in super::super) fn snap_start(&self, x: f64, until: f64, run_gap: f64) -> Snap {
    let column = self.clamp(x);
    if let Some(inside) = [column, column.saturating_sub(1), column + 1]
      .into_iter()
      .find(|&column| self.at(column))
    {
      let start = self
        .run_left(inside, word_gap(run_gap), inside)
        .unwrap_or(inside);
      return inside_word(column, start, column.saturating_sub(start), inside, run_gap);
    }
    let limit = self.clamp(until).max(column);
    let next = (column..=limit)
      .take_while(|&next| !self.blocked(next))
      .find(|&next| self.at(next))
      .unwrap_or(column);
    Snap::edge(next)
  }

  /// Where a selection ending at `x` ends. Inside a word it ends there, or at
  /// the word's end when that is within reach; in a gap, at the end of the
  /// word before it back to `until`.
  pub(in super::super) fn snap_end(&self, x: f64, until: f64, run_gap: f64) -> Snap {
    let column = self.clamp(x);
    if let Some(inside) = [column, column + 1, column.saturating_sub(1)]
      .into_iter()
      .find(|&column| self.at(column))
    {
      let end = self
        .run_right(inside, word_gap(run_gap), inside)
        .unwrap_or(inside);
      return inside_word(column, end, end.saturating_sub(column), inside, run_gap);
    }
    let limit = self.clamp(until).min(column);
    let previous = (limit..=column)
      .rev()
      .take_while(|&previous| !self.blocked(previous))
      .find(|&previous| self.at(previous))
      .unwrap_or(column);
    Snap::edge(previous)
  }
}

/// An end let go at `column` inside a word, `apart` from the word's `edge` it
/// would grow to: at the edge when that is within reach, else cutting the word
/// where it was let go.
fn inside_word(column: usize, edge: usize, apart: usize, ink: usize, run_gap: f64) -> Snap {
  let reach = (run_gap * EDGE_SNAP / RUN_GAP).max(2.0);
  let (at, cut) = if apart as f64 <= reach {
    (edge, false)
  } else {
    (column, true)
  };
  Snap { at, cut, ink }
}

/// A word space, from the run gap it is a fixed share of.
fn word_gap(run_gap: f64) -> f64 {
  (run_gap * WORD_GAP / RUN_GAP).max(2.0)
}
