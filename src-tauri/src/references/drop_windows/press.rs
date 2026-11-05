// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Noticing a press anywhere on the desktop: a low-level mouse hook on a
//! thread of its own, which only wakes the drop watch on the main thread and
//! hands the press on at once, so input never waits on the app.

use windows::Win32::{
  Foundation::{LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HC_ACTION,
    MSG, WH_MOUSE_LL, WM_LBUTTONDOWN, WM_RBUTTONDOWN,
  },
};

pub(super) fn listen() {
  let spawned = std::thread::Builder::new()
    .name("reference-drop-press".to_owned())
    .spawn(|| {
      // The hook is called on this thread, which must take messages for it.
      if let Err(error) = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(callback), None, 0) } {
        eprintln!("Alt-dragging a picture is off: no mouse hook: {error}");
        return;
      }
      let mut message = MSG::default();
      while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
        unsafe {
          let _ = TranslateMessage(&message);
          DispatchMessageW(&message);
        }
      }
    });
  if let Err(error) = spawned {
    eprintln!("Alt-dragging a picture is off: {error}");
  }
}

unsafe extern "system" fn callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  // Either button: which is primary is the watch's to read, as it reads the
  // button held.
  if code == HC_ACTION as i32 && matches!(wparam.0 as u32, WM_LBUTTONDOWN | WM_RBUTTONDOWN) {
    if let Some(app) = super::APP.get() {
      let _ = app.run_on_main_thread(super::super::drop_watch::pressed);
    }
  }
  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
