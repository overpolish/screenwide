// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_accent::SystemAccent;

/// Reads the current accent. `None` means the platform has no accent to
/// follow, so the app keeps its own brand colour.
#[tauri::command]
pub fn get_system_accent() -> Option<SystemAccent> {
  screenwide_accent::current()
}
