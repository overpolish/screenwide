// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Timestamped keyboard-shortcut effects shared by preview, export, and still composition.

use serde_json::Value;
use std::path::Path;

mod animation;
use animation::{ease_out, pop_spring, replacement_enter_progress, replacement_exit_progress};
mod data;
use data::{parse_v1, read_values, reconstruct_v2};
mod gpu_wire;
use gpu_wire::MAX_KEYS;
pub(crate) use gpu_wire::{KeyboardKey, KeyboardOverlay};
mod geometry;
mod layout;
mod settings;
pub(crate) use settings::{KeyboardAnimation, KeyboardAppearance, KeyboardEffectSettings};
mod state;
use state::{KeyboardStateTimeline, TransitionKind};

const ENTRANCE_SECONDS: f64 = 0.6;
const EXIT_SECONDS: f64 = 0.6;
const MICROS_PER_SECOND: f64 = 1_000_000.0;

#[derive(Clone, Debug)]
pub(crate) struct KeyboardCompositor {
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
    Ok(Self::from_shortcuts_with_legacy(shortcuts, version < 2))
  }
  #[cfg(test)]
  fn from_shortcuts(shortcuts: Vec<Shortcut>) -> Self {
    Self::from_shortcuts_with_legacy(shortcuts, false)
  }
  fn from_shortcuts_with_legacy(shortcuts: Vec<Shortcut>, legacy_modifier_expansion: bool) -> Self {
    let mut timeline = KeyboardStateTimeline::from_shortcuts(&shortcuts);
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
      geometry::maximum_width(&timeline.visuals, &slots, legacy_modifier_expansion);
    Self {
      maximum_width,
      timeline,
      slots,
    }
  }
  pub(crate) fn maximum_width_units(&self) -> u16 {
    self.maximum_width.ceil().clamp(0.0, f64::from(u16::MAX)) as u16
  }
  pub(crate) fn maximum_size_percent(&self, width: u32, height: u32) -> f64 {
    geometry::maximum_size_percent(self.maximum_width, width, height)
  }
  pub(crate) fn evaluate_fitted(
    &self,
    position_ms: u64,
    settings: KeyboardEffectSettings,
    dimensions: (u32, u32),
  ) -> Option<KeyboardOverlay> {
    let fitted_scale = (settings
      .size_percent
      .min(self.maximum_size_percent(dimensions.0, dimensions.1))
      / 100.0) as f32;
    let mut overlay = self.evaluate(position_ms, settings)?;
    overlay.scale = fitted_scale;
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
      maximum_width: self.maximum_width as f32,
      requested_scale: size_scale,
      ..Default::default()
    };
    let mut visible = self
      .timeline
      .visuals
      .iter()
      .filter(|key| key.visible_at(now))
      .collect::<Vec<_>>();
    let slots = &self.slots;
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
      }) * (settings.size_percent / 100.0) as f32;
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
