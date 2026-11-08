// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island's window: an overlay like the rest that never takes the
//! keyboard, and lets presses through everywhere but the island.
//!
//! It is shown and hidden with `show_without_activation` rather than through
//! tao, which would activate it: shown over the Ruler or OCR, the island then
//! took the keyboard from their overlay, and only Escape, which the island's
//! page answers, still worked.

use tauri::WebviewWindow;
use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE},
};

use super::super::initialize_overlay;
use super::windows_api::passthrough_style;

/// The island: an overlay like the rest, never activated, and without the
/// rounded corner the other panels take, as it draws its own shape and
/// shadow on a transparent window, which the corner would clip. tao is told
/// it cannot take focus too, so the one rewrite here, before it is ever
/// shown, keeps `WS_EX_NOACTIVATE`.
pub fn initialize_island(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)?;
  window.set_focusable(false)?;
  set_island_input(window, true)
}

/// The island's extended style: never activated, and letting presses through
/// where `passthrough`. The watcher compares the window with this and puts it
/// back, should anything have changed it.
const fn island_style(style: isize, passthrough: bool) -> isize {
  passthrough_style(style, passthrough) | WS_EX_NOACTIVATE.0 as isize
}

fn ex_style(window: &WebviewWindow) -> tauri::Result<isize> {
  Ok(unsafe { GetWindowLongPtrW(HWND(window.hwnd()?.0), GWL_EXSTYLE) })
}

/// Whether the island's window has its own extended style for `passthrough`.
/// Reading it is safe from any thread.
pub fn island_input_holds(window: &WebviewWindow, passthrough: bool) -> tauri::Result<bool> {
  let current = ex_style(window)?;
  Ok(island_style(current, passthrough) == current)
}

/// Gives the island's window its own extended style for `passthrough`.
pub fn set_island_input(window: &WebviewWindow, passthrough: bool) -> tauri::Result<()> {
  let current = ex_style(window)?;
  let next = island_style(current, passthrough);
  if next != current {
    unsafe { SetWindowLongPtrW(HWND(window.hwnd()?.0), GWL_EXSTYLE, next) };
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::island_style;
  use windows::Win32::UI::WindowsAndMessaging::{
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
  };

  #[test]
  fn the_island_style_survives_a_rewrite_of_the_word() {
    // A style written without the island's own bits: topmost alone.
    let rewritten = WS_EX_TOPMOST.0 as isize;
    let restored = island_style(rewritten, true);
    assert_ne!(restored, rewritten);
    for bit in [
      WS_EX_TRANSPARENT,
      WS_EX_LAYERED,
      WS_EX_NOACTIVATE,
      WS_EX_TOPMOST,
    ] {
      assert_ne!(restored & bit.0 as isize, 0);
    }
    // Restored, it holds; taking presses keeps the island unactivated.
    assert_eq!(island_style(restored, true), restored);
    let taking = island_style(restored, false);
    assert_eq!(taking & WS_EX_TRANSPARENT.0 as isize, 0);
    assert_ne!(taking & WS_EX_NOACTIVATE.0 as isize, 0);
  }
}
