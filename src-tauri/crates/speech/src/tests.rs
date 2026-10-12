// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::analysis::SpeechMap;
use super::pause_gate::PauseGate;
use super::silences::plan;

const WINDOW_MS: f64 = 32.0;

/// A listen that hears speech over each of `spoken`, in milliseconds, and
/// nothing else, over `duration_ms`.
fn speech(spoken: &[(u64, u64)], duration_ms: u64) -> SpeechMap {
  let windows = (duration_ms as f64 / WINDOW_MS).ceil() as usize;
  SpeechMap {
    probabilities: (0..windows)
      .map(|index| {
        let ms = index as f64 * WINDOW_MS;
        if spoken
          .iter()
          .any(|&(start, end)| ms >= start as f64 && ms < end as f64)
        {
          0.9
        } else {
          0.05
        }
      })
      .collect(),
    window_ms: WINDOW_MS,
  }
}

/// Speech starting and ending on window edges, so the cuts land exactly.
fn at(window: u64) -> u64 {
  window * WINDOW_MS as u64
}

#[test]
fn cuts_a_long_pause_and_keeps_a_natural_one() {
  let spoken = [(0, at(100)), (at(200), at(300))];
  let cuts = plan(&speech(&spoken, at(300)), at(300));
  assert_eq!(cuts, vec![(at(100) + 250, at(200) - 150)]);
}

#[test]
fn leaves_a_breath_alone() {
  let spoken = [(0, at(100)), (at(110), at(200))];
  assert_eq!(plan(&speech(&spoken, at(200)), at(200)), vec![]);
}

#[test]
fn cuts_the_quiet_before_the_first_words_and_after_the_last() {
  let spoken = [(at(100), at(200))];
  let duration = at(300);
  let cuts = plan(&speech(&spoken, duration), duration);
  assert_eq!(cuts, vec![(0, at(100) - 150), (at(200) + 250, duration)]);
}

#[test]
fn nothing_said_leaves_nothing_to_cut() {
  assert_eq!(plan(&speech(&[], 60_000), 60_000), vec![]);
}

/// Phrases separated by `pause_ms`, then one pause of `long_ms`, then a
/// last phrase.
fn speaker(pause_ms: u64, long_ms: u64) -> (Vec<(u64, u64)>, u64) {
  let mut spoken = Vec::new();
  let mut ms = 0;
  for _ in 0..5 {
    spoken.push((ms, ms + 2_000));
    ms += 2_000 + pause_ms;
  }
  ms += long_ms - pause_ms;
  spoken.push((ms, ms + 2_000));
  (spoken, ms + 2_000)
}

#[test]
fn a_slow_speakers_long_pause_is_their_rhythm() {
  // Pausing 640 ms as a habit, 1.1 s is no more than a breath for them.
  let (spoken, duration) = speaker(640, 1_120);
  assert_eq!(plan(&speech(&spoken, duration), duration), vec![]);
}

#[test]
fn a_quick_speakers_same_pause_is_dead_air() {
  // Pausing 320 ms as a habit, the same 1.1 s stands out.
  let (spoken, duration) = speaker(320, 1_120);
  assert_eq!(plan(&speech(&spoken, duration), duration).len(), 1);
}

#[test]
fn quiets_pauses_but_not_speech_or_its_margins() {
  // At 1 kHz a sample is a millisecond.
  let spoken = [(at(100), at(200))];
  let length = at(400);
  let mut gate = PauseGate::new(&speech(&spoken, length), 1_000, length);
  let mut samples = vec![1.0_f32; length as usize];
  // Applied in pieces, as the track is read.
  for (index, piece) in samples.chunks_mut(997).enumerate() {
    gate.apply(index as u64 * 997, piece);
  }
  let gain = |ms: u64| samples[ms as usize];
  assert!(gain(at(150)) == 1.0, "speech is untouched");
  assert!(
    gain(at(100) - 150) == 1.0,
    "the margin before speech is kept"
  );
  assert!(
    gain(at(200) + 249) == 1.0,
    "the margin after speech is kept"
  );
  assert!(gain(at(50)) < 0.02, "a pause well before speech is quiet");
  assert!(gain(at(300)) < 0.02, "a pause well after speech is quiet");
  let fade: Vec<f32> = (at(200) + 250..at(200) + 400).map(gain).collect();
  assert!(
    fade.windows(2).all(|pair| pair[1] <= pair[0]),
    "the gain eases down rather than stepping back up"
  );
}
