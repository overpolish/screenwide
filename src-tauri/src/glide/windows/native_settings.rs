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
  pub enabled: bool,
  pub mouse_modifier: NativeControl,
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
  key.is_down()
    || (key.uses_observed_state()
      && PRESSED
        .lock()
        .is_ok_and(|pressed| pressed.iter().any(|value| key.matches(*value))))
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
    enabled: settings.enabled,
    mouse_modifier: NativeControl::from_control(settings.mouse_modifier)
      .expect("validated Glide mouse control"),
    thirds_modifier: NativeControl::from_control(settings.thirds_modifier)
      .expect("validated Glide thirds control"),
    window_gap: settings.window_gap,
  }
}
