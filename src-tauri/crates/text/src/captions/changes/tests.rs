// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::captions::CaptionWord;

/// A caption of `lines`, each word said over the next 100 ms of the source.
fn caption(lines: &[&[&str]], spaced: bool) -> Caption {
  let mut at = 0.0;
  let lines: Vec<Vec<CaptionWord>> = lines
    .iter()
    .map(|line| {
      line
        .iter()
        .map(|word| {
          at += 100.0;
          CaptionWord {
            text: word.to_uppercase(),
            said: (*word).to_owned(),
            start_ms: at - 100.0,
            end_ms: at,
            source_start_ms: at - 100.0,
            source_end_ms: at,
            emphasis: false,
          }
        })
        .collect()
    })
    .collect();
  Caption {
    id: 0,
    start_ms: 0.0,
    end_ms: at,
    text: String::new(),
    lines,
    spaced,
  }
}

fn corrected(start_ms: f64, end_ms: f64, text: &str) -> Correction {
  Correction {
    start_ms,
    end_ms,
    text: text.to_owned(),
  }
}

#[test]
fn leaves_a_caption_typed_back_as_it_was_unchanged() {
  let said = caption(&[&["we", "built"], &["it"]], true);
  assert_eq!(
    caption_changes(&said, "we built\nit"),
    CaptionChanges::default()
  );
  // Lines and spaces are the layout's, not the words'.
  assert_eq!(
    caption_changes(&said, "we  built it"),
    CaptionChanges::default()
  );
}

#[test]
fn corrects_only_the_words_spelled_another_way() {
  let said = caption(&[&["we", "bilt", "the", "thnig"]], true);
  let changes = caption_changes(&said, "we built the thing");
  assert_eq!(
    changes.corrections,
    [
      corrected(100.0, 200.0, "built"),
      corrected(300.0, 400.0, "thing")
    ]
  );
  assert!(changes.hidden.is_empty());
}

#[test]
fn hides_a_word_taken_out_whole() {
  let said = caption(&[&["so", "um", "yes"]], true);
  let changes = caption_changes(&said, "so yes");
  assert!(changes.corrections.is_empty());
  assert_eq!(changes.hidden, [[100.0, 200.0]]);
}

#[test]
fn joins_a_typed_word_to_the_one_kept_before_it() {
  let said = caption(&[&["a", "cat"]], true);
  assert_eq!(
    caption_changes(&said, "a big cat").corrections,
    [corrected(0.0, 100.0, "a big")]
  );
  // At the start it joins the word after.
  assert_eq!(
    caption_changes(&said, "oh a cat").corrections,
    [corrected(0.0, 100.0, "oh a")]
  );
}

#[test]
fn respells_several_words_as_fewer_in_one_correction() {
  let said = caption(&[&["any", "way", "then"]], true);
  let changes = caption_changes(&said, "anyway then");
  assert_eq!(changes.corrections, [corrected(0.0, 200.0, "anyway")]);
  assert!(changes.hidden.is_empty());
}

#[test]
fn compares_a_caption_without_spaces_a_line_at_a_time() {
  let said = caption(&[&["今日", "は"], &["晴れ"]], false);
  let changes = caption_changes(&said, "今日は\n雨");
  assert_eq!(changes.corrections, [corrected(200.0, 300.0, "雨")]);
}

/// Each typed word of `typed` in `said`, as its text and its stretch.
fn typed_words<'a>(said: &Caption, typed: &'a str) -> Vec<(&'a str, [f64; 2])> {
  typed_caption_words(said, typed)
    .into_iter()
    .map(|word| (&typed[word.bytes], word.span))
    .collect()
}

#[test]
fn reads_each_typed_word_as_the_words_it_stands_for() {
  let said = caption(&[&["any", "way", "a", "cat"], &["sat"]], true);
  // Respelled as fewer, the word stands for both; a word typed between
  // kept ones stands for the one it joins; one past the merged pair is
  // still its own.
  assert_eq!(
    typed_words(&said, "anyway a big cat\nsat"),
    [
      ("anyway", [0.0, 200.0]),
      ("a", [200.0, 300.0]),
      ("big", [200.0, 300.0]),
      ("cat", [300.0, 400.0]),
      ("sat", [400.0, 500.0]),
    ]
  );
  // A word taken out leaves the words after it standing for their own.
  assert_eq!(
    typed_words(&said, "any way cat sat"),
    [
      ("any", [0.0, 100.0]),
      ("way", [100.0, 200.0]),
      ("cat", [300.0, 400.0]),
      ("sat", [400.0, 500.0]),
    ]
  );
}

#[test]
fn sets_apart_the_typed_words_said_where_the_made_captions_set_one_apart() {
  let said = caption(&[&["we", "bilt", "it"]], true);
  let mut made = said.clone();
  made.lines[0][1].emphasis = true;
  assert_eq!(
    typed_emphasis(&said, "we built it", &[made]),
    [TextSpan { start: 3, end: 8 }]
  );
}
