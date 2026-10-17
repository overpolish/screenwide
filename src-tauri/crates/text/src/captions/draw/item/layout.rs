// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where each byte of a text sits as its look sets it, words set apart in
//! their own face and size: what its box is sized by, and what the caret,
//! the selection and a press on the text go by while it is typed into. It
//! is read from the same words its pieces are drawn from, so what is typed
//! lands where it shows.

use super::super::place::{place, Ground};
use super::{item_caption, item_style};
use crate::captions::{CaptionFrame, Measure};
use crate::emphasis::TextSpan;
use crate::engine::typing::TypingLines;
use crate::style::{TextFace, TextStyle};

/// How far the caret reaches above and below the middle of the type beside
/// it, in ems of that type: from about its ascenders to its descenders.
const CARET_REACH: f64 = 0.6;

/// One word of a line as placed, from the block's top-left corner.
#[derive(Clone, Copy, Debug)]
struct ItemToken {
  span: TextSpan,
  left: f64,
  width: f64,
  middle: f64,
  size: f64,
  face: TextFace,
}

#[derive(Clone, Debug)]
struct ItemLine {
  top: f64,
  bottom: f64,
  tokens: Vec<ItemToken>,
}

/// A text's words as placed in its block, in the pixels its type size is
/// in, from the block's top-left corner.
#[derive(Clone, Debug)]
pub struct TextItemLayout {
  /// How large the block is: what the text's box holds.
  pub block: [f64; 2],
  /// Where the block's top-left corner is in what it is set on: about its
  /// middle for a text placed by hand, in the picture for a caption.
  pub origin: [f64; 2],
  all_caps: bool,
  lines: Vec<ItemLine>,
}

impl TextItemLayout {
  /// `text` as `look` sets it at type size `size`, the words of `chosen`
  /// set apart, its lines lined up by `align`, a share of the block's width
  /// from the left, about the block's middle.
  pub fn new(
    text: &str,
    look: &TextStyle,
    chosen: &[TextSpan],
    size: f64,
    align: f64,
    measure: &mut Measure,
  ) -> Self {
    let ground = Ground::Block {
      centre: [0.0; 2],
      size,
      align,
    };
    Self::on(text, look, chosen, ground, measure)
  }

  /// `text` as `look` sets a caption of it in a picture `frame` in size,
  /// where the look's place puts it, the words of `chosen` set apart.
  pub fn in_frame(
    text: &str,
    look: &TextStyle,
    chosen: &[TextSpan],
    frame: CaptionFrame,
    measure: &mut Measure,
  ) -> Self {
    Self::on(text, look, chosen, frame.into(), measure)
  }

  fn on(
    text: &str,
    look: &TextStyle,
    chosen: &[TextSpan],
    ground: Ground,
    measure: &mut Measure,
  ) -> Self {
    let style = item_style(look);
    let item = item_caption(text, &style, chosen, 0.0);
    let placement = place(&item.caption, &style, ground, measure);
    let corner = [0, 1].map(|axis| placement.centre[axis] - placement.block[axis] / 2.0);
    let mut placed = placement.words.into_iter();
    let lines = item
      .spans
      .iter()
      .map(|spans| {
        let mut band = [0.0; 2];
        let tokens = spans
          .iter()
          .zip(placed.by_ref())
          .map(|(&span, word)| {
            band = word.line;
            ItemToken {
              span,
              left: word.centre[0] - word.width / 2.0 - corner[0],
              width: word.width,
              middle: word.centre[1] - corner[1],
              size: word.size,
              face: word.face,
            }
          })
          .collect();
        ItemLine {
          top: band[0] - corner[1],
          bottom: band[1] - corner[1],
          tokens,
        }
      })
      .collect();
    Self {
      block: placement.block,
      origin: corner,
      all_caps: style.all_caps,
      lines,
    }
  }

  /// The line `offset` is on, and the word it is in or at the end of.
  fn token_at(&self, offset: usize) -> (usize, &ItemToken) {
    let line = self
      .lines
      .iter()
      .position(|line| {
        line
          .tokens
          .last()
          .is_some_and(|last| offset <= last.span.end)
      })
      .unwrap_or(self.lines.len() - 1);
    let tokens = &self.lines[line].tokens;
    let token = tokens
      .iter()
      .find(|token| offset <= token.span.end)
      .unwrap_or(&tokens[tokens.len() - 1]);
    (line, token)
  }

