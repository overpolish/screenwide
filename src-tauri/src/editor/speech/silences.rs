// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which pauses to cut: the stretches between speech long enough to be dead
//! air for this speaker, less a natural pause kept at each end.

use super::analysis::SpeechMap;

/// A window this likely to be speech starts speech...
const SPEECH_ON: f32 = 0.5;
/// ...and speech goes on until a window falls below this. The gap between
/// the two keeps a word's quiet tail from flickering in and out.
const SPEECH_OFF: f32 = 0.35;
/// Speech shorter than this is a cough or a click, not words.
const SHORTEST_SPEECH_MS: f64 = 120.0;
/// A gap shorter than this sits inside a phrase and says nothing about how
/// long the speaker pauses between them.
const SHORTEST_PAUSE_MS: f64 = 150.0;
/// A pause is dead air once it runs this many times the speaker's usual
/// pause...
const USUAL_PAUSE_FACTOR: f64 = 2.0;
/// ...within these bounds: a quick speaker's breaths are never cut, and a
/// slow speaker's long silences always are.
const DEAD_AIR_MS: (f64, f64) = (700.0, 1_500.0);
/// Without enough pauses to learn from, the speaker's usual pause is taken
/// to give this.
const DEFAULT_DEAD_AIR_MS: f64 = 1_000.0;
/// How many pauses it takes to learn the speaker's usual one.
const PAUSES_TO_LEARN: usize = 3;
/// Kept after speech ends, so its last sound and breath finish...
pub(super) const KEEP_AFTER_SPEECH_MS: f64 = 250.0;
/// ...and before it starts again, so the first sound is not clipped.
pub(super) const KEEP_BEFORE_SPEECH_MS: f64 = 150.0;
/// A cut shorter than this would only make the picture jump.
const SHORTEST_CUT_MS: u64 = 500;

/// The stretches of the recording to cut, in milliseconds, sorted. Only the
/// speech decides: a pause is cut whatever happens on screen in it, and the
/// markers on the ruler bring back any that should have stayed.
pub(super) fn plan(speech: &SpeechMap, duration_ms: u64) -> Vec<(u64, u64)> {
  let spoken = speech_ranges(speech, duration_ms as f64);
  let (Some(first), Some(last)) = (spoken.first(), spoken.last()) else {
    // Nothing said: no speech to pause between.
    return Vec::new();
  };
  let pauses: Vec<f64> = spoken
    .windows(2)
    .map(|pair| pair[1].0 - pair[0].1)
    .filter(|&gap| gap >= SHORTEST_PAUSE_MS)
    .collect();
  let dead_air = dead_air_ms(pauses);

  // Before the first words and after the last there is no speech on the far
  // side to keep a pause for.
  let mut gaps = vec![(0.0, first.0 - KEEP_BEFORE_SPEECH_MS, first.0)];
  gaps.extend(spoken.windows(2).map(|pair| {
    (
      pair[0].1 + KEEP_AFTER_SPEECH_MS,
      pair[1].0 - KEEP_BEFORE_SPEECH_MS,
      pair[1].0 - pair[0].1,
    )
  }));
  gaps.push((
    last.1 + KEEP_AFTER_SPEECH_MS,
    duration_ms as f64,
    duration_ms as f64 - last.1,
  ));

  gaps
    .into_iter()
    .filter(|&(_, _, length)| length >= dead_air)
    .map(|(start, end, _)| (start.max(0.0) as u64, end.max(0.0) as u64))
    .filter(|&(start, end)| end.saturating_sub(start) >= SHORTEST_CUT_MS)
    .collect()
}

/// How long a pause has to run to be dead air, from how long the speaker
/// usually pauses.
fn dead_air_ms(mut pauses: Vec<f64>) -> f64 {
  if pauses.len() < PAUSES_TO_LEARN {
    return DEFAULT_DEAD_AIR_MS;
  }
  pauses.sort_by(f64::total_cmp);
  let usual = pauses[pauses.len() / 2];
  (usual * USUAL_PAUSE_FACTOR).clamp(DEAD_AIR_MS.0, DEAD_AIR_MS.1)
}

/// Where the speech is, in milliseconds.
pub(super) fn speech_ranges(speech: &SpeechMap, duration_ms: f64) -> Vec<(f64, f64)> {
  let mut ranges = Vec::new();
  let mut start: Option<usize> = None;
  for (index, &chance) in speech.probabilities.iter().enumerate() {
    match start {
      None if chance >= SPEECH_ON => start = Some(index),
      Some(from) if chance < SPEECH_OFF => {
        ranges.push((speech.start_ms(from), speech.start_ms(index)));
        start = None;
      }
      _ => {}
    }
  }
  if let Some(from) = start {
    ranges.push((
      speech.start_ms(from),
      speech.start_ms(speech.probabilities.len()),
    ));
  }
  ranges
    .into_iter()
    .map(|(start, end)| (start, end.min(duration_ms)))
    .filter(|&(start, end)| end - start >= SHORTEST_SPEECH_MS)
    .collect()
}
