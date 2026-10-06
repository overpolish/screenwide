// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const MS: i64 = 1_000_000;

fn frame(pts_ms: i64, keyframe: bool) -> EncodedFrame<i64> {
  EncodedFrame {
    keyframe,
    pts_ns: pts_ms * MS,
    sample: pts_ms,
  }
}

/// Frames every 100ms with a keyframe every 500ms, up to `end_ms`.
fn filled(horizon_ms: i64, end_ms: i64) -> VideoRing<i64> {
  let mut ring = VideoRing::new(horizon_ms * MS);
  for at in (0..=end_ms).step_by(100) {
    ring.push(frame(at, at % 500 == 0));
  }
  ring
}

fn pts(frames: &[EncodedFrame<i64>]) -> Vec<i64> {
  frames.iter().map(|frame| frame.sample).collect()
}

#[test]
fn keeps_the_keyframe_a_full_length_clip_has_to_start_from() {
  let ring = filled(1_000, 3_000);

  // The newest frame is at 3s, so a full clip wants to start at 2s: the
  // keyframe there is kept, and nothing before it is.
  assert_eq!(ring.keyframe_at_or_before(2_000 * MS), Some(2_000 * MS));
  assert_eq!(ring.keyframe_at_or_before(0), Some(2_000 * MS));
}

#[test]
fn a_clip_starting_between_keyframes_backs_up_to_the_one_before() {
  let ring = filled(10_000, 3_000);

  let start = ring.keyframe_at_or_before(1_300 * MS).unwrap();
  assert_eq!(start, 1_000 * MS);
  assert_eq!(
    pts(&ring.frames(start, 1_600 * MS)),
    [1_000, 1_100, 1_200, 1_300, 1_400, 1_500, 1_600]
  );
}

#[test]
fn a_follower_starts_at_its_first_keyframe_after_the_chosen_start() {
  let ring = filled(10_000, 3_000);

  assert_eq!(ring.keyframe_at_or_after(1_300 * MS), Some(1_500 * MS));
  assert_eq!(ring.keyframe_at_or_after(3_100 * MS), None);
}

#[test]
fn a_ring_with_sparse_keyframes_never_drops_the_only_one_it_has() {
  let mut ring = VideoRing::new(1_000 * MS);
  ring.push(frame(0, true));
  for at in (100..=5_000).step_by(100) {
    ring.push(frame(at, false));
  }

  assert_eq!(ring.keyframe_at_or_before(4_000 * MS), Some(0));
  assert_eq!(ring.frames(0, 5_000 * MS).len(), 51);
}

fn chunk(pts_ms: i64, frames: usize) -> AudioChunk {
  AudioChunk {
    pts_ns: pts_ms * MS,
    samples: vec![0.5; frames * 2],
  }
}

#[test]
fn audio_is_trimmed_to_start_exactly_at_the_clip() {
  // 1kHz stereo, 100 frames per chunk: 100ms each.
  let mut ring = AudioRing::new(2, 1_000, 10_000 * MS);
  for at in (0..1_000).step_by(100) {
    ring.push(chunk(at, 100));
  }

  let clip = ring.clip(250 * MS, 500 * MS);
  assert_eq!(
    clip
      .iter()
      .map(|chunk| chunk.pts_ns / MS)
      .collect::<Vec<_>>(),
    [250, 300, 400]
  );
  assert_eq!(clip[0].samples.len(), 50 * 2);
  assert_eq!(ring.end_ns(), Some(1_000 * MS));
}

#[test]
fn audio_older_than_the_horizon_is_let_go() {
  let mut ring = AudioRing::new(2, 1_000, 300 * MS);
  for at in (0..1_000).step_by(100) {
    ring.push(chunk(at, 100));
  }

  assert_eq!(
    ring
      .clip(0, 1_000 * MS)
      .first()
      .map(|chunk| chunk.pts_ns / MS),
    Some(500)
  );
}
