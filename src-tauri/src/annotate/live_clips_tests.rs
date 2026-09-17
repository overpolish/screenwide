// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use super::*;
use crate::editor::annotations::model::new_arrow;
use crate::editor::annotations::timing::validate_clips;
use crate::editor::annotations::{AnnotationPoint, AnnotationShape};
use crate::recording::cursor::CursorSourceKind;

/// A 1000x500 point display, at the desktop origin, recorded at 2x.
fn source() -> CursorSource {
  CursorSource {
    height: 500.0,
    kind: CursorSourceKind::Screen,
    platform_id: "1".to_owned(),
    video_height: 1_000,
    video_width: 2_000,
    width: 1_000.0,
    x: 0.0,
    y: 0.0,
  }
}

/// A short arrow starting at `x`, in global logical points.
fn annotation(id: &str, x: f64) -> Annotation {
  new_arrow(
    id.to_owned(),
    AnnotationPoint { x, y: 10.0 },
    AnnotationPoint {
      x: x + 100.0,
      y: 110.0,
    },
    None,
  )
}

/// Begins a recording whose first frame has already landed at `origin`.
fn started(live: &mut LiveAnnotations, origin: Instant) {
  let shared = Arc::new(OnceLock::new());
  shared.set(origin).unwrap();
  live.start(shared, source());
}

#[test]
fn annotations_on_screen_when_recording_starts_are_in_it_from_zero() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  live.add(annotation("first", 10.0), origin);
  live.add(annotation("second", 200.0), origin);
  started(&mut live, origin);
  live.add(annotation("third", 400.0), origin + Duration::from_secs(5));
  live.clear(origin + Duration::from_secs(8));

  let clips = live.stop(origin + Duration::from_secs(10));

  assert_eq!(clips.len(), 3);
  // Cleared at eight seconds, so each clip carries the closing phase's 750ms
  // past that: the annotation is whole when it goes and leaves afterwards.
  assert_eq!((clips[0].start_ms, clips[0].end_ms), (0, 8_750));
  assert_eq!((clips[1].start_ms, clips[1].end_ms), (0, 8_750));
  assert_eq!((clips[2].start_ms, clips[2].end_ms), (5_000, 8_750));
  assert!(clips
    .iter()
    .all(|clip| clip.track_id == AnnotationTrack::Primary));
  assert!(validate_clips(&clips).is_ok());
}

#[test]
fn an_annotation_still_on_screen_at_the_stop_ends_there() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  live.add(annotation("a", 10.0), origin + Duration::from_secs(5));

  let clips = live.stop(origin + Duration::from_secs(10));

  assert_eq!(clips.len(), 1);
  assert_eq!((clips[0].start_ms, clips[0].end_ms), (5_000, 10_000));
}

#[test]
fn a_stroke_drawn_during_a_pause_starts_at_the_resume() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  live.pause(origin + Duration::from_secs(3));
  live.add(annotation("a", 10.0), origin + Duration::from_secs(4));
  live.resume(origin + Duration::from_secs(6));

  let clips = live.stop(origin + Duration::from_secs(8));

  // Three seconds of recording ran before the pause and two after the resume,
  // so wall time is three seconds ahead of the movie throughout.
  assert_eq!((clips[0].start_ms, clips[0].end_ms), (3_000, 5_000));
}

#[test]
fn a_stroke_drawn_during_a_pause_the_recording_never_leaves_is_dropped() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  live.add(annotation("before", 10.0), origin);
  started(&mut live, origin);
  live.pause(origin + Duration::from_secs(3));
  live.add(annotation("during", 200.0), origin + Duration::from_secs(4));

  let clips = live.stop(origin + Duration::from_secs(6));

  assert_eq!(clips.len(), 1);
  assert_eq!(clips[0].annotation.id, "before");
  assert_eq!(clips[0].end_ms, 3_000);
}

#[test]
fn an_undone_annotation_keeps_the_clip_it_earned() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  live.add(annotation("a", 10.0), origin + Duration::from_secs(1));
  assert!(live.remove_last(origin + Duration::from_secs(4)));

  let clips = live.stop(origin + Duration::from_secs(9));

  assert_eq!(clips.len(), 1);
  assert_eq!((clips[0].start_ms, clips[0].end_ms), (1_000, 4_750));
}

