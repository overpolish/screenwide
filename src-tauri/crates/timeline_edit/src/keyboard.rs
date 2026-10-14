// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  DeletedKeyboardShortcutFragment, DeletedKeyboardShortcutRange, RecordingTimelineSegment,
};

#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, serde::Serialize, ts_rs::TS)]
pub struct RecordingTimelineKeyboardDeletions {
  #[serde(default, rename = "deletedKeyboardShortcutFragments")]
  #[ts(as = "Option<Vec<DeletedKeyboardShortcutFragment>>", optional)]
  pub fragments: Vec<DeletedKeyboardShortcutFragment>,
  #[serde(default, rename = "deletedKeyboardShortcutIds")]
  #[ts(as = "Option<Vec<u64>>", optional)]
  pub shortcut_ids: Vec<u64>,
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
  Ok(())
}
