// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The end of a native window drag on Windows. The page is not relied on to
//! report the release after one, so a window that settles itself when it is
//! dropped watches the button directly from its first move.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use ::windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};

fn button_held() -> bool {
  let state = unsafe { GetAsyncKeyState(VK_LBUTTON.0.into()) };
  state < 0
}

/// Runs `dropped` once the left button is released, if it is held now. A move
/// with the button up is not a drag and does nothing. `active` is the window's
/// own flag, so the many moves one drag reports start one watch between them.
pub(crate) fn after_mouse_up(active: &'static AtomicBool, dropped: impl FnOnce() + Send + 'static) {
  if !button_held() || active.swap(true, Ordering::Relaxed) {
    return;
  }
  tauri::async_runtime::spawn_blocking(move || {
    while button_held() {
      std::thread::sleep(Duration::from_millis(8));
    }
    dropped();
    active.store(false, Ordering::Relaxed);
  });
}
