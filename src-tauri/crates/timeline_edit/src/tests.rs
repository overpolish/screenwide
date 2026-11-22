// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_timeline::{source_after_output_duration_us, TimelineRange};

#[test]
fn old_edits_default_deleted_shortcuts_to_empty() {
  let edit: RecordingTimelineEdit = serde_json::from_value(serde_json::json!({
    "artifactId": 1,
    "nextSegmentId": 2,
    "segments": [{"id": 1, "sourceStart": 0.0, "sourceEnd": 1.0}]
  }))
  .unwrap();
  assert!(edit.keyboard_deletions.fragments.is_empty());
  assert!(edit.keyboard_deletions.shortcut_ids.is_empty());
}

#[test]
fn duplicate_deleted_shortcut_ids_are_rejected() {
  let mut edit = edit(1, 0.5);
  edit.keyboard_deletions.shortcut_ids = vec![2, 2];
  assert!(validate(&edit).is_err());
}

#[test]
fn fragment_deletions_map_to_their_segment_source_range() {
  let mut timeline = edit(1, 0.25);
  timeline
    .keyboard_deletions
    .fragments
    .push(DeletedKeyboardShortcutFragment {
      segment_id: 0,
      shortcut_id: 7,
    });
  let serialized = serde_json::to_value(&timeline).unwrap();
  assert_eq!(
    serialized["deletedKeyboardShortcutFragments"][0]["segmentId"],
    0
  );
  let plan = TimelinePlan::from_edit(&timeline, 8_000).unwrap();
  assert_eq!(
    plan.deleted_keyboard_shortcut_ranges(),
    &[DeletedKeyboardShortcutRange {
      end_ms: 2_000,
      shortcut_id: 7,
      start_ms: 0,
    }]
  );
}

fn edit(artifact_id: u64, split: f64) -> RecordingTimelineEdit {
  RecordingTimelineEdit {
    audio_clips: Vec::new(),
    muted_audio: Vec::new(),
    annotation_clips: Vec::new(),
    artifact_id,
    keyboard_deletions: Box::default(),
    silence_cuts: Vec::new(),
    transcript_cuts: Vec::new(),
    transcript_corrections: Vec::new(),
    transcript_allowed_words: Vec::new(),
    captions: None,
    scene_clips: Vec::new(),
    camera_background_ranges: Vec::new(),
    next_segment_id: 2,
    segments: vec![
      RecordingTimelineSegment {
        id: 0,
        source_end: split,
        source_start: 0.0,
        playback_rate: 1.0,
        background: None,
      },
      RecordingTimelineSegment {
        id: 1,
        source_end: 1.0,
        source_start: split,
        playback_rate: 1.0,
        background: None,
      },
    ],
    kept_lulls: Vec::new(),
  }
}

#[test]
fn rejects_overlapping_or_empty_segments() {
  let mut invalid = edit(1, 0.5);
  invalid.segments[1].source_start = 0.25;
  assert!(validate(&invalid).is_err());
  invalid.segments.clear();
  assert!(validate(&invalid).is_err());
}

#[test]
fn a_gap_plays_past_the_recording_and_never_joins_it() {
  // An 8 s recording split in half, a 2 s gap between the halves: its own
  // stretch of the empty source past the end, 2 s being a quarter of 8.
  let mut timeline = edit(1, 0.5);
  timeline.next_segment_id = 3;
  timeline.segments.insert(
    1,
    RecordingTimelineSegment {
      id: 2,
      source_end: 3.25,
      source_start: 3.0,
      playback_rate: 1.0,
      background: None,
    },
  );
  assert!(validate(&timeline).is_ok());
  let plan = TimelinePlan::from_edit(&timeline, 8_000).unwrap();
  assert_eq!(plan.duration_ms(), 10_000);
  let gap = &plan.gap_scenes()[0];
  assert_eq!(
    (plan.gap_scenes().len(), gap.start_ms, gap.end_ms),
    (1, 4_000, 6_000)
  );
  assert_eq!(plan.source_to_output_us(24_500_000), Some(4_500_000));
  assert_eq!(plan.source_to_output_us(5_000_000), Some(7_000_000));
  // A gap plays at speed one, and a timeline of gaps alone plays nothing
  // of the recording.
  let mut hurried = timeline.clone();
  hurried.segments[1].playback_rate = 2.0;
  assert!(validate(&hurried).is_err());
  let mut only_gap = timeline;
  only_gap.segments.retain(RecordingTimelineSegment::is_gap);
  assert!(validate(&only_gap).is_err());
}

#[test]
fn export_plan_coalesces_cuts_and_maps_retained_source_time() {
  let mut timeline = edit(1, 0.25);
  timeline.next_segment_id = 4;
  timeline.segments.push(RecordingTimelineSegment {
    id: 3,
    source_end: 1.0,
    source_start: 0.75,
    playback_rate: 1.0,
    background: None,
  });
  timeline.segments[1].source_end = 0.5;

  let plan = TimelinePlan::from_edit(&timeline, 8_000).unwrap();
  assert_eq!(plan.duration_ms(), 6_000);
  assert_eq!(plan.ranges().len(), 2);
  assert_eq!(plan.source_to_output_us(3_000_000), Some(3_000_000));
  assert_eq!(plan.source_to_output_us(5_000_000), None);
  assert_eq!(plan.source_to_output_us(7_000_000), Some(5_000_000));
}

