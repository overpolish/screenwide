// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The first edit a freshly captured recording is handed.

use super::*;
use screenwide_timeline_edit::RecordingTimelineSegment;

/// Writes the first edit a recording has ever had, carrying the annotations
/// that were drawn live while it was being captured and the auto zooms made
/// from it. The editor loads this as the recording's own timeline, so both
/// arrive already editable, the annotations stacked the way the live overlay
/// drew them. A project that already has an edit keeps it.
///
/// The timeline itself is the untouched whole: one segment at full rate.
pub(in crate::editor) fn persist_initial_edit(
  project: &Path,
  artifact_id: u64,
  mut annotation_clips: Vec<screenwide_annotations::timing::RecordingAnnotationClip>,
  scene_clips: Vec<screenwide_scenes::RecordingSceneClip>,
) -> Result<(), String> {
  screenwide_timeline_edit::stacking::stack_by_kind(&mut annotation_clips);
  persist(
    project,
    artifact_id,
    0,
    RecordingTimelineEdit {
      annotation_clips,
      scene_clips,
      ..untouched(artifact_id)
    },
  )
}

/// The edit of a recording nothing has been done to: the whole of it, once,
/// at full rate.
pub(in crate::editor) fn untouched(artifact_id: u64) -> RecordingTimelineEdit {
  RecordingTimelineEdit {
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
    next_segment_id: 1,
    segments: vec![RecordingTimelineSegment {
      id: 0,
      playback_rate: 1.0,
      source_end: 1.0,
      source_start: 0.0,
    }],
  }
}