#[test]
fn a_short_clip_leaves_over_the_closing_phase_it_is_allowed() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  live.add(annotation("a", 10.0), origin + Duration::from_secs(1));
  // Visible for 600ms, so the closing phase is capped to a third of the clip
  // rather than the full 750. The clip is the length that still puts the
  // leaving after the annotation went: 600 + 300.
  assert!(live.remove_last(origin + Duration::from_millis(1_600)));

  let clips = live.stop(origin + Duration::from_secs(9));

  assert_eq!((clips[0].start_ms, clips[0].end_ms), (1_000, 1_900));
}

#[test]
fn a_recording_that_ends_first_keeps_the_closing_phase_it_has_room_for() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  live.add(annotation("a", 10.0), origin);
  live.clear(origin + Duration::from_secs(5));

  // Stopped 200ms after the clear, so only 200ms of the closing phase fits.
  let clips = live.stop(origin + Duration::from_millis(5_200));

  assert_eq!((clips[0].start_ms, clips[0].end_ms), (0, 5_200));
  assert!(validate_clips(&clips).is_ok());
}

#[test]
fn a_cancelled_recording_leaves_no_clips_and_keeps_the_annotations() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  live.add(annotation("a", 10.0), origin + Duration::from_secs(1));
  live.cancel();

  assert!(live.stop(origin + Duration::from_secs(2)).is_empty());

  // The annotation is still on screen, so the next recording still records it.
  let second = origin + Duration::from_secs(10);
  started(&mut live, second);
  let clips = live.stop(second + Duration::from_secs(3));

  assert_eq!(clips.len(), 1);
  assert_eq!((clips[0].start_ms, clips[0].end_ms), (0, 3_000));
}

#[test]
fn annotations_that_outlive_a_recording_are_timed_fresh_by_the_next_one() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  live.add(annotation("a", 10.0), origin + Duration::from_secs(2));
  assert_eq!(live.stop(origin + Duration::from_secs(4)).len(), 1);

  let second = origin + Duration::from_secs(20);
  started(&mut live, second);
  let clips = live.stop(second + Duration::from_secs(3));

  assert_eq!((clips[0].start_ms, clips[0].end_ms), (0, 3_000));
}

#[test]
fn an_annotation_is_recorded_in_the_source_pixels_it_was_drawn_over() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  live.add(
    new_arrow(
      "a".to_owned(),
      AnnotationPoint { x: 100.0, y: 50.0 },
      AnnotationPoint { x: 200.0, y: 150.0 },
      None,
    ),
    origin,
  );

  let clips = live.stop(origin + Duration::from_secs(1));
  let AnnotationShape::Arrow { start, end, .. } = clips[0].annotation.shape else {
    unreachable!()
  };

  assert_eq!((start.x, start.y), (200.0, 100.0));
  assert_eq!((end.x, end.y), (400.0, 300.0));
  // The stroke's weight is pixels, as an editor annotation's is, so the
  // preset it was drawn with is the preset the clip carries.
  assert_eq!(clips[0].annotation.style.width, 8.0);
  assert!(validate_clips(&clips).is_ok());
}

#[test]
fn an_annotation_outside_the_recorded_pixels_produces_no_clip() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  // A second display, to the right of the one being recorded.
  live.add(annotation("elsewhere", 1_400.0), origin);

  assert!(live.stop(origin + Duration::from_secs(1)).is_empty());
}

#[test]
fn an_annotation_cleared_in_the_millisecond_it_appeared_still_spans_one() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  let at = origin + Duration::from_secs(5);
  live.add(annotation("a", 10.0), at);
  live.clear(at);

  let clips = live.stop(origin + Duration::from_secs(6));

  assert_eq!((clips[0].start_ms, clips[0].end_ms), (5_000, 5_001));
  assert!(validate_clips(&clips).is_ok());
}

#[test]
fn nothing_is_recorded_while_no_recording_runs() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  live.add(annotation("a", 10.0), origin);
  assert!(live.remove_last(origin + Duration::from_secs(1)));
  live.add(annotation("b", 200.0), origin + Duration::from_secs(2));
  live.clear(origin + Duration::from_secs(3));

  assert!(live.stop(origin + Duration::from_secs(4)).is_empty());
}

#[test]
fn a_repeated_annotation_id_is_refused() {
  let origin = Instant::now();
  let mut live = LiveAnnotations::default();
  started(&mut live, origin);
  live.add(annotation("a", 10.0), origin + Duration::from_secs(1));
  live.add(annotation("a", 200.0), origin + Duration::from_secs(2));

  let clips = live.stop(origin + Duration::from_secs(3));

  assert_eq!(clips.len(), 1);
  assert!(validate_clips(&clips).is_ok());
}
