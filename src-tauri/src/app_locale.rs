// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_i18n::AppLocale;

/// The language the app is in, for a webview to read before its first render.
#[tauri::command]
pub fn get_app_locale() -> AppLocale {
  screenwide_i18n::app_locale()
}
