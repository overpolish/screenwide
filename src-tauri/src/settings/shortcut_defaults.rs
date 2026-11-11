// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;
use ts_rs::TS;

/// Return the same defaults used when native settings are first initialized,
/// with the system's keys Screenwide can take on this platform. The UI never
/// has to maintain a second set of platform-specific bindings.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ShortcutDefaults {
  pub shortcuts: crate::shortcuts::ShortcutSettings,
  pub glide: crate::glide::settings::GlideSettings,
  pub ruler: crate::ruler::settings::RulerSettings,
  pub ocr: crate::text_recognition::settings::OcrSettings,
  pub system_shortcuts: Vec<crate::shortcuts::SystemShortcut>,
}

#[tauri::command]
pub fn get_shortcut_defaults() -> ShortcutDefaults {
  ShortcutDefaults {
    shortcuts: Default::default(),
    glide: Default::default(),
    ruler: Default::default(),
    ocr: Default::default(),
    system_shortcuts: crate::shortcuts::system_shortcuts(),
  }
}
