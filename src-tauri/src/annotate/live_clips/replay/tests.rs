// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::tests::{annotation, source};
use super::*;

const SECOND_US: u64 = 1_000_000;

fn replaying(live: &mut LiveAnnotations, origin: Instant) {
  let shared = Arc::new(OnceLock::new());
  shared.set(origin).unwrap();
  live.start_replay(shared, source(), Duration::from_secs(30));
}

fn spans(clips: &[RecordingAnnotationClip]) -> Vec<(String, u64, u64)> {
  clips
    .iter()
    .map(|clip| (clip.annotation.id.clone(), clip.start_ms, clip.end_ms))
    .collect()
}

#[test]
fn a_clip_cut_from_the_middle_starts_inherited_annotations_at_zero() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  replaying(&mut live, origin);
  live.add(annotation("early", 10.0), origin + Duration::from_secs(2));
  live.add(annotation("late", 200.0), origin + Duration::from_secs(12));

  let clips = live.replay_clips(10 * SECOND_US, 15 * SECOND_US);

  assert_eq!(
    spans(&clips),
    [
      ("early".to_owned(), 0, 5_000),
      ("late".to_owned(), 2_000, 5_000)
    ]
  );
}

#[test]
fn an_annotation_gone_before_the_clip_is_left_out_and_one_gone_inside_ends_there() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  replaying(&mut live, origin);
  live.add(annotation("before", 10.0), origin + Duration::from_secs(1));
  assert!(live.remove_last(origin + Duration::from_secs(3)));
  live.add(annotation("inside", 10.0), origin + Duration::from_secs(6));
  live.clear(origin + Duration::from_secs(8));

  let clips = live.replay_clips(5 * SECOND_US, 10 * SECOND_US);

  assert_eq!(clips.len(), 1);
  assert_eq!(clips[0].annotation.id, "inside");
  assert_eq!(clips[0].start_ms, 1_000);
  // The clip carries its closing phase past the clear, within the clip.
  assert!(clips[0].end_ms > 3_000 && clips[0].end_ms <= 5_000);
}

#[test]
fn replay_timing_does_not_disturb_a_recording_running_beside_it() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  replaying(&mut live, origin);
  let shared = Arc::new(OnceLock::new());
  shared.set(origin + Duration::from_secs(4)).unwrap();
  live.start(shared, source());
  live.add(annotation("a", 10.0), origin + Duration::from_secs(6));
  live.clear(origin + Duration::from_secs(7));

  let recorded = live.stop(origin + Duration::from_secs(9));
  let replayed = live.replay_clips(0, 9 * SECOND_US);

  assert_eq!(recorded[0].start_ms, 2_000);
  assert_eq!(replayed[0].start_ms, 6_000);
}

#[test]
fn stopping_the_replay_forgets_its_timing() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  replaying(&mut live, origin);
  live.add(annotation("a", 10.0), origin);
  live.stop_replay();

  assert!(live.replay_clips(0, 5 * SECOND_US).is_empty());
  assert!(live.annotations[0].replay_shown_at_us.is_none());
}
