// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the recording is transcribed: the stretches of speech the engine
//! hears, each on its own, in samples for [`Job::Transcribe`]. Chunks are
//! planned from the listen Remove silences already made, so transcription
//! hears the same speech it cuts around.
//!
//! [`Job::Transcribe`]: screenwide_transcriber::Job::Transcribe

use screenwide_transcriber::SAMPLE_RATE;

use super::super::analysis::SpeechMap;
use super::super::silences;

/// Added to each end of a stretch, so a word's first and last sound are not
/// clipped.
const PAD_MS: f64 = 200.0;
/// A pause shorter than this stays inside one stretch, so a phrase split by a
/// breath is heard whole.
const MERGE_GAP_MS: f64 = 1_000.0;
/// A stretch longer than this is split, so the engine never holds a long
/// recording's speech all at once.
const MAX_STRETCH_MS: f64 = 30_000.0;

/// The stretches of the recording to transcribe, in samples at
/// [`SAMPLE_RATE`], sorted and apart. Nothing where nothing is said.
pub(super) fn plan(speech: &SpeechMap, duration_ms: f64) -> Vec<[u64; 2]> {
  let spoken = silences::speech_ranges(speech, duration_ms);
  if spoken.is_empty() {
    return Vec::new();
  }
  let padded = spoken
    .iter()
    .map(|&(start, end)| ((start - PAD_MS).max(0.0), (end + PAD_MS).min(duration_ms)));
  let mut merged: Vec<(f64, f64)> = Vec::new();
  for (start, end) in padded {
    if let Some(last) = merged.last_mut() {
      if start - last.1 <= MERGE_GAP_MS {
        last.1 = last.1.max(end);
        continue;
      }
    }
    merged.push((start, end));
  }
  let mut out = Vec::new();
  for (start, end) in merged {
    split(start, end, &spoken, &mut out);
  }
  out
    .into_iter()
    .map(|(start, end)| [ms_to_samples(start), ms_to_samples(end)])
    .collect()
}

/// The stretch from `start` to `end`, pushed whole when it is short enough,
/// and otherwise cut at its longest pause, so a cut never lands through a
/// word while a pause will do. Only when a stretch has no pause at all is it
/// cut through whatever is said.
fn split(start: f64, end: f64, spoken: &[(f64, f64)], out: &mut Vec<(f64, f64)>) {
  if end - start <= MAX_STRETCH_MS {
    out.push((start, end));
    return;
  }
  let longest = spoken
    .iter()
    .copied()
    .filter(|&(from, to)| to > start && from < end)
    .collect::<Vec<_>>()
    .windows(2)
    .map(|pair| (pair[1].0 - pair[0].1, pair[0].1, pair[1].0))
    .max_by(|a, b| a.0.total_cmp(&b.0));
  match longest {
    Some((_, before, after)) => {
      let mid = (before + after) / 2.0;
      split(start, mid, spoken, out);
      split(mid, end, spoken, out);
    }
    None => {
      split(start, start + MAX_STRETCH_MS, spoken, out);
      split(start + MAX_STRETCH_MS, end, spoken, out);
    }
  }
}

fn ms_to_samples(ms: f64) -> u64 {
  (ms * f64::from(SAMPLE_RATE) / 1_000.0).round() as u64
}

#[cfg(test)]
mod tests {
  use super::*;

  const WINDOW_MS: f64 = 32.0;

  /// A listen that hears speech over each of `spoken`, in milliseconds, and
  /// nothing else, over `duration_ms`.
  fn speech(spoken: &[(f64, f64)], duration_ms: u64) -> SpeechMap {
    let windows = (duration_ms as f64 / WINDOW_MS).ceil() as usize;
    SpeechMap {
      probabilities: (0..windows)
        .map(|index| {
          let ms = index as f64 * WINDOW_MS;
          if spoken.iter().any(|&(start, end)| ms >= start && ms < end) {
            0.9
          } else {
            0.05
          }
        })
        .collect(),
      window_ms: WINDOW_MS,
    }
  }

  /// Speech starting and ending on window edges, so the ranges land exactly.
  fn at(window: u64) -> f64 {
    window as f64 * WINDOW_MS
  }

  fn samples(ms: f64) -> u64 {
    (ms * f64::from(SAMPLE_RATE) / 1_000.0).round() as u64
  }

  #[test]
  fn nothing_said_leaves_nothing_to_transcribe() {
    assert_eq!(plan(&speech(&[], 60_000), 60_000.0), Vec::<[u64; 2]>::new());
  }

  #[test]
  fn a_short_phrase_is_padded_by_a_beat() {
    let planned = plan(&speech(&[(at(32), at(64))], at(100) as u64), at(100));
    assert_eq!(
      planned,
      vec![[samples(at(32) - 200.0), samples(at(64) + 200.0)]]
    );
  }

  #[test]
  fn a_short_pause_keeps_the_phrase_whole() {
    let planned = plan(
      &speech(&[(at(0), at(30)), (at(45), at(80))], at(100) as u64),
      at(100),
    );
    assert_eq!(planned, vec![[0, samples(at(80) + 200.0)]]);
  }

  #[test]
  fn a_long_pause_makes_two_stretches() {
    let planned = plan(
      &speech(&[(at(0), at(30)), (at(90), at(120))], at(130) as u64),
      at(130),
    );
    assert_eq!(
      planned,
      vec![
        [0, samples(at(30) + 200.0)],
        [samples(at(90) - 200.0), samples(at(120) + 200.0)],
      ]
    );
  }

  #[test]
  fn a_long_stretch_is_split_at_its_longest_pause() {
    // Forty seconds of speech with one 320 ms pause: cut in that pause,
    // never through the words.
    let planned = plan(
      &speech(&[(at(0), at(600)), (at(610), at(1200))], at(1250) as u64),
      at(1250),
    );
    let mid = samples((at(600) + at(610)) / 2.0);
    assert_eq!(planned, vec![[0, mid], [mid, samples(at(1200) + 200.0)]]);
  }
}
