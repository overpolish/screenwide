// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod time_mapping;
pub(crate) use time_mapping::{output_at_us, rate_at, source_to_output_us};
mod validation;
use validation::validate;
mod initial_edit;
pub(in crate::editor) use initial_edit::persist_initial_edit;
mod stacking;

pub(crate) use time_mapping::source_after_output_duration_us;
pub(crate) use time_mapping::source_before_output_duration_us;

use std::path::Path;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Annotations are drawn in document order.
const FORMAT_VERSION: u16 = 2;
const MAX_SEGMENTS: usize = 100_000;

mod keyboard;
pub use keyboard::{KeyboardShortcutPositionRange, RecordingTimelineKeyboardDeletions};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingTimelineSegment {
  pub id: u64,
  pub source_end: f64,
  pub source_start: f64,
  #[serde(default = "default_playback_rate")]
  #[ts(as = "Option<f64>", optional)]
  pub playback_rate: f64,
}

const fn default_playback_rate() -> f64 {
  1.0
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeletedKeyboardShortcutFragment {
  pub segment_id: u64,
  pub shortcut_id: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeletedKeyboardShortcutRange {
  pub end_ms: u64,
  pub shortcut_id: u64,
  pub start_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingTimelineEdit {
  pub artifact_id: u64,
  #[serde(default)]
  #[ts(
    as = "Option<Vec<super::annotations::timing::RecordingAnnotationClip>>",
    optional
  )]
  pub annotation_clips: Vec<super::annotations::timing::RecordingAnnotationClip>,
  #[serde(flatten)]
  pub keyboard_deletions: Box<RecordingTimelineKeyboardDeletions>,
  /// Sorted by start, never overlapping; see [`super::scenes`].
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub scene_clips: Vec<super::scenes::RecordingSceneClip>,
  pub next_segment_id: u64,
  pub segments: Vec<RecordingTimelineSegment>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineRange {
  pub output_start_us: u64,
  pub source_end_us: u64,
  pub source_start_us: u64,
  pub playback_rate: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelinePlan {
  annotation_clips: Vec<super::annotations::timing::RecordingAnnotationClip>,
  deleted_keyboard_shortcut_ids: Vec<u64>,
  deleted_keyboard_shortcut_ranges: Vec<DeletedKeyboardShortcutRange>,
  keyboard_shortcut_positions: Vec<KeyboardShortcutPositionRange>,
  /// Read by the export, which arranges each frame as they have it.
  pub(crate) scene_clips: Vec<super::scenes::RecordingSceneClip>,
  duration_us: u64,
  ranges: Vec<TimelineRange>,
}

impl TimelinePlan {
  pub fn from_edit(edit: &RecordingTimelineEdit, duration_ms: u64) -> Option<Self> {
    if duration_ms == 0 || validate(edit).is_err() {
      return None;
    }
    let source_duration_us = duration_ms.saturating_mul(1_000);
    let mut ranges: Vec<TimelineRange> = Vec::new();
    for segment in &edit.segments {
      let source_start_us = (segment.source_start * source_duration_us as f64).round() as u64;
      let source_end_us = (segment.source_end * source_duration_us as f64).round() as u64;
      if ranges.last().is_some_and(|range| {
        range.source_end_us == source_start_us && range.playback_rate == segment.playback_rate
      }) {
        ranges.last_mut()?.source_end_us = source_end_us;
        continue;
      }
      let output_start_us = ranges.last().map_or(0, |range| {
        range.output_start_us.saturating_add(
          ((range.source_end_us.saturating_sub(range.source_start_us)) as f64 / range.playback_rate)
            .round() as u64,
        )
      });
      ranges.push(TimelineRange {
        output_start_us,
        source_end_us,
        source_start_us,
        playback_rate: segment.playback_rate,
      });
    }
    let duration_us = ranges.last().map_or(0, |range| {
      range.output_start_us.saturating_add(
        ((range.source_end_us.saturating_sub(range.source_start_us)) as f64 / range.playback_rate)
          .round() as u64,
      )
    });
    let plan = Self {
      annotation_clips: edit.annotation_clips.clone(),
      deleted_keyboard_shortcut_ids: edit.keyboard_deletions.shortcut_ids.clone(),
      deleted_keyboard_shortcut_ranges: keyboard::ranges(
        &edit.keyboard_deletions,
        &edit.segments,
        duration_ms,
      ),
      keyboard_shortcut_positions: keyboard::position_ranges(
        &edit.keyboard_deletions,
        &edit.segments,
        duration_ms,
      ),
      duration_us,
      scene_clips: edit.scene_clips.clone(),
      ranges,
    };
    (!plan.annotation_clips.is_empty()
      || !plan.scene_clips.is_empty()
      || !plan.is_identity(source_duration_us)
      || !plan.deleted_keyboard_shortcut_ids.is_empty()
      || !plan.deleted_keyboard_shortcut_ranges.is_empty()
      || !plan.keyboard_shortcut_positions.is_empty())
    .then_some(plan)
  }

  pub(crate) fn annotation_clips(&self) -> &[super::annotations::timing::RecordingAnnotationClip] {
    &self.annotation_clips
  }

  pub fn duration_ms(&self) -> u64 {
    self.duration_us.div_ceil(1_000)
  }

  pub fn ranges(&self) -> &[TimelineRange] {
    &self.ranges
  }

  pub fn deleted_keyboard_shortcut_ids(&self) -> &[u64] {
    &self.deleted_keyboard_shortcut_ids
  }

  pub fn deleted_keyboard_shortcut_ranges(&self) -> &[DeletedKeyboardShortcutRange] {
    &self.deleted_keyboard_shortcut_ranges
  }

  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub fn source_to_output_us(&self, source_us: u64) -> Option<u64> {
    source_to_output_us(&self.ranges, source_us)
  }

  fn is_identity(&self, source_duration_us: u64) -> bool {
    self.ranges.as_slice()
      == [TimelineRange {
        output_start_us: 0,
        source_end_us: source_duration_us,
        source_start_us: 0,
        playback_rate: 1.0,
      }]
  }
}

/// The edit as a project keeps it. The revision orders saves that can arrive
/// out of turn, so a slow older save never lands over a newer one.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PersistedTimelineEdit {
  edit: RecordingTimelineEdit,
  revision: u64,
  version: u16,
}

/// The project's edit and its revision, bound to `artifact_id`. An edit this
/// version cannot read, or one that fails validation, is treated as absent.
pub fn for_project(project: &Path, artifact_id: u64) -> Option<(u64, RecordingTimelineEdit)> {
  let PersistedTimelineEdit {
    mut edit,
    revision,
    version,
  } = crate::project::read(project).ok()?.timeline?;
  if version != FORMAT_VERSION || validate(&edit).is_err() {
    return None;
  }
  edit.artifact_id = artifact_id;
  Some((revision, edit))
}

pub fn snapshot_fields(
  project: &Path,
  artifact_id: u64,
) -> (Option<u64>, Option<RecordingTimelineEdit>) {
  for_project(project, artifact_id).map_or((None, None), |(revision, edit)| {
    (Some(revision), Some(edit))
  })
}

/// The plan an export or its estimate follows: the edit the window sent, if
/// it is this recording's, otherwise the one saved in the project.
pub fn export_plan(
  sent: Option<&RecordingTimelineEdit>,
  project: &Path,
  artifact_id: u64,
  duration_ms: u64,
) -> Option<TimelinePlan> {
  match sent.filter(|edit| edit.artifact_id == artifact_id) {
    Some(edit) => TimelinePlan::from_edit(edit, duration_ms),
    None => for_project(project, artifact_id)
      .and_then(|(_, edit)| TimelinePlan::from_edit(&edit, duration_ms)),
  }
}

pub fn persist(
  project: &Path,
  artifact_id: u64,
  revision: u64,
  edit: RecordingTimelineEdit,
) -> Result<(), String> {
  if edit.artifact_id != artifact_id {
    return Err("That timeline belongs to another recording".to_owned());
  }
  validate(&edit)?;
  crate::project::update(project, |manifest| {
    if manifest
      .timeline
      .as_ref()
      .is_some_and(|current| current.revision >= revision)
    {
      return false;
    }
    manifest.timeline = Some(PersistedTimelineEdit {
      edit,
      revision,
      version: FORMAT_VERSION,
    });
    true
  })
}

#[cfg(test)]
mod tests;
