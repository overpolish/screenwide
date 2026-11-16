// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! More voices than the speaker model has room for. Nemotron 3 Diarization
//! keeps eight voices apart over a call; a call with more fills every slot
//! and from then on merges voices or hears no one. Such a call is told apart
//! again in overlapping windows, each with eight fresh slots, a window that
//! still fills them in halves, and the windows' voices are joined into the
//! call's speakers by where the windows agree and by each voice's
//! fingerprint (`link`, `fingerprint`).

mod fbank;
mod fingerprint;
mod link;

pub use fingerprint::Fingerprints;
use link::{Local, Tie};
use screenwide_transcriber::SpeakerTurn;

use super::alone::{self, RATE};

/// A window of the call, in milliseconds: long enough that each holds fewer
/// voices than the model's slots in most calls.
pub const WINDOW_MS: u64 = 300_000;
/// How much consecutive windows share, in milliseconds: where they agree on
/// who spoke ties their voices together.
const OVERLAP_MS: u64 = 60_000;
/// The most of a voice's speech its fingerprint is taken from, in
/// milliseconds, longest stretches first.
const SPEECH_MAX_MS: u64 = 30_000;
/// How alike two voices' fingerprints must be to be one speaker (cosine),
/// tuned on VoxConverse calls of 7 to 17 voices.
const ALIKE: f32 = 0.6;

/// Where each window starts and ends in the call, in milliseconds.
pub fn windows(call_ms: u64) -> Vec<(u64, u64)> {
  let mut windows = Vec::new();
  let mut start = 0;
  loop {
    let end = (start + WINDOW_MS).min(call_ms);
    windows.push((start, end));
    if end >= call_ms {
      return windows;
    }
    start += WINDOW_MS - OVERLAP_MS;
  }
}

/// The least a window is halved to, in milliseconds: a shorter one holds too
/// little of each voice for its fingerprint.
const SHORTEST_MS: u64 = 60_000;

/// `window` in two halves that share a fifth of it, or none when they would
/// be shorter than [`SHORTEST_MS`].
pub fn halves((start, end): (u64, u64)) -> Option<[(u64, u64); 2]> {
  let length = end - start;
  let half = (length + length / 5) / 2;
  (half >= SHORTEST_MS).then(|| [(start, start + half), (end - half, end)])
}

/// The call's speaker turns from each window's own, in call time, numbered
/// per window from 1. `pcm` is the whole call, 16 kHz in [-1, 1].
pub fn joined(
  voices: &Fingerprints,
  pcm: &[f32],
  windows: &[(u64, u64)],
  per_window: &[Vec<SpeakerTurn>],
) -> Result<Vec<SpeakerTurn>, String> {
  // Each window's voices, in the order they first speak.
  let mut wanted: Vec<(usize, u32)> = Vec::new();
  let mut index: Vec<Vec<(u32, usize)>> = Vec::new();
  for (window, turns) in per_window.iter().enumerate() {
    let mut here: Vec<(u32, usize)> = Vec::new();
    for turn in turns {
      if !here.iter().any(|(speaker, _)| *speaker == turn.speaker) {
        here.push((turn.speaker, wanted.len()));
        wanted.push((window, turn.speaker));
      }
    }
    index.push(here);
  }
  // A fingerprint takes a fraction of a second, a long call's voices tens
  // of them on one core, while the notes are being finished.
  let cores = std::thread::available_parallelism().map_or(1, usize::from);
  let chunk = wanted.len().div_ceil(cores).max(1);
  let locals: Vec<Local> = std::thread::scope(|scope| {
    let workers: Vec<_> = wanted
      .chunks(chunk)
      .map(|jobs| {
        scope.spawn(move || {
          jobs
            .iter()
            .map(|&(window, speaker)| {
              let (fingerprint, heard_ms) =
                fingerprint_of(voices, pcm, &per_window[window], speaker)?;
              Ok(Local {
                window,
                fingerprint,
                heard_ms,
              })
            })
            .collect::<Result<Vec<_>, String>>()
        })
      })
      .collect();
    workers
      .into_iter()
      .map(|worker| {
        worker
          .join()
          .map_err(|_| "A voice fingerprint failed".to_owned())?
      })
      .collect::<Result<Vec<_>, String>>()
  })?
  .into_iter()
  .flatten()
  .collect();
  let local = |window: usize, speaker: u32| {
    index[window]
      .iter()
      .find(|(s, _)| *s == speaker)
      .map(|(_, at)| *at)
  };

  // Where consecutive windows agree in their overlap.
  let mut ties = Vec::new();
  for w in 0..windows.len().saturating_sub(1) {
    let overlap = (windows[w + 1].0, windows[w].1);
    for &(a, la) in &index[w] {
      for &(b, lb) in &index[w + 1] {
        let a_turns = clipped(&per_window[w], a, overlap);
        let b_turns = clipped(&per_window[w + 1], b, overlap);
        let together_ms = shared_ms(&a_turns, &b_turns);
        if together_ms > 0 {
          ties.push(Tie {
            a: la,
            b: lb,
            together_ms,
            a_ms: total_ms(&a_turns),
            b_ms: total_ms(&b_turns),
          });
        }
      }
    }
  }

  let speakers = link::link(&locals, &ties, ALIKE);

  // Each moment from the window whose middle it is nearest: the first half
  // of an overlap from the earlier window, the second from the later.
  let mut turns = Vec::new();
  for (w, window_turns) in per_window.iter().enumerate() {
    let from = if w == 0 {
      windows[w].0
    } else {
      (windows[w].0 + windows[w - 1].1) / 2
    };
    let to = if w + 1 == windows.len() {
      windows[w].1
    } else {
      (windows[w + 1].0 + windows[w].1) / 2
    };
    for turn in window_turns {
      let (start_ms, end_ms) = (turn.start_ms.max(from), turn.end_ms.min(to));
      if start_ms >= end_ms {
        continue;
      }
      let Some(at) = local(w, turn.speaker) else {
        continue;
      };
      turns.push(SpeakerTurn {
        start_ms,
        end_ms,
        speaker: u32::try_from(speakers[at] + 1).unwrap_or(u32::MAX),
      });
    }
  }
  turns.sort_by_key(|turn| turn.start_ms);
  Ok(turns)
}

