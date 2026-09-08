// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;

use super::SystemAccent;

/// Platforms without a system accent keep the app's own brand colour.
pub(super) fn current() -> Option<SystemAccent> {
  None
}

pub(super) fn initialize(_app: &AppHandle, _changed: fn(AppHandle)) {}
