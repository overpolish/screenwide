// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const FRAME: CaptionFrame = CaptionFrame {
  width: 1_000,
  height: 1_000,
};

fn measure(text: &str, _face: CaptionFace, size: f64) -> f64 {
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

fn styled(template: &str, highlight_words: bool) -> RecordingCaptions {
  RecordingCaptions {
    enabled: true,
    style: CaptionStyle {
      highlight_words,
      ..CaptionTemplateId::from(template).shipped_style()
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

/// A word in `ink`, outlined in the dark the templates ship with where
/// `outlined`.
fn word_in(ink: &str, outlined: bool) -> CaptionPieceKind {
  CaptionPieceKind::Word {
    ink: ink.to_owned(),
    outline: outlined.then(|| "#000000ff".to_owned()),
  }
}

fn white_word() -> CaptionPieceKind {
  word_in("#ffffffff", true)
}

#[test]
fn draws_nothing_off_or_between_captions() {
  assert!(pieces(&styled("clean", false), 500.0).is_empty());
  assert!(pieces(&styled("clean", false), 3_000.0).is_empty());
  let mut off = styled("clean", false);
  off.enabled = false;
  assert!(pieces(&off, 1_500.0).is_empty());
}

#[test]
fn sets_each_line_centred_with_the_last_ending_low_in_the_picture() {
  let drawn = pieces(&styled("clean", false), 1_500.0);
  let centres: Vec<[f64; 2]> = drawn.iter().map(|piece| piece.centre).collect();
  // 50 px type on 62.5 px lines, the last ending at 900 px; "one two" is
  // 70 px wide with its 10 px space, centred on 500 px.
  assert_eq!(centres, [[480.0, 806.25], [520.0, 806.25], [500.0, 868.75]]);
}

#[test]
fn holds_a_caption_in_its_corner_with_its_lines_lined_up_to_that_side() {
  let placed = |position| {
    let mut recording = styled("clean", false);
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

/// Whether `a` and `b` are the same point, give or take rounding.
fn near(a: [f64; 2], b: [f64; 2]) -> bool {
  (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

#[test]
fn sets_a_word_apart_in_the_accent_standing_on_its_lines_baseline() {
  let mut recording = styled("clean", false);
  recording.style.accent = CaptionAccent::Word;
  recording.style.accent_size_percent = 200.0;
  recording.style.accent_color = "#ff0000".to_owned();
  let mut said = captions();
  said[0].lines[0][1].emphasis = true;
  let drawn = caption_pieces(&said, &recording, FRAME, 1_500.0, &mut measure);
  assert_eq!((drawn[1].face, drawn[1].size), (CaptionFace::Accent, 100.0));
  assert_eq!(drawn[1].kind, word_in("#ff0000ff", true));
  assert_eq!((drawn[0].face, drawn[0].size), (CaptionFace::Main, 50.0));
  // "two" at twice the size makes its line 125 px tall and 110 px wide,
  // with the accent's wider space; "one" sits lower, on the same baseline.
  let expected = [[460.0, 793.0], [525.0, 775.0], [500.0, 868.75]];
  for (piece, centre) in drawn.iter().zip(expected) {
    assert!(
      near(piece.centre, centre),
      "{:?} != {centre:?}",
      piece.centre
    );
  }
}

#[test]
fn steps_each_line_of_a_staircase_further_along() {
  let mut recording = styled("clean", false);
  recording.style.arrangement = CaptionArrangement::Staircase;
  let lefts: Vec<f64> = pieces(&recording, 1_500.0)
    .iter()
    .map(|piece| piece.centre[0] - measure(&piece.text, piece.face, piece.size) / 2.0)
    .collect();
  // "three" starts 1.5 ems, 75 px, further along than "one two", and the
  // block, 125 px wide, is centred.
  assert_eq!(lefts, [437.5, 477.5, 512.5]);
}

#[test]
fn colours_the_word_being_said_where_asked() {
  let drawn = pieces(&styled("clean", true), 1_500.0);
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
  let mut recording = styled("highlight", true);
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
fn outlines_words_in_the_colour_picked_or_not_at_all() {
  let mut recording = styled("clean", false);
  recording.style.outline_color = "#3050ff".to_owned();
  let outlined = |recording: &RecordingCaptions| match &pieces(recording, 1_500.0)[0].kind {
    CaptionPieceKind::Word { outline, .. } => outline.clone(),
    other => panic!("{other:?}"),
  };
  assert_eq!(outlined(&recording).as_deref(), Some("#3050ffff"));
  recording.style.outline = false;
  assert_eq!(outlined(&recording), None);
}

#[test]
fn puts_the_box_behind_the_lines_and_the_pill_under_the_word_said() {
  let boxed = pieces(&styled("box", false), 1_500.0);
  assert_eq!(
    boxed[0].kind,
    CaptionPieceKind::Backdrop {
      color: "#0000009e".to_owned(),
      // "one two" over "three", in 50 px type on 62.5 px lines.
      block: [70.0, 125.0],
    }
  );
  assert_eq!(boxed[0].text, "one two\nthree");
  let pill = pieces(&styled("highlight", true), 1_900.0);
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
  // Not highlighting the word being said takes its pill away too.
  let unmarked = pieces(&styled("highlight", false), 1_900.0);
  assert_eq!(
    kinds(&unmarked),
    [
      ("one", white_word()),
      ("two", white_word()),
      ("three", white_word())
    ]
  );
}

/// Clean, with its words revealed as they are said, popping where asked.
fn revealed(popping: bool) -> RecordingCaptions {
  let mut recording = styled("clean", false);
  recording.style.reveal_words = true;
  if popping {
    recording.style.motion = CaptionTemplateId::from("pop").shipped_style().motion;
  }
  recording
}

#[test]
fn reveals_each_word_as_it_is_said() {
  let drawn = pieces(&revealed(false), 1_445.0);
  assert_eq!(
    kinds(&drawn),
    [("one", white_word()), ("two", white_word())]
  );
  assert_eq!(drawn[1].opacity[0], 0.5);
  // Popping as well, each word pops as it is said rather than with the
  // caption: "two", said 400 ms in, starts small while "one" has settled.
  let popped = pieces(&revealed(true), 1_400.0);
  assert_eq!(popped[0].scale[0], 1.0);
  assert_eq!(popped[1].scale[0], 0.75);
}

#[test]
fn grows_the_box_over_the_words_revealed_so_far() {
  let mut recording = styled("box", false);
  recording.style.reveal_words = true;
  let block = |at_ms| match pieces(&recording, at_ms)[0].kind.clone() {
    CaptionPieceKind::Backdrop { block, .. } => block,
    kind => panic!("{kind:?} is no box"),
  };
  // "one", then "one two", then both lines, in 50 px type on 62.5 px
  // lines.
  assert_eq!(block(1_100.0), [30.0, 62.5]);
  assert_eq!(block(1_500.0), [70.0, 62.5]);
  assert_eq!(block(1_900.0), [70.0, 125.0]);
}

#[test]
fn pops_in_past_its_size_and_settles() {
  let at = |ms: f64| pieces(&styled("pop", false), 1_000.0 + ms)[0].clone();
  assert_eq!((at(0.0).scale[0], at(0.0).opacity[0]), (0.75, 0.0));
  let peak = (0..22)
    .map(|step| at(f64::from(step) * 10.0).scale[0])
    .fold(0.0, f64::max);
  assert!((1.03..1.06).contains(&peak), "{peak}");
  assert_eq!((at(400.0).scale[0], at(400.0).opacity[0]), (1.0, 1.0));
  // Each word grows where it sits, sharp: it never moves, and its size a
  // frame ago is its size now.
  assert_eq!(at(0.0).centre, at(400.0).centre);
  assert_eq!(at(100.0).scale[0], at(100.0).scale[1]);
}

#[test]
fn moves_each_piece_by_its_motions_offset_in_ems_of_its_type() {
  let mut recording = styled("clean", false);
  recording.style.motion = serde_json::from_value::<CaptionStyle>(serde_json::json!({
    "motion": { "caption": [{ "property": "offsetY", "keyframes": [
      { "atMs": 0, "value": 0.5 },
      { "atMs": 200, "value": 0 }
    ] }] }
  }))
  .unwrap()
  .motion;
  let at = |ms: f64| pieces(&recording, 1_000.0 + ms)[0].clone();
  let settled = at(400.0);
  let moving = at(0.0);
  assert_eq!(moving.centre[0], settled.centre[0]);
  assert_eq!(moving.centre[1] - settled.centre[1], 0.5 * settled.size);
}

#[test]
fn readies_the_next_captions_words_unseen_and_set_as_they_will_be() {
  let track = CaptionTrack {
    captions: captions(),
    recording: styled("clean", false),
  };
  let upcoming = track.upcoming_pieces(FRAME, 500.0);
  let shown = pieces(&track.recording, 2_000.0);
  let texts: Vec<&str> = upcoming.iter().map(|piece| piece.text.as_str()).collect();
  assert_eq!(texts, ["one", "two", "three"]);
  for (ready, drawn) in upcoming.iter().zip(&shown) {
    assert_eq!(ready.opacity, [0.0; 2]);
    assert_eq!(ready.size, drawn.size);
    let outline = |kind: &CaptionPieceKind| match kind {
      CaptionPieceKind::Word { outline, .. } => outline.clone(),
      other => panic!("{other:?}"),
    };
    assert_eq!(outline(&ready.kind), outline(&drawn.kind));
  }
  // Nothing comes after the last caption.
  assert!(track.upcoming_pieces(FRAME, 1_500.0).is_empty());
}

#[test]
fn readies_the_next_caption_only_once_the_one_showing_has_come_on() {
  let mut following = captions();
  following.push(Caption {
    id: 3_000,
    start_ms: 3_000.0,
    end_ms: 4_000.0,
    text: "four".to_owned(),
    lines: vec![vec![word("four", 3_000.0)]],
    spaced: true,
  });
  let track = CaptionTrack {
    captions: following,
    recording: styled("pop", false),
  };
  let texts = |at_ms| {
    track
      .upcoming_pieces(FRAME, at_ms)
      .into_iter()
      .map(|piece| piece.text)
      .collect::<Vec<_>>()
  };
  // While "one two three" pops in, readying "four" would take frames from
  // it; once it has settled, "four" is readied.
  assert!(texts(1_100.0).is_empty());
  assert_eq!(texts(1_500.0), ["four"]);
}
