// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording's edit: the segments its timeline keeps and their speeds, the
//! annotation and scene clips laid over it, the songs and sound effects
//! laid under it, where the camera's background is taken away, the captions
//! asked of its transcript, and what Remove silences, the Transcript panel
//! and keyboard deletions cut. A project file keeps it; the plan an export
//! or the preview follows is made from it. Annotation, scene and audio clips
//! and background ranges are timed on the timeline; the edits that change
//! the segments re-time the annotation and scene clips as they are made,
//! and leave the audio clips where they are.

mod plan;
pub use plan::TimelinePlan;
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
/// which need not be source order, annotation and scene clips are times on
/// the timeline, audio clips may be laid under it, and segments past the
/// recording's end are gaps ([`RecordingTimelineSegment::is_gap`]).
pub const FORMAT_VERSION: u16 = 6;
/// Version 2 edits always kept segments in source order, which later
/// versions still read the same way.
const OLDEST_READABLE_VERSION: u16 = 2;
/// The version from which annotation and scene clips are timeline times
/// rather than recording times.
const TIMELINE_CLIPS_VERSION: u16 = 4;

mod sound;
mod timeline_time;

const MAX_SEGMENTS: usize = 100_000;

mod keyboard;
pub use keyboard::RecordingTimelineKeyboardDeletions;

/// A stretch the timeline plays: of the recording, as shares of its length,
/// at a speed of its own; or, past its end (a source of one or more), a gap,
/// time with nothing recorded behind it, always at speed one, which may show
/// a background of its own. Each gap keeps a stretch of that empty source of
/// its own, so no two ever meet.
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
  /// A gap's own background, over the recording's; never set on a stretch
  /// of the recording.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub background: Option<screenwide_background::Background>,
}

impl RecordingTimelineSegment {
  /// Whether the segment is a gap rather than a stretch of the recording.
  pub fn is_gap(&self) -> bool {
    self.source_start >= 1.0
  }
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
  /// Songs and sound effects, timed on the timeline; see
  /// [`screenwide_sound_track`].
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  #[ts(
    optional,
    as = "Option<Vec<screenwide_sound_track::RecordingAudioClip>>"
  )]
  pub audio_clips: Vec<screenwide_sound_track::RecordingAudioClip>,
  /// The kinds of audio clip left out of the preview and the export.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  #[ts(optional, as = "Option<Vec<screenwide_sound_track::AudioClipKind>>")]
  pub muted_audio: Vec<screenwide_sound_track::AudioClipKind>,
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
  /// [`screenwide_text::captions`].
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub captions: Option<screenwide_text::captions::RecordingCaptions>,
  /// Where each lull kept as recorded, out of the squeeze, starts, as a
  /// share of the recording. A lull's speed alone cannot say it is kept
  /// while nothing is squeezed, since every lull then plays as recorded;
  /// the strip finds the lull each point falls in. Plays and exports as
  /// nothing.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  #[ts(optional, as = "Option<Vec<f64>>")]
  pub kept_lulls: Vec<f64>,
  pub next_segment_id: u64,
  pub segments: Vec<RecordingTimelineSegment>,
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
