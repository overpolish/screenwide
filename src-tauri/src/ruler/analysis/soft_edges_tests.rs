// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn a_four_pixel_soft_left_edge_of_a_rounded_card_is_detected_beside_a_plus() {
  let mut canvas = Canvas::new(SIZE, SIZE, 0xFF);

  // The card's visible solid bounds are x=30..83, y=28..69. Its left edge
  // fades in over four pixels, while the other edges remain easy to see.
  for y in 28..70 {
    for x in 30..84 {
      let dx = if x < 37 {
        37 - x
      } else if x > 76 {
        x - 76
      } else {
        0
      };
      let dy = if y < 35 {
        35 - y
      } else if y > 62 {
        y - 62
      } else {
        0
      };
      if dx * dx + dy * dy <= 49 {
        canvas.set(x, y, 0xE5);
      }
    }
  }
  for y in 34..64 {
    canvas.set(30, y, 0xFB);
    canvas.set(31, y, 0xF5);
    canvas.set(32, y, 0xED);
    canvas.set(33, y, 0xE5);
  }
  // A dark plus sits six pixels to the right of the card's edge.
  canvas.fill(90, 45, 18, 4, 40);
  canvas.fill(97, 38, 4, 18, 40);

  let boxes = canvas.boxes(24);
  let card = boxes
    .iter()
    .find(|candidate| candidate.x <= 32 && candidate.y <= 30)
    .unwrap_or_else(|| panic!("no card box in {boxes:?}"));
  assert_box(card, (30, 28, 54, 42), 2);
  assert!(
    boxes
      .iter()
      .all(|candidate| { candidate.x + candidate.width <= 84 || candidate.x >= 90 }),
    "card and plus merged: {boxes:?}"
  );
}

#[test]
fn a_sharp_box_keeps_exact_bounds_at_the_same_threshold() {
  let mut canvas = Canvas::new(SIZE, SIZE, 0xFF);
  canvas.fill(20, 20, 40, 30, 0xE5);

  let boxes = canvas.boxes(24);
  assert_eq!(boxes.len(), 1, "unexpected boxes: {boxes:?}");
  assert_box(&boxes[0], (20, 20, 40, 30), 1);
}

#[test]
fn a_close_plus_does_not_extend_the_card_bounds() {
  let mut canvas = Canvas::new(SIZE, SIZE, 0xFF);
  canvas.fill(30, 30, 54, 40, 0xE5);
  canvas.fill(86, 44, 18, 4, 40);
  canvas.fill(93, 37, 4, 18, 40);

  let boxes = canvas.boxes(24);
  let card = boxes
    .iter()
    .find(|candidate| candidate.x <= 32 && candidate.y <= 32)
    .unwrap_or_else(|| panic!("no card box in {boxes:?}"));
  assert_box(card, (30, 30, 54, 40), 1);
  assert!(
    boxes
      .iter()
      .all(|candidate| { candidate.x + candidate.width <= 84 || candidate.x >= 86 }),
    "card and close plus merged: {boxes:?}"
  );
}
