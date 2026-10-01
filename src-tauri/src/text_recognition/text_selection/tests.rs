// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::text_recognition::{RecognizedCharacter, RecognizedLine};

fn bounds(x: f64, y: f64, width: f64, height: f64) -> TextRect {
  TextRect {
    x,
    y,
    width,
    height,
  }
}

fn line(text: &str, bounds: TextRect, characters: Vec<RecognizedCharacter>) -> RecognizedLine {
  RecognizedLine {
    text: text.to_owned(),
    confidence: 1.0,
    bounds,
    characters,
  }
}

fn result() -> TextRecognitionResult {
  TextRecognitionResult {
    lines: vec![
      line(
        "hello",
        bounds(0.1, 0.1, 0.5, 0.1),
        vec![
          RecognizedCharacter {
            start: 0,
            end: 1,
            bounds: bounds(0.1, 0.1, 0.1, 0.1),
          },
          RecognizedCharacter {
            start: 1,
            end: 2,
            bounds: bounds(0.2, 0.1, 0.1, 0.1),
          },
        ],
      ),
      line("world", bounds(0.1, 0.5, 0.5, 0.1), Vec::new()),
    ],
    qr_codes: Vec::new(),
    text: "hello\nworld".to_owned(),
  }
}

#[test]
fn position_matches_character_midpoints_and_nearest_line() {
  let result = result();
  assert_eq!(
    text_position_at(&result, Point { x: 0.12, y: 0.12 }),
    Some(TextPosition { line: 0, offset: 0 })
  );
  assert_eq!(
    text_position_at(&result, Point { x: 0.29, y: 0.4 }),
    Some(TextPosition { line: 1, offset: 2 })
  );
}

#[test]
fn line_hit_testing_uses_both_axes_for_columns_on_the_same_row() {
  let result = TextRecognitionResult {
    lines: vec![
      line("left", bounds(0.06, 0.22, 0.05, 0.02), Vec::new()),
      line("right", bounds(0.45, 0.22, 0.08, 0.02), Vec::new()),
    ],
    qr_codes: Vec::new(),
    text: "left\nright".to_owned(),
  };

  assert_eq!(
    text_position_at(&result, Point { x: 0.46, y: 0.23 }),
    Some(TextPosition { line: 1, offset: 1 })
  );
}

#[test]
fn overlapping_character_boxes_advance_at_ordered_caret_boundaries() {
  let result = TextRecognitionResult {
    lines: vec![line(
      "ab",
      bounds(0.1, 0.1, 0.4, 0.1),
      vec![
        RecognizedCharacter {
          start: 0,
          end: 1,
          bounds: bounds(0.1, 0.1, 0.3, 0.1),
        },
        RecognizedCharacter {
          start: 1,
          end: 2,
          bounds: bounds(0.2, 0.1, 0.3, 0.1),
        },
      ],
    )],
    qr_codes: Vec::new(),
    text: "ab".to_owned(),
  };

  assert_eq!(
    text_position_at(&result, Point { x: 0.37, y: 0.15 }),
    Some(TextPosition { line: 0, offset: 2 })
  );
}

#[test]
fn reverse_ranges_use_character_boxes_and_fallback_slices() {
  let result = result();
  let range = ordered_range(
    TextPosition { line: 1, offset: 3 },
    TextPosition { line: 0, offset: 1 },
  );
  let rects = selection_rects(&result, &[range]);

  assert_rect_close(rects[0], bounds(0.2, 0.1, 0.1, 0.1));
  assert_rect_close(rects[1], bounds(0.1, 0.5, 0.3, 0.1));
}

fn assert_rect_close(actual: TextRect, expected: TextRect) {
  assert!((actual.x - expected.x).abs() < f64::EPSILON * 2.0);
  assert!((actual.y - expected.y).abs() < f64::EPSILON * 2.0);
  assert!((actual.width - expected.width).abs() < f64::EPSILON * 2.0);
  assert!((actual.height - expected.height).abs() < f64::EPSILON * 2.0);
}

#[test]
fn overlapping_ranges_merge_for_text_extraction() {
  let result = result();
  let ranges = [
    TextRange {
      start: TextPosition { line: 0, offset: 0 },
      end: TextPosition { line: 0, offset: 3 },
    },
    TextRange {
      start: TextPosition { line: 0, offset: 2 },
      end: TextPosition { line: 1, offset: 2 },
    },
  ];
  assert_eq!(selected_text(&result, &ranges), "hello\nwo");
}

#[test]
fn selection_offsets_are_utf16_like_the_frontend() {
  let result = TextRecognitionResult {
    lines: vec![line("A😀B", bounds(0.0, 0.0, 1.0, 1.0), Vec::new())],
    qr_codes: Vec::new(),
    text: "A😀B".to_owned(),
  };
  let range = TextRange {
    start: TextPosition { line: 0, offset: 1 },
    end: TextPosition { line: 0, offset: 3 },
  };
  assert_eq!(selected_text(&result, &[range]), "😀");
  assert_eq!(
    selection_rects(&result, &[range])[0],
    bounds(0.25, 0.0, 0.5, 1.0)
  );
}

#[test]
fn pointer_drag_additive_double_click_and_select_all_match_legacy_behavior() {
  let mut selection = TextSelection::new(result());
  assert!(selection.pointer_down(Point { x: 0.12, y: 0.12 }, false, false));
  assert!(selection.pointer_move(Point { x: 0.29, y: 0.55 }));
  assert!(selection.pointer_up(Point { x: 0.29, y: 0.55 }));
  assert_eq!(selection.selected_text(), "hello\nwo");

  assert!(selection.pointer_down(Point { x: 0.2, y: 0.52 }, true, true));
  assert_eq!(selection.selected_text(), "hello\nworld");

  assert!(selection.select_all());
  assert_eq!(selection.selected_text(), "hello\nworld");
}

#[test]
fn paragraph_collapses_trimmed_lines() {
  assert_eq!(
    paragraph(" first \n\n second\r\n third "),
    "first second third"
  );
}
