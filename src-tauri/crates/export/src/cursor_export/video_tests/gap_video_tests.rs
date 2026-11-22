// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Time with no recording behind it, through the real video export: a gap
//! inserted into the timeline and a blank scene each show the background,
//! and what is drawn over it, where the screen would be.

use super::scene_video_tests::{blue, clip, directory, exported, frame_at, pixel, red, timeline};
use super::*;
use screenwide_annotations::arrow::new_arrow;
use screenwide_annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use screenwide_annotations::AnnotationPoint;
use screenwide_scenes::RecordingSceneClip;
use screenwide_timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};

fn background(pixel: [u8; 3]) -> bool {
  pixel.iter().all(|channel| *channel < 40)
}

fn yellow(pixel: &[u8; 3]) -> bool {
  pixel[0] > 160 && pixel[1] > 100 && pixel[2] < 80
}

/// A 2 s recording, blue for its first second and red for its second.
fn halves(directory: &Path) -> PathBuf {
  let screen = directory.join("screen.mov");
  clip(
    &screen,
    "color=c=blue:s=640x360:r=30:d=1[a];color=c=red:s=640x360:r=30:d=1[b];[a][b]concat=n=2:v=1:a=0",
  );
  screen
}

/// The 2 s recording's timeline as `segments` play it, with `annotations`
/// and `scenes` over it.
fn plan(
  segments: Vec<RecordingTimelineSegment>,
  (annotations, scenes): (Vec<RecordingAnnotationClip>, Vec<RecordingSceneClip>),
) -> TimelinePlan {
  TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      audio_clips: Vec::new(),
      muted_audio: Vec::new(),
      annotation_clips: annotations,
      artifact_id: 1,
      keyboard_deletions: Box::default(),
      silence_cuts: Vec::new(),
      transcript_cuts: Vec::new(),
      transcript_corrections: Vec::new(),
      transcript_allowed_words: Vec::new(),
      captions: None,
      scene_clips: scenes,
      camera_background_ranges: Vec::new(),
      next_segment_id: 3,
      segments,
      kept_lulls: Vec::new(),
    },
    2_000,
  )
  .expect("a timeline with a gap is a plan")
}

fn segment(id: u64, source_start: f64, source_end: f64) -> RecordingTimelineSegment {
  RecordingTimelineSegment {
    id,
    playback_rate: 1.0,
    source_end,
    source_start,
    background: None,
  }
}

