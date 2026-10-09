// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! When the system audio makes way, frame by frame: fully while the voice
//! speaks, not at all while it is quiet, and easing between the two. The
//! speech the voice activity model hears is padded by the margins the pause
//! gate keeps, and gaps between words shorter than a breath are bridged, so
//! the audio does not bob up and down within a sentence.

use super::super::analysis::SpeechMap;
use super::super::silences::{speech_ranges, KEEP_AFTER_SPEECH_MS, KEEP_BEFORE_SPEECH_MS};

/// Gaps in speech shorter than this keep the audio down.
const BRIDGE_MS: f64 = 600.0;
/// How quickly the audio makes way when the voice starts, and comes back
/// when it stops, as the time each takes to go most of the way.
const DOWN_MS: f64 = 60.0;
const UP_MS: f64 = 350.0;

/// How far the audio makes way in each frame, 0 to 1, for frames whose
/// middles are `frame_ms(frame)` into the recording.
pub(super) fn gate(
  speech: &SpeechMap,
  duration_ms: f64,
  frames: usize,
  frame_ms: impl Fn(usize) -> f64,
  step_ms: f64,
) -> Vec<f32> {
  let mut spans: Vec<(f64, f64)> = Vec::new();
  for (start, end) in speech_ranges(speech, duration_ms) {
    let (start, end) = (start - KEEP_BEFORE_SPEECH_MS, end + KEEP_AFTER_SPEECH_MS);
    match spans.last_mut() {
      Some(last) if start - last.1 < BRIDGE_MS => last.1 = last.1.max(end),
      _ => spans.push((start, end)),
    }
  }
  let (down, up) = (step(step_ms, DOWN_MS), step(step_ms, UP_MS));
  let (mut next, mut level) = (0, 0.0_f32);
  (0..frames)
    .map(|frame| {
      let at = frame_ms(frame);
      while spans.get(next).is_some_and(|&(_, end)| end <= at) {
        next += 1;
      }
      let target = if spans.get(next).is_some_and(|&(start, _)| start <= at) {
        1.0
      } else {
        0.0
      };
      level += if target > level { down } else { up } * (target - level);
      level
    })
    .collect()
}

/// The share of the way a one-pole follower with time constant `ms` goes
/// in a step `step_ms` long.
fn step(step_ms: f64, ms: f64) -> f32 {
  (1.0 - (-step_ms / ms).exp()) as f32
}
