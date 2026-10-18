// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A timeline whose segments play out of source order, through the real
//! video export: each stretch shows its own footage where the timeline puts
//! it.

use super::scene_video_tests::{blue, clip, directory, frame_at, pixel, red};
use super::tests::output;
use super::*;
use screenwide_timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};

#[test]
fn exports_reordered_segments_in_the_order_they_play() {
  let directory = directory("reordered");
  let screen = directory.join("screen.mov");
  // Blue for the first second, red for the second.
  clip(
    &screen,
    "color=c=blue:s=640x360:r=30:d=1[a];color=c=red:s=640x360:r=30:d=1[b];[a][b]concat=n=2:v=1:a=0",
  );
  let segment = |id, source_start, source_end| RecordingTimelineSegment {
    id,
    playback_rate: 1.0,
    source_end,
    source_start,
  };
  let timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      annotation_clips: Vec::new(),
      artifact_id: 1,
      keyboard_deletions: Box::default(),
      silence_cuts: Vec::new(),
      transcript_cuts: Vec::new(),
      transcript_corrections: Vec::new(),
      transcript_allowed_words: Vec::new(),
      captions: None,
      scene_clips: Vec::new(),
      camera_background_ranges: Vec::new(),
      next_segment_id: 2,
      segments: vec![segment(1, 0.5, 1.0), segment(0, 0.0, 0.5)],
    },
    2_000,
  )
  .expect("a reordered timeline is a plan");
  let destination = directory.join("reordered.mp4");
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    camera_effects: None,
    annotation_track: screenwide_annotations::timing::AnnotationTrack::Primary,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: None,
    camera_on_top: true,
    cancelled: &cancelled,
    captions: None,
    cursor: None,
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings::default(),
    destination: &destination,
    duration_ms: 2_000,
    height: 360,
    on_progress: &mut |_| {},
    output: &output(640, 360),
    screen: &screen,
    selection: &TrackSelection::default(),
    timeline: Some(&timeline),
    video: VideoExportOptions {
      compression: 1,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
    width: 640,
  })
  .unwrap();
  assert_eq!(result, ExportRunResult::Completed);
  assert!(
    media_preview::duration_ms(&destination).is_some_and(|duration| duration.abs_diff(2_000) < 100)
  );
  assert!(red(pixel(&frame_at(&destination, 0.5), 320, 180)));
  assert!(blue(pixel(&frame_at(&destination, 1.5), 320, 180)));
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn exports_the_stretch_after_a_long_cut() {
  let directory = directory("long-cut");
  let screen = directory.join("screen.mov");
  // Blue for a second, green for five, red for the last: the cut drops the
  // green, which is long enough that the reader seeks over it.
  clip(
    &screen,
    "color=c=blue:s=640x360:r=30:d=1[a];color=c=green:s=640x360:r=30:d=5[b];color=c=red:s=640x360:r=30:d=1[c];[a][b][c]concat=n=3:v=1:a=0",
  );
  let segment = |id, source_start, source_end| RecordingTimelineSegment {
    id,
    playback_rate: 1.0,
    source_end,
    source_start,
  };
  let timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      annotation_clips: Vec::new(),
      artifact_id: 1,
      keyboard_deletions: Box::default(),
      silence_cuts: Vec::new(),
      transcript_cuts: Vec::new(),
      transcript_corrections: Vec::new(),
      transcript_allowed_words: Vec::new(),
      captions: None,
      scene_clips: Vec::new(),
      camera_background_ranges: Vec::new(),
      next_segment_id: 2,
      segments: vec![segment(0, 0.0, 1.0 / 7.0), segment(1, 6.0 / 7.0, 1.0)],
    },
    7_000,
  )
  .expect("a cut timeline is a plan");
  let destination = directory.join("long-cut.mp4");
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    camera_effects: None,
    annotation_track: screenwide_annotations::timing::AnnotationTrack::Primary,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: None,
    camera_on_top: true,
    cancelled: &cancelled,
    captions: None,
    cursor: None,
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings::default(),
    destination: &destination,
    duration_ms: 7_000,
    height: 360,
    on_progress: &mut |_| {},
    output: &output(640, 360),
    screen: &screen,
    selection: &TrackSelection::default(),
    timeline: Some(&timeline),
    video: VideoExportOptions {
      compression: 1,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
    width: 640,
  })
  .unwrap();
  assert_eq!(result, ExportRunResult::Completed);
  assert!(
    media_preview::duration_ms(&destination).is_some_and(|duration| duration.abs_diff(2_000) < 100)
  );
  assert!(blue(pixel(&frame_at(&destination, 0.5), 320, 180)));
  // The first frame after the cut, and one later, are red: the seek lands
  // on the stretch kept, not on the green it skipped.
  assert!(red(pixel(&frame_at(&destination, 1.05), 320, 180)));
  assert!(red(pixel(&frame_at(&destination, 1.5), 320, 180)));
  let _ = std::fs::remove_dir_all(directory);
}
