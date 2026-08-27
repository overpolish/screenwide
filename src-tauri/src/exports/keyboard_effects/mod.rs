// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Timestamped keyboard-shortcut effects shared by preview, export, and still composition.

use crate::exports::timeline_edit::{DeletedKeyboardShortcutRange, KeyboardShortcutPositionRange};
use serde_json::Value;
use std::{
  collections::HashSet,
  path::Path,
  sync::{Arc, RwLock},
};

mod animation;
use animation::{ease_out, pop_spring, replacement_enter_progress, replacement_exit_progress};
mod data;
use data::{parse_v1, read_values, reconstruct_v2};
mod gpu_wire;
use gpu_wire::{KEY_CENTER_DEFAULT, KEY_CENTER_INHERIT, MAX_KEYS};
pub(crate) use gpu_wire::{KeyboardKey, KeyboardOverlay};
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
    };
    compositor.rebake();
    compositor
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
    let mut timeline = KeyboardStateTimeline::from_shortcuts(
      &self.shortcuts,
      ChainContext {
        deleted_ids: &deleted_ids,
        deleted_ranges: &deleted_ranges,
        positions: &positions,
      },
    );
    layout::attach_tracks(&mut timeline.visuals);
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
    self
      .maximum_width()
      .ceil()
      .clamp(0.0, f64::from(u16::MAX)) as u16
  }
  pub(crate) fn maximum_size_percent(&self, width: u32, height: u32) -> f64 {
    geometry::maximum_size_percent(self.maximum_width(), width, height)
  }
  pub(crate) fn evaluate_fitted(
    &self,
    position_ms: u64,
    settings: KeyboardEffectSettings,
    dimensions: (u32, u32),
  ) -> Option<KeyboardOverlay> {
    let mut overlay = self.evaluate(position_ms, settings)?;
    overlay.scale = overlay
      .requested_scale
      .min((self.maximum_size_percent(dimensions.0, dimensions.1) / 100.0) as f32);
    Some(overlay)
  }
  pub(crate) fn evaluate(
    &self,
    position_ms: u64,
    settings: KeyboardEffectSettings,
  ) -> Option<KeyboardOverlay> {
    let settings = settings.normalized();
    if !settings.bake {
      return None;
    }
    let now = position_ms.saturating_mul(1_000);
    let baked = self
      .baked
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let animation = match settings.animation {
      KeyboardAnimation::Pop => KeyboardOverlay::ANIMATION_POP,
      KeyboardAnimation::Fade => KeyboardOverlay::ANIMATION_FADE,
      KeyboardAnimation::None => KeyboardOverlay::ANIMATION_NONE,
    };
    let size_scale = (settings.size_percent / 100.0) as f32;
    let appearance = match settings.appearance {
      KeyboardAppearance::Dark => KeyboardOverlay::APPEARANCE_DARK,
      KeyboardAppearance::Light => KeyboardOverlay::APPEARANCE_LIGHT,
    };
    let mut overlay = KeyboardOverlay {
      key_count: 0,
      animation,
      appearance,
      scale: size_scale,
      progress: 1.0,
      maximum_width: baked.maximum_width as f32,
      requested_scale: size_scale,
      center_x: settings
        .position_x_percent
        .map_or(-1.0, |position| (position / 100.0) as f32),
      center_y: settings
        .position_y_percent
        .map_or(-1.0, |position| (position / 100.0) as f32),
      ..Default::default()
    };
    let deleted_ids = self
      .deleted_shortcut_ids
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let deleted_ranges = self
      .deleted_shortcut_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let visible = baked
      .timeline
      .visuals
      .iter()
      .filter(|key| {
        let shortcut_id = key.source_shortcut as u64;
        !deleted_ids.contains(&shortcut_id)
          && !deleted_ranges.iter().any(|range| {
            range.shortcut_id == shortcut_id
              && position_ms >= range.start_ms
              && position_ms < range.end_ms
          })
      })
      .filter(|key| key.visible_at(now))
      .collect::<Vec<_>>();
    // Manual placements are per shortcut, so during a transition two groups
    // can occupy different spots. The overlay centre follows the newest
    // visible group as before; every key additionally carries its own
    // group's centre so a differently placed badge finishes its animation
    // where the user put it instead of teleporting to the successor.
    let positions = self
      .shortcut_positions
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone();
    let placement_at = |shortcut: usize| {
      positions
        .iter()
        .find(|position| {
          position.shortcut_id == shortcut as u64
            && position_ms >= position.start_ms
            && position_ms < position.end_ms
        })
        .map(|position| (position.center_x, position.center_y, position.size_percent))
    };
    let base_center = (overlay.center_x, overlay.center_y);
    self.apply_shortcut_position(&mut overlay, &visible, position_ms);
    let overlay_placement = visible
      .iter()
      .max_by_key(|key| key.enter_us)
      .and_then(|key| placement_at(key.source_shortcut));
    // Wire slot indices and layout masks are bit positions, so they must be
    // compact. Fresh badge groups keep allocating new slot ids for the whole
    // recording; only the slots this frame actually references are wired,
    // in the global ordering so rows stay stable.
    let mut frame_slots: Vec<u32> = Vec::new();
    for key in &visible {
      let sample = key.layout.sample(now);
      for slot in std::iter::once(&key.slot_id)
        .chain(sample.from)
        .chain(sample.to)
      {
        if !frame_slots.contains(slot) {
          frame_slots.push(*slot);
        }
      }
    }
    frame_slots.sort_by_key(|slot| {
      baked
        .slots
        .iter()
        .position(|known| known == slot)
        .unwrap_or(usize::MAX)
    });
    let slots = &frame_slots;
    let mut visible = visible;
    visible.sort_by_key(|key| {
      (
        slots
          .iter()
          .position(|slot| *slot == key.slot_id)
          .unwrap_or(usize::MAX),
        key.enter_us,
      )
    });
    for key in visible {
      let exit = key.exit.filter(|(exit_us, _)| now >= *exit_us);
      let exit_progress = exit.map(|(exit_us, _)| {
        ((now.saturating_sub(exit_us) as f64 / MICROS_PER_SECOND) / EXIT_SECONDS).clamp(0.0, 1.0)
          as f32
      });
      let layout_tail_visible = key
        .layout_anchor_until_us
        .is_some_and(|until_us| now < until_us);
      let artwork_hidden = exit_progress
        .is_some_and(|progress| settings.animation == KeyboardAnimation::None || progress >= 1.0);
      if artwork_hidden && !layout_tail_visible {
        continue;
      }
      if overlay.key_count as usize >= MAX_KEYS {
        break;
      }
      // Inherit the overlay centre when this key's group placement matches
      // the group the overlay follows; otherwise pin the key to its own
      // group's spot so it animates there.
      let placement = placement_at(key.source_shortcut);
      let (center_x, center_y, group_scale) = if placement == overlay_placement {
        (KEY_CENTER_INHERIT, KEY_CENTER_INHERIT, overlay.requested_scale)
      } else if let Some((x, y, size)) = placement {
        (
          x as f32,
          y as f32,
          size.map_or(size_scale, |size| (size / 100.0) as f32),
        )
      } else {
        (
          if base_center.0 >= 0.0 {
            base_center.0
          } else {
            KEY_CENTER_DEFAULT
          },
          if base_center.1 >= 0.0 {
            base_center.1
          } else {
            KEY_CENTER_DEFAULT
          },
          size_scale,
        )
      };
      let scale_ratio = group_scale / overlay.requested_scale.max(0.001);
      let entrance_seconds = if key.replacement_enter {
        EXIT_SECONDS
      } else {
        ENTRANCE_SECONDS
      };
      let raw_entrance_progress = ((now.saturating_sub(key.animation_enter_us) as f64
        / MICROS_PER_SECOND)
        / entrance_seconds)
        .clamp(0.0, 1.0) as f32;
      let entrance_progress = if key.replacement_enter {
        replacement_enter_progress(raw_entrance_progress)
      } else {
        raw_entrance_progress
      };
      let key_progress = exit_progress.map_or(entrance_progress, |progress| {
        if exit.is_some_and(|(_, kind)| kind == TransitionKind::Replacement) {
          1.0 - replacement_exit_progress(progress)
        } else {
          1.0 - progress
        }
      });
      let detached_amount = exit
        .filter(|(_, kind)| *kind == TransitionKind::Detached)
        .and(exit_progress)
        .map(|progress| 1.0 - pop_spring(progress).clamp(0.0, 1.0));
      let key_alpha = if artwork_hidden {
        0.0
      } else if let Some(amount) = detached_amount {
        amount
      } else if animation == KeyboardOverlay::ANIMATION_FADE
        || key.replacement_enter
        || exit.is_some_and(|(_, kind)| kind == TransitionKind::Replacement)
      {
        ease_out(key_progress)
      } else {
        1.0
      };
      let key_scale = (if artwork_hidden {
        0.0
      } else if animation == KeyboardOverlay::ANIMATION_POP {
        detached_amount.unwrap_or_else(|| pop_spring(key_progress))
      } else {
        1.0
      }) * overlay.requested_scale;
      let index = overlay.key_count as usize;
      let layout = key.layout.sample(now);
      overlay.keys[index] = KeyboardKey {
        key_code: key.key_code,
        modifier_mask: key.modifier_mask,
        visible: if exit_progress.is_some() { 2 } else { 1 },
        progress: key_progress,
        alpha: key_alpha,
        scale: key_scale,
        layout_progress: layout.progress,
        slot: slots
          .iter()
          .position(|slot| *slot == key.slot_id)
          .unwrap_or_default() as u32,
        layout_from_mask: layout::mask(layout.from, slots),
        layout_to_mask: layout::mask(layout.to, slots),
        center_x,
        center_y,
        scale_ratio,
      };
      overlay.key_count += 1;
    }
    if overlay.key_count == 0 {
      return None;
    }
    Some(overlay)
  }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod timeline_tests;
