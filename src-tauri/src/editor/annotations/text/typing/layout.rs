// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The edits that go by what is on screen rather than by the text alone: a
//! line up or down, and a press on the text.

use super::super::metrics::LINE_HEIGHT;
use super::TextTyping;

/// How a box's lines are set, for the edits that go by what is on screen: a
/// line up or down, and a press on the text. `width` measures one line at
/// `font`, in the pixels the block and every point are in; each line sits in
/// the block by `align`: 0 left, 1 centred, 2 right.
pub(crate) struct TypingLayout<'a> {
  pub(crate) width: &'a dyn Fn(&str) -> f64,
  pub(crate) block_width: f64,
  pub(crate) font: f64,
  pub(crate) align: u32,
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

impl TextTyping {
  /// Moves the caret a line up or down, keeping to where across the block
  /// the run of line moves began. Past the first or last line it goes to
  /// the text's start or end.
  pub(crate) fn go_line(&mut self, down: bool, extend: bool, layout: &TypingLayout<'_>) {
    let goal = self.goal.unwrap_or_else(|| self.caret_point(layout).0);
    let line = self.text[..self.caret].matches('\n').count();
    let caret = if !down && line == 0 {
      0
    } else if down && line + 1 >= self.text.split('\n').count() {
      self.text.len()
    } else {
      self.offset_in_line(if down { line + 1 } else { line - 1 }, goal, layout)
    };
    self.caret = caret;
    if !extend {
      self.anchor = caret;
    }
    self.settle();
    self.goal = Some(goal);
  }

  /// The offset nearest `point`, a point in the block's own space.
  pub(crate) fn offset_at(&self, point: (f64, f64), layout: &TypingLayout<'_>) -> usize {
    let lines = self.text.split('\n').count();
    let height = layout.line_height();
    let line = if height > 0.0 {
      ((point.1 / height).floor().max(0.0) as usize).min(lines - 1)
    } else {
      0
    };
    self.offset_in_line(line, point.0, layout)
  }

  /// Where the caret stands in the block: across it, and the top of its line.
  pub(crate) fn caret_point(&self, layout: &TypingLayout<'_>) -> (f64, f64) {
    let (start, end) = self.line_bounds(self.caret);
    let line = &self.text[start..end];
    let line_index = self.text[..start].matches('\n').count();
    (
      layout.line_left(line) + (layout.width)(&line[..self.caret - start]),
      line_index as f64 * layout.line_height(),
    )
  }

  fn offset_in_line(&self, line: usize, x: f64, layout: &TypingLayout<'_>) -> usize {
    let start: usize = self
      .text
      .split('\n')
      .take(line)
      .map(|line| line.len() + 1)
      .sum();
    let text = self.text[start..].split('\n').next().unwrap_or_default();
    let left = layout.line_left(text);
    text
      .char_indices()
      .map(|(index, _)| index)
      .chain([text.len()])
      .min_by(|a, b| {
        let distance = |at: &usize| (left + (layout.width)(&text[..*at]) - x).abs();
        distance(a).total_cmp(&distance(b))
      })
      .map_or(start, |offset| start + offset)
  }
}
