// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which few seconds of a voice its clip keeps, and how loud. Where the
//! speaker model says a voice speaks is not all speech: a turn takes in the
//! pauses, breaths and thinking between its words. So the clip is the
//! window with the most speech in it among where the voice speaks, and it
//! is brought to one level, so a quiet voice or a call played softly is
//! heard as well as a loud one.

use super::alone::RATE;

/// How long a clip is at most, in milliseconds.
pub const CLIP_MS: u64 = 5_000;
/// How long a frame is, in milliseconds: the step a window moves by, and
/// the span whose loudness says whether it is speech.
const FRAME_MS: u64 = 20;
const FRAME: usize = (FRAME_MS * RATE) as usize;
/// The share of a voice's frames taken as its quiet between words: the
/// level below which that many of them lie.
const FLOOR_SHARE: f64 = 0.1;
/// How much louder than its quiet a frame is to be speech, in mean square:
/// about 10 dB.
const SPEECH_OVER_FLOOR: u64 = 9;
/// The least mean square a frame is speech at, whatever the quiet: about
/// −50 dBFS, so a stretch of near silence never reads as speech.
const SPEECH_AT_LEAST: u64 = 100 * 100;
/// The peak a clip is brought to: about −3 dBFS, room left for the fades.
const TARGET_PEAK: f32 = 0.7 * i16::MAX as f32;
/// How much a clip is raised at most, about 18 dB, so a clip of hiss is
/// not raised into a roar.
const MOST_GAIN: f32 = 8.0;

/// The window of at most `CLIP_MS` with the most speech in `audio` among
/// `spans`, in milliseconds, the earliest of equals; louder wins between
/// windows of as much speech. None when no span holds a frame.
pub fn liveliest(audio: &[i16], spans: &[(u64, u64)]) -> Option<(u64, u64)> {
  let framed: Vec<(u64, Vec<u64>)> = spans
    .iter()
    .map(|&(start, end)| (start, frames(audio, start, end)))
    .filter(|(_, frames)| !frames.is_empty())
    .collect();
  let mut all: Vec<u64> = framed
    .iter()
    .flat_map(|(_, frames)| frames.iter().copied())
    .collect();
  if all.is_empty() {
    return None;
  }
  let at = ((all.len() as f64 * FLOOR_SHARE) as usize).min(all.len() - 1);
  let floor = *all.select_nth_unstable(at).1;
  let speech = floor.saturating_mul(SPEECH_OVER_FLOOR).max(SPEECH_AT_LEAST);
  let longest = (CLIP_MS / FRAME_MS) as usize;
  let mut best: Option<((usize, u64), (u64, u64))> = None;
  for (start, frames) in &framed {
    let width = longest.min(frames.len());
    let mut spoken: Vec<usize> = vec![0];
    let mut loudness: Vec<u64> = vec![0];
    for &frame in frames {
      spoken.push(spoken.last().copied().unwrap_or(0) + usize::from(frame > speech));
      loudness.push(loudness.last().copied().unwrap_or(0) + frame);
    }
    for first in 0..=frames.len() - width {
      let score = (
        spoken[first + width] - spoken[first],
        loudness[first + width] - loudness[first],
      );
      if best.is_none_or(|(most, _)| score > most) {
        let from = start + first as u64 * FRAME_MS;
        best = Some((score, (from, from + width as u64 * FRAME_MS)));
      }
    }
  }
  best.map(|(_, window)| window)
}

/// The mean square of each whole frame of `audio` from `start_ms` to
/// `end_ms`.
fn frames(audio: &[i16], start_ms: u64, end_ms: u64) -> Vec<u64> {
  let from = ((start_ms * RATE) as usize).min(audio.len());
  let to = ((end_ms * RATE) as usize).min(audio.len());
  audio[from..to]
    .as_chunks::<FRAME>()
    .0
    .iter()
    .map(|frame| {
      let sum: u64 = frame
        .iter()
        .map(|&sample| (i64::from(sample) * i64::from(sample)) as u64)
        .sum();
      sum / FRAME as u64
    })
    .collect()
}

/// `samples` brought to `TARGET_PEAK`, raised by `MOST_GAIN` at most.
pub fn evened(samples: &mut [i16]) {
  let peak = samples
    .iter()
    .map(|sample| sample.unsigned_abs())
    .max()
    .unwrap_or(0);
  if peak == 0 {
    return;
  }
  let gain = (TARGET_PEAK / f32::from(peak)).min(MOST_GAIN);
  for sample in samples {
    *sample = (f32::from(*sample) * gain).round() as i16;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// `ms` of a tone `amplitude` high, or of silence at 0.
  fn sound(ms: u64, amplitude: i16) -> Vec<i16> {
    (0..ms * RATE)
      .map(|i| if i % 2 == 0 { amplitude } else { -amplitude })
      .collect()
  }

  #[test]
  fn a_turn_that_opens_in_silence_is_clipped_where_the_voice_is() {
    // Three seconds of quiet, then six of speech, in one eight-second stretch.
    let audio = [sound(3_000, 20), sound(6_000, 4_000)].concat();
    let window = liveliest(&audio, &[(0, 9_000)]);
    assert_eq!(window, Some((3_000, 8_000)));
  }

  #[test]
  fn the_stretch_with_speech_wins_over_a_longer_one_without() {
    let audio = [sound(8_000, 20), sound(1_000, 0), sound(3_000, 4_000)].concat();
    let window = liveliest(&audio, &[(0, 8_000), (9_000, 12_000)]);
    assert_eq!(window, Some((9_000, 12_000)));
  }

  #[test]
  fn even_sound_keeps_the_earliest_window() {
    let audio = sound(8_000, 1_000);
    assert_eq!(liveliest(&audio, &[(250, 8_000)]), Some((250, 5_250)));
  }

  #[test]
  fn a_quiet_voice_is_raised_and_a_loud_one_lowered_to_one_peak() {
    let mut quiet = sound(100, 3_000);
    evened(&mut quiet);
    assert_eq!(quiet[0], TARGET_PEAK.round() as i16);
    let mut loud = sound(100, i16::MAX);
    evened(&mut loud);
    assert_eq!(loud[0], TARGET_PEAK.round() as i16);
  }

  #[test]
  fn hiss_is_raised_only_so_far() {
    let mut hiss = sound(100, 50);
    evened(&mut hiss);
    assert_eq!(hiss[0], (50.0 * MOST_GAIN) as i16);
  }
}
