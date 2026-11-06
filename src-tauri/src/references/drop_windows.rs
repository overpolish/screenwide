// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Alt-dragging a picture on Windows: what the drop watch asks of the
//! platform (`references::drop_watch`). A low-level mouse hook tells it of
//! each press; the buttons and Alt are read as they stand. Windows says what
//! a drag carries only to the window under it, so a drag counts once the
//! pointer has moved the system's drag distance from the press, and the
//! catchers learn whether it carries a picture as it reaches them.
//! Everything but the hook runs on the main thread.

mod catchers;
mod press;

use std::sync::OnceLock;

use tauri::AppHandle;
use windows::Win32::{
  Foundation::POINT,
  UI::{
    Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_MENU, VK_RBUTTON},
    WindowsAndMessaging::{GetCursorPos, GetSystemMetrics, SM_CXDRAG, SM_CYDRAG, SM_SWAPBUTTON},
  },
};

pub(super) use catchers::Catchers;

/// Where the pointer was as the button went down.
pub(super) type Mark = POINT;

/// Only a window under the drag is told what it carries.
pub(super) const SEES_PICTURE_BEFORE_CATCHERS: bool = false;

static APP: OnceLock<AppHandle> = OnceLock::new();

/// Tells the watch of every press on the desktop.
pub(super) fn listen(app: &AppHandle) {
  let _ = APP.set(app.clone());
  press::listen();
}

fn is_down(key: u16) -> bool {
  // The high bit says the key is down now.
  let state = unsafe { GetAsyncKeyState(i32::from(key)) };
  state < 0
}

/// Whether the primary button is held. The key state reads the buttons as
/// they are, not as the person has swapped them.
pub(super) fn held() -> bool {
  let swapped = unsafe { GetSystemMetrics(SM_SWAPBUTTON) } != 0;
  is_down(if swapped { VK_RBUTTON.0 } else { VK_LBUTTON.0 })
}

pub(super) fn modifier() -> bool {
  is_down(VK_MENU.0)
}

fn pointer() -> POINT {
  let mut at = POINT::default();
  let _ = unsafe { GetCursorPos(&mut at) };
  at
}

pub(super) fn mark() -> Mark {
  pointer()
}

/// Whether the pointer has gone the system's drag distance from the press
/// at `mark`, as a drag begins.
pub(super) fn dragging(mark: Mark) -> bool {
  let at = pointer();
  let (across, down) = unsafe { (GetSystemMetrics(SM_CXDRAG), GetSystemMetrics(SM_CYDRAG)) };
  (at.x - mark.x).abs() > across / 2 || (at.y - mark.y).abs() > down / 2
}