#[test]
#[cfg_attr(screenwide_no_gpu, ignore = "requires a hardware GPU")]
fn a_gap_parts_the_recording_and_shows_the_background_between() {
  let directory = directory("gap-between");
  let screen = halves(&directory);
  // A 1 s gap, half the recording's length, in a slot past its end.
  let timeline = plan(
    vec![
      segment(0, 0.0, 0.5),
      segment(2, 3.0, 3.5),
      segment(1, 0.5, 1.0),
    ],
    (Vec::new(), Vec::new()),
  );
  let destination = directory.join("gap.mp4");
  exported(&screen, None, &timeline, &destination);
  assert!(
    media_preview::duration_ms(&destination).is_some_and(|duration| duration.abs_diff(3_000) < 100)
  );
  assert!(blue(pixel(&frame_at(&destination, 0.2), 320, 180)));
  assert!(background(pixel(&frame_at(&destination, 1.5), 320, 180)));
  assert!(red(pixel(&frame_at(&destination, 2.8), 320, 180)));
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
#[cfg_attr(screenwide_no_gpu, ignore = "requires a hardware GPU")]
fn a_gap_can_start_the_video() {
  let directory = directory("gap-first");
  let screen = halves(&directory);
  let timeline = plan(
    vec![segment(2, 3.0, 3.5), segment(0, 0.0, 1.0)],
    (Vec::new(), Vec::new()),
  );
  let destination = directory.join("gap-first.mp4");
  exported(&screen, None, &timeline, &destination);
  assert!(background(pixel(&frame_at(&destination, 0.3), 320, 180)));
  assert!(blue(pixel(&frame_at(&destination, 1.8), 320, 180)));
  assert!(red(pixel(&frame_at(&destination, 2.8), 320, 180)));
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
#[cfg_attr(screenwide_no_gpu, ignore = "requires a hardware GPU")]
fn a_blank_scene_hides_the_screen_while_the_recording_plays_on() {
  let directory = directory("blank-scene");
  let screen = directory.join("screen.mov");
  clip(&screen, "color=c=blue:s=640x360:r=30:d=2");
  let destination = directory.join("blank.mp4");
  exported(
    &screen,
    None,
    &timeline(serde_json::json!({
      "id": "b", "startMs": 500, "endMs": 1_900, "preset": "blank"
    })),
    &destination,
  );
  assert!(blue(pixel(&frame_at(&destination, 0.25), 320, 180)));
  assert!(background(pixel(&frame_at(&destination, 1.2), 320, 180)));
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
#[cfg_attr(screenwide_no_gpu, ignore = "requires a hardware GPU")]
fn what_is_drawn_shows_over_a_gap_and_a_blank_scene() {
  let directory = directory("gap-drawn");
  let screen = halves(&directory);
  let mut arrow = new_arrow(
    "across".into(),
    AnnotationPoint { x: 80.0, y: 180.0 },
    AnnotationPoint { x: 560.0, y: 180.0 },
    None,
  );
  arrow.style.width = 16.0;
  let drawn = RecordingAnnotationClip {
    path_ms: None,
    pin: None,
    recording_span: None,
    scene_shapes: Vec::new(),
    annotation: arrow,
    track_id: AnnotationTrack::Primary,
    start_ms: 1_100,
    end_ms: 2_950,
  };
  // Blue, then a 1 s gap, then red under a blank scene to the end.
  let blank: RecordingSceneClip = serde_json::from_value(serde_json::json!({
    "id": "b", "startMs": 2_000, "endMs": 3_000, "preset": "blank"
  }))
  .unwrap();
  let timeline = plan(
    vec![
      segment(0, 0.0, 0.5),
      segment(2, 3.0, 3.5),
      segment(1, 0.5, 1.0),
    ],
    (vec![drawn], vec![blank]),
  );
  let destination = directory.join("drawn.mp4");
  exported(&screen, None, &timeline, &destination);
  let yellow_in = |seconds: f64| {
    frame_at(&destination, seconds)
      .as_chunks::<3>()
      .0
      .iter()
      .filter(|rgb| yellow(rgb))
      .count()
  };
  assert!(yellow_in(1.6) > 500, "the arrow over the gap");
  assert!(yellow_in(2.8) > 500, "the arrow over the blank scene");
  let settled = frame_at(&destination, 2.8);
  assert!(background(pixel(&settled, 320, 60)), "the screen hidden");
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
#[cfg_attr(screenwide_no_gpu, ignore = "requires a hardware GPU")]
fn a_gap_wears_its_own_background_and_no_scene_plays_over_it() {
  let directory = directory("gap-background");
  let screen = halves(&directory);
  let gap = RecordingTimelineSegment {
    background: serde_json::from_value(serde_json::json!({ "kind": "solid", "color": "#00FF00" }))
      .ok(),
    ..segment(2, 3.0, 3.5)
  };
  // A scene drawn across the gap, which plays only either side of it.
  let across: RecordingSceneClip = serde_json::from_value(serde_json::json!({
    "id": "across", "startMs": 500, "endMs": 2_500, "preset": "screen-only"
  }))
  .unwrap();
  let timeline = plan(
    vec![segment(0, 0.0, 0.5), gap, segment(1, 0.5, 1.0)],
    (Vec::new(), vec![across]),
  );
  let destination = directory.join("gap-background.mp4");
  exported(&screen, None, &timeline, &destination);
  let green = |[red, green, blue]: [u8; 3]| green > 200 && red < 60 && blue < 60;
  assert!(green(pixel(&frame_at(&destination, 1.5), 320, 180)));
  assert!(red(pixel(&frame_at(&destination, 2.8), 320, 180)));
  let _ = std::fs::remove_dir_all(directory);
}
