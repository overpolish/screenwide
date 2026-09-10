// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;

/// Return the same defaults used when native settings are first initialized.
/// The UI never has to maintain a second set of platform-specific bindings.
#[derive(Serialize, Default)]
pub struct ShortcutDefaults {
  pub shortcuts: crate::shortcuts::ShortcutSettings,
  pub glide: crate::glide::settings::GlideSettings,
  pub ruler: crate::ruler::settings::RulerSettings,
  pub ocr: crate::text_recognition::settings::OcrSettings,
}

#[tauri::command]
pub fn get_shortcut_defaults() -> ShortcutDefaults {
  ShortcutDefaults::default()
}
