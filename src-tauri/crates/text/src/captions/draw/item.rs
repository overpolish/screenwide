// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A text placed by hand, drawn in its look's motion as a caption is: its
//! lines as typed, set about the middle of its block, the whole coming on
//! and going off as the look's motion has it, and, where the look reveals
//! its words, each word coming on in turn. A text is not said, so its words
//! come on evenly over its time before its exit, and nothing marks a word
//! being said. Its box is not among the pieces: the text's own box draws it.

use super::caption::pieces_of;
use super::place::Ground;
use super::{CaptionMoment, CaptionPiece, CaptionPieceKind};
use crate::captions::{Caption, CaptionWord, Measure};
use crate::motion::Mirror;
use crate::style::TextStyle;

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

/// The pieces of `text`, as `look` shows it, drawn at `moment` into its time
/// as `setting` places it. `measure` gives a text's width set in a face at a
/// type size, both in pixels.
pub fn text_item_pieces(
  text: &str,
  look: &TextStyle,
  setting: TextItemSetting,
  moment: CaptionMoment,
  measure: &mut Measure,
) -> Vec<CaptionPiece> {
  let style = TextStyle {
    highlight_words: false,
    visible_words: 0,
    ..look.normalized()
  };
  let duration_ms = setting.duration_ms.max(0.0);
  let lines: Vec<Vec<&str>> = text
    .split('\n')
    .map(|line| line.split_whitespace().collect())
    .collect();
  let count = lines.iter().map(Vec::len).sum::<usize>().max(1);
  // Over the time before the text starts going off, so the last word is in
  // before it leaves.
  let span_ms = (duration_ms - style.motion.exit_ms()).max(0.0);
  let mut at = 0;
  let mut word = |text: &str, counted: bool| {
    let start_ms = span_ms * at as f64 / count as f64;
    at += usize::from(counted);
    CaptionWord {
      text: text.to_owned(),
      start_ms,
      end_ms: duration_ms,
      source_start_ms: start_ms,
      source_end_ms: duration_ms,
      emphasis: false,
    }
  };
  let caption = Caption {
    id: 0,
    start_ms: 0.0,
    end_ms: duration_ms,
    text: text.to_owned(),
    lines: lines
      .iter()
      .map(|line| {
        if line.is_empty() {
          // An empty line keeps its height, as the text's box does.
          vec![word("", false)]
        } else {
          line.iter().map(|text| word(text, true)).collect()
        }
      })
      .collect(),
    spaced: true,
  };
  let ground = Ground::Block {
    centre: setting.centre,
    size: setting.size,
    align: setting.align,
  };
  let mirror = Mirror {
    across: setting.align >= 1.0,
    down: false,
  };
  pieces_of(&caption, &style, ground, moment, measure, mirror)
    .into_iter()
    .filter(|piece| matches!(piece.kind, CaptionPieceKind::Word { .. }) && !piece.text.is_empty())
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_texts_words_come_on_evenly_over_its_time_in_its_lines_as_typed() {
    let look = TextStyle {
      reveal_words: true,
      ..TextStyle::default()
    };
    assert!(moves_text(&look));
    assert!(!moves_text(&TextStyle::default()));
    let setting = TextItemSetting {
      centre: [500.0, 300.0],
      size: 20.0,
      align: 0.0,
      duration_ms: 4_000.0,
    };
    let mut measure = |text: &str, _, size: f64| text.chars().count() as f64 * size * 0.5;
    let mut at = |at_ms: f64| {
      text_item_pieces(
        "one two\n\nthree four",
        &look,
        setting,
        CaptionMoment::still(at_ms),
        &mut measure,
      )
    };
    let words = |pieces: Vec<CaptionPiece>| -> Vec<String> {
      pieces.into_iter().map(|piece| piece.text).collect()
    };
    assert_eq!(words(at(500.0)), ["one"]);
    assert_eq!(words(at(2_500.0)), ["one", "two", "three"]);
    let all = at(3_900.0);
    assert_eq!(words(all.clone()), ["one", "two", "three", "four"]);
    // Lined up from the left of the block, the empty line between keeping
    // its height: three lines 25 pixels apart about the middle.
    let [one, _, three, _] = &all[..] else {
      unreachable!()
    };
    assert_eq!(one.centre[0] - three.centre[0], (3.0 - 5.0) * 10.0 / 2.0);
    assert_eq!(three.centre[1] - one.centre[1], 2.0 * 20.0 * 1.25);
  }
}
