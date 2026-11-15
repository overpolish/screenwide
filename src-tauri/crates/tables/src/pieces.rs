// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A reader's lines cut into the pieces a table's cells hold.
//!
//! A reader often gives a row of cells as one line, and a cell as a line of
//! its own beside it, sometimes both for the same words. Each line is cut
//! where its words are clearly further apart than a space sets words, which
//! leaves a sentence whole and parts the cells of most rows; a bullet or a
//! list's number stays with what it marks; and a piece read twice is kept
//! once. A tight table's cells can be no further apart than a small face's
//! words, so a piece keeps its words, to be cut again where it crosses a
//! gutter that the table's other rows show (`split_at`).

use crate::ink::{ink_runs, Picture};
use crate::{Line, Rect, Source};

/// The gap between two words, as a share of the line's height, past which
/// they are surely in two cells: wider than a space, even a small face's or
/// a justified line's.
const CELL_GAP: f64 = 0.42;
/// Two pieces with the same words, this much of the smaller covered by the
/// other, are one piece read twice.
const SAME_PIECE: f64 = 0.5;
const BULLETS: [&str; 9] = ["•", "◦", "▪", "‣", "·", "*", "-", "–", "—"];

/// Words, or the words of a cell, with where they are.
#[derive(Clone, Debug)]
pub(crate) struct Piece {
  pub text: String,
  pub bounds: Rect,
  /// Where its words are among the lines; more than one for a piece read
  /// twice.
  pub sources: Vec<Source>,
  /// Its words, in its first source's line.
  pub words: Vec<Word>,
}

/// A word: its text, its UTF-16 offsets in its line, and where it starts
/// and ends across the picture.
#[derive(Clone, Debug)]
pub(crate) struct Word {
  pub text: String,
  pub start: usize,
  pub end: usize,
  pub from: f64,
  pub to: f64,
}

impl Piece {
  fn of(line: usize, bounds: Rect, words: Vec<Word>) -> Self {
    let (first, last) = (&words[0], &words[words.len() - 1]);
    Piece {
      text: words
        .iter()
        .map(|word| word.text.as_str())
        .collect::<Vec<_>>()
        .join(" "),
      bounds: Rect {
        x: first.from,
        width: last.to - first.from,
        ..bounds
      },
      sources: vec![Source {
        line,
        start: first.start,
        end: last.end,
      }],
      words,
    }
  }

  /// The piece in two, before word `index` and from it.
  pub(crate) fn split_at(&self, index: usize) -> (Piece, Piece) {
    let line = self.sources[0].line;
    (
      Piece::of(line, self.bounds, self.words[..index].to_vec()),
      Piece::of(line, self.bounds, self.words[index..].to_vec()),
    )
  }
}

/// `aspect` is the picture's height over its width, so a gap across a line
/// and the line's height, each a share of its own side, are measured alike.
pub(crate) fn pieces(lines: &[Line], picture: Picture, aspect: f64) -> Vec<Piece> {
  let mut found: Vec<Piece> = Vec::new();
  for (index, line) in lines.iter().enumerate() {
    for piece in line_pieces(index, line, picture, CELL_GAP * line.bounds.height * aspect) {
      match found
        .iter_mut()
        .find(|other| other.text == piece.text && overlap(other.bounds, piece.bounds) >= SAME_PIECE)
      {
        Some(other) => other.sources.extend(piece.sources),
        None => found.push(piece),
      }
    }
  }
  found
}

fn line_pieces(index: usize, line: &Line, picture: Picture, cell_gap: f64) -> Vec<Piece> {
  let mut groups: Vec<Vec<Word>> = Vec::new();
  for word in words(line, picture, cell_gap) {
    match groups.last_mut() {
      Some(group) if word.from - group[group.len() - 1].to < cell_gap || is_marker(group) => {
        group.push(word)
      }
      _ => groups.push(vec![word]),
    }
  }
  groups
    .into_iter()
    .map(|words| Piece::of(index, line.bounds, words))
    .collect()
}

/// Whether `words` are only a bullet or a list's number, which stay with
/// what they mark however far apart.
fn is_marker(words: &[Word]) -> bool {
  let [word] = words else {
    return false;
  };
  let text = word.text.as_str();
  let numbered = text.strip_suffix(['.', ')']).is_some_and(|number| {
    !number.is_empty() && number.len() <= 3 && number.chars().all(|c| c.is_ascii_alphanumeric())
  });
  BULLETS.contains(&text) || numbered
}

/// The line's words, each with where it starts and ends across the picture:
/// read off the picture where its ink parts into as many words as the line
/// has, or as near as that by joining its closest runs, and otherwise from
/// the reader's own boxes.
fn words(line: &Line, picture: Picture, cell_gap: f64) -> Vec<Word> {
  let units: Vec<u16> = line.text.encode_utf16().collect();
  let mut words: Vec<Word> = Vec::new();
  let mut start = None;
  let mut push = |from: usize, end: usize| {
    words.push(Word {
      text: String::from_utf16_lossy(&units[from..end]),
      start: from,
      end,
      from: 0.0,
      to: 0.0,
    });
  };
  for (offset, unit) in units.iter().enumerate() {
    let space = char::from_u32(u32::from(*unit)).is_some_and(char::is_whitespace);
    match (space, start) {
      (false, None) => start = Some(offset),
      (true, Some(from)) => {
        push(from, offset);
        start = None;
      }
      _ => {}
    }
  }
  if let Some(from) = start {
    push(from, units.len());
  }
  if let Some(runs) = matched_runs(ink_runs(picture, line.bounds), words.len(), cell_gap) {
    for (word, (from, to)) in words.iter_mut().zip(runs) {
      (word.from, word.to) = (from, to);
    }
    return words;
  }
  for word in &mut words {
    let glyphs = line
      .glyphs
      .iter()
      .filter(|glyph| glyph.start >= word.start && glyph.end <= word.end);
    let (from, to) = glyphs.fold((f64::MAX, f64::MIN), |(from, to), glyph| {
      (
        from.min(glyph.bounds.x),
        to.max(glyph.bounds.x + glyph.bounds.width),
      )
    });
    (word.from, word.to) = if from <= to {
      (from, to)
    } else {
      (line.bounds.x, line.bounds.x)
    };
  }
  words
}

/// `runs` joined to `count`, closest first, unless that would join two a
/// cell's gap apart, when which words they hold is not known.
fn matched_runs(mut runs: Vec<(f64, f64)>, count: usize, cell_gap: f64) -> Option<Vec<(f64, f64)>> {
  if runs.len() < count || count == 0 {
    return None;
  }
  while runs.len() > count {
    let (index, gap) = runs
      .windows(2)
      .enumerate()
      .map(|(index, pair)| (index, pair[1].0 - pair[0].1))
      .min_by(|one, other| one.1.total_cmp(&other.1))?;
    if gap >= cell_gap {
      return None;
    }
    runs[index].1 = runs[index + 1].1;
    runs.remove(index + 1);
  }
  Some(runs)
}

/// How much of the smaller of `one` and `other` the other covers.
fn overlap(one: Rect, other: Rect) -> f64 {
  let width = (one.x + one.width).min(other.x + other.width) - one.x.max(other.x);
  let height = (one.y + one.height).min(other.y + other.height) - one.y.max(other.y);
  if width <= 0.0 || height <= 0.0 {
    return 0.0;
  }
  let smaller = (one.width * one.height).min(other.width * other.height);
  if smaller <= 0.0 {
    0.0
  } else {
    width * height / smaller
  }
}
