// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::Win32::UI::WindowsAndMessaging::ShowCursor;

pub(super) fn hide_cursor() {
  while unsafe { ShowCursor(false) } >= 0 {}
}

pub(super) fn show_cursor() {
  while unsafe { ShowCursor(true) } < 0 {}
}
