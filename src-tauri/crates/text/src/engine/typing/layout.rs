// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The edits that go by what is on screen rather than by the text alone: a
//! line up or down, and a press on the text.

use super::super::metrics::LINE_HEIGHT;
use super::TextTyping;

/// Where a box's text sits on screen, for the edits that go by it. Every
/// point is in the block's own space, from its top-left corner; `text` is
/// always the text being typed.
pub trait TypingLines {
  /// Where `offset` stands: across the block, and the top of its line.
  fn point_of(&mut self, text: &str, offset: usize) -> (f64, f64);
  /// The line at `y` down the block.
  fn line_at(&mut self, text: &str, y: f64) -> usize;
  /// The offset in line `line` nearest `x` across the block.
  fn offset_in_line(&mut self, text: &str, line: usize, x: f64) -> usize;
}

/// A box's lines set in one face at one size. `width` measures one line at
/// `font`, in the pixels the block and every point are in; each line sits
/// in the block by `align`: 0 left, 1 centred, 2 right.
pub struct TypingLayout<'a> {
  pub width: &'a dyn Fn(&str) -> f64,
  pub block_width: f64,
  pub font: f64,
  pub align: u32,
}

impl TypingLayout<'_> {
  fn line_height(&self) -> f64 {
    self.font * LINE_HEIGHT
  }

  fn line_left(&self, line: &str) -> f64 {
    let share = match self.align {
      1 => 0.5,
      2 => 1.0,
      _ => 0.0,
    };
    (self.block_width - (self.width)(line)) * share
  }
}

/// Where line `line` of `text` starts, and the line itself.
fn line_of(text: &str, line: usize) -> (usize, &str) {
  let start: usize = text.split('\n').take(line).map(|line| line.len() + 1).sum();
  let start = start.min(text.len());
  (start, text[start..].split('\n').next().unwrap_or_default())
}

impl TypingLines for TypingLayout<'_> {
  fn point_of(&mut self, text: &str, offset: usize) -> (f64, f64) {
    let start = text[..offset].rfind('\n').map_or(0, |at| at + 1);
    let line = text[start..].split('\n').next().unwrap_or_default();
    let index = text[..start].matches('\n').count();
    (
      self.line_left(line) + (self.width)(&line[..offset - start]),
      index as f64 * self.line_height(),
    )
  }

  fn line_at(&mut self, text: &str, y: f64) -> usize {
    let lines = text.split('\n').count();
    let height = self.line_height();
    if height > 0.0 {
      ((y / height).floor().max(0.0) as usize).min(lines - 1)
    } else {
      0
    }
  }

  fn offset_in_line(&mut self, text: &str, line: usize, x: f64) -> usize {
    let (start, text) = line_of(text, line);
    let left = self.line_left(text);
    text
      .char_indices()
      .map(|(index, _)| index)
      .chain([text.len()])
      .min_by(|a, b| {
        let distance = |at: &usize| (left + (self.width)(&text[..*at]) - x).abs();
        distance(a).total_cmp(&distance(b))
      })
      .map_or(start, |offset| start + offset)
  }
}

impl TextTyping {
  /// Moves the caret a line up or down, keeping to where across the block
  /// the run of line moves began. Past the first or last line it goes to
  /// the text's start or end.
  pub fn go_line(&mut self, down: bool, extend: bool, lines: &mut dyn TypingLines) {
    let goal = self
      .goal
      .unwrap_or_else(|| lines.point_of(&self.text, self.caret).0);
    let line = self.text[..self.caret].matches('\n').count();
    let caret = if !down && line == 0 {
      0
    } else if down && line + 1 >= self.text.split('\n').count() {
      self.text.len()
    } else {
      lines.offset_in_line(&self.text, if down { line + 1 } else { line - 1 }, goal)
    };
    self.caret = caret;
    if !extend {
      self.anchor = caret;
    }
    self.settle();
    self.goal = Some(goal);
  }

  /// The offset nearest `point`, a point in the block's own space.
  pub fn offset_at(&self, point: (f64, f64), lines: &mut dyn TypingLines) -> usize {
    let line = lines.line_at(&self.text, point.1);
    lines.offset_in_line(&self.text, line, point.0)
  }

  /// Where the caret stands in the block: across it, and the top of its line.
  pub fn caret_point(&self, lines: &mut dyn TypingLines) -> (f64, f64) {
    lines.point_of(&self.text, self.caret)
  }
}
