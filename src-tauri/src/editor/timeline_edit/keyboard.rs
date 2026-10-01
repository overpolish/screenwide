// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  DeletedKeyboardShortcutFragment, DeletedKeyboardShortcutRange, RecordingTimelineSegment,
  TimelinePlan,
};

#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardShortcutPositionFragment {
  pub center_x: f64,
  pub center_y: f64,
  pub segment_id: u64,
  pub shortcut_id: u64,
  #[serde(default)]
  pub size_percent: Option<f64>,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardShortcutPositionRange {
  pub center_x: f64,
  pub center_y: f64,
  pub end_ms: u64,
  pub shortcut_id: u64,
  pub start_ms: u64,
  #[serde(default)]
  pub size_percent: Option<f64>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, serde::Serialize)]
pub struct RecordingTimelineKeyboardDeletions {
  #[serde(default, rename = "deletedKeyboardShortcutFragments")]
  pub fragments: Vec<DeletedKeyboardShortcutFragment>,
  #[serde(default, rename = "deletedKeyboardShortcutIds")]
  pub shortcut_ids: Vec<u64>,
  #[serde(default, rename = "keyboardShortcutPositions")]
  pub positions: Vec<KeyboardShortcutPositionFragment>,
}

impl TimelinePlan {
  pub fn keyboard_shortcut_positions(&self) -> &[KeyboardShortcutPositionRange] {
    &self.keyboard_shortcut_positions
  }
}

pub(super) fn ranges(
  deletions: &RecordingTimelineKeyboardDeletions,
  segments: &[RecordingTimelineSegment],
  duration_ms: u64,
) -> Vec<DeletedKeyboardShortcutRange> {
  deletions
    .fragments
    .iter()
    .filter_map(|fragment| {
      let segment = segments
        .iter()
        .find(|segment| segment.id == fragment.segment_id)?;
      Some(DeletedKeyboardShortcutRange {
        end_ms: (segment.source_end * duration_ms as f64).round() as u64,
        shortcut_id: fragment.shortcut_id,
        start_ms: (segment.source_start * duration_ms as f64).round() as u64,
      })
    })
    .collect()
}

pub(super) fn position_ranges(
  keyboard: &RecordingTimelineKeyboardDeletions,
  segments: &[RecordingTimelineSegment],
  duration_ms: u64,
) -> Vec<KeyboardShortcutPositionRange> {
  keyboard
    .positions
    .iter()
    .filter_map(|position| {
      let segment = segments
        .iter()
        .find(|segment| segment.id == position.segment_id)?;
      Some(KeyboardShortcutPositionRange {
        center_x: position.center_x,
        center_y: position.center_y,
        end_ms: (segment.source_end * duration_ms as f64).round() as u64,
        shortcut_id: position.shortcut_id,
        start_ms: (segment.source_start * duration_ms as f64).round() as u64,
        size_percent: position.size_percent,
      })
    })
    .collect()
}

pub(super) fn validate(
  deletions: &RecordingTimelineKeyboardDeletions,
  maximum: usize,
) -> Result<(), String> {
  if deletions.shortcut_ids.len() > maximum {
    return Err("The timeline contains a reasonable number of deleted shortcuts".to_owned());
  }
  let mut shortcut_ids = std::collections::HashSet::with_capacity(deletions.shortcut_ids.len());
  if deletions
    .shortcut_ids
    .iter()
    .any(|id| !shortcut_ids.insert(id))
  {
    return Err("The timeline contains duplicate deleted shortcut identities".to_owned());
  }
  if deletions.fragments.len() > maximum {
    return Err(
      "The timeline contains a reasonable number of deleted shortcut fragments".to_owned(),
    );
  }
  let mut fragments = std::collections::HashSet::with_capacity(deletions.fragments.len());
  if deletions
    .fragments
    .iter()
    .any(|fragment| !fragments.insert(*fragment))
  {
    return Err("The timeline contains duplicate deleted shortcut fragments".to_owned());
  }
  if deletions.positions.len() > maximum {
    return Err("The timeline contains a reasonable number of shortcut positions".to_owned());
  }
  let mut positions = std::collections::HashSet::with_capacity(deletions.positions.len());
  if deletions.positions.iter().any(|position| {
    !position.center_x.is_finite()
      || !position.center_y.is_finite()
      || !(0.0..=1.0).contains(&position.center_x)
      || !(0.0..=1.0).contains(&position.center_y)
      || position
        .size_percent
        .is_some_and(|size| !size.is_finite() || !(5.0..=500.0).contains(&size))
      || !positions.insert((position.shortcut_id, position.segment_id))
  }) {
    return Err("The timeline contains invalid or duplicate shortcut positions".to_owned());
  }
  Ok(())
}
