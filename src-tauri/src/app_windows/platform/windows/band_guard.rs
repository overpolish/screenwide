// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeps this process's always-on-top windows in the topmost band when another
//! application takes the foreground.
//!
//! Windows has no window levels: the band is one contiguous run at the top of
//! the z-list, and a later window is inserted below the first window found
//! without `WS_EX_TOPMOST`. Some applications break the run. Windows 11 Paint
//! places its main window above every topmost window while carrying no
//! topmost bit, so from then on each newly activated window lands above the
//! overlays too, and a monitor-sized overlay is additionally pushed under the
//! taskbar by the fullscreen rule. The overlays' own styles stay intact
//! throughout, so nothing on this side can notice by inspection. The recovery
//! is to re-assert the band whenever the foreground moves to another process.

use std::sync::atomic::{AtomicIsize, Ordering};

use windows::Win32::{
  Foundation::HWND,
  System::Threading::GetCurrentThreadId,
  UI::{
    Accessibility::{SetWinEventHook, HWINEVENTHOOK},
    WindowsAndMessaging::{
      GetTopWindow, GetWindow, GetWindowLongPtrW, GetWindowThreadProcessId, IsWindowVisible,
      SetWindowPos, EVENT_SYSTEM_FOREGROUND, GWL_EXSTYLE, GW_HWNDNEXT, HWND_TOPMOST,
      SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
      WS_EX_TOPMOST,
    },
  },
};

/// The hook lives as long as the process; it is never unhooked.
static HOOK: AtomicIsize = AtomicIsize::new(0);

/// Installs the foreground watch. Must run on the thread that owns the overlay
/// windows and pumps messages - the main thread - because an out-of-context
/// hook delivers on the installing thread's message loop, and the re-assertion
/// reorders that thread's windows synchronously.
pub fn initialize() -> std::io::Result<()> {
  if HOOK.load(Ordering::Acquire) != 0 {
    return Ok(());
  }
  let hook = unsafe {
    SetWinEventHook(
      EVENT_SYSTEM_FOREGROUND,
      EVENT_SYSTEM_FOREGROUND,
      None,
      Some(on_foreground),
      0,
      0,
      WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
    )
  };
  if hook.is_invalid() {
    return Err(std::io::Error::last_os_error());
  }
  HOOK.store(hook.0 as isize, Ordering::Release);
  Ok(())
}

unsafe extern "system" fn on_foreground(
  _hook: HWINEVENTHOOK,
  _event: u32,
  _window: HWND,
  _object: i32,
  _child: i32,
  _thread: u32,
  _time: u32,
) {
  reassert_band();
}

/// Moves every visible topmost window of the calling thread back to the top
/// of the z-list, keeping their order among themselves. Windows on other
/// threads are left alone: reordering them would block on their message loop.
fn reassert_band() {
  let thread = unsafe { GetCurrentThreadId() };
  let mut ours = Vec::new();
  let mut window = unsafe { GetTopWindow(None) }.unwrap_or_default();
  while !window.is_invalid() {
    if unsafe { GetWindowThreadProcessId(window, None) } == thread
      && unsafe { IsWindowVisible(window) }.as_bool()
      && unsafe { GetWindowLongPtrW(window, GWL_EXSTYLE) } & WS_EX_TOPMOST.0 as isize != 0
    {
      ours.push(window);
    }
    window = unsafe { GetWindow(window, GW_HWNDNEXT) }.unwrap_or_default();
  }
  // Bottom-most first: each call lands at the top of the band, so walking up
  // the list rebuilds it in the order the windows already had.
  for window in ours.into_iter().rev() {
    let _ = unsafe {
      SetWindowPos(
        window,
        Some(HWND_TOPMOST),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      )
    };
  }
}
