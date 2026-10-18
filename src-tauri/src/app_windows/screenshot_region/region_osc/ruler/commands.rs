// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay uses the shared Ruler settings matcher so its shortcuts stay
//! in sync with the Settings UI. Keys arrive as the platform's own codes,
//! which the bindings name per platform.

pub(crate) use crate::ruler::settings::KeyCommand;

/// Whether key codes are the Mac's rather than Windows' virtual keys.
pub(crate) const MAC_KEYS: bool = cfg!(target_os = "macos");

pub(crate) fn key_command(
  key: u16,
  modifiers: u32,
  repeat: bool,
  latched: bool,
) -> Option<KeyCommand> {
  crate::ruler::settings::key_command(key, modifiers, MAC_KEYS, repeat, latched)
}

/// `shortcut` as this platform delivers it: its key code and modifier word.
#[cfg(test)]
pub(crate) fn press(shortcut: &str) -> (u16, u32) {
  let binding = screenwide_osc::shortcut::parse(shortcut).unwrap();
  let key = if MAC_KEYS {
    binding.mac_key
  } else {
    binding.windows_key
  };
  (key, binding.modifiers)
}