  /// How far across the block `offset` of `token` stands.
  fn x_in(&self, text: &str, token: &ItemToken, offset: usize, measure: &mut Measure) -> f64 {
    let offset = offset.clamp(token.span.start, token.span.end);
    let before = if offset == token.span.start {
      0.0
    } else if offset == token.span.end {
      token.width
    } else {
      let prefix = text.get(token.span.start..offset).unwrap_or_default();
      let shown = if self.all_caps {
        prefix.to_uppercase()
      } else {
        prefix.to_owned()
      };
      measure(&shown, token.face, token.size)
    };
    token.left + before
  }

  /// How many lines the text has.
  pub fn line_count(&self) -> usize {
    self.lines.len()
  }

  /// The line at `y` down the block: the one it is over, or the nearest.
  pub fn line_at(&self, y: f64) -> usize {
    self
      .lines
      .iter()
      .position(|line| y < line.bottom)
      .unwrap_or(self.lines.len() - 1)
  }

  /// Where `offset` stands: across the block, and the top of its line.
  pub fn point_of(&self, text: &str, offset: usize, measure: &mut Measure) -> (f64, f64) {
    let (line, token) = self.token_at(offset);
    (
      self.x_in(text, token, offset, measure),
      self.lines[line].top,
    )
  }

  /// The caret at `offset`: across the block, and its top and bottom, as
  /// tall as the type beside it.
  pub fn caret(&self, text: &str, offset: usize, measure: &mut Measure) -> [f64; 3] {
    let (_, token) = self.token_at(offset);
    let reach = CARET_REACH * token.size;
    [
      self.x_in(text, token, offset, measure),
      token.middle - reach,
      token.middle + reach,
    ]
  }

  /// The selection from `start` to `end` as one rectangle on each line it
  /// covers, each `[left, top, right, bottom]` across the line's height.
  pub fn selection(
    &self,
    text: &str,
    start: usize,
    end: usize,
    measure: &mut Measure,
  ) -> Vec<[f64; 4]> {
    self
      .lines
      .iter()
      .filter_map(|line| {
        let first = line.tokens.first()?;
        let last = line.tokens.last()?;
        let (from, to) = (start.max(first.span.start), end.min(last.span.end));
        (from < to).then(|| {
          let x = |offset: usize, measure: &mut Measure| {
            let token = line
              .tokens
              .iter()
              .find(|token| offset <= token.span.end)
              .unwrap_or(last);
            self.x_in(text, token, offset, measure)
          };
          [x(from, measure), line.top, x(to, measure), line.bottom]
        })
      })
      .collect()
  }

  /// The offset in line `line` nearest `x` across the block.
  pub fn offset_in_line(&self, text: &str, line: usize, x: f64, measure: &mut Measure) -> usize {
    let Some(line) = self.lines.get(line) else {
      return text.len();
    };
    let mut nearest = (f64::INFINITY, line.tokens[0].span.start);
    for token in &line.tokens {
      let word = text
        .get(token.span.start..token.span.end)
        .unwrap_or_default();
      for offset in word
        .char_indices()
        .map(|(index, _)| token.span.start + index)
        .chain([token.span.end])
      {
        let distance = (self.x_in(text, token, offset, measure) - x).abs();
        if distance < nearest.0 {
          nearest = (distance, offset);
        }
      }
    }
    nearest.1
  }

  /// The offset nearest `point`, a point in the block's own space.
  pub fn offset_at(&self, text: &str, point: (f64, f64), measure: &mut Measure) -> usize {
    self.offset_in_line(text, self.line_at(point.1), point.0, measure)
  }
}

/// A text item's layout as the typing goes by it, measuring with `measure`.
/// The text it is asked about is the one it was laid out from.
pub struct ItemTyping<'l, 'm> {
  pub layout: &'l TextItemLayout,
  pub measure: &'l mut Measure<'m>,
}

impl TypingLines for ItemTyping<'_, '_> {
  fn point_of(&mut self, text: &str, offset: usize) -> (f64, f64) {
    self.layout.point_of(text, offset, self.measure)
  }

  fn line_at(&mut self, _: &str, y: f64) -> usize {
    self.layout.line_at(y)
  }

  fn offset_in_line(&mut self, text: &str, line: usize, x: f64) -> usize {
    self.layout.offset_in_line(text, line, x, self.measure)
  }
}

#[cfg(test)]
mod tests;
