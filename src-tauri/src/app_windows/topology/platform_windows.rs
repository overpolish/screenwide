// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;

/// Windows topology adapter boundary. The Windows overlay pass only needs to
/// observe WM_DISPLAYCHANGE / WM_SETTINGCHANGE and call `changed`; all window
/// policies and geometry reconciliation are already shared Rust code.
pub(super) fn initialize(_app: &AppHandle, _changed: fn(AppHandle)) {}
