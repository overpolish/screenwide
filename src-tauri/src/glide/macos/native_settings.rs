// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The settings as the event tap needs them: native flags, already resolved,
//! behind one process-global lock. Every tap callback reads this - several
//! times per event - so it must never reach for Tauri's managed state, which
//! would mean a handle lookup on the input thread for a value that changes
//! about once a session at most. The thirds modifier used to be a const up in
//! `macos.rs` that promised to become a user setting one day; this is that day.

use std::sync::{LazyLock, RwLock};

use core_graphics::event::CGEventFlags;

use crate::glide::settings::{GlideModifier, GlideSettings};

/// What the tap actually asks about, with the modifiers already mapped.
#[derive(Clone, Copy)]
pub(super) struct NativeGlideSettings {
  pub enabled: bool,
  pub haptics: bool,
  pub mouse_modifier: CGEventFlags,
  pub rest_ms: f64,
  pub thirds_modifier: CGEventFlags,
  pub window_gap: u32,
  pub cursor_follows: bool,
  pub double_tap_center: bool,
}

/// Seeded from the defaults so the tap has an answer even in the window between
/// the tap starting and the stored settings being applied.
static NATIVE: LazyLock<RwLock<NativeGlideSettings>> =
  LazyLock::new(|| RwLock::new(native(&GlideSettings::default())));

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
    mouse_modifier: flags(settings.mouse_modifier),
    rest_ms: settings.pacing.rest_ms(),
    thirds_modifier: flags(settings.thirds_modifier),
    window_gap: settings.window_gap,
    cursor_follows: settings.cursor_follows,
    double_tap_center: settings.double_tap_center,
  }
}

fn flags(modifier: GlideModifier) -> CGEventFlags {
  match modifier {
    GlideModifier::Command => CGEventFlags::CGEventFlagCommand,
    GlideModifier::Option => CGEventFlags::CGEventFlagAlternate,
    GlideModifier::Control => CGEventFlags::CGEventFlagControl,
    GlideModifier::Shift => CGEventFlags::CGEventFlagShift,
  }
}
