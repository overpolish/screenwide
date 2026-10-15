// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::RecordingTimelineSegment;
use screenwide_annotations::timing::RecordingAnnotationClip;
use screenwide_scenes::RecordingSceneClip;

fn annotation(id: &str, start_ms: u64, end_ms: u64) -> RecordingAnnotationClip {
  serde_json::from_value(serde_json::json!({
    "annotation": {
      "id": id,
      "shape": {"kind": "arrow", "start": {"x": 0, "y": 0},
        "control": {"x": 1, "y": 1}, "end": {"x": 2, "y": 2}},
      "style": {"color": "#ff0000", "width": 4}
    },
    "trackId": "primary",
    "startMs": start_ms,
    "endMs": end_ms
  }))
  .unwrap()
}

fn scene(id: &str, start_ms: u64, end_ms: u64) -> RecordingSceneClip {
  serde_json::from_value(serde_json::json!({
    "id": id, "startMs": start_ms, "endMs": end_ms, "preset": "full"
  }))
  .unwrap()
}

fn segment(id: u64, source_start: f64, source_end: f64) -> RecordingTimelineSegment {
  RecordingTimelineSegment {
    id,
    playback_rate: 1.0,
    source_end,
    source_start,
  }
}

/// A 10 s recording: `segments`, with clips in recording time.
fn edit(segments: Vec<RecordingTimelineSegment>) -> RecordingTimelineEdit {
  RecordingTimelineEdit {
    annotation_clips: vec![annotation("a", 3_000, 7_000)],
    artifact_id: 1,
    keyboard_deletions: Box::default(),
    silence_cuts: Vec::new(),
    transcript_cuts: Vec::new(),
    transcript_corrections: Vec::new(),
    transcript_allowed_words: Vec::new(),
    captions: None,
    scene_clips: vec![scene("s", 3_000, 7_000)],
    camera_background_ranges: Vec::new(),
    next_segment_id: 3,
    segments,
  }
}

fn spans<T>(clips: &[T], span: impl Fn(&T) -> [u64; 2]) -> Vec<[u64; 2]> {
  clips.iter().map(span).collect()
}

/// 4-6 s cut away: the clip loses the footage the cut took and plays on
/// across the join.
#[test]
fn a_cut_shortens_a_clip_onto_the_timeline() {
  let mut cut = edit(vec![segment(0, 0.0, 0.4), segment(1, 0.6, 1.0)]);
  carry_clips_onto_timeline(&mut cut, 10_000);
  assert_eq!(
    spans(&cut.annotation_clips, |clip| [clip.start_ms, clip.end_ms]),
    [[3_000, 5_000]]
  );
  assert_eq!(
    spans(&cut.scene_clips, |clip| [clip.start_ms, clip.end_ms]),
    [[3_000, 5_000]]
  );
}

/// 5-10 s plays first, then 0-5 s: the clip's footage plays in two places,
/// so it becomes two clips, in timeline order, the first keeping its id.
#[test]
fn a_reordered_timeline_carries_a_clip_into_each_place_it_plays() {
  let mut reordered = edit(vec![segment(1, 0.5, 1.0), segment(0, 0.0, 0.5)]);
  carry_clips_onto_timeline(&mut reordered, 10_000);
  assert_eq!(
    spans(&reordered.annotation_clips, |clip| [
      clip.start_ms,
      clip.end_ms
    ]),
    [[0, 2_000], [8_000, 10_000]]
  );
  assert_eq!(reordered.annotation_clips[0].annotation.id, "a");
  assert_eq!(reordered.annotation_clips[1].annotation.id, "a-2");
  assert_eq!(
    spans(&reordered.scene_clips, |clip| [clip.start_ms, clip.end_ms]),
    [[0, 2_000], [8_000, 10_000]]
  );
  assert!(crate::validate(&reordered).is_ok());
}
