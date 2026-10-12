// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where each word starts. The transcriber's alignment puts a word's start
//! partway into it, about a fifth of a second after the sound begins, so a
//! highlight follows the voice late and a cut leaves the word's first sound
//! behind. A word that opens speech after a silence starts where the voice
//! activity listen heard the voice begin; any other word is moved back by the
//! usual lag, never past the word before it.

use super::RecordingTranscriptWord;

/// How far behind the sound the alignment usually puts a word's start. Words
/// spoken with known starts came out 108 to 432 ms late, half of them more
/// than 190 ms; this stays under the smallest of those, so a word may still
/// start a little late but seldom early, where a cut would clip the word
/// before it.
const ALIGNMENT_LAG_MS: u64 = 150;
/// A word starting just ahead of speech the listen heard belongs to it.
const LEAD_IN_MS: f64 = 150.0;
/// The first word of a stretch of speech starting further in than this is
/// not taken to open it: the listen heard something the transcriber did not
/// write, and the voice's start is not this word's.
const OPENING_REACH_MS: f64 = 500.0;
/// Each word keeps at least this much after the start of the one before it,
/// as word ends allow for.
const SHORTEST_WORD_MS: u64 = 40;

/// Settles every word's start from `speech`, the stretches the voice was
/// heard in, in milliseconds, sorted and apart. `words` are in the order said;
/// their starts are as the transcriber aligned them.
pub fn settle_word_starts(words: &mut [RecordingTranscriptWord], speech: &[(f64, f64)]) {
  let mut previous: Option<(u64, Option<usize>)> = None;
  for word in words.iter_mut() {
    let aligned = word.start_ms;
    let stretch = speech
      .iter()
      .position(|&(from, to)| aligned as f64 + LEAD_IN_MS > from && (aligned as f64) < to);
    let opens = stretch.is_some_and(|index| {
      previous.is_none_or(|(_, before)| before != Some(index))
        && aligned as f64 <= speech[index].0 + OPENING_REACH_MS
    });
    let mut start = match stretch {
      Some(index) if opens => speech[index].0.round() as u64,
      Some(index) => aligned
        .saturating_sub(ALIGNMENT_LAG_MS)
        .max(speech[index].0.round() as u64),
      None => aligned.saturating_sub(ALIGNMENT_LAG_MS),
    };
    if let Some((before, _)) = previous {
      start = start.max(before + SHORTEST_WORD_MS);
    }
    word.start_ms = start;
    previous = Some((start, stretch));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn starts(aligned: &[u64], speech: &[(f64, f64)]) -> Vec<u64> {
    let mut words = aligned
      .iter()
      .map(|&start_ms| RecordingTranscriptWord {
        end_ms: start_ms,
        probability: 1.0,
        start_ms,
        text: "word".to_owned(),
      })
      .collect::<Vec<_>>();
    settle_word_starts(&mut words, speech);
    words.iter().map(|word| word.start_ms).collect()
  }

  #[test]
  fn a_word_opening_speech_starts_with_the_voice() {
    assert_eq!(
      starts(
        &[1_900, 2_400, 6_300],
        &[(1_650.0, 3_000.0), (6_050.0, 7_000.0)]
      ),
      [1_650, 2_250, 6_050]
    );
  }

  #[test]
  fn a_word_inside_speech_moves_back_by_the_lag_but_not_past_the_one_before() {
    assert_eq!(
      starts(&[1_000, 1_300, 1_320], &[(900.0, 2_000.0)]),
      [900, 1_150, 1_190]
    );
  }

  #[test]
  fn a_word_far_into_speech_heard_before_it_is_not_taken_to_open_it() {
    assert_eq!(starts(&[2_000], &[(1_000.0, 3_000.0)]), [1_850]);
  }

  #[test]
  fn a_word_the_listen_missed_still_moves_back_by_the_lag() {
    assert_eq!(starts(&[5_000, 100], &[(0.0, 1_000.0)]), [4_850, 4_890]);
  }
}
