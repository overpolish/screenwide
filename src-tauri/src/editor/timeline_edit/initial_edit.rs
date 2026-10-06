// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The first edit a freshly captured recording is handed.

use super::*;

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
  mut annotation_clips: Vec<crate::editor::annotations::timing::RecordingAnnotationClip>,
  scene_clips: Vec<crate::editor::scenes::RecordingSceneClip>,
) -> Result<(), String> {
  super::stacking::stack_by_kind(&mut annotation_clips);
  persist(
    project,
    artifact_id,
    0,
    RecordingTimelineEdit {
      annotation_clips,
      artifact_id,
      keyboard_deletions: Box::default(),
      scene_clips,
      next_segment_id: 1,
      segments: vec![RecordingTimelineSegment {
        id: 0,
        playback_rate: 1.0,
        source_end: 1.0,
        source_start: 0.0,
      }],
    },
  )
}
