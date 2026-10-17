// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A text placed by hand, drawn in its look as a caption is: its lines as
//! typed, set about the middle of its block, the whole coming on and going
//! off as the look's motion has it, and, where the look reveals its words,
//! each word coming on in turn. A text is not said, so its words come on
//! [`WORD_STEP_MS`] apart from its start, closer together where its time
//! before its exit is too short for that, and the look's accent and
//! highlight fall on the words chosen in it. Its box is not among the
//! pieces: the text's own box draws it.
//!
//! Each line is split at every white space, so the words keep the spaces
//! typed between them: two in a row leave an empty word between, which takes
//! no room but the space after it. [`layout`] reads where each byte of the
//! text sits from the same words.

use super::caption::pieces_of;
use super::marked::Highlighted;
use super::place::Ground;
use super::{CaptionMoment, CaptionPiece, CaptionPieceKind};
use crate::captions::{Caption, CaptionWord, Measure};
use crate::emphasis::{chosen_words, is_chosen, TextSpan};
use crate::motion::Mirror;
use crate::style::{TextAccent, TextStyle};

pub mod layout;

/// How long after one word of a text revealed word by word the next comes
/// on, in milliseconds.
const WORD_STEP_MS: f64 = 80.0;

/// Where a text's type is set and when it is drawn: the middle of its block
/// and its type size, in the canvas's pixels; how its lines line up, as a
/// share of the block's width, 0 from the left and 1 from the right; and
/// how far into its time of `duration_ms` the moment is.
#[derive(Clone, Copy, Debug)]
pub struct TextItemSetting {
  pub centre: [f64; 2],
  pub size: f64,
  pub align: f64,
  pub duration_ms: f64,
}

/// Whether `look` moves a text at all: it comes on, goes off, or reveals
/// its words. One that does none of these stands as the text's box does.
pub fn moves_text(look: &TextStyle) -> bool {
  let motion = &look.motion;
  look.reveal_words
    || !motion.caption.is_empty()
    || !motion.caption_out.is_empty()
    || !motion.word_out.is_empty()
}

/// `text`'s lines, each as the bytes of its words: split at every white
/// space but a line break, which starts the next line.
fn tokens(text: &str) -> Vec<Vec<TextSpan>> {
  let mut lines = Vec::new();
  let mut line = Vec::new();
  let mut start = 0;
  for (index, character) in text.char_indices() {
    if character.is_whitespace() {
      line.push(TextSpan { start, end: index });
      start = index + character.len_utf8();
      if character == '\n' {
        lines.push(std::mem::take(&mut line));
      }
    }
  }
  line.push(TextSpan {
    start,
    end: text.len(),
  });
  lines.push(line);
  lines
}

/// A text as the caption it is drawn as: its words, the bytes each takes
/// in the text, and which of them the highlight falls on.
pub(super) struct ItemCaption {
  pub(super) caption: Caption,
  pub(super) spans: Vec<Vec<TextSpan>>,
  pub(super) highlighted: Vec<bool>,
}

/// `text` as `style`, normalized, sets it over a time of `duration_ms`,
/// `chosen` set apart as the style sets words apart.
pub(super) fn item_caption(
  text: &str,
  style: &TextStyle,
  chosen: &[TextSpan],
  duration_ms: f64,
) -> ItemCaption {
  let chosen = chosen_words(text, chosen);
  let spans = tokens(text);
  let count = spans
    .iter()
    .flatten()
    .filter(|span| span.start < span.end)
    .count()
    .max(1);
  // Every word is in before the text starts going off, however short its
  // time.
  let span_ms = (duration_ms - style.motion.exit_ms()).max(0.0);
  let mut at = 0;
  let mut highlighted = Vec::new();
  let lines = spans
    .iter()
    .map(|line| {
      line
        .iter()
        .map(|&span| {
          let start_ms = (WORD_STEP_MS * at as f64).min(span_ms * at as f64 / count as f64);
          let word = &text[span.start..span.end];
          at += usize::from(!word.is_empty());
          let set_apart = !word.is_empty() && is_chosen(&chosen, span);
          highlighted.push(set_apart && style.highlight_words);
          CaptionWord {
            text: if style.all_caps {
              word.to_uppercase()
            } else {
              word.to_owned()
            },
            said: word.to_owned(),
            start_ms,
            end_ms: duration_ms,
            source_start_ms: start_ms,
            source_end_ms: duration_ms,
            emphasis: set_apart && style.accent == TextAccent::Word,
          }
        })
        .collect()
    })
    .collect();
  ItemCaption {
    caption: Caption {
      id: 0,
      start_ms: 0.0,
      end_ms: duration_ms,
      text: text.to_owned(),
      lines,
      spaced: true,
    },
    spans,
    highlighted,
  }
}

/// `look` as a text sets it: normalized, every word at once.
pub(super) fn item_style(look: &TextStyle) -> TextStyle {
  TextStyle {
    visible_words: 0,
    ..look.normalized()
  }
}

/// The pieces of `text`, as `look` shows it with the words of `chosen` set
/// apart, drawn at `moment` into its time as `setting` places it. `measure`
/// gives a text's width set in a face at a type size, both in pixels.
pub fn text_item_pieces(
  text: &str,
  look: &TextStyle,
  chosen: &[TextSpan],
  setting: TextItemSetting,
  moment: CaptionMoment,
  measure: &mut Measure,
) -> Vec<CaptionPiece> {
  let style = item_style(look);
  let item = item_caption(text, &style, chosen, setting.duration_ms.max(0.0));
  let ground = Ground::Block {
    centre: setting.centre,
    size: setting.size,
    align: setting.align,
  };
  let mirror = Mirror {
    across: setting.align >= 1.0,
    down: false,
  };
  pieces_of(
    &item.caption,
    &style,
    (ground, moment),
    measure,
    mirror,
    Highlighted::Chosen(&item.highlighted),
  )
  .into_iter()
  .filter(|piece| match piece.kind {
    CaptionPieceKind::Backdrop { .. } => false,
    CaptionPieceKind::Word { .. } => !piece.text.is_empty(),
    _ => true,
  })
  .collect()
}

#[cfg(test)]
mod tests;
