// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use ::windows::Win32::UI::WindowsAndMessaging::{WS_EX_LAYERED, WS_EX_TRANSPARENT};
use tauri::WebviewWindow;

/// Lets presses through a window to whatever is underneath, or takes them
/// back.
///
/// The extended style is edited in place rather than through
/// `set_ignore_cursor_events`, which rebuilds the whole word from tao's own
/// flags: that drops the always-on-top band the window was holding, and it is
/// applied through the event loop, so a raise issued right after it can land
/// before the rebuild and be undone. Layering is added but never removed,
/// because a transparent window's own alpha depends on it.
pub fn set_pointer_passthrough(window: &WebviewWindow, passthrough: bool) -> tauri::Result<()> {
  use ::windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE},
  };

  let hwnd = HWND(window.hwnd()?.0);
  let current = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
  let next = passthrough_style(current, passthrough);
  if next != current {
    unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next) };
  }
  Ok(())
}

pub(super) const fn passthrough_style(style: isize, passthrough: bool) -> isize {
  let transparent = WS_EX_TRANSPARENT.0 as isize;
  if passthrough {
    style | transparent | WS_EX_LAYERED.0 as isize
  } else {
    style & !transparent
  }
}

#[cfg(test)]
mod tests {
  use super::passthrough_style;
  use ::windows::Win32::UI::WindowsAndMessaging::{
    WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
  };

  #[test]
  fn passthrough_keeps_every_unrelated_style_including_the_topmost_band() {
    let base = 0x0012_3456_isize | WS_EX_TOPMOST.0 as isize;
    let through = passthrough_style(base, true);
    assert_ne!(through & WS_EX_TRANSPARENT.0 as isize, 0);
    assert_ne!(through & WS_EX_LAYERED.0 as isize, 0);
    assert_ne!(through & WS_EX_TOPMOST.0 as isize, 0);
    // Taking presses back leaves the layering a transparent window needs.
    let back = passthrough_style(through, false);
    assert_eq!(back & WS_EX_TRANSPARENT.0 as isize, 0);
    assert_ne!(back & WS_EX_LAYERED.0 as isize, 0);
    assert_ne!(back & WS_EX_TOPMOST.0 as isize, 0);
  }
}
