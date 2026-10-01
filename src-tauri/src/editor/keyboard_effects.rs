// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Timestamped keyboard-shortcut effects shared by preview, export, and still composition.

mod evaluation;

use crate::editor::timeline_edit::{DeletedKeyboardShortcutRange, KeyboardShortcutPositionRange};
use serde_json::Value;
use std::{
  collections::HashSet,
  path::Path,
  sync::{Arc, RwLock},
};

mod animation;
use animation::{ease_out, pop_spring, replacement_enter_progress, replacement_exit_progress};
mod clock;
use clock::AnimationClock;
mod data;
use data::{parse_v1, read_values, reconstruct_v2};
mod gpu_wire;
pub(crate) use gpu_wire::{KeyboardKey, KeyboardOverlay};
use gpu_wire::{KEY_CENTER_DEFAULT, KEY_CENTER_INHERIT, MAX_KEYS};
mod geometry;
mod layout;
mod settings;
pub(crate) use settings::{KeyboardAnimation, KeyboardAppearance, KeyboardEffectSettings};
mod state;
use state::{ChainContext, KeyboardStateTimeline, TransitionKind};
mod timeline;
pub(crate) use timeline::KeyboardTimelineItem;

const ENTRANCE_SECONDS: f64 = 0.6;
const EXIT_SECONDS: f64 = 0.4;
const MICROS_PER_SECOND: f64 = 1_000_000.0;
pub(super) const HOLD_US: u64 = 750_000;

#[derive(Clone, Debug)]
pub(crate) struct KeyboardCompositor {
  /// Display state derived from the shortcuts plus the timeline edits.
  /// Deletions and manual placements decide badge continuity, so every edit
  /// rebakes this rather than being patched over a stale lifecycle.
  baked: Arc<RwLock<BakedTimeline>>,
  shortcuts: Vec<Shortcut>,
  legacy_modifier_expansion: bool,
  deleted_shortcut_ids: Arc<RwLock<HashSet<u64>>>,
  deleted_shortcut_ranges: Arc<RwLock<Vec<DeletedKeyboardShortcutRange>>>,
  shortcut_positions: Arc<RwLock<Vec<KeyboardShortcutPositionRange>>>,
  /// The retained playback ranges of the current edit. Animation durations
  /// run on the output clock, so decisions baked from them (fade pulled
  /// forward to finish by a press, the chord-roll assembly window) need the
  /// rate mapping at bake time, not only at evaluation.
  animation_ranges: Arc<RwLock<Option<Vec<crate::editor::timeline_edit::TimelineRange>>>>,
}

#[derive(Debug, Default)]
struct BakedTimeline {
  maximum_width: f64,
  timeline: KeyboardStateTimeline,
  slots: Vec<u32>,
}
#[derive(Clone, Debug)]
pub(super) struct Shortcut {
  keys: Vec<KeyPress>,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct KeyPress {
  key_code: u16,
  modifier_mask: u32,
  down_us: u64,
  up_us: Option<u64>,
}

impl KeyboardCompositor {
  pub(crate) fn open(path: &Path) -> Result<Self, String> {
    Self::open_with_deleted(path, &[], &[])
  }
  pub(crate) fn open_with_deleted(
    path: &Path,
    deleted_ids: &[u64],
    deleted_ranges: &[DeletedKeyboardShortcutRange],
  ) -> Result<Self, String> {
    let records = read_values(path)?;
    let version = records
      .first()
      .and_then(|value| value.get("version"))
      .and_then(Value::as_u64)
      .unwrap_or(1);
    let shortcuts = if version >= 2 {
      reconstruct_v2(&records)
    } else {
      parse_v1(&records)
    };
    let compositor = Self::from_shortcuts_with_legacy(shortcuts, version < 2);
    compositor.set_deleted_shortcuts(deleted_ids, deleted_ranges);
    Ok(compositor)
  }
  #[cfg(test)]
  fn from_shortcuts(shortcuts: Vec<Shortcut>) -> Self {
    Self::from_shortcuts_with_legacy(shortcuts, false)
  }
  fn from_shortcuts_with_legacy(shortcuts: Vec<Shortcut>, legacy_modifier_expansion: bool) -> Self {
    let compositor = Self {
      baked: Arc::new(RwLock::new(BakedTimeline::default())),
      shortcuts,
      legacy_modifier_expansion,
      deleted_shortcut_ids: Arc::new(RwLock::new(HashSet::new())),
      deleted_shortcut_ranges: Arc::new(RwLock::new(Vec::new())),
      shortcut_positions: Arc::new(RwLock::new(Vec::new())),
      animation_ranges: Arc::new(RwLock::new(None)),
    };
    compositor.rebake();
    compositor
  }

  /// Adopts the edit's playback ranges for bake-time animation decisions,
  /// rebaking only when they changed. Called by every evaluation entry point,
  /// so a compositor always bakes against the timeline it renders for.
  pub(super) fn set_animation_timeline(
    &self,
    ranges: Option<&[crate::editor::timeline_edit::TimelineRange]>,
  ) {
    {
      let mut known = self
        .animation_ranges
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      if known.as_deref() == ranges {
        return;
      }
      *known = ranges.map(<[_]>::to_vec);
    }
    self.rebake();
  }

  /// Rebuilds the display lifecycle from the shortcuts and the current
  /// timeline edits. Called whenever deletions or manual placements change,
  /// because they decide whether consecutive chords continue one badge.
  pub(super) fn rebake(&self) {
    let deleted_ids = self
      .deleted_shortcut_ids
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .iter()
      .copied()
      .collect::<Vec<_>>();
    let deleted_ranges = self
      .deleted_shortcut_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone();
    let positions = self
      .shortcut_positions
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone();
    let animation_ranges = self
      .animation_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone();
    let mut timeline = KeyboardStateTimeline::from_shortcuts(
      &self.shortcuts,
      ChainContext {
        deleted_ids: &deleted_ids,
        deleted_ranges: &deleted_ranges,
        positions: &positions,
        ranges: animation_ranges.as_deref(),
      },
    );
    layout::attach_tracks(&mut timeline.visuals, animation_ranges.as_deref());
    let mut slots = timeline
      .visuals
      .iter()
      .map(|visual| visual.slot_id)
      .collect::<Vec<_>>();
    slots.sort_unstable();
    slots.dedup();
    slots.sort_by_key(|slot| {
      let order = timeline
        .visuals
        .iter()
        .filter(|visual| visual.slot_id == *slot)
        .map(|visual| visual.role.order())
        .min()
        .unwrap_or(u8::MAX);
      (order, *slot)
    });
    let maximum_width =
      geometry::maximum_width(&timeline.visuals, &slots, self.legacy_modifier_expansion);
    *self
      .baked
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = BakedTimeline {
      maximum_width,
      timeline,
      slots,
    };
  }

  #[cfg(test)]
  fn visuals_snapshot(&self) -> Vec<state::VisualKey> {
    self
      .baked
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .timeline
      .visuals
      .clone()
  }

  fn maximum_width(&self) -> f64 {
    self
      .baked
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .maximum_width
  }

  pub(crate) fn shortcut_count(&self) -> usize {
    self.shortcuts.len()
  }

  pub(crate) fn maximum_width_units(&self) -> u16 {
    self.maximum_width().ceil().clamp(0.0, f64::from(u16::MAX)) as u16
  }
  pub(crate) fn maximum_size_percent(&self, width: u32, height: u32) -> f64 {
    geometry::maximum_size_percent(self.maximum_width(), width, height)
  }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod timeline_tests;
