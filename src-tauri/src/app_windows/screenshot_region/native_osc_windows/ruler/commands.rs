// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows uses the shared Ruler settings matcher so its shortcuts stay in
//! sync with macOS and the Settings UI.

pub(crate) use crate::ruler::settings::KeyCommand;

pub(crate) fn key_command(
  key: u16,
  modifiers: u32,
  repeat: bool,
  latched: bool,
) -> Option<KeyCommand> {
  crate::ruler::settings::key_command(key, modifiers, false, repeat, latched)
}
