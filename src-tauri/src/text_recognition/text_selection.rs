// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Portable recognized-text selection. Platform adapters provide normalized
//! pointer positions and render the returned normalized rectangles.

use crate::osc::geometry::Point;

use super::{TextRecognitionResult, TextRect};
use extraction::{line_length, selected_text, selection_rects};

mod extraction;
pub(super) use extraction::paragraph;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextPosition {
  pub line: usize,
  pub offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRange {
  pub start: TextPosition,
  pub end: TextPosition,
}

pub struct TextSelection {
  result: TextRecognitionResult,
  ranges: Vec<TextRange>,
  anchor: Option<TextPosition>,
  focus: Option<TextPosition>,
  selecting: bool,
}

impl TextSelection {
  pub fn new(result: TextRecognitionResult) -> Self {
    Self {
      result,
      ranges: Vec::new(),
      anchor: None,
      focus: None,
      selecting: false,
    }
  }

  pub fn result(&self) -> &TextRecognitionResult {
    &self.result
  }

  pub fn pointer_down(&mut self, point: Point, additive: bool, double: bool) -> bool {
    let Some(position) = text_position_at(&self.result, point) else {
      return false;
    };
    if double {
      let range = TextRange {
        start: TextPosition {
          line: position.line,
          offset: 0,
        },
        end: TextPosition {
          line: position.line,
          offset: line_length(&self.result.lines[position.line].text),
        },
      };
      if additive {
        self.ranges.push(range);
      } else {
        self.ranges = vec![range];
      }
      self.clear_live();
      return true;
    }
    if !additive {
      self.ranges.clear();
    }
    self.anchor = Some(position);
    self.focus = Some(position);
    self.selecting = true;
    true
  }

  pub fn pointer_move(&mut self, point: Point) -> bool {
    if !self.selecting {
      return false;
    }
    let Some(position) = text_position_at(&self.result, point) else {
      return false;
    };
    let changed = self.focus != Some(position);
    self.focus = Some(position);
    changed
  }

  pub fn pointer_up(&mut self, point: Point) -> bool {
    if !self.selecting {
      return false;
    }
    if let Some(position) = text_position_at(&self.result, point) {
      self.focus = Some(position);
    }
    if let (Some(anchor), Some(focus)) = (self.anchor, self.focus) {
      self.ranges.push(ordered_range(anchor, focus));
    }
    self.clear_live();
    true
  }

  pub fn select_all(&mut self) -> bool {
    let Some(last) = self.result.lines.last() else {
      return false;
    };
    self.ranges = vec![TextRange {
      start: TextPosition { line: 0, offset: 0 },
      end: TextPosition {
        line: self.result.lines.len() - 1,
        offset: line_length(&last.text),
      },
    }];
    self.clear_live();
    true
  }

  pub fn rectangles(&self) -> Vec<TextRect> {
    selection_rects(&self.result, &self.current_ranges())
  }

  pub fn selected_text(&self) -> String {
    selected_text(&self.result, &self.current_ranges())
  }

  pub fn all_text(&self) -> &str {
    &self.result.text
  }

  fn current_ranges(&self) -> Vec<TextRange> {
    let mut ranges = self.ranges.clone();
    if let (Some(anchor), Some(focus)) = (self.anchor, self.focus) {
      ranges.push(ordered_range(anchor, focus));
    }
    ranges
  }

  fn clear_live(&mut self) {
    self.anchor = None;
    self.focus = None;
    self.selecting = false;
  }
}

fn ordered_range(anchor: TextPosition, focus: TextPosition) -> TextRange {
  if (anchor.line, anchor.offset) <= (focus.line, focus.offset) {
    TextRange {
      start: anchor,
      end: focus,
    }
  } else {
    TextRange {
      start: focus,
      end: anchor,
    }
  }
}

fn text_position_at(result: &TextRecognitionResult, point: Point) -> Option<TextPosition> {
  let (line_index, line) = result.lines.iter().enumerate().min_by(|(_, a), (_, b)| {
    line_distance(a.bounds, point).total_cmp(&line_distance(b.bounds, point))
  })?;
  if let Some(character) = line.characters.iter().min_by(|a, b| {
    character_distance(a.bounds, point.x).total_cmp(&character_distance(b.bounds, point.x))
  }) {
    return Some(TextPosition {
      line: line_index,
      offset: if point.x < character.bounds.x + character.bounds.width / 2.0 {
        character.start
      } else {
        character.end
      },
    });
  }
  let fraction = ((point.x - line.bounds.x) / line.bounds.width.max(0.0001)).clamp(0.0, 1.0);
  Some(TextPosition {
    line: line_index,
    offset: (fraction * line_length(&line.text) as f64).round() as usize,
  })
}

fn line_distance(bounds: TextRect, point: Point) -> f64 {
  if point.y < bounds.y {
    bounds.y - point.y
  } else if point.y > bounds.y + bounds.height {
    point.y - bounds.y - bounds.height
  } else {
    0.0
  }
}

fn character_distance(bounds: TextRect, x: f64) -> f64 {
  if x >= bounds.x && x <= bounds.x + bounds.width {
    0.0
  } else {
    (x - bounds.x - bounds.width / 2.0).abs()
  }
}

#[cfg(test)]
#[path = "text_selection_tests.rs"]
mod tests;
