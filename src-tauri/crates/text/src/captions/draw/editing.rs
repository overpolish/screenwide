// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A caption being typed into on the picture. While it is, it stands still
//! with every word showing, its words as typed so far laid out where the
//! style's place puts a caption of them, so the caret and a press on the
//! words land where they show. Its key words keep the accent; the word being
//! said is not marked, since nothing is said while the video stands still.

use super::caption::pieces_of;
use super::item::{item_caption, item_style};
use super::marked::Highlighted;
use super::place::Ground;
use super::{CaptionMoment, CaptionPiece, CaptionPieceKind, CaptionTrack, LINE_HEIGHT};
use crate::captions::{Caption, CaptionFrame, Measure, TextItemLayout};
use crate::emphasis::TextSpan;
use crate::motion::{Mirror, TextMotion};
use crate::style::TextStyle;

/// The caption being typed into: which caption it is, by its id, when it
/// shows on the edited video, in milliseconds, and its words as typed so far
/// with its key words, each by its bytes in the text.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptionEditing {
  pub id: u64,
  pub shows_ms: [f64; 2],
  pub text: String,
  pub emphasis: Vec<TextSpan>,
}

impl CaptionEditing {
  /// `caption` opened for typing: its words as the transcript spells them,
  /// its lines as it shows them, and its key words.
  pub fn of(caption: &Caption) -> Self {
    let gap = if caption.spaced { " " } else { "" };
    let mut text = String::new();
    let mut emphasis = Vec::new();
    for (index, line) in caption.lines.iter().enumerate() {
      if index > 0 {
        text.push('\n');
      }
      for (at, word) in line.iter().enumerate() {
        if at > 0 {
          text.push_str(gap);
        }
        let start = text.len();
        text.push_str(&word.said);
        if word.emphasis {
          emphasis.push(TextSpan {
            start,
            end: text.len(),
          });
        }
      }
    }
    Self {
      id: caption.id,
      shows_ms: [caption.start_ms, caption.end_ms],
      text,
      emphasis,
    }
  }
}

/// `style` as a caption being typed into is drawn in: standing still, every
/// word showing at once, and no word marked as said.
pub fn editing_style(style: &TextStyle) -> TextStyle {
  TextStyle {
    motion: TextMotion {
      caption: Vec::new(),
      word: Vec::new(),
      caption_out: Vec::new(),
      word_out: Vec::new(),
    },
    reveal_words: false,
    highlight_words: false,
    visible_words: 0,
    ..style.normalized()
  }
}

impl CaptionTrack {
  /// The caption showing at `at_ms` into the edited video, if any.
  pub fn showing_at(&self, at_ms: f64) -> Option<&Caption> {
    super::showing(&self.captions, at_ms)
  }

  /// The caption being typed into, where it shows at `at_ms`.
  pub(super) fn editing_at(&self, at_ms: f64) -> Option<&CaptionEditing> {
    self
      .editing
      .as_ref()
      .filter(|editing| editing.shows_ms[0] <= at_ms && at_ms < editing.shows_ms[1])
  }

  /// The caption being typed into as it is laid out in a picture `frame` in
  /// size: what the caret, the selection and a press on it go by.
  pub fn editing_layout(
    &self,
    frame: CaptionFrame,
    measure: &mut Measure,
  ) -> Option<TextItemLayout> {
    let editing = self.editing.as_ref()?;
    let style = editing_style(&self.recording.style);
    Some(TextItemLayout::in_frame(
      &editing.text,
      &style,
      &editing.emphasis,
      frame,
      measure,
    ))
  }

  /// Which word of the caption showing at `moment`, counted across its
  /// lines, is drawn under `point`, in the pixels of a picture `frame` in
  /// size. None over no word, and while a caption is typed into.
  pub fn word_at(
    &self,
    frame: CaptionFrame,
    moment: CaptionMoment,
    point: [f64; 2],
    measure: &mut Measure,
  ) -> Option<usize> {
    if self.editing.is_some() {
      return None;
    }
    self
      .pieces_at(frame, moment, measure)
      .iter()
      .filter(|piece| piece.opacity[0] > 0.01)
      .find_map(|piece| {
        let word = piece.word?;
        let half = [
          measure(&piece.text, piece.face, piece.size) * piece.scale[0] / 2.0,
          piece.size * LINE_HEIGHT * piece.scale[0] / 2.0,
        ];
        ((point[0] - piece.centre[0]).abs() <= half[0]
          && (point[1] - piece.centre[1]).abs() <= half[1])
          .then_some(word)
      })
  }
}

/// What the caption being typed into draws, set as `style` sets a caption
/// in a picture `frame` in size.
pub(super) fn editing_pieces(
  editing: &CaptionEditing,
  style: &TextStyle,
  frame: CaptionFrame,
  measure: &mut Measure,
) -> Vec<CaptionPiece> {
  let style = item_style(&editing_style(style));
  let item = item_caption(&editing.text, &style, &editing.emphasis, 0.0);
  let [across, down] = style.position.shares();
  let mirror = Mirror {
    across: across >= 1.0,
    down: down <= 0.0,
  };
  pieces_of(
    &item.caption,
    &style,
    (Ground::from(frame), CaptionMoment::still(0.0)),
    measure,
    mirror,
    Highlighted::Chosen(&item.highlighted),
  )
  .into_iter()
  .filter(|piece| !matches!(piece.kind, CaptionPieceKind::Word { .. }) || !piece.text.is_empty())
  .collect()
}

#[cfg(test)]
mod tests;