/// The fingerprint of `speaker`'s voice, from their longest stretches where
/// they speak alone, and how much speech it was taken from.
fn fingerprint_of(
  voices: &Fingerprints,
  pcm: &[f32],
  turns: &[SpeakerTurn],
  speaker: u32,
) -> Result<(Option<Vec<f32>>, u64), String> {
  let alone = alone::stretches(turns, speaker);
  let mut samples = Vec::new();
  let mut heard_ms = 0;
  for (start, end) in alone {
    if heard_ms >= SPEECH_MAX_MS {
      break;
    }
    let end = end.min(start + SPEECH_MAX_MS - heard_ms);
    let (from, to) = (
      (start * RATE) as usize,
      ((end * RATE) as usize).min(pcm.len()),
    );
    if from < to {
      samples.extend_from_slice(&pcm[from..to]);
      heard_ms += end - start;
    }
  }
  if samples.is_empty() {
    return Ok((None, 0));
  }
  Ok((voices.of(&samples)?, heard_ms))
}

fn clipped(turns: &[SpeakerTurn], speaker: u32, (from, to): (u64, u64)) -> Vec<(u64, u64)> {
  turns
    .iter()
    .filter(|turn| turn.speaker == speaker)
    .map(|turn| (turn.start_ms.max(from), turn.end_ms.min(to)))
    .filter(|(start, end)| end > start)
    .collect()
}

fn total_ms(spans: &[(u64, u64)]) -> u64 {
  spans.iter().map(|(start, end)| end - start).sum()
}

fn shared_ms(a: &[(u64, u64)], b: &[(u64, u64)]) -> u64 {
  a.iter()
    .flat_map(|&(s1, e1)| {
      b.iter()
        .map(move |&(s2, e2)| e1.min(e2).saturating_sub(s1.max(s2)))
    })
    .sum()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn windows_overlap_and_cover_the_call() {
    assert_eq!(windows(100_000), vec![(0, 100_000)]);
    assert_eq!(
      windows(700_000),
      vec![(0, 300_000), (240_000, 540_000), (480_000, 700_000)]
    );
  }

  #[test]
  fn halves_share_a_fifth_and_stop_at_a_minute() {
    assert_eq!(
      halves((0, 300_000)),
      Some([(0, 180_000), (120_000, 300_000)])
    );
    assert_eq!(halves((0, 90_000)), None);
  }
}
