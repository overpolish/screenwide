// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use transcribe_cpp::{Segment, Transcript, Word};

use super::{dictated, shared_out, Said, Stretch, Written};

/// The span from `start` to `end` milliseconds into the dictation, in
/// samples at the protocol's rate.
fn span(start: usize, end: usize) -> Stretch {
  Stretch {
    start: start * 16,
    end: end * 16,
  }
}

/// The words of `transcript`, of audio from the start, as dictation answers
/// them, with their times in milliseconds.
fn answered(transcript: &Transcript, stretches: &[Said]) -> Vec<(String, u64, u64)> {
  let written = Written {
    open: shared_out(transcript, 0, stretches),
    ..Written::default()
  };
  dictated(&written, 0, true)
    .words
    .into_iter()
    .map(|word| (word.text, word.start_ms, word.end_ms))
    .collect()
}

#[test]
fn a_word_takes_the_time_of_the_run_of_speech_it_was_said_in() {
  // The model runs each word on into the silence after it.
  let word = |text: &str, t0_ms, t1_ms| Word {
    text: text.to_owned(),
    t0_ms,
    t1_ms,
    ..Word::default()
  };
  let transcript = Transcript {
    words: vec![
      word("Sam.", 800, 1_440),
      word("New", 1_440, 1_680),
      word("line.", 1_920, 2_560),
      word("Thanks", 2_560, 3_040),
    ],
    ..Transcript::default()
  };
  let said = Said {
    stretch: span(400, 3_200),
    runs: vec![span(400, 1_100), span(1_650, 2_200), span(2_800, 3_200)],
  };
  let times: Vec<(u64, u64)> = answered(&transcript, &[said])
    .iter()
    .map(|word| (word.1, word.2))
    .collect();
  assert_eq!(
    times,
    [(400, 1_100), (1_650, 2_200), (1_650, 2_200), (2_800, 3_200)]
  );
}

#[test]
fn a_segment_untimed_by_word_takes_its_stretch_time() {
  let segment = |text: &str, t0_ms, t1_ms| Segment {
    text: text.to_owned(),
    t0_ms,
    t1_ms,
    ..Segment::default()
  };
  let transcript = Transcript {
    segments: vec![
      segment(" Dear Sam.", 0, 2_000),
      segment(" New line.", 2_000, 4_000),
    ],
    ..Transcript::default()
  };
  let stretches = [
    Said {
      stretch: span(400, 1_000),
      runs: vec![span(400, 600), span(800, 1_000)],
    },
    Said {
      stretch: span(2_600, 3_200),
      runs: vec![span(2_600, 3_200)],
    },
  ];
  let words = answered(&transcript, &stretches);
  let said: Vec<(&str, u64, u64)> = words
    .iter()
    .map(|(text, start, end)| (text.as_str(), *start, *end))
    .collect();
  assert_eq!(
    said,
    [
      ("Dear", 400, 1_000),
      ("Sam.", 400, 1_000),
      ("New", 2_600, 3_200),
      ("line.", 2_600, 3_200),
    ]
  );
}
