// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Screenwide's own shortcuts that must not show in a recording's keystrokes:
//! the keys that place a moment are a note to whoever edits, not part of
//! what the recording shows.

use std::sync::RwLock;

use super::KeyboardModifier;
use crate::osc::shortcut::{self, Binding};

static HIDDEN: RwLock<Vec<Binding>> = RwLock::new(Vec::new());

/// Leaves the shortcuts `values` out of keyboard sidecars from now on, in
/// place of any left out before.
pub(crate) fn set_hidden_shortcuts(values: &[String]) {
  let bindings = values
    .iter()
    .filter_map(|value| shortcut::parse(value).ok())
    .collect();
  *HIDDEN
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = bindings;
}

/// Whether `key_code` pressed with `modifiers` is a hidden shortcut. Key
/// codes are macOS virtual key codes on every platform.
pub(super) fn is_hidden(key_code: u16, modifiers: &[KeyboardModifier]) -> bool {
  let mask = modifiers.iter().fold(0, |mask, modifier| {
    mask
      | match modifier {
        KeyboardModifier::Command => 1,
        KeyboardModifier::Control => 2,
        KeyboardModifier::Option => 4,
        KeyboardModifier::Shift => 8,
        KeyboardModifier::Function => 0,
      }
  });
  HIDDEN
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .iter()
    .any(|binding| binding.mac_key == key_code && binding.modifiers == mask)
}
