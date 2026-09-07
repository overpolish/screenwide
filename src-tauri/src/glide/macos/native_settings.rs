// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The settings as the event tap needs them: native flags, already resolved,
//! behind one process-global lock. Every tap callback reads this - several
//! times per event - so it must never reach for Tauri's managed state, which
//! would mean a handle lookup on the input thread for a value that changes
//! about once a session at most. The thirds modifier used to be a const up in
//! `macos.rs` that promised to become a user setting one day; this is that day.

use std::{
  collections::HashMap,
  sync::{LazyLock, Mutex, RwLock},
  time::{Duration, Instant},
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
  pub monitors_modifier: NativeControl,
  pub spaces_modifier: NativeControl,
  pub thirds_modifier: NativeControl,
  pub window_gap: u32,
  pub cursor_follows: bool,
  pub double_tap_center: bool,
}

/// Seeded from the defaults so the tap has an answer even in the window between
/// the tap starting and the stored settings being applied.
static NATIVE: LazyLock<RwLock<NativeGlideSettings>> =
  LazyLock::new(|| RwLock::new(native(&GlideSettings::default())));
#[derive(Clone, Copy)]
struct Observed {
  since: Instant,
  up_samples: u8,
  recover_release: bool,
}
static PRESSED: LazyLock<Mutex<HashMap<i64, Observed>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

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
    monitors_modifier: NativeControl::from_control(settings.monitors_modifier)
      .expect("validated Glide monitors control"),
    spaces_modifier: NativeControl::from_control(settings.spaces_modifier)
      .expect("validated Glide Spaces control"),
    thirds_modifier: NativeControl::from_control(settings.thirds_modifier)
      .expect("validated Glide thirds control"),
    window_gap: settings.window_gap,
    cursor_follows: settings.cursor_follows,
    double_tap_center: settings.double_tap_center,
  }
}

pub(super) fn observe(event_type: CGEventType, event: &CGEvent) {
  if super::spaces::is_synthetic(event) {
    return;
  }
  let (code, pressed) = match event_type {
    CGEventType::KeyDown => (keyboard_code(event), true),
    CGEventType::KeyUp => (keyboard_code(event), false),
    CGEventType::OtherMouseDown => (mouse_code(event), true),
    CGEventType::OtherMouseUp => (mouse_code(event), false),
    CGEventType::FlagsChanged => {
      let code = keyboard_code(event);
      (code, flags_pressed(code, event.get_flags()))
    }
    _ => return,
  };
  if let Ok(mut keys) = PRESSED.lock() {
    if pressed {
      keys.insert(
        code,
        Observed {
          since: Instant::now(),
          up_samples: 0,
          recover_release: matches!(
            event_type,
            CGEventType::FlagsChanged | CGEventType::OtherMouseDown
          ),
        },
      );
    } else {
      keys.remove(&code);
    }
  }
}

pub(super) fn reconcile() -> Vec<i64> {
  let now = Instant::now();
  let mut recovered = Vec::new();
  if let Ok(mut keys) = PRESSED.lock() {
    keys.retain(|code, observed| {
      // Ordinary key-downs can be swallowed by our tap. Polling can then
      // report them as released while held; only their key-up ends the hold.
      if !observed.recover_release {
        return true;
      }
      let hardware = if *code >= MOUSE_STATE_BASE {
        super::hardware::button_down((*code - MOUSE_STATE_BASE) as u32)
      } else {
        super::hardware::key_down(*code as u16)
      };
      if hardware {
        observed.up_samples = 0;
        true
      } else if now.duration_since(observed.since) < Duration::from_millis(40) {
        true
      } else {
        observed.up_samples = observed.up_samples.saturating_add(1);
        if observed.up_samples >= 2 {
          recovered.push(*code);
          false
        } else {
          true
        }
      }
    });
  }
  for code in &recovered {
    crate::glide::core::trace::input("mac-release", format!("recovered code={code}"));
  }
  recovered
}

pub(super) fn is_down(key: NativeControl) -> bool {
  PRESSED
    .lock()
    .is_ok_and(|pressed| pressed.keys().any(|code| key.matches_state(*code)))
}

fn keyboard_code(event: &CGEvent) -> i64 {
  event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
}

fn mouse_code(event: &CGEvent) -> i64 {
  MOUSE_STATE_BASE + event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER)
}

fn flags_pressed(code: i64, flags: core_graphics::event::CGEventFlags) -> bool {
  let flag = match code {
    54 | 55 => core_graphics::event::CGEventFlags::CGEventFlagCommand,
    56 | 60 => core_graphics::event::CGEventFlags::CGEventFlagShift,
    59 | 62 => core_graphics::event::CGEventFlags::CGEventFlagControl,
    58 | 61 => core_graphics::event::CGEventFlags::CGEventFlagAlternate,
    57 => core_graphics::event::CGEventFlags::CGEventFlagAlphaShift,
    _ => return false,
  };
  flags.contains(flag)
}
