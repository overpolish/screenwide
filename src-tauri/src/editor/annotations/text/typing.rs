// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Typing into a text box where no native text view can be laid over the
//! compositor's box. Windows draws the caret and the selection into the box's
//! own type, so the text, the selection and every edit a key makes are kept
//! here, apart from the window that feeds the keys in.
//!
//! Offsets are bytes into the text, always on a character boundary.

use history::{History, Run, Snapshot};

/// The caret and the selection a box being typed into is drawn with. The
/// selection runs from `start` to `end`; where they meet, the caret stands
/// there, drawn only while `caret` says the blink has it showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TypingMarks {
  pub(crate) start: usize,
  pub(crate) end: usize,
  pub(crate) caret: bool,
}

/// Where a key moves the caret to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Motion {
  Left,
  Right,
  WordLeft,
  WordRight,
  LineStart,
  LineEnd,
  Start,
  End,
}

#[derive(Clone, Debug)]
pub(crate) struct TextTyping {
  pub(super) text: String,
  /// Where the selection was started from, and where it has been taken to;
  /// equal when nothing is selected.
  pub(super) anchor: usize,
  pub(super) caret: usize,
  /// How far across the block a run of line moves is aiming, so going
  /// through a short line does not leave the caret at its end.
  pub(super) goal: Option<f64>,
  history: History,
}

/// Text as a box holds it: line breaks as `\n`, and no other control
/// characters, which a box has no way to show.
fn normalised(text: &str) -> String {
  text
    .replace("\r\n", "\n")
    .replace('\r', "\n")
    .chars()
    .filter(|c| *c == '\n' || !c.is_control())
    .collect()
}

fn is_word(c: char) -> bool {
  c.is_alphanumeric() || c == '_'
}

impl TextTyping {
  /// Typing into `text`, with the caret at its end.
  pub(crate) fn new(text: &str) -> Self {
    let text = normalised(text);
    let end = text.len();
    Self {
      text,
      anchor: end,
      caret: end,
      goal: None,
      history: History::default(),
    }
  }

  pub(crate) fn text(&self) -> &str {
    &self.text
  }

  fn range(&self) -> (usize, usize) {
    (self.anchor.min(self.caret), self.anchor.max(self.caret))
  }

  pub(crate) fn marks(&self, caret: bool) -> TypingMarks {
    let (start, end) = self.range();
    TypingMarks { start, end, caret }
  }

  pub(crate) fn selected(&self) -> &str {
    let (start, end) = self.range();
    &self.text[start..end]
  }

  pub(crate) fn select_all(&mut self) {
    self.anchor = 0;
    self.caret = self.text.len();
    self.settle();
  }

  /// Puts the caret at `offset`, or takes the selection there.
  pub(crate) fn place(&mut self, offset: usize, extend: bool) {
    self.caret = self.boundary(offset);
    if !extend {
      self.anchor = self.caret;
    }
    self.settle();
  }

  /// Selects the word at `offset`, or the one character there when it is
  /// not in a word.
  pub(crate) fn select_word(&mut self, offset: usize) {
    let offset = self.boundary(offset);
    let mut start = offset;
    while let Some(c) = self.text[..start]
      .chars()
      .next_back()
      .filter(|c| is_word(*c))
    {
      start -= c.len_utf8();
    }
    let mut end = offset;
    while let Some(c) = self.text[end..].chars().next().filter(|c| is_word(*c)) {
      end += c.len_utf8();
    }
    if start == end {
      end = self.next(end);
    }
    self.anchor = start;
    self.caret = end;
    self.settle();
  }

  /// Types `typed` over the selection. Consecutive typing is one undo step.
  pub(crate) fn insert(&mut self, typed: &str) -> bool {
    let typed = normalised(typed);
    let (start, end) = self.range();
    if typed.is_empty() && start == end {
      return false;
    }
    self.record(Run::Insert);
    self.text.replace_range(start..end, &typed);
    self.caret = start + typed.len();
    self.anchor = self.caret;
    self.goal = None;
    true
  }

  /// Pastes `text` over the selection, as an undo step of its own.
  pub(crate) fn paste(&mut self, text: &str) -> bool {
    self.history.settle();
    let changed = self.insert(text);
    self.history.settle();
    changed
  }

