// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The settings as the event tap needs them: native flags, already resolved,
//! behind one process-global lock. Every tap callback reads this - several
//! times per event - so it must never reach for Tauri's managed state, which
//! would mean a handle lookup on the input thread for a value that changes
//! about once a session at most. The thirds modifier used to be a const up in
//! `macos.rs` that promised to become a user setting one day; this is that day.

use std::{
  collections::HashSet,
  sync::{LazyLock, Mutex, RwLock},
};

use core_graphics::event::{CGEvent, CGEventType, EventField};

use super::control::{NativeControl, MOUSE_STATE_BASE};
use crate::glide::settings::GlideSettings;

/// What the tap actually asks about, with the modifiers already mapped.
#[derive(Clone, Copy)]
pub(super) struct NativeGlideSettings {
  pub enabled: bool,
  pub haptics: bool,
  pub mouse_modifier: NativeControl,
  pub thirds_modifier: NativeControl,
  pub window_gap: u32,
  pub cursor_follows: bool,
  pub double_tap_center: bool,
}

/// Seeded from the defaults so the tap has an answer even in the window between
/// the tap starting and the stored settings being applied.
static NATIVE: LazyLock<RwLock<NativeGlideSettings>> =
  LazyLock::new(|| RwLock::new(native(&GlideSettings::default())));
static PRESSED: LazyLock<Mutex<HashSet<i64>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// The settings as they stand, cheap enough to take once per event.
pub(super) fn snapshot() -> NativeGlideSettings {
  *NATIVE
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Refreshes what the tap reads. Called once at startup and on every save, from
/// whichever thread the command arrived on.
pub(super) fn apply(settings: &GlideSettings) {
  *NATIVE
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = native(settings);
}

fn native(settings: &GlideSettings) -> NativeGlideSettings {
  NativeGlideSettings {
    enabled: settings.enabled,
    haptics: settings.haptics,
    mouse_modifier: NativeControl::from_control(settings.mouse_modifier)
      .expect("validated Glide mouse control"),
    thirds_modifier: NativeControl::from_control(settings.thirds_modifier)
      .expect("validated Glide thirds control"),
    window_gap: settings.window_gap,
    cursor_follows: settings.cursor_follows,
    double_tap_center: settings.double_tap_center,
  }
}

pub(super) fn observe(event_type: CGEventType, event: &CGEvent) {
  let (code, pressed) = match event_type {
    CGEventType::KeyDown => (keyboard_code(event), true),
    CGEventType::KeyUp => (keyboard_code(event), false),
    CGEventType::OtherMouseDown => (mouse_code(event), true),
    CGEventType::OtherMouseUp => (mouse_code(event), false),
    CGEventType::FlagsChanged => {
      let code = keyboard_code(event);
      let currently_pressed = PRESSED.lock().is_ok_and(|pressed| pressed.contains(&code));
      (code, !currently_pressed)
    }
    _ => return,
  };
  if let Ok(mut keys) = PRESSED.lock() {
    if pressed {
      keys.insert(code);
    } else {
      keys.remove(&code);
    }
  }
}

pub(super) fn is_down(key: NativeControl) -> bool {
  PRESSED
    .lock()
    .is_ok_and(|pressed| pressed.iter().any(|code| key.matches_state(*code)))
}

fn keyboard_code(event: &CGEvent) -> i64 {
  event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
}

fn mouse_code(event: &CGEvent) -> i64 {
  MOUSE_STATE_BASE + event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER)
}
