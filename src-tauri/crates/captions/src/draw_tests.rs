// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const FRAME: CaptionFrame = CaptionFrame {
  width: 1_000,
  height: 1_000,
};

fn measure(text: &str, _size: f64) -> f64 {
  text.chars().count() as f64 * 10.0
}

fn word(text: &str, start_ms: f64) -> CaptionWord {
  CaptionWord {
    text: text.to_owned(),
    start_ms,
    end_ms: start_ms + 300.0,
    source_start_ms: start_ms,
    source_end_ms: start_ms + 300.0,
  }
}

/// "one two" over "three", from 1 s to 3 s.
fn captions() -> Vec<Caption> {
  vec![Caption {
    id: 1_000,
    start_ms: 1_000.0,
    end_ms: 3_000.0,
    text: "one two\nthree".to_owned(),
    lines: vec![
      vec![word("one", 1_000.0), word("two", 1_400.0)],
      vec![word("three", 1_800.0)],
    ],
    spaced: true,
  }]
}

fn styled(template: CaptionTemplate, highlight_words: bool) -> RecordingCaptions {
  RecordingCaptions {
    enabled: true,
    style: CaptionStyle {
      highlight_words,
      ..template.style()
    },
    ..Default::default()
  }
}

fn pieces(recording: &RecordingCaptions, at_ms: f64) -> Vec<CaptionPiece> {
  caption_pieces(&captions(), recording, FRAME, at_ms, &mut measure)
}

fn kinds(pieces: &[CaptionPiece]) -> Vec<(&str, CaptionPieceKind)> {
  pieces
    .iter()
    .map(|piece| (piece.text.as_str(), piece.kind.clone()))
    .collect()
}

fn word_in(ink: &str, outlined: bool) -> CaptionPieceKind {
  CaptionPieceKind::Word {
    ink: ink.to_owned(),
    outlined,
  }
}

fn white_word() -> CaptionPieceKind {
  word_in("#ffffffff", true)
}

#[test]
fn draws_nothing_off_or_between_captions() {
  assert!(pieces(&styled(CaptionTemplate::Clean, false), 500.0).is_empty());
  assert!(pieces(&styled(CaptionTemplate::Clean, false), 3_000.0).is_empty());
  let mut off = styled(CaptionTemplate::Clean, false);
  off.enabled = false;
  assert!(pieces(&off, 1_500.0).is_empty());
}

#[test]
fn sets_each_line_centred_with_the_last_ending_low_in_the_picture() {
  let drawn = pieces(&styled(CaptionTemplate::Clean, false), 1_500.0);
  let centres: Vec<[f64; 2]> = drawn.iter().map(|piece| piece.centre).collect();
  // 50 px type on 62.5 px lines, the last ending at 900 px; "one two" is
  // 70 px wide with its 10 px space, centred on 500 px.
  assert_eq!(centres, [[480.0, 806.25], [520.0, 806.25], [500.0, 868.75]]);
}

#[test]
fn holds_a_caption_in_its_corner_with_its_lines_lined_up_to_that_side() {
  let placed = |position| {
    let mut recording = styled(CaptionTemplate::Clean, false);
    recording.style.position = position;
    let drawn = pieces(&recording, 1_500.0);
    drawn.iter().map(|piece| piece.centre).collect::<Vec<_>>()
  };
  // 100 px in from every edge; the caption is 70 px wide and 125 px tall,
  // and "three" lines up with the side the caption keeps to.
  assert_eq!(
    placed(CaptionPosition::TopLeft),
    [[115.0, 131.25], [155.0, 131.25], [125.0, 193.75]]
  );
  assert_eq!(
    placed(CaptionPosition::BottomRight),
    [[845.0, 806.25], [885.0, 806.25], [875.0, 868.75]]
  );
}

#[test]
fn colours_the_word_being_said_where_asked() {
  let drawn = pieces(&styled(CaptionTemplate::Clean, true), 1_500.0);
  assert_eq!(
    kinds(&drawn),
    [
      ("one", white_word()),
      ("two", word_in("#ffd60aff", true)),
      ("three", white_word())
    ]
  );
}

#[test]
fn draws_in_the_colours_the_style_picks() {
  let mut recording = styled(CaptionTemplate::Highlight, false);
  recording.style.text_color = "#ff0000".to_owned();
  // A dark pill takes light type on it.
  recording.style.highlight_color = "#202080".to_owned();
  let drawn = pieces(&recording, 1_900.0);
  assert_eq!(
    kinds(&drawn),
    [
      (
        "three",
        CaptionPieceKind::Pill {
          color: "#202080ff".to_owned()
        }
      ),
      ("one", word_in("#ff0000ff", true)),
      ("two", word_in("#ff0000ff", true)),
      ("three", word_in("#ffffffff", false)),
    ]
  );
}

#[test]
fn puts_the_box_behind_the_lines_and_the_pill_under_the_word_said() {
  let boxed = pieces(&styled(CaptionTemplate::Box, false), 1_500.0);
  assert_eq!(
    boxed[0].kind,
    CaptionPieceKind::Backdrop {
      color: "#0000009e".to_owned()
    }
  );
  assert_eq!(boxed[0].text, "one two\nthree");
  let pill = pieces(&styled(CaptionTemplate::Highlight, false), 1_900.0);
  assert_eq!(
    kinds(&pill),
    [
      (
        "three",
        CaptionPieceKind::Pill {
          color: "#ffd60aff".to_owned()
        }
      ),
      ("one", white_word()),
      ("two", white_word()),
      ("three", word_in("#000000ff", false)),
    ]
  );
}

#[test]
fn reveals_each_word_as_it_is_said() {
  let drawn = pieces(&styled(CaptionTemplate::Reveal, false), 1_445.0);
  assert_eq!(
    kinds(&drawn),
    [("one", white_word()), ("two", white_word())]
  );
  assert_eq!(drawn[1].opacity[0], 0.5);
}

#[test]
fn pops_in_past_its_size_and_settles() {
  let at = |ms: f64| pieces(&styled(CaptionTemplate::Pop, false), 1_000.0 + ms)[0].clone();
  assert_eq!((at(0.0).scale[0], at(0.0).opacity[0]), (0.75, 0.0));
  let peak = (0..22)
    .map(|step| at(f64::from(step) * 10.0).scale[0])
    .fold(0.0, f64::max);
  assert!((1.03..1.06).contains(&peak), "{peak}");
  assert_eq!((at(400.0).scale[0], at(400.0).opacity[0]), (1.0, 1.0));
  // It grows as one block: a word left of the middle moves further left.
  assert!(at(0.0).centre[0] > at(400.0).centre[0]);
}