#[test]
fn a_moment_cut_away_shows_the_frame_beside_it() {
  // From 8 s, 0-2 s and 5-6 s are cut: 2-5 s plays from 0, 6-8 s from 3 s.
  let mut timeline = edit(1, 0.25);
  timeline.next_segment_id = 4;
  timeline.segments.remove(0);
  timeline.segments[0].source_end = 0.625;
  timeline.segments.push(RecordingTimelineSegment {
    id: 3,
    source_end: 1.0,
    source_start: 0.75,
    playback_rate: 1.0,
    background: None,
  });
  let plan = TimelinePlan::from_edit(&timeline, 8_000).unwrap();
  assert_eq!(plan.output_frame_at_us(0), 0);
  assert_eq!(plan.output_frame_at_us(3_000_000), 1_000_000);
  assert_eq!(plan.output_frame_at_us(5_500_000), 3_000_000);

  // With 4-8 s cut, a playhead in the cut shows the last frame kept.
  let mut end_cut = edit(1, 0.5);
  end_cut.segments.truncate(1);
  let plan = TimelinePlan::from_edit(&end_cut, 8_000).unwrap();
  assert_eq!(plan.output_frame_at_us(6_000_000), 3_999_999);
}

#[test]
fn export_plan_ignores_cut_only_timelines() {
  assert!(TimelinePlan::from_edit(&edit(1, 0.5), 8_000).is_none());
}

#[test]
fn reordered_segments_are_valid_while_no_source_plays_twice() {
  let mut reordered = edit(1, 0.5);
  reordered.segments.swap(0, 1);
  assert!(validate(&reordered).is_ok());
  reordered.segments[1].source_end = 0.6;
  assert!(validate(&reordered).is_err());
}

#[test]
fn export_plan_plays_reordered_segments_in_list_order() {
  let mut reordered = edit(1, 0.25);
  reordered.segments.swap(0, 1);
  let plan = TimelinePlan::from_edit(&reordered, 8_000).unwrap();
  assert_eq!(plan.duration_ms(), 8_000);
  assert_eq!(plan.ranges().len(), 2);
  assert_eq!(plan.ranges()[0].source_start_us, 2_000_000);
  assert_eq!(plan.source_to_output_us(1_000_000), Some(7_000_000));
  assert_eq!(plan.source_to_output_us(2_000_000), Some(0));
}

#[test]
fn edits_this_build_cannot_read_are_absent() {
  let persisted = |version| PersistedTimelineEdit {
    edit: edit(1, 0.5),
    revision: 1,
    version,
  };
  assert!(persisted(2).into_current(8_000).is_some());
  assert!(persisted(FORMAT_VERSION).into_current(8_000).is_some());
  assert!(persisted(1).into_current(8_000).is_none());
  assert!(persisted(FORMAT_VERSION + 1).into_current(8_000).is_none());
}

#[test]
fn export_plan_scales_ranges_by_playback_rate_without_coalescing_different_rates() {
  let mut timeline = edit(1, 0.5);
  timeline.segments[0].playback_rate = 2.0;
  timeline.segments[1].playback_rate = 0.5;
  let plan = TimelinePlan::from_edit(&timeline, 8_000).unwrap();
  assert_eq!(plan.duration_ms(), 10_000);
  assert_eq!(plan.ranges().len(), 2);
  assert_eq!(plan.ranges()[0].output_start_us, 0);
  assert_eq!(plan.ranges()[1].output_start_us, 2_000_000);
  assert_eq!(plan.source_to_output_us(6_000_000), Some(6_000_000));
}

#[test]
fn fixed_output_duration_maps_back_through_playback_rate() {
  let fast = [TimelineRange {
    output_start_us: 0,
    source_end_us: 5_000_000,
    source_start_us: 0,
    playback_rate: 2.0,
  }];
  let slow = [TimelineRange {
    playback_rate: 0.5,
    ..fast[0]
  }];

  assert_eq!(
    source_after_output_duration_us(Some(&fast), 2_000_000, 400_000),
    Some(2_800_000)
  );
  assert_eq!(
    source_after_output_duration_us(Some(&slow), 2_000_000, 400_000),
    Some(2_200_000)
  );
}

#[test]
fn fixed_output_duration_crosses_speed_boundaries() {
  let ranges = [
    TimelineRange {
      output_start_us: 0,
      source_end_us: 2_200_000,
      source_start_us: 0,
      playback_rate: 2.0,
    },
    TimelineRange {
      output_start_us: 1_100_000,
      source_end_us: 5_000_000,
      source_start_us: 2_200_000,
      playback_rate: 0.5,
    },
  ];

  assert_eq!(
    source_after_output_duration_us(Some(&ranges), 2_050_000, 400_000),
    Some(2_362_500)
  );
}
