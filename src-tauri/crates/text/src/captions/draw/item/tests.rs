// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::style::{TextBackground, TextFace};

/// Every character half an em wide, in either face.
fn measure(text: &str, _: TextFace, size: f64) -> f64 {
  text.chars().count() as f64 * size * 0.5
}

fn setting() -> TextItemSetting {
  TextItemSetting {
    centre: [500.0, 300.0],
    size: 20.0,
    align: 0.0,
    duration_ms: 4_000.0,
  }
}

fn span(start: usize, end: usize) -> TextSpan {
  TextSpan { start, end }
}

fn still(text: &str, look: &TextStyle, chosen: &[TextSpan]) -> Vec<CaptionPiece> {
  text_item_pieces(
    text,
    look,
    chosen,
    setting(),
    CaptionMoment::still(0.0),
    &mut measure,
  )
}

#[test]
fn a_texts_words_come_on_one_after_another_in_its_lines_as_typed() {
  let look = TextStyle {
    reveal_words: true,
    ..TextStyle::default()
  };
  assert!(moves_text(&look));
  assert!(!moves_text(&TextStyle::default()));
  let at = |at_ms: f64, duration_ms: f64| {
    text_item_pieces(
      "one two\n\nthree four",
      &look,
      &[],
      TextItemSetting {
        duration_ms,
        ..setting()
      },
      CaptionMoment::still(at_ms),
      &mut measure,
    )
  };
  let words = |pieces: Vec<CaptionPiece>| -> Vec<String> {
    pieces.into_iter().map(|piece| piece.text).collect()
  };
  // A step apart from the start, whatever the text's time, the empty line
  // taking none.
  assert_eq!(words(at(50.0, 4_000.0)), ["one"]);
  assert_eq!(words(at(170.0, 4_000.0)), ["one", "two", "three"]);
  // A time too short for the steps takes the words closer together, so
  // the last is in before the text goes.
  assert_eq!(words(at(110.0, 200.0)), ["one", "two", "three"]);
  let all = at(3_900.0, 4_000.0);
  assert_eq!(words(all.clone()), ["one", "two", "three", "four"]);
  // Lined up from the left of the block, the empty line between keeping
  // its height: three lines 25 pixels apart about the middle.
  let [one, _, three, _] = &all[..] else {
    unreachable!()
  };
  assert_eq!(one.centre[0] - three.centre[0], (3.0 - 5.0) * 10.0 / 2.0);
  assert_eq!(three.centre[1] - one.centre[1], 2.0 * 20.0 * 1.25);
}

#[test]
fn keeps_the_spaces_typed_between_words() {
  let look = TextStyle::default();
  let single = still("a b", &look, &[]);
  let double = still("a  b", &look, &[]);
  // A second space puts the next word another space along.
  let gap = |pieces: &[CaptionPiece]| pieces[1].centre[0] - pieces[0].centre[0];
  assert_eq!(gap(&double) - gap(&single), 10.0);
}

#[test]
fn sets_only_the_chosen_words_in_the_accent() {
  let look = TextStyle {
    accent: TextAccent::Word,
    ..TextStyle::default()
  };
  let faces = |chosen: &[TextSpan]| -> Vec<TextFace> {
    still("save your work", &look, chosen)
      .iter()
      .map(|piece| piece.face)
      .collect()
  };
  // Nothing is set apart until a word is chosen: no word is picked by rule.
  assert_eq!(faces(&[]), [TextFace::Main; 3]);
  assert_eq!(
    faces(&[span(5, 9)]),
    [TextFace::Main, TextFace::Accent, TextFace::Main]
  );
}

#[test]
fn puts_a_pill_behind_every_chosen_word_and_one_line_under_a_run() {
  let pill = TextStyle {
    highlight_words: true,
    background: TextBackground::Word,
    ..TextStyle::default()
  };
  let pills = still("one two three", &pill, &[span(0, 3), span(8, 13)])
    .iter()
    .filter(|piece| matches!(piece.kind, CaptionPieceKind::Pill { .. }))
    .count();
  assert_eq!(pills, 2);
  let underline = TextStyle {
    background: TextBackground::Underline,
    ..pill
  };
  let lines: Vec<f64> = still("one two three", &underline, &[span(0, 3), span(4, 7)])
    .iter()
    .filter_map(|piece| match piece.kind {
      CaptionPieceKind::Underline { block, .. } => Some(block[0]),
      _ => None,
    })
    .collect();
  // "one two", a space included, at ten pixels a character.
  assert_eq!(lines, [70.0]);
}