  /// Erases the selection, or else the character or word before or after
  /// the caret.
  pub(crate) fn erase(&mut self, backward: bool, word: bool) -> bool {
    let (mut start, mut end) = self.range();
    if start == end {
      let other = match (backward, word) {
        (true, false) => self.previous(start),
        (true, true) => self.word_left(start),
        (false, false) => self.next(end),
        (false, true) => self.word_right(end),
      };
      (start, end) = (start.min(other), end.max(other));
    }
    if start == end {
      return false;
    }
    self.record(Run::Erase);
    self.text.replace_range(start..end, "");
    self.caret = start;
    self.anchor = start;
    self.goal = None;
    true
  }

  /// Moves the caret, and with `extend` the selection's moving end.
  pub(crate) fn go(&mut self, motion: Motion, extend: bool) {
    let (start, end) = self.range();
    let collapsing = !extend && start != end;
    let caret = match motion {
      Motion::Left if collapsing => start,
      Motion::Right if collapsing => end,
      Motion::Left => self.previous(self.caret),
      Motion::Right => self.next(self.caret),
      Motion::WordLeft => self.word_left(self.caret),
      Motion::WordRight => self.word_right(self.caret),
      Motion::LineStart => self.line_bounds(self.caret).0,
      Motion::LineEnd => self.line_bounds(self.caret).1,
      Motion::Start => 0,
      Motion::End => self.text.len(),
    };
    self.caret = caret;
    if !extend {
      self.anchor = caret;
    }
    self.settle();
  }

  pub(crate) fn undo(&mut self) -> bool {
    self.step(true)
  }

  pub(crate) fn redo(&mut self) -> bool {
    self.step(false)
  }

  pub(super) fn line_bounds(&self, offset: usize) -> (usize, usize) {
    let start = self.text[..offset].rfind('\n').map_or(0, |at| at + 1);
    let end = self.text[offset..]
      .find('\n')
      .map_or(self.text.len(), |at| offset + at);
    (start, end)
  }

  pub(super) fn boundary(&self, offset: usize) -> usize {
    let mut offset = offset.min(self.text.len());
    while !self.text.is_char_boundary(offset) {
      offset -= 1;
    }
    offset
  }

  pub(super) fn previous(&self, offset: usize) -> usize {
    self.text[..offset]
      .char_indices()
      .next_back()
      .map_or(0, |(index, _)| index)
  }

  pub(super) fn next(&self, offset: usize) -> usize {
    offset + self.text[offset..].chars().next().map_or(0, char::len_utf8)
  }

  /// The start of the word before `offset`, past any space before it.
  fn word_left(&self, offset: usize) -> usize {
    let before = &self.text[..offset];
    let trimmed = before.trim_end_matches(|c: char| !is_word(c));
    trimmed.trim_end_matches(is_word).len()
  }

  /// The start of the word after the one `offset` is in, which is where
  /// Windows takes Ctrl+Right.
  fn word_right(&self, offset: usize) -> usize {
    let rest = self.text[offset..]
      .trim_start_matches(is_word)
      .trim_start_matches(|c: char| !is_word(c));
    self.text.len() - rest.len()
  }

  /// A move or a selection ends the undo step typing was collecting.
  pub(super) fn settle(&mut self) {
    self.goal = None;
    self.history.settle();
  }

  fn record(&mut self, run: Run) {
    let (text, anchor, caret) = (&self.text, self.anchor, self.caret);
    self.history.record(run, || Snapshot {
      text: text.clone(),
      anchor,
      caret,
    });
  }

  fn step(&mut self, back: bool) -> bool {
    let mut current = Snapshot {
      text: std::mem::take(&mut self.text),
      anchor: self.anchor,
      caret: self.caret,
    };
    let stepped = self.history.step(back, &mut current);
    (self.text, self.anchor, self.caret) = (current.text, current.anchor, current.caret);
    self.goal = None;
    stepped
  }
}

#[path = "typing_history.rs"]
mod history;

#[path = "typing_layout.rs"]
mod layout;
pub(crate) use layout::TypingLayout;

#[cfg(test)]
#[path = "typing_tests.rs"]
mod tests;
