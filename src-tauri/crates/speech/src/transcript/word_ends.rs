// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where each word stops. Its start is settled first (`word_starts`), but the
//! transcriber's idea of where a word ends often lands on the start, which
//! would leave words with no length and every pause between them unseen. A
//! word is taken to last until the next word starts or the voice stops,
//! whichever is first, as the voice activity listen heard it.

use super::RecordingTranscriptWord;

/// A word no longer than this, however long the voice runs on unheard.
const LONGEST_WORD_MS: u64 = 1_500;
/// A word the voice activity listen missed lasts about this long.
const UNHEARD_WORD_MS: u64 = 250;
/// No word is drawn shorter than this, so every word can be chosen and cut.
const SHORTEST_WORD_MS: u64 = 40;
/// A word starting just ahead of speech the listen heard belongs to it.
const LEAD_IN_MS: f64 = 150.0;

/// Settles every word's end from the words after it and `speech`, the
/// stretches the voice was heard in, in milliseconds, sorted and apart.
/// `words` are in the order said.
pub fn settle_word_ends(words: &mut [RecordingTranscriptWord], speech: &[(f64, f64)]) {
  for index in 0..words.len() {
    let start = words[index].start_ms;
    let next_start = words
      .get(index + 1)
      .map_or(u64::MAX, |next| next.start_ms.max(start));
    let voice_end = speech
      .iter()
      .find(|(from, to)| *to > start as f64 && *from <= start as f64 + LEAD_IN_MS)
      .map(|(_, to)| to.round() as u64);
    let end = voice_end
      .unwrap_or_else(|| words[index].end_ms.max(start + UNHEARD_WORD_MS))
      .min(next_start)
      .min(start + LONGEST_WORD_MS);
    words[index].end_ms = end.max(start + SHORTEST_WORD_MS);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn word(start_ms: u64, end_ms: u64) -> RecordingTranscriptWord {
    RecordingTranscriptWord {
      end_ms,
      probability: 1.0,
      start_ms,
      text: "word".to_owned(),
    }
  }

  fn ends(mut words: Vec<RecordingTranscriptWord>, speech: &[(f64, f64)]) -> Vec<u64> {
    settle_word_ends(&mut words, speech);
    words.iter().map(|word| word.end_ms).collect()
  }

  #[test]
  fn a_word_runs_to_the_next_one_inside_speech() {
    assert_eq!(
      ends(vec![word(100, 100), word(400, 400)], &[(80.0, 900.0)]),
      [400, 900]
    );
  }

  #[test]
  fn a_word_stops_where_the_voice_stops_so_the_pause_after_it_shows() {
    assert_eq!(
      ends(
        vec![word(100, 100), word(2_000, 2_000)],
        &[(80.0, 600.0), (1_950.0, 2_400.0)]
      ),
      [600, 2_400]
    );
  }

  #[test]
  fn a_word_the_listen_missed_keeps_a_short_length() {
    assert_eq!(ends(vec![word(5_000, 5_000)], &[(0.0, 1_000.0)]), [5_250]);
  }

  #[test]
  fn no_word_runs_on_without_end_or_shrinks_to_nothing() {
    assert_eq!(ends(vec![word(0, 0)], &[(0.0, 60_000.0)]), [1_500]);
    assert_eq!(
      ends(vec![word(500, 500), word(500, 500)], &[(0.0, 1_000.0)]),
      [540, 1_000]
    );
  }
}
