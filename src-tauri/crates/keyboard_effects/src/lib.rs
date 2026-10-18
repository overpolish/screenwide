// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Timestamped keyboard-shortcut effects shared by preview, export, and still composition.

mod evaluation;

use screenwide_timeline_edit::DeletedKeyboardShortcutRange;
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
use gpu_wire::MAX_KEYS;
pub use gpu_wire::{KeyboardKey, KeyboardOverlay};
mod geometry;
mod layout;
mod settings;
pub use settings::{KeyboardAnimation, KeyboardAppearance, KeyboardEffectSettings};
mod state;
use state::{ChainContext, KeyboardStateTimeline, TransitionKind};
mod timeline;
pub use timeline::KeyboardTimelineItem;

const ENTRANCE_SECONDS: f64 = 0.6;
const EXIT_SECONDS: f64 = 0.4;
const MICROS_PER_SECOND: f64 = 1_000_000.0;
pub const HOLD_US: u64 = 750_000;

#[derive(Clone, Debug)]
pub struct KeyboardCompositor {
  /// Display state derived from the shortcuts plus the timeline edits.
  /// Deletions decide badge continuity, so every edit rebakes this rather
  /// than being patched over a stale lifecycle.
  baked: Arc<RwLock<BakedTimeline>>,
  shortcuts: Vec<Shortcut>,
  legacy_modifier_expansion: bool,
  deleted_shortcut_ids: Arc<RwLock<HashSet<u64>>>,
  deleted_shortcut_ranges: Arc<RwLock<Vec<DeletedKeyboardShortcutRange>>>,
  /// The retained playback ranges of the current edit. Animation durations
  /// run on the output clock, so decisions baked from them (fade pulled
  /// forward to finish by a press, the chord-roll assembly window) need the
  /// rate mapping at bake time, not only at evaluation.
  animation_ranges: Arc<RwLock<Option<Vec<screenwide_timeline::TimelineRange>>>>,
}

#[derive(Debug, Default)]
struct BakedTimeline {
  maximum_width: f64,
  timeline: KeyboardStateTimeline,
  slots: Vec<u32>,
}
#[derive(Clone, Debug)]
pub struct Shortcut {
  keys: Vec<KeyPress>,
}
#[derive(Clone, Copy, Debug)]
pub struct KeyPress {
  key_code: u16,
  modifier_mask: u32,
  down_us: u64,
  up_us: Option<u64>,
}

impl Shortcut {
  /// Whether the chord is a shortcut rather than typing: some key in it other
  /// than a modifier was pressed with Command or Control.
  fn is_shortcut(&self) -> bool {
    const COMMAND_OR_CONTROL: u32 = 1 | 2;
    self.keys.iter().any(|press| {
      !screenwide_records::keyboard::is_modifier_key(press.key_code)
        && !screenwide_records::keyboard::types(
          press.key_code,
          press.modifier_mask & COMMAND_OR_CONTROL != 0,
        )
    })
  }
}

impl KeyboardCompositor {
  pub fn open(path: &Path) -> Result<Self, String> {
    Self::open_with_deleted(path, &[], &[])
  }
  pub fn open_with_deleted(
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
    let mut shortcuts = if version >= 2 {
      reconstruct_v2(&records)
    } else {
      parse_v1(&records)
    };
    // Typing shows no keys, only shortcuts do. A chord that typed is emptied
    // rather than removed, so every other keeps the id its deletions name.
    for shortcut in &mut shortcuts {
      if !shortcut.is_shortcut() {
        shortcut.keys.clear();
      }
    }
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
      animation_ranges: Arc::new(RwLock::new(None)),
    };
    compositor.rebake();
    compositor
  }

  /// Adopts the edit's playback ranges for bake-time animation decisions,
  /// rebaking only when they changed. Called by every evaluation entry point,
  /// so a compositor always bakes against the timeline it renders for.
  pub fn set_animation_timeline(&self, ranges: Option<&[screenwide_timeline::TimelineRange]>) {
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
  /// timeline edits. Called whenever deletions change, because they decide
  /// whether consecutive chords continue one badge.
  pub fn rebake(&self) {
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

  pub fn shortcut_count(&self) -> usize {
    self.shortcuts.len()
  }

  pub(crate) fn maximum_size_percent(&self, width: u32, height: u32) -> f64 {
    geometry::maximum_size_percent(self.maximum_width(), width, height)
  }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod timeline_tests;
