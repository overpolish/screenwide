// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::decode::{read, Dictionary, Symbol};
use super::detect::{boxes, model_size, TextBox};

const CONFIG: &str = "PostProcess:\n  name: CTCLabelDecode\n  character_dict:\n  - l\n  - o\n  - ''''\n  - '#'\nPreProcess:\n  - nothing\n";

/// One row of chances per step, sure of `class` at `chance`.
fn steps(classes: &[usize], chance: f32, dictionary: &Dictionary) -> Vec<f32> {
  let mut rows = vec![0.0; classes.len() * dictionary.classes()];
  for (step, &class) in classes.iter().enumerate() {
    rows[step * dictionary.classes() + class] = chance;
  }
  rows
}

#[test]
fn the_dictionary_reads_plain_and_quoted_characters_and_ends_at_the_list() {
  let dictionary = Dictionary::from_config(CONFIG).unwrap();
  // Blank, four characters, space.
  assert_eq!(dictionary.classes(), 6);
  let text = |classes: &[usize]| {
    read(&steps(classes, 0.9, &dictionary), &dictionary)
      .symbols
      .into_iter()
      .map(|symbol| symbol.text)
      .collect::<String>()
  };
  assert_eq!(text(&[3, 0, 4, 5, 1]), "'# l");
}

#[test]
fn a_held_class_is_one_character_and_a_blank_between_repeats_makes_two() {
  let dictionary = Dictionary::from_config(CONFIG).unwrap();
  let reading = read(
    &steps(&[0, 1, 1, 0, 1, 2, 2, 0], 0.8, &dictionary),
    &dictionary,
  );
  assert_eq!(
    reading.symbols,
    [
      Symbol {
        text: "l".into(),
        steps: (1, 3)
      },
      Symbol {
        text: "l".into(),
        steps: (4, 5)
      },
      Symbol {
        text: "o".into(),
        steps: (5, 7)
      },
    ]
  );
  assert!((reading.confidence - 0.8).abs() < 1e-6);
}

#[test]
fn a_line_of_blanks_reads_as_nothing_and_is_not_sure() {
  let dictionary = Dictionary::from_config(CONFIG).unwrap();
  let reading = read(&steps(&[0, 0, 0], 0.99, &dictionary), &dictionary);
  assert!(reading.symbols.is_empty());
  assert_eq!(reading.confidence, 0.0);
}

#[test]
fn a_configuration_without_characters_is_refused() {
  assert!(Dictionary::from_config("PostProcess:\n  name: CTCLabelDecode\n").is_err());
}

#[test]
fn small_pictures_are_scaled_up_and_large_ones_down_in_steps_of_32() {
  assert_eq!(model_size(232, 232), (736, 736));
  assert_eq!(model_size(1920, 1080), (1920, 1088));
  assert_eq!(model_size(4000, 3000), (2016, 1504));
}

/// A chance map `width` by `height` with `areas` marked sure.
fn chances(width: usize, height: usize, areas: &[([usize; 4], f32)]) -> Vec<f32> {
  let mut map = vec![0.0; width * height];
  for &([left, top, right, bottom], chance) in areas {
    for y in top..=bottom {
      for x in left..=right {
        map[y * width + x] = chance;
      }
    }
  }
  map
}

#[test]
fn a_line_is_grown_back_out_from_the_centre_the_model_marks() {
  let map = chances(100, 50, &[([10, 20, 49, 29], 0.9)]);
  // A 39 by 9 area grows by 39 * 9 * 1.4 / (2 * 48) on every side.
  let grow = 39.0 * 9.0 * 1.4 / 96.0;
  let found = boxes(&map, (100, 50), (200, 100));
  assert_eq!(found.len(), 1);
  let expected = TextBox {
    left: (10.0 - grow) * 2.0,
    top: (20.0 - grow) * 2.0,
    right: (49.0 + grow) * 2.0,
    bottom: (29.0 + grow) * 2.0,
  };
  for (got, want) in [
    (found[0].left, expected.left),
    (found[0].top, expected.top),
    (found[0].right, expected.right),
    (found[0].bottom, expected.bottom),
  ] {
    assert!((got - want).abs() < 1e-3, "{found:?} against {expected:?}");
  }
}

#[test]
fn thin_and_unsure_areas_are_not_lines() {
  let map = chances(
    100,
    60,
    &[
      // Two pixels tall.
      ([10, 5, 60, 7], 0.9),
      // Each pixel just sure enough to count, the area not sure enough.
      ([10, 30, 60, 40], 0.3),
    ],
  );
  assert!(boxes(&map, (100, 60), (100, 60)).is_empty());
}

#[test]
fn lines_on_one_row_read_left_to_right_whichever_sits_higher() {
  let map = chances(
    200,
    100,
    &[
      ([120, 20, 180, 30], 0.9),
      ([10, 24, 80, 34], 0.9),
      ([10, 60, 80, 70], 0.9),
    ],
  );
  let lefts = boxes(&map, (200, 100), (200, 100))
    .iter()
    .map(|found| (found.left < 100.0, found.top < 50.0))
    .collect::<Vec<_>>();
  assert_eq!(lefts, [(true, true), (false, true), (true, false)]);
}
