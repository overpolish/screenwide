// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "timeline/labels.rs"]
mod labels;
use labels::shortcut_label;

use super::{
  state::TransitionKind, state::VisualKey, state::EXIT_US, KeyboardCompositor, KeyboardOverlay,
  Shortcut, HOLD_US,
};
use crate::editor::timeline_edit::{
  source_after_output_duration_us, DeletedKeyboardShortcutRange, KeyboardShortcutPositionRange,
  TimelineRange,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KeyboardTimelineItem {
  pub id: u64,
  pub start_ms: u64,
  pub end_ms: u64,
  pub label: String,
}

impl KeyboardCompositor {
  pub(crate) fn set_deleted_shortcuts(&self, ids: &[u64], ranges: &[DeletedKeyboardShortcutRange]) {
    {
      let mut deleted = self
        .deleted_shortcut_ids
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      deleted.clear();
      deleted.extend(ids.iter().copied());
      *self
        .deleted_shortcut_ranges
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = ranges.to_vec();
    }
    self.rebake();
  }

  pub(crate) fn set_shortcut_positions(&self, positions: &[KeyboardShortcutPositionRange]) {
    {
      *self
        .shortcut_positions
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = positions.to_vec();
    }
    self.rebake();
  }

  pub(super) fn apply_shortcut_position(
    &self,
    overlay: &mut KeyboardOverlay,
    visible: &[&VisualKey],
    position_ms: u64,
  ) {
    let position = visible
      .iter()
      .max_by_key(|visual| visual.enter_us)
      .map(|visual| visual.source_shortcut as u64)
      .and_then(|shortcut_id| {
        self
          .shortcut_positions
          .read()
          .unwrap_or_else(|poisoned| poisoned.into_inner())
          .iter()
          .find(|position| {
            position.shortcut_id == shortcut_id
              && position_ms >= position.start_ms
              && position_ms < position.end_ms
          })
          .map(|position| {
            (
              position.center_x as f32,
              position.center_y as f32,
              position.size_percent.map(|size| (size / 100.0) as f32),
            )
          })
      });
    if let Some((center_x, center_y, scale)) = position {
      overlay.center_x = center_x;
      overlay.center_y = center_y;
      if let Some(scale) = scale {
        overlay.scale = scale;
        overlay.requested_scale = scale;
      }
    }
  }

  /// Returns captured shortcut groups in reconstruction order, using the same
  /// parsed data consumed by preview and export.
  #[cfg(test)]
  pub(crate) fn timeline_items(&self) -> Vec<KeyboardTimelineItem> {
    self.timeline_items_with_timeline(None)
  }

  /// Returns lane bounds in source coordinates, adjusted so animation-only
  /// tails occupy their fixed duration on the edited output timeline. The
  /// frontend can therefore use its normal source-to-output lane mapping while
  /// matching the compositor at every playback rate.
  pub(crate) fn timeline_items_with_timeline(
    &self,
    ranges: Option<&[TimelineRange]>,
  ) -> Vec<KeyboardTimelineItem> {
    self.set_animation_timeline(ranges);
    let baked = self
      .baked
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut items: Vec<KeyboardTimelineItem> = self
      .shortcuts
      .iter()
      .enumerate()
      .filter(|(index, _)| {
        !self
          .deleted_shortcut_ids
          .read()
          .unwrap_or_else(|poisoned| poisoned.into_inner())
          .contains(&(*index as u64))
      })
      .filter_map(|(index, shortcut)| {
        let start_us = shortcut.keys.iter().map(|key| key.down_us).min()?;
        let last_key_us = shortcut
          .keys
          .iter()
          .map(|key| key.up_us.unwrap_or(key.down_us))
          .max()
          .unwrap_or(start_us);
        // Released keys remain visible through the compositor's exit
        // lifetime, so the lane ends when the artwork is actually gone
        // rather than at the physical key-up event. Replacements and
        // detachments are the exception: they hand this key's place to the
        // continuing badge at the exit moment, so the morph and pop tails -
        // and any layout reflow that follows them - belong to the
        // successor's clip and the lane ends at the handover.
        let end_us = baked
          .timeline
          .visuals
          .iter()
          .filter(|visual| visual.source_shortcut == index)
          .filter_map(|visual| {
            visual.exit.and_then(|(exit_us, kind)| match kind {
              TransitionKind::Replacement | TransitionKind::Detached => {
                source_after_output_duration_us(ranges, exit_us, 0)
              }
              _ => source_after_output_duration_us(ranges, exit_us, EXIT_US),
            })
          })
          .max()
          .unwrap_or_else(|| {
            source_after_output_duration_us(ranges, last_key_us.saturating_add(HOLD_US), EXIT_US)
              .unwrap_or(last_key_us)
          });
        Some(KeyboardTimelineItem {
          id: index as u64,
          start_ms: start_us / 1_000,
          // Floored like start_ms: a handover end shares its instant with the
          // successor's start, and rounding it up would fabricate a 1ms
          // overlap that stacks the two clips into separate sublanes.
          end_ms: end_us / 1_000,
          label: shortcut_label(shortcut, self.legacy_modifier_expansion),
        })
      })
      .collect();
    // A successor press pulls its predecessor's fade to finish by that press
    // on the output clock. When the physical release leaves less than a full
    // fade of output time, the residue past the press is a frame at most, so
    // the lane clamps to the successor rather than overlapping it.
    for index in 1..items.len() {
      let start_ms = items[index].start_ms;
      let previous = &mut items[index - 1];
      previous.end_ms = previous.end_ms.min(start_ms).max(previous.start_ms);
    }
    items
  }
}
