// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The first edit a freshly captured recording is handed.

use super::*;

/// Writes the first edit a recording has ever had, carrying the marks that
/// were drawn live while it was being captured. The editor loads this as the
/// recording's own timeline, so those marks arrive already editable.
///
/// The timeline itself is the untouched whole: one segment at full rate.
pub(in crate::editor) fn persist_initial_annotation_clips(
  recording: &Path,
  artifact_id: u64,
  annotation_clips: Vec<crate::editor::annotations::timing::RecordingAnnotationClip>,
) -> Result<(), String> {
  persist(
    recording,
    artifact_id,
    0,
    RecordingTimelineEdit {
      annotation_clips,
      artifact_id,
      keyboard_deletions: Box::default(),
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
