// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The plan an export or the preview follows, made from an edit: the ranges
//! of the recording, and the gaps past its end, the timeline plays, and what
//! is laid over and under them.

use screenwide_scenes::GapScene;
use screenwide_timeline::{output_at_us, source_to_output_us, TimelineRange};

use super::{keyboard, validate, DeletedKeyboardShortcutRange, RecordingTimelineEdit};

#[derive(Clone, Debug, PartialEq)]
pub struct TimelinePlan {
  annotation_clips: Vec<screenwide_annotations::timing::RecordingAnnotationClip>,
  deleted_keyboard_shortcut_ids: Vec<u64>,
  deleted_keyboard_shortcut_ranges: Vec<DeletedKeyboardShortcutRange>,
  /// Read by the export, which arranges each frame as they have it.
  pub scene_clips: Vec<screenwide_scenes::RecordingSceneClip>,
  /// Where the gaps play, each drawn as a blank scene of its own.
  gap_scenes: Vec<GapScene>,
  /// The songs and sound effects, with their files once the app has found
  /// them ([`Self::find_sounds`]).
  pub(crate) sound_track: screenwide_sound_track::SoundTrack,
  duration_us: u64,
  source_duration_us: u64,
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
      // A gap never runs on into the recording, nor the recording into one.
      if ranges.last().is_some_and(|range| {
        range.source_end_us == source_start_us
          && range.playback_rate == segment.playback_rate
          && range.is_gap(source_duration_us) == segment.is_gap()
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
      gap_scenes: gap_scenes(edit, source_duration_us),
      annotation_clips: edit.annotation_clips.clone(),
      deleted_keyboard_shortcut_ids: edit.keyboard_deletions.shortcut_ids.clone(),
      deleted_keyboard_shortcut_ranges: keyboard::ranges(
        &edit.keyboard_deletions,
        &edit.segments,
        duration_ms,
      ),
      duration_us,
      source_duration_us,
      scene_clips: edit.scene_clips.clone(),
      sound_track: screenwide_sound_track::SoundTrack {
        clips: edit.audio_clips.clone(),
        muted: edit.muted_audio.clone(),
        ..Default::default()
      },
      ranges,
    };
    (!plan.annotation_clips.is_empty()
      || !plan.scene_clips.is_empty()
      || !plan.sound_track.clips.is_empty()
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

  /// How long the recording the plan edits is, in microseconds: where the
  /// gaps' empty source begins.
  pub fn source_duration_us(&self) -> u64 {
    self.source_duration_us
  }

  /// Where the gaps play on the timeline, in order, with the backgrounds of
  /// their own.
  pub fn gap_scenes(&self) -> &[GapScene] {
    &self.gap_scenes
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

  /// Where the frame shown for the source moment `source_us` sits in the
  /// output. A moment the edit cuts away, such as a playhead resting at 0
  /// in a recording whose start is cut, shows the frame restoring it would
  /// sit beside; past the last kept moment, the last frame.
  pub fn output_frame_at_us(&self, source_us: u64) -> u64 {
    self.source_to_output_us(source_us).unwrap_or_else(|| {
      let landed = output_at_us(&self.ranges, source_us as f64).round() as u64;
      landed.min(self.duration_us.saturating_sub(1))
    })
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

/// The gaps `edit` plays on a recording `source_duration_us` long, in order.
/// Two that meet are two blank scenes, which hand over with nothing to see.
fn gap_scenes(edit: &RecordingTimelineEdit, source_duration_us: u64) -> Vec<GapScene> {
  let mut output_us = 0.0_f64;
  edit
    .segments
    .iter()
    .filter_map(|segment| {
      let start_ms = (output_us / 1_000.0).round() as u64;
      output_us += (segment.source_end - segment.source_start) * source_duration_us as f64
        / segment.playback_rate;
      segment.is_gap().then(|| GapScene {
        start_ms,
        end_ms: (output_us / 1_000.0).round() as u64,
        background: segment.background.clone(),
      })
    })
    .collect()
}
