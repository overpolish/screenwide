// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording's edit: the segments its timeline keeps and their speeds, the
//! annotation and scene clips laid over it, where the camera's background is
//! taken away, the captions asked of its transcript, and what Remove
//! silences, the Transcript panel and keyboard deletions cut. A project file
//! keeps it; the plan an export or the preview follows is made from it.
//! Annotation and scene clips and background ranges are timed on the
//! timeline; the edits that change the segments re-time them as they are
//! made.

use screenwide_timeline::{source_to_output_us, TimelineRange};
mod validation;
pub use validation::validate;
mod silences;
pub mod stacking;
pub use silences::RecordingSilenceCut;
mod transcript;
pub use transcript::{RecordingTranscriptCorrection, RecordingTranscriptCut};
mod camera_backgrounds;
pub use camera_backgrounds::{CameraBackgroundRange, CameraRangeBackground};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Annotations are drawn in document order, segments play in list order,
/// which need not be source order, and annotation and scene clips are times
/// on the timeline.
pub const FORMAT_VERSION: u16 = 4;
/// Version 2 edits always kept segments in source order, which later
/// versions still read the same way.
const OLDEST_READABLE_VERSION: u16 = 2;
/// The version from which annotation and scene clips are timeline times
/// rather than recording times.
const TIMELINE_CLIPS_VERSION: u16 = 4;

mod timeline_time;

const MAX_SEGMENTS: usize = 100_000;

mod keyboard;
pub use keyboard::RecordingTimelineKeyboardDeletions;

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
    as = "Option<Vec<screenwide_annotations::timing::RecordingAnnotationClip>>",
    optional
  )]
  pub annotation_clips: Vec<screenwide_annotations::timing::RecordingAnnotationClip>,
  #[serde(flatten)]
  pub keyboard_deletions: Box<RecordingTimelineKeyboardDeletions>,
  /// Sorted by start, never overlapping; see [`super::scenes`].
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub scene_clips: Vec<screenwide_scenes::RecordingSceneClip>,
  /// Where the camera's background is taken away; sorted by start, never
  /// overlapping. See [`camera_backgrounds`].
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub camera_background_ranges: Vec<CameraBackgroundRange>,
  /// What Remove silences cut, for Restore all to bring back; see
  /// [`silences`].
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub silence_cuts: Vec<RecordingSilenceCut>,
  /// What the Transcript panel cut, for Restore all to bring back; see
  /// [`transcript`].
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub transcript_cuts: Vec<RecordingTranscriptCut>,
  /// Words the Transcript panel corrected, which change nothing that plays
  /// or exports; see [`transcript`].
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub transcript_corrections: Vec<RecordingTranscriptCorrection>,
  /// Folded text the Transcript panel was told is not profanity; marks only.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub transcript_allowed_words: Vec<String>,
  /// The captions made from the transcript, once any were asked for; see
  /// [`screenwide_captions`].
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub captions: Option<screenwide_captions::RecordingCaptions>,
  pub next_segment_id: u64,
  pub segments: Vec<RecordingTimelineSegment>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelinePlan {
  annotation_clips: Vec<screenwide_annotations::timing::RecordingAnnotationClip>,
  deleted_keyboard_shortcut_ids: Vec<u64>,
  deleted_keyboard_shortcut_ranges: Vec<DeletedKeyboardShortcutRange>,
  /// Read by the export, which arranges each frame as they have it.
  pub scene_clips: Vec<screenwide_scenes::RecordingSceneClip>,
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
      duration_us,
      scene_clips: edit.scene_clips.clone(),
      ranges,
    };
    (!plan.annotation_clips.is_empty()
      || !plan.scene_clips.is_empty()
      || !plan.is_identity(source_duration_us)
      || !plan.deleted_keyboard_shortcut_ids.is_empty()
      || !plan.deleted_keyboard_shortcut_ranges.is_empty())
    .then_some(plan)
  }

  pub fn annotation_clips(&self) -> &[screenwide_annotations::timing::RecordingAnnotationClip] {
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
pub struct PersistedTimelineEdit {
  pub edit: RecordingTimelineEdit,
  pub revision: u64,
  pub version: u16,
}

impl PersistedTimelineEdit {
  /// The edit as this build works on it, for a recording `duration_ms`
  /// long: an older one's clips carried from recording time onto the
  /// timeline. `None` for a version this build cannot read, or an edit that
  /// fails validation.
  pub fn into_current(self, duration_ms: u64) -> Option<RecordingTimelineEdit> {
    if !(OLDEST_READABLE_VERSION..=FORMAT_VERSION).contains(&self.version) {
      return None;
    }
    let mut edit = self.edit;
    if self.version < TIMELINE_CLIPS_VERSION {
      timeline_time::carry_clips_onto_timeline(&mut edit, duration_ms);
    }
    validate(&edit).ok()?;
    Some(edit)
  }
}

#[cfg(test)]
mod tests;
