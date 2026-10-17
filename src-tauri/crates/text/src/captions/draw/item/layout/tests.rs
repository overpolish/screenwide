// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::style::TextAccent;

/// Every character half an em wide, in either face.
fn measure(text: &str, _: TextFace, size: f64) -> f64 {
  text.chars().count() as f64 * size * 0.5
}

fn laid_out(text: &str, chosen: &[TextSpan]) -> TextItemLayout {
  let look = TextStyle {
    accent: TextAccent::Word,
    accent_size_percent: 200.0,
    ..TextStyle::default()
  };
  TextItemLayout::new(text, &look, chosen, 20.0, 0.0, &mut measure)
}

#[test]
fn sizes_its_block_by_the_words_set_apart() {
  let plain = laid_out("big cat\nsat", &[]);
  assert_eq!(plain.block, [70.0, 2.0 * 25.0]);
  let set_apart = laid_out("big cat\nsat", &[TextSpan { start: 4, end: 7 }]);
  // The accent takes "cat" twice as wide and its line twice as tall.
  assert!(set_apart.block[0] > plain.block[0]);
  assert_eq!(
    set_apart.block[1],
    25.0 + set_apart.lines[0].bottom - set_apart.lines[0].top
  );
}

#[test]
fn reads_presses_and_the_caret_from_where_the_words_are_drawn() {
  let text = "big cat\nsat";
  let layout = laid_out(text, &[TextSpan { start: 4, end: 7 }]);
  let mut measure = measure;
  let cat = layout.point_of(text, 4, &mut measure).0;
  let end = layout.point_of(text, 7, &mut measure).0;
  // Each letter of the word set apart is twice as wide as a plain one.
  let one = layout.point_of(text, 5, &mut measure).0 - cat;
  assert_eq!(one, 20.0);
  assert_eq!(end - cat, 60.0);
  // A press lands on the nearest boundary, on the line it is over.
  let bottom = layout.lines[0].bottom;
  assert_eq!(
    layout.offset_at(text, (cat + one * 1.4, 1.0), &mut measure),
    5
  );
  assert_eq!(layout.offset_at(text, (0.0, bottom + 1.0), &mut measure), 8);
  assert_eq!(layout.offset_at(text, (1_000.0, 1.0), &mut measure), 7);
  // The caret is as tall as the type it stands in.
  let tall = layout.caret(text, 5, &mut measure);
  let short = layout.caret(text, 1, &mut measure);
  assert!(tall[2] - tall[1] > short[2] - short[1]);
  // A selection across the break takes a rectangle on each line.
  assert_eq!(layout.selection(text, 5, 10, &mut measure).len(), 2);
}
