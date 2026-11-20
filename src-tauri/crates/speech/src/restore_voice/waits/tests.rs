// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::heard;

const QUIET_DB: f32 = -75.0;
const VOICE_DB: f32 = -30.0;

/// A track of `blocks` 10 ms blocks, quiet and unlikely speech, with speech
/// the detector is sure of over each of `spoken`.
fn track(blocks: usize, spoken: &[std::ops::Range<usize>]) -> (Vec<f32>, Vec<f32>) {
  let mut levels = vec![QUIET_DB; blocks];
  let mut chances = vec![0.0; blocks];
  for range in spoken {
    levels[range.clone()].fill(VOICE_DB);
    chances[range.clone()].fill(1.0);
  }
  (levels, chances)
}

#[test]
fn a_long_wait_between_speech_is_left_out_with_half_a_second_kept_either_side() {
  // Speech, a minute's wait, speech.
  let (levels, chances) = track(7_000, &[0..500, 6_500..7_000]);
  assert_eq!(heard(&levels, &chances), vec![0..550, 6_450..7_000]);
}

#[test]
fn pauses_between_sentences_are_rebuilt_with_the_speech() {
  let (levels, chances) = track(1_000, &[0..400, 650..1_000]);
  assert_eq!(heard(&levels, &chances), vec![0..1_000]);
}

#[test]
fn a_soft_word_the_detector_misses_is_kept_for_its_loudness() {
  let (mut levels, chances) = track(7_000, &[0..500, 6_500..7_000]);
  levels[3_000..3_040].fill(-50.0);
  assert_eq!(
    heard(&levels, &chances),
    vec![0..550, 2_950..3_090, 6_450..7_000]
  );
}

#[test]
fn quiet_at_the_start_and_end_is_left_out_only_when_long() {
  // A tenth of a second before the first word stays; four and a half
  // seconds after the last are left out.
  let (levels, chances) = track(3_000, &[10..500, 2_000..2_500]);
  assert_eq!(heard(&levels, &chances), vec![0..550, 1_950..2_550]);
  let (levels, chances) = track(3_000, &[100..1_000, 1_100..2_900]);
  assert_eq!(heard(&levels, &chances), vec![0..3_000]);
}
