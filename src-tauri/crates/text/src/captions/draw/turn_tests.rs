// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::captions::CaptionWord;
use crate::TextTemplateId;

const FRAME: CaptionFrame = CaptionFrame {
  width: 1_000,
  height: 1_000,
};

fn measure(text: &str, _face: TextFace, size: f64) -> f64 {
  text.chars().count() as f64 * size / 5.0
}

fn word(text: &str, start_ms: f64) -> CaptionWord {
  CaptionWord {
    text: text.to_owned(),
    start_ms,
    end_ms: start_ms + 300.0,
    source_start_ms: start_ms,
    source_end_ms: start_ms + 300.0,
    emphasis: false,
  }
}

/// "one two", said from 1 s, over Clean set to turn as `motion` says.
fn turned(motion: serde_json::Value, reveal_words: bool) -> Vec<CaptionPiece> {
  let captions = [Caption {
    id: 1_000,
    start_ms: 1_000.0,
    end_ms: 3_000.0,
    text: "one two".to_owned(),
    lines: vec![vec![word("one", 1_000.0), word("two", 1_000.0)]],
    spaced: true,
  }];
  let mut style = TextTemplateId::from("clean").shipped_style();
  style.reveal_words = reveal_words;
  style.motion = serde_json::from_value::<TextStyle>(serde_json::json!({ "motion": motion }))
    .unwrap()
    .motion;
  let recording = RecordingCaptions {
    enabled: true,
    style,
    ..Default::default()
  };
  caption_pieces(
    &captions,
    &recording,
    FRAME,
    CaptionMoment {
      at_ms: 1_050.0,
      exposure_ms: 25.0,
    },
    &mut measure,
  )
}

/// Turning from 90 degrees about `axis` to none over 100 ms.
fn flip(axis: &str) -> serde_json::Value {
  serde_json::json!([{ "property": axis, "keyframes": [
    { "atMs": 0, "value": 90 }, { "atMs": 100, "value": 0 }
  ] }])
}

#[test]
fn turns_each_word_about_its_own_middle_as_it_comes_on() {
  let pieces = turned(serde_json::json!({ "word": flip("rotateY") }), true);
  // Half-way round now, three quarters as the exposure began, each word
  // on its own, and the line not turning at all.
  for piece in &pieces {
    assert_eq!(piece.turn, [[0.0, 45.0, 0.0], [0.0, 67.5, 0.0]]);
    assert_eq!(piece.line_turn, None);
  }
}

#[test]
fn turns_a_whole_caption_as_one_about_the_middle_of_its_lines() {
  let pieces = turned(serde_json::json!({ "caption": flip("rotate") }), false);
  let [one, two] = [&pieces[0], &pieces[1]];
  assert_eq!(one.turn, [[0.0; 3]; 2]);
  let line = one.line_turn.as_ref().unwrap();
  assert_eq!(line.turn, [[0.0, 0.0, 45.0], [0.0, 0.0, 67.5]]);
  // The pivot is between the words, the same for both, and still.
  assert_eq!(two.line_turn.as_ref().unwrap().pivot, line.pivot);
  assert_eq!(line.pivot[0], line.pivot[1]);
  assert!(one.centre[0] < line.pivot[0][0] && line.pivot[0][0] < two.centre[0]);
  assert!(line.reach > 0.0);
}
