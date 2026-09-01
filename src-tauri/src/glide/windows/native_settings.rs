// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The settings snapshot read by the Windows raw-input adapter.

use std::sync::{LazyLock, RwLock};

use windows::Win32::UI::Input::KeyboardAndMouse::{
  GetAsyncKeyState, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};

use crate::glide::settings::{GlideModifier, GlideSettings};

#[derive(Clone, Copy)]
pub(super) struct NativeGlideSettings {
  pub cursor_follows: bool,
  pub enabled: bool,
  pub mouse_modifier: GlideModifier,
  pub rest_ms: f64,
  pub thirds_modifier: GlideModifier,
  pub window_gap: u32,
}

static NATIVE: LazyLock<RwLock<NativeGlideSettings>> =
  LazyLock::new(|| RwLock::new(native(&GlideSettings::default())));

pub(super) fn snapshot() -> NativeGlideSettings {
  *NATIVE
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn apply(settings: &GlideSettings) {
  *NATIVE
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = native(settings);
}

pub(super) fn is_down(modifier: GlideModifier) -> bool {
  keys(modifier)
    .iter()
    .any(|key| unsafe { GetAsyncKeyState(key.0 as i32) } < 0)
}

pub(super) fn matches(modifier: GlideModifier, key: u32) -> bool {
  keys(modifier)
    .iter()
    .any(|candidate| u32::from(candidate.0) == key)
}

fn native(settings: &GlideSettings) -> NativeGlideSettings {
  NativeGlideSettings {
    cursor_follows: settings.cursor_follows,
    enabled: settings.enabled,
    mouse_modifier: settings.mouse_modifier,
    rest_ms: settings.pacing.rest_ms(),
    thirds_modifier: settings.thirds_modifier,
    window_gap: settings.window_gap,
  }
}

fn keys(modifier: GlideModifier) -> &'static [VIRTUAL_KEY] {
  match modifier {
    GlideModifier::Command => &[VK_LWIN, VK_RWIN],
    GlideModifier::Option => &[VK_MENU],
    GlideModifier::Control => &[VK_CONTROL],
    GlideModifier::Shift => &[VK_SHIFT],
  }
}
