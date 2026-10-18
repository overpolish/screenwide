// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn character(start: usize, x: f64, width: f64) -> RecognizedCharacter {
  RecognizedCharacter {
    start,
    end: start + 1,
    bounds: TextRect {
      x,
      y: 0.4,
      width,
      height: 0.2,
    },
  }
}

#[test]
fn a_word_box_shared_by_its_letters_is_divided_between_them() {
  // "HI OK": Vision's answer for every letter is its word's box.
  let mut characters = vec![
    character(0, 0.125, 0.25),
    character(1, 0.125, 0.25),
    character(3, 0.5, 0.5),
    character(4, 0.5, 0.5),
  ];
  split_shared_boxes(&mut characters);
  let spans = characters
    .iter()
    .map(|character| (character.bounds.x, character.bounds.width))
    .collect::<Vec<_>>();
  assert_eq!(
    spans,
    [(0.125, 0.125), (0.25, 0.125), (0.5, 0.25), (0.75, 0.25)]
  );
}

#[test]
fn letters_with_boxes_of_their_own_keep_them() {
  let mut characters = vec![character(0, 0.1, 0.05), character(1, 0.16, 0.05)];
  split_shared_boxes(&mut characters);
  assert_eq!(characters[0].bounds.x, 0.1);
  assert_eq!(characters[1].bounds.x, 0.16);
  assert_eq!(characters[1].bounds.width, 0.05);
}
