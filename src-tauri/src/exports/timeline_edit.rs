// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const FORMAT_VERSION: u16 = 1;
const MAX_SEGMENTS: usize = 100_000;

#[path = "timeline_edit_keyboard.rs"]
mod keyboard;
pub use keyboard::{KeyboardShortcutPositionRange, RecordingTimelineKeyboardDeletions};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingTimelineSegment {
  pub id: u64,
  pub source_end: f64,
  pub source_start: f64,
  #[serde(default = "default_playback_rate")]
  pub playback_rate: f64,
}

const fn default_playback_rate() -> f64 {
  1.0
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletedKeyboardShortcutFragment {
  pub segment_id: u64,
  pub shortcut_id: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletedKeyboardShortcutRange {
  pub end_ms: u64,
  pub shortcut_id: u64,
  pub start_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingTimelineEdit {
  pub artifact_id: u64,
  #[serde(flatten)]
  pub keyboard_deletions: Box<RecordingTimelineKeyboardDeletions>,
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

pub(crate) fn source_to_output_us(ranges: &[TimelineRange], source_us: u64) -> Option<u64> {
  let range = ranges.iter().find(|range| {
    source_us >= range.source_start_us
      && (source_us < range.source_end_us
        || source_us == range.source_end_us && range.source_end_us == range.source_start_us)
  })?;
  Some(range.output_start_us.saturating_add(
    ((source_us.saturating_sub(range.source_start_us)) as f64 / range.playback_rate).round() as u64,
  ))
}

fn output_to_source_us(ranges: &[TimelineRange], output_us: u64) -> Option<u64> {
  for range in ranges {
    let output_end_us = range.output_start_us.saturating_add(
      ((range.source_end_us.saturating_sub(range.source_start_us)) as f64 / range.playback_rate)
        .round() as u64,
    );
    if output_us <= output_end_us {
      return Some(range.source_start_us.saturating_add(
        ((output_us.saturating_sub(range.output_start_us)) as f64 * range.playback_rate).round()
          as u64,
      ));
    }
  }
  ranges.last().map(|range| range.source_end_us)
}

/// Converts a source-time animation anchor plus an output-time duration back
/// into a source coordinate. Timed lanes can map that coordinate normally and
/// still match animations whose duration must not stretch with playback rate.
pub(crate) fn source_after_output_duration_us(
  ranges: Option<&[TimelineRange]>,
  anchor_us: u64,
  duration_us: u64,
) -> Option<u64> {
  let Some(ranges) = ranges else {
    return Some(anchor_us.saturating_add(duration_us));
  };
  let output_anchor_us = source_to_output_us(ranges, anchor_us)?;
  output_to_source_us(ranges, output_anchor_us.saturating_add(duration_us))
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelinePlan {
  deleted_keyboard_shortcut_ids: Vec<u64>,
  deleted_keyboard_shortcut_ranges: Vec<DeletedKeyboardShortcutRange>,
  keyboard_shortcut_positions: Vec<KeyboardShortcutPositionRange>,
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
      ranges,
    };
    (!plan.is_identity(source_duration_us)
      || !plan.deleted_keyboard_shortcut_ids.is_empty()
      || !plan.deleted_keyboard_shortcut_ranges.is_empty()
      || !plan.keyboard_shortcut_positions.is_empty())
    .then_some(plan)
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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedTimelineEdit {
  edit: RecordingTimelineEdit,
  revision: u64,
  version: u16,
}

fn sidecar_path(recording: &Path, slot: char) -> Option<PathBuf> {
  let stem = recording.file_stem()?.to_str()?;
  Some(recording.with_file_name(format!("{stem}.timeline-edit-{slot}.json")))
}

fn validate(edit: &RecordingTimelineEdit) -> Result<(), String> {
  if edit.segments.is_empty() || edit.segments.len() > MAX_SEGMENTS {
    return Err("The timeline must contain a reasonable number of segments".to_owned());
  }
  let mut previous_end = 0.0;
  let mut ids = std::collections::HashSet::with_capacity(edit.segments.len());
  for segment in &edit.segments {
    if !segment.source_start.is_finite()
      || !segment.source_end.is_finite()
      || segment.source_start < previous_end
      || segment.source_start < 0.0
      || segment.source_end <= segment.source_start
      || segment.source_end > 1.0
      || !segment.playback_rate.is_finite()
      || !(0.25..=4.0).contains(&segment.playback_rate)
      || !ids.insert(segment.id)
    {
      return Err("The timeline contains an invalid segment".to_owned());
    }
    previous_end = segment.source_end;
  }
  if edit.next_segment_id
    <= edit
      .segments
      .iter()
      .map(|segment| segment.id)
      .max()
      .unwrap_or(0)
  {
    return Err("The timeline's next segment identity is invalid".to_owned());
  }
  keyboard::validate(&edit.keyboard_deletions, MAX_SEGMENTS)?;
  Ok(())
}

fn read_slot(recording: &Path, slot: char) -> Option<PersistedTimelineEdit> {
  let bytes = std::fs::read(sidecar_path(recording, slot)?).ok()?;
  let persisted: PersistedTimelineEdit = serde_json::from_slice(&bytes).ok()?;
  (persisted.version == FORMAT_VERSION && validate(&persisted.edit).is_ok()).then_some(persisted)
}

pub fn for_recording(recording: &Path, artifact_id: u64) -> Option<(u64, RecordingTimelineEdit)> {
  let mut persisted = ['a', 'b']
    .into_iter()
    .filter_map(|slot| read_slot(recording, slot))
    .max_by_key(|candidate| candidate.revision)?;
  persisted.edit.artifact_id = artifact_id;
  Some((persisted.revision, persisted.edit))
}

pub fn snapshot_fields(
  recording: &Path,
  artifact_id: u64,
) -> (Option<u64>, Option<RecordingTimelineEdit>) {
  for_recording(recording, artifact_id).map_or((None, None), |(revision, edit)| {
    (Some(revision), Some(edit))
  })
}

pub fn persist(
  recording: &Path,
  artifact_id: u64,
  revision: u64,
  edit: RecordingTimelineEdit,
) -> Result<(), String> {
  if edit.artifact_id != artifact_id {
    return Err("That timeline belongs to another recording".to_owned());
  }
  validate(&edit)?;
  if for_recording(recording, artifact_id).is_some_and(|(current, _)| current >= revision) {
    return Ok(());
  }

  let slot = if revision.is_multiple_of(2) { 'a' } else { 'b' };
  let target = sidecar_path(recording, slot)
    .ok_or_else(|| "The recording has no valid timeline sidecar name".to_owned())?;
  let temporary = target.with_extension("json.tmp");
  let bytes = serde_json::to_vec(&PersistedTimelineEdit {
    edit,
    revision,
    version: FORMAT_VERSION,
  })
  .map_err(|error| error.to_string())?;
  let mut file = File::create(&temporary).map_err(|error| error.to_string())?;
  file.write_all(&bytes).map_err(|error| error.to_string())?;
  file.sync_all().map_err(|error| error.to_string())?;
  drop(file);
  if target.exists() {
    std::fs::remove_file(&target).map_err(|error| error.to_string())?;
  }
  std::fs::rename(&temporary, target).map_err(|error| error.to_string())
}

pub fn remove_for_recording(recording: &Path) {
  for slot in ['a', 'b'] {
    if let Some(path) = sidecar_path(recording, slot) {
      let _ = std::fs::remove_file(&path);
      let _ = std::fs::remove_file(path.with_extension("json.tmp"));
    }
  }
}

pub fn sweep_unclaimed(directory: &Path, keep: Option<&Path>) {
  let keep_stem = keep
    .and_then(Path::file_stem)
    .and_then(|stem| stem.to_str());
  let Ok(entries) = std::fs::read_dir(directory) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
      continue;
    };
    let is_timeline =
      name.contains(".timeline-edit-") && (name.ends_with(".json") || name.ends_with(".json.tmp"));
    if is_timeline && !keep_stem.is_some_and(|stem| name.starts_with(&format!("{stem}."))) {
      let _ = std::fs::remove_file(path);
    }
  }
}

#[cfg(test)]
#[path = "timeline_edit_tests.rs"]
mod tests;
