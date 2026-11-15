// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window to paste into, kept by its handle. Windows lets the program in
//! front choose which window comes forward, so when a press on Screenwide's
//! own window has brought it in front, it can hand the front back.

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
  GetForegroundWindow, GetWindowThreadProcessId, IsWindow, SetForegroundWindow,
};

use crate::OnMain;

/// A window, by its handle's value, which unlike the handle can be sent
/// between threads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Target(isize);

impl Target {
  pub(crate) fn current() -> Option<Self> {
    // Safety: a plain query.
    let window = unsafe { GetForegroundWindow() };
    (!window.is_invalid() && !is_ours(window)).then_some(Self(window.0 as isize))
  }

  fn window(self) -> HWND {
    HWND(self.0 as *mut _)
  }
}

fn is_ours(window: HWND) -> bool {
  let mut process = 0;
  // Safety: plain queries.
  unsafe {
    GetWindowThreadProcessId(window, Some(&raw mut process));
    process == GetCurrentProcessId()
  }
}

/// Brings `target` in front if Screenwide is. False when nothing was done.
pub(crate) fn bring_back(target: Target, _on_main: OnMain<'_>) -> bool {
  // Safety: the window is only checked and brought forward.
  unsafe {
    let front = GetForegroundWindow();
    if front.is_invalid() || !is_ours(front) || !IsWindow(Some(target.window())).as_bool() {
      return false;
    }
    SetForegroundWindow(target.window()).as_bool()
  }
}

pub(crate) fn is_in_front(target: Target, _on_main: OnMain<'_>) -> bool {
  // Safety: a plain query.
  unsafe { GetForegroundWindow() == target.window() }
}
