// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The settings snapshot read by the Windows raw-input adapter.

use std::{
  collections::HashSet,
  sync::{LazyLock, Mutex, RwLock},
};

use super::control::NativeControl;
use crate::glide::settings::GlideSettings;

#[derive(Clone, Copy)]
pub(super) struct NativeGlideSettings {
  pub cursor_follows: bool,
  pub double_tap_center: bool,
  pub enabled: bool,
  pub mouse_modifier: NativeControl,
  pub monitors_modifier: NativeControl,
  pub spaces_modifier: NativeControl,
  pub thirds_modifier: NativeControl,
  pub window_gap: u32,
}

static NATIVE: LazyLock<RwLock<NativeGlideSettings>> =
  LazyLock::new(|| RwLock::new(native(&GlideSettings::default())));
static PRESSED: LazyLock<Mutex<HashSet<u32>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub(super) fn snapshot() -> NativeGlideSettings {
  *NATIVE
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn trace_state(scope: &str) {
  if std::env::var_os("SCREENWIDE_GLIDE_TRACE").is_none() {
    return;
  }
  let settings = snapshot();
  let observed = |control: NativeControl| {
    PRESSED
      .lock()
      .is_ok_and(|pressed| pressed.iter().any(|key| control.matches(*key)))
  };
  crate::glide::core::trace::input(
    "windows-controls",
    format!(
      "{scope} mouse={:?}/physical={}/observed={}/effective={} monitors={:?}/physical={}/observed={}/effective={} spaces={:?}/physical={}/observed={}/effective={}",
      settings.mouse_modifier,
      settings.mouse_modifier.is_down(),
      observed(settings.mouse_modifier),
      is_down(settings.mouse_modifier),
      settings.monitors_modifier,
      settings.monitors_modifier.is_down(),
      observed(settings.monitors_modifier),
      is_down(settings.monitors_modifier),
      settings.spaces_modifier,
      settings.spaces_modifier.is_down(),
      observed(settings.spaces_modifier),
      is_down(settings.spaces_modifier),
    ),
  );
}

pub(super) fn apply(settings: &GlideSettings) {
  let settings = native(settings);
  *NATIVE
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings;
  if let Ok(mut pressed) = PRESSED.lock() {
    pressed.clear();
  }
}

pub(super) fn is_down(key: NativeControl) -> bool {
  if !key.uses_observed_state() {
    return key.is_down();
  }
  PRESSED
    .lock()
    .is_ok_and(|pressed| pressed.iter().any(|value| key.matches(*value)))
}

pub(super) fn observe(key: u32, pressed: bool) {
  if let Ok(mut keys) = PRESSED.lock() {
    if pressed {
      keys.insert(key);
    } else {
      keys.remove(&key);
    }
  }
}

pub(super) fn matches(configured: NativeControl, key: u32) -> bool {
  configured.matches(key)
}

fn native(settings: &GlideSettings) -> NativeGlideSettings {
  NativeGlideSettings {
    cursor_follows: settings.cursor_follows,
    double_tap_center: settings.double_tap_center,
    enabled: settings.enabled,
    mouse_modifier: NativeControl::from_control(settings.mouse_modifier)
      .expect("validated Glide mouse control"),
    monitors_modifier: NativeControl::from_control(settings.monitors_modifier)
      .expect("validated Glide monitors control"),
    spaces_modifier: NativeControl::from_control(settings.spaces_modifier)
      .expect("validated Glide Spaces control"),
    thirds_modifier: NativeControl::from_control(settings.thirds_modifier)
      .expect("validated Glide thirds control"),
    window_gap: settings.window_gap,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::glide::settings::GlideControl;
  use keyboard_types::Code;

  #[test]
  fn observed_controls_ignore_stale_async_state_until_fresh_transition() {
    let z = NativeControl::from_control(GlideControl::Key(Code::KeyZ)).unwrap();
    observe(90, false);
    assert!(!is_down(z));
    observe(90, true);
    assert!(is_down(z));
    observe(90, false);
    assert!(!is_down(z));
  }

  #[test]
  fn mouse_controls_are_transition_owned() {
    let mouse = NativeControl::from_control(GlideControl::MouseMiddle).unwrap();
    observe(super::super::control::MOUSE_MIDDLE, false);
    assert!(!is_down(mouse));
    observe(super::super::control::MOUSE_MIDDLE, true);
    assert!(is_down(mouse));
    observe(super::super::control::MOUSE_MIDDLE, false);
    assert!(!is_down(mouse));
  }
}
