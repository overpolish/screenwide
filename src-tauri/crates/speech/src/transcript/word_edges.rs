// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where two words meet inside speech. The alignment's boundary is only near
//! the gap between them, so it is moved to the quietest moment close by: a
//! cut made there falls between the sounds rather than through one, and the
//! highlight moves on as the next word is heard.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use screenwide_transcriber::SAMPLE_RATE;

use super::RecordingTranscriptWord;

/// How far either way of the alignment's boundary the quiet point is looked
/// for. The start of a word is already settled to within about this.
const REACH_MS: u64 = 50;
/// How long a stretch of the audio each candidate point is measured over.
const FRAME_MS: u64 = 10;
/// How far apart the candidate points are.
const STEP_MS: u64 = 5;
/// Each word keeps at least this much, as word ends allow for.
const SHORTEST_WORD_MS: u64 = 40;
/// A point counts as quieter only below this share of the loudness at the
/// alignment's boundary, so steady sound, where no point is truly quieter,
/// leaves the boundary alone.
const QUIETER_SHARE: f32 = 0.5;

/// [`settle_word_edges`] over the decoded audio at `audio`: 32-bit floats in
/// native byte order, mono at [`SAMPLE_RATE`], as the transcriber reads it.
/// Only the few milliseconds around each boundary are read, never the whole
/// track, which for an hour's recording would be a quarter of a gigabyte.
pub fn settle_word_edges_from(
  words: &mut [RecordingTranscriptWord],
  audio: &Path,
) -> Result<(), String> {
  let mut file = File::open(audio).map_err(|error| format!("Could not read the audio: {error}"))?;
  settle_word_edges(words, &mut |from, to| {
    let mut bytes = vec![0_u8; (to - from) * 4];
    let read = file
      .seek(SeekFrom::Start(from as u64 * 4))
      .and_then(|_| {
        let mut filled = 0;
        while filled < bytes.len() {
          match file.read(&mut bytes[filled..])? {
            0 => break,
            count => filled += count,
          }
        }
        Ok(filled)
      })
      .unwrap_or(0);
    bytes[..read]
      .as_chunks::<4>()
      .0
      .iter()
      .map(|chunk| f32::from_ne_bytes(*chunk))
      .collect()
  });
  Ok(())
}

/// Moves every boundary between two touching words to the quietest point of
/// the audio within reach of it, keeping each word at least the shortest
/// length. `read` gives the samples, mono at [`SAMPLE_RATE`], from one index
/// up to another, fewer where the audio ends. Words apart, with silence
/// between them, are left as they are: their edges are where the voice stops
/// and starts.
fn settle_word_edges(
  words: &mut [RecordingTranscriptWord],
  read: &mut dyn FnMut(usize, usize) -> Vec<f32>,
) {
  for index in 1..words.len() {
    let boundary = words[index].start_ms;
    if words[index - 1].end_ms != boundary {
      continue;
    }
    let earliest = boundary
      .saturating_sub(REACH_MS)
      .max(words[index - 1].start_ms + SHORTEST_WORD_MS);
    let latest = (boundary + REACH_MS).min(words[index].end_ms.saturating_sub(SHORTEST_WORD_MS));
    if earliest >= latest {
      continue;
    }
    let window_ms = earliest.saturating_sub(FRAME_MS / 2);
    let window = read(
      sample_at(window_ms),
      sample_at(latest.max(boundary) + FRAME_MS / 2),
    );
    let loudness = |at_ms: u64| loudness(&window, sample_at(at_ms) - sample_at(window_ms));
    let quietest = (earliest..=latest)
      .step_by(STEP_MS as usize)
      .map(|at| (loudness(at), at.abs_diff(boundary), at))
      .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let Some((level, _, at)) = quietest else {
      continue;
    };
    if level >= loudness(boundary) * QUIETER_SHARE {
      continue;
    }
    words[index - 1].end_ms = at;
    words[index].start_ms = at;
  }
}

fn sample_at(ms: u64) -> usize {
  (ms * u64::from(SAMPLE_RATE) / 1_000) as usize
}

/// The mean square of `window` over the frame centred `centre` samples in.
fn loudness(window: &[f32], centre: usize) -> f32 {
  let half = sample_at(FRAME_MS / 2);
  let from = centre.saturating_sub(half).min(window.len());
  let frame = &window[from..(centre + half).min(window.len())];
  if frame.is_empty() {
    return f32::INFINITY;
  }
  frame.iter().map(|sample| sample * sample).sum::<f32>() / frame.len() as f32
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

  /// One second of a steady tone, silent between `quiet` milliseconds.
  fn tone_with_gap(quiet: (u64, u64)) -> Vec<f32> {
    (0..SAMPLE_RATE)
      .map(|index| {
        let ms = u64::from(index) * 1_000 / u64::from(SAMPLE_RATE);
        if (quiet.0..quiet.1).contains(&ms) {
          0.0
        } else {
          (index as f32 * 0.1).sin() * 0.5
        }
      })
      .collect()
  }

  /// Settles `words` over `samples`, written out as the transcriber's audio
  /// file is, so the reading is tested too.
  fn settle(words: &mut [RecordingTranscriptWord], samples: &[f32]) {
    let path = std::env::temp_dir().join(format!(
      "screenwide-word-edges-{}-{:?}.f32",
      std::process::id(),
      std::thread::current().id()
    ));
    let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_ne_bytes()).collect();
    std::fs::write(&path, bytes).unwrap();
    settle_word_edges_from(words, &path).unwrap();
    std::fs::remove_file(path).unwrap();
  }

  #[test]
  fn a_boundary_moves_into_the_quiet_between_two_words() {
    let samples = tone_with_gap((520, 545));
    let mut words = vec![word(200, 500), word(500, 900)];
    settle(&mut words, &samples);
    assert!((520..545).contains(&words[0].end_ms), "{}", words[0].end_ms);
    assert_eq!(words[0].end_ms, words[1].start_ms);
  }

  #[test]
  fn a_boundary_with_no_quiet_near_it_stays_where_it_was() {
    let samples = tone_with_gap((800, 820));
    let mut words = vec![word(200, 500), word(500, 900)];
    settle(&mut words, &samples);
    assert_eq!((words[0].end_ms, words[1].start_ms), (500, 500));
  }

  #[test]
  fn words_apart_and_short_words_keep_their_edges() {
    let samples = tone_with_gap((395, 425));
    let mut apart = vec![word(200, 380), word(470, 900)];
    settle(&mut apart, &samples);
    assert_eq!((apart[0].end_ms, apart[1].start_ms), (380, 470));
    // The quiet lies inside the first word's shortest length.
    let mut short = vec![word(400, 440), word(440, 900)];
    settle(&mut short, &samples);
    assert_eq!((short[0].end_ms, short[1].start_ms), (440, 440));
  }
}
