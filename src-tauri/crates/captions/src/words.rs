// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The words a caption can show: the transcript as it reads after its
//! corrections, each word placed where the edited video plays it, or left out
//! where the edit cuts it.

use screenwide_timeline::TimelineRange;

/// A word as the transcriber heard it, timed in source milliseconds.
#[derive(Clone, Debug, PartialEq)]
pub struct HeardWord {
  pub start_ms: f64,
  pub end_ms: f64,
  pub text: String,
}

/// Text written over the words heard across a stretch of the source, in
/// milliseconds: it replaces every word whose middle falls inside it.
#[derive(Clone, Debug, PartialEq)]
pub struct Correction {
  pub start_ms: f64,
  pub end_ms: f64,
  pub text: String,
}

/// One word as the transcript reads it, where the source has it and, while
/// the edit keeps it, where the edited video plays it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Token {
  pub text: String,
  pub source_start_ms: f64,
  pub source_end_ms: f64,
  /// The kept stretch that plays it, by its place in the play order, and its
  /// output start and end in milliseconds; `None` where the edit cuts it.
  pub played: Option<(usize, f64, f64)>,
}

fn middle(start: f64, end: f64) -> f64 {
  (start + end) / 2.0
}

/// The words a correction covers under its text, or the word on its own, in
/// source order, each with the stretch it covers.
fn corrected_words(words: &[HeardWord], corrections: &[Correction]) -> Vec<(String, f64, f64)> {
  let mut read = Vec::with_capacity(words.len());
  let mut next = 0;
  let mut first = 0;
  while first < words.len() {
    let heard = &words[first];
    let heard_middle = middle(heard.start_ms, heard.end_ms);
    while corrections
      .get(next)
      .is_some_and(|correction| correction.end_ms <= heard_middle)
    {
      next += 1;
    }
    let covers = |correction: &Correction, word: &HeardWord| {
      let at = middle(word.start_ms, word.end_ms);
      correction.start_ms <= at && at < correction.end_ms
    };
    let Some(correction) = corrections
      .get(next)
      .filter(|correction| covers(correction, heard))
    else {
      read.push((heard.text.clone(), heard.start_ms, heard.end_ms));
      first += 1;
      continue;
    };
    let mut last = first;
    while words
      .get(last + 1)
      .is_some_and(|word| covers(correction, word))
    {
      last += 1;
    }
    read.push((
      correction.text.clone(),
      words[first].start_ms,
      words[last].end_ms,
    ));
    first = last + 1;
  }
  read
}

/// The output millisecond the source moment `source_ms` plays at, in the
/// stretch `index` of `ranges`, held to that stretch.
fn output_of(ranges: &[TimelineRange], index: usize, source_ms: f64) -> f64 {
  let range = &ranges[index];
  let start = range.source_start_us as f64 / 1_000.0;
  let end = range.source_end_us as f64 / 1_000.0;
  range.output_start_us as f64 / 1_000.0
    + (source_ms.clamp(start, end) - start) / range.playback_rate
}

/// The transcript's words as they read, split on spaces with a run's stretch
/// shared out over its words by their length, and placed on the edited
/// video: a word plays where its middle does, from the later of its start
/// and its stretch's, to the earlier of its end and its stretch's.
pub(crate) fn tokens(
  words: &[HeardWord],
  corrections: &[Correction],
  ranges: &[TimelineRange],
) -> Vec<Token> {
  corrected_words(words, corrections)
    .into_iter()
    .flat_map(|(text, start, end)| {
      let parts: Vec<&str> = text.split_whitespace().collect();
      let total: usize = parts.iter().map(|part| part.chars().count()).sum();
      let mut at = start;
      parts
        .into_iter()
        .map(|part| {
          let until = at + (end - start) * part.chars().count() as f64 / total.max(1) as f64;
          let token = (part.to_owned(), at, until);
          at = until;
          token
        })
        .collect::<Vec<_>>()
    })
    .map(|(text, source_start_ms, source_end_ms)| {
      let at_us = middle(source_start_ms, source_end_ms) * 1_000.0;
      let played = ranges
        .iter()
        .position(|range| {
          range.source_start_us as f64 <= at_us && at_us < range.source_end_us as f64
        })
        .map(|index| {
          (
            index,
            output_of(ranges, index, source_start_ms),
            output_of(ranges, index, source_end_ms),
          )
        });
      Token {
        text,
        source_start_ms,
        source_end_ms,
        played,
      }
    })
    .collect()
}
