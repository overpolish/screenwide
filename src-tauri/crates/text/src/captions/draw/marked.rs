// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which words a style's highlight falls on: in a caption the word being
//! said, which moves along as the words are; in a text the words chosen in
//! it, which hold it throughout.

use super::caption::DrawnWord;
use super::glow::said_strength;
use super::underline::{under_run, underline, UnderlineAt};
use crate::captions::CaptionWord;

#[derive(Clone, Copy, Debug)]
pub enum Highlighted<'a> {
  /// The word being said.
  Said,
  /// The words chosen, by their place among the caption's words.
  Chosen(&'a [bool]),
}

/// A caption's words at one moment, read for what its highlight falls on.
pub(super) struct Marks<'a, 'w> {
  highlighted: Highlighted<'a>,
  words: &'w [DrawnWord<'w>],
  /// The word being said, and when it started, where words are said.
  said: Option<(usize, f64)>,
  at_ms: f64,
}

impl<'a, 'w> Marks<'a, 'w> {
  pub(super) fn new(highlighted: Highlighted<'a>, words: &'w [DrawnWord<'w>], at_ms: f64) -> Self {
    // The word being said: the last that has started, held through the
    // pause after it until the next one starts, and on through the
    // caption's exit.
    let said = matches!(highlighted, Highlighted::Said)
      .then(|| {
        words
          .iter()
          .filter(|drawn| drawn.word.start_ms <= at_ms)
          .max_by_key(|drawn| drawn.index)
          .map(|drawn| (drawn.index, drawn.word.start_ms))
      })
      .flatten();
    Self {
      highlighted,
      words,
      said,
      at_ms,
    }
  }

  /// The word being said, where words are said.
  pub(super) fn said(&self) -> Option<usize> {
    self.said.map(|(index, _)| index)
  }

  /// Whether the highlight falls on word `index`.
  pub(super) fn has(&self, index: usize) -> bool {
    match self.highlighted {
      Highlighted::Said => self.said() == Some(index),
      Highlighted::Chosen(chosen) => chosen.get(index).copied().unwrap_or(false),
    }
  }

  /// How strongly word `index` glows, from none to full.
  pub(super) fn glow(&self, index: usize) -> f64 {
    match self.highlighted {
      Highlighted::Said => said_strength(index, self.said, self.at_ms),
      Highlighted::Chosen(_) => f64::from(u8::from(self.has(index))),
    }
  }

  /// The lines under what the highlight falls on: under the word being
  /// said, sliding from the one before, or under each run of chosen words
  /// on one line. Each with its middle now and as the exposure began, how
  /// wide and thick it is, and the word it starts under.
  pub(super) fn underlines(&self, opened_ms: f64) -> Vec<(UnderlineAt, &'w CaptionWord)> {
    if let Some(said) = self.said() {
      let word = self.words.iter().find(|drawn| drawn.index == said);
      return word
        .zip(underline(self.words, said, self.at_ms, opened_ms))
        .map(|(drawn, line)| vec![(line, drawn.word)])
        .unwrap_or_default();
    }
    let mut lines = Vec::new();
    let mut run: Option<(&DrawnWord, &DrawnWord)> = None;
    for drawn in self.words {
      let joins = |(first, last): (&DrawnWord, &DrawnWord)| {
        drawn.index == last.index + 1 && (drawn.placed.line[0] - first.placed.line[0]).abs() < 0.5
      };
      match (self.has(drawn.index), run) {
        (true, Some(open)) if joins(open) => run = Some((open.0, drawn)),
        (true, _) => {
          if let Some((first, last)) = run.replace((drawn, drawn)) {
            lines.push((under_run(first, last), first.word));
          }
        }
        (false, _) => {
          if let Some((first, last)) = run.take() {
            lines.push((under_run(first, last), first.word));
          }
        }
      }
    }
    if let Some((first, last)) = run {
      lines.push((under_run(first, last), first.word));
    }
    lines
  }
}
