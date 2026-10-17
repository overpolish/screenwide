// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::captions::{CaptionWord, RecordingCaptions};
use crate::style::TextFace;

/// Every character half an em wide, in either face.
fn measure(text: &str, _: TextFace, size: f64) -> f64 {
  text.chars().count() as f64 * size * 0.5
}

const FRAME: CaptionFrame = CaptionFrame {
  width: 1920,
  height: 1080,
};

fn word(said: &str, at: f64, emphasis: bool) -> CaptionWord {
  CaptionWord {
    text: said.to_uppercase(),
    said: said.to_owned(),
    start_ms: at,
    end_ms: at + 300.0,
    source_start_ms: at,
    source_end_ms: at + 300.0,
    emphasis,
  }
}

fn track() -> CaptionTrack {
  CaptionTrack {
    captions: vec![Caption {
      id: 0,
      start_ms: 0.0,
      end_ms: 900.0,
      text: "WE BUILT\nIT".to_owned(),
      lines: vec![
        vec![word("we", 0.0, false), word("built", 300.0, true)],
        vec![word("it", 600.0, false)],
      ],
      spaced: true,
    }],
    recording: RecordingCaptions {
      enabled: true,
      style: TextStyle {
        accent: crate::style::TextAccent::Word,
        ..TextStyle::default()
      },
      ..Default::default()
    },
    editing: None,
  }
}

#[test]
fn opens_a_caption_as_the_transcript_spells_it_in_its_lines() {
  let editing = CaptionEditing::of(&track().captions[0]);
  assert_eq!(editing.text, "we built\nit");
  assert_eq!(editing.emphasis, [TextSpan { start: 3, end: 8 }]);
  assert_eq!(editing.shows_ms, [0.0, 900.0]);
}

#[test]
fn draws_the_words_typed_where_the_caption_stood() {
  let mut track = track();
  let still = CaptionMoment::still(500.0);
  let before = track.pieces_at(FRAME, still, &mut measure);
  let mut editing = CaptionEditing::of(&track.captions[0]);
  editing.text = "we built\nthat".to_owned();
  track.editing = Some(editing);
  let typed = track.pieces_at(FRAME, still, &mut measure);
  let words = |pieces: &[CaptionPiece]| -> Vec<String> {
    pieces
      .iter()
      .filter(|piece| piece.word.is_some())
      .map(|piece| piece.text.clone())
      .collect()
  };
  // The caption shows its words as its rules show them; typed into, it
  // shows them as the transcript spells them, as typed so far.
  assert_eq!(words(&before), ["WE", "BUILT", "IT"]);
  assert_eq!(words(&typed), ["we", "built", "that"]);
  // The first line stands where it stood.
  assert_eq!(before[0].centre, typed[0].centre);
  // The layout the caret goes by is the one drawn: the first word's middle
  // sits half its width into the block.
  let layout = track.editing_layout(FRAME, &mut measure).unwrap();
  let we = &typed[0];
  assert!(
    (layout.origin[0] + measure("we", TextFace::Main, we.size) / 2.0 - we.centre[0]).abs() < 1e-6
  );
  // Past its time it is the captions as they are again.
  assert!(track
    .pieces_at(FRAME, CaptionMoment::still(950.0), &mut measure)
    .is_empty());
}

#[test]
fn finds_the_word_under_a_point() {
  let track = track();
  let still = CaptionMoment::still(700.0);
  let pieces = track.pieces_at(FRAME, still, &mut measure);
  let it = pieces.iter().find(|piece| piece.word == Some(2)).unwrap();
  assert_eq!(
    track.word_at(FRAME, still, it.centre, &mut measure),
    Some(2)
  );
  assert_eq!(track.word_at(FRAME, still, [0.0, 0.0], &mut measure), None);
}
