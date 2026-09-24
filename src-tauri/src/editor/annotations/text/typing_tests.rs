// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Motion, TextTyping, TypingLayout, TypingMarks};

/// Every character ten wide, lines twelve and a half high.
fn layout(width: &dyn Fn(&str) -> f64, block_width: f64, align: u32) -> TypingLayout<'_> {
  TypingLayout {
    width,
    block_width,
    font: 10.0,
    align,
  }
}

fn ten(line: &str) -> f64 {
  line.chars().count() as f64 * 10.0
}

#[test]
fn typing_replaces_the_selection_and_undoes_as_one_step() {
  let mut typing = TextTyping::new("Hello world");
  typing.place(6, false);
  typing.place(11, true);
  for key in ["t", "h", "e", "r", "e"] {
    assert!(typing.insert(key));
  }
  assert_eq!(typing.text(), "Hello there");
  // Moving ends the step, so the next typing is a step of its own.
  typing.go(Motion::End, false);
  typing.insert("!");
  assert!(typing.undo());
  assert_eq!(typing.text(), "Hello there");
  assert!(typing.undo());
  assert_eq!(typing.text(), "Hello world");
  assert_eq!(
    typing.marks(false),
    TypingMarks {
      start: 6,
      end: 11,
      caret: false
    }
  );
  assert!(!typing.undo());
  assert!(typing.redo());
  assert_eq!(typing.text(), "Hello there");
}

#[test]
fn pasted_text_keeps_its_line_breaks_and_drops_what_a_box_cannot_show() {
  let mut typing = TextTyping::new("");
  assert!(typing.paste("one\r\ntwo\rthree\u{7}\tfour"));
  assert_eq!(typing.text(), "one\ntwo\nthreefour");
  assert!(!typing.paste("\u{1b}"));
}

#[test]
fn erasing_takes_whole_characters_and_whole_words() {
  let mut typing = TextTyping::new("café 👋");
  assert!(typing.erase(true, false));
  assert_eq!(typing.text(), "café ");
  typing.go(Motion::Left, false);
  assert!(typing.erase(true, false));
  assert_eq!(typing.text(), "caf ");
  let mut words = TextTyping::new("one two, three");
  words.go(Motion::Start, false);
  assert!(words.erase(false, true));
  assert_eq!(words.text(), "two, three");
  words.go(Motion::End, false);
  assert!(words.erase(true, true));
  assert_eq!(words.text(), "two, ");
}

#[test]
fn word_and_line_keys_move_the_way_windows_text_fields_do() {
  let mut typing = TextTyping::new("one two\nthree");
  typing.go(Motion::Start, false);
  typing.go(Motion::WordRight, false);
  assert_eq!(typing.marks(true).start, 4, "the start of the next word");
  typing.go(Motion::LineEnd, true);
  assert_eq!(typing.selected(), "two");
  typing.go(Motion::WordRight, false);
  typing.go(Motion::LineStart, false);
  assert_eq!(typing.marks(true).start, 8, "the start of the second line");
  typing.go(Motion::WordLeft, false);
  assert_eq!(typing.marks(true).start, 4);
  typing.select_all();
  assert_eq!(typing.selected(), "one two\nthree");
}

#[test]
fn an_arrow_key_collapses_a_selection_to_the_edge_it_points_at() {
  let mut typing = TextTyping::new("abcdef");
  typing.place(2, false);
  typing.place(4, true);
  typing.go(Motion::Left, false);
  assert_eq!(
    typing.marks(true),
    TypingMarks {
      start: 2,
      end: 2,
      caret: true
    }
  );
  typing.place(4, true);
  typing.go(Motion::Right, false);
  assert_eq!(typing.marks(true).start, 4);
}

#[test]
fn line_moves_keep_to_where_they_began_across_a_short_line() {
  let width = ten;
  let set = layout(&width, 80.0, 0);
  let mut typing = TextTyping::new("abcdefgh\nab\nabcdefgh");
  typing.place(6, false);
  typing.go_line(true, false, &set);
  assert_eq!(typing.marks(true).start, 11, "the end of the short line");
  typing.go_line(true, false, &set);
  assert_eq!(typing.marks(true).start, 18, "back to the sixth column");
  typing.go_line(true, true, &set);
  assert_eq!(
    typing.marks(true),
    TypingMarks {
      start: 18,
      end: 20,
      caret: true
    }
  );
}

#[test]
fn a_press_lands_on_the_nearest_gap_of_the_line_it_is_on() {
  let width = ten;
  // Centred in a block 80 wide, "ab" runs from 30 to 50.
  let set = layout(&width, 80.0, 1);
  let typing = TextTyping::new("abcdefgh\nab");
  assert_eq!(typing.offset_at((44.0, 13.0), &set), 9 + 1);
  assert_eq!(typing.offset_at((5.0, 13.0), &set), 9);
  assert_eq!(typing.offset_at((200.0, 900.0), &set), 11);
  assert_eq!(typing.offset_at((26.0, 0.0), &set), 3);
}

#[test]
fn a_double_click_selects_the_word_it_lands_in() {
  let mut typing = TextTyping::new("say hello_there , now");
  typing.select_word(7);
  assert_eq!(typing.selected(), "hello_there");
  // Beside a comma there is no word, so the comma alone.
  typing.select_word(16);
  assert_eq!(typing.selected(), ",");
}
