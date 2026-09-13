// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Every workspace at once: a webview that has just loaded has to render the
/// right one and cannot know which change events it missed.
#[tauri::command]
pub fn get_editor_snapshot(app: AppHandle) -> EditorSnapshots {
  snapshots(&app)
}
