// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;

/// No capture runs on this platform, so there is nothing to set aside.
pub(super) fn initialize(_: &AppHandle, _: fn(&AppHandle), _: fn(&AppHandle)) {}
