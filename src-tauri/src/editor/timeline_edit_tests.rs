// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

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
  assert!(edit.keyboard_deletions.positions.is_empty());
}

#[test]
fn shortcut_positions_map_to_their_fragment_source_range() {
  let mut timeline = edit(1, 0.25);
  timeline
    .keyboard_deletions
    .positions
    .push(keyboard::KeyboardShortcutPositionFragment {
      center_x: 0.25,
      center_y: 0.75,
      segment_id: 1,
      shortcut_id: 7,
      size_percent: Some(125.0),
    });
  let plan = TimelinePlan::from_edit(&timeline, 8_000).unwrap();
  assert_eq!(
    plan.keyboard_shortcut_positions(),
    &[KeyboardShortcutPositionRange {
      center_x: 0.25,
      center_y: 0.75,
      end_ms: 8_000,
      shortcut_id: 7,
      start_ms: 2_000,
      size_percent: Some(125.0),
    }]
  );
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
    artifact_id,
    keyboard_deletions: Box::default(),
    next_segment_id: 2,
    segments: vec![
      RecordingTimelineSegment {
        id: 0,
        source_end: split,
        source_start: 0.0,
        playback_rate: 1.0,
      },
      RecordingTimelineSegment {
        id: 1,
        source_end: 1.0,
        source_start: split,
        playback_rate: 1.0,
      },
    ],
  }
}

#[test]
fn restores_the_newest_valid_slot_and_rebinds_the_artifact() {
  let directory =
    std::env::temp_dir().join(format!("screenwide-timeline-edit-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  let recording = directory.join("recording-test.mov");
  std::fs::write(&recording, []).unwrap();

  persist(&recording, 7, 1, edit(7, 0.25)).unwrap();
  persist(&recording, 7, 2, edit(7, 0.75)).unwrap();
  persist(&recording, 7, 1, edit(7, 0.5)).unwrap();
  let (revision, restored) = for_recording(&recording, 99).unwrap();
  assert_eq!(revision, 2);
  assert_eq!(restored.artifact_id, 99);
  assert_eq!(restored.segments[0].source_end, 0.75);

  std::fs::write(sidecar_path(&recording, 'a').unwrap(), b"truncated").unwrap();
  let (revision, restored) = for_recording(&recording, 99).unwrap();
  assert_eq!(revision, 1);
  assert_eq!(restored.segments[0].source_end, 0.25);
  let _ = std::fs::remove_dir_all(directory);
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
fn export_plan_coalesces_cuts_and_maps_retained_source_time() {
  let mut timeline = edit(1, 0.25);
  timeline.next_segment_id = 4;
  timeline.segments.push(RecordingTimelineSegment {
    id: 3,
    source_end: 1.0,
    source_start: 0.75,
    playback_rate: 1.0,
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
fn export_plan_ignores_cut_only_timelines() {
  assert!(TimelinePlan::from_edit(&edit(1, 0.5), 8_000).is_none());
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
