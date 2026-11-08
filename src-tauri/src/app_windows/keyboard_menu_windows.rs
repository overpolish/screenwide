// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Alt pressed and let go on its own asks Windows for the window's menu bar.
//! Screenwide's windows have none, so nothing shows, but the window enters
//! the system's keyboard menu mode, and while it lasts the app's event loop
//! runs none of its queued work: the page's calls to Rust wait, the webview
//! keeps its old size and a shortcut's tool never opens. Each window drops
//! that request. Alt+Space, which names the window menu itself, still opens
//! it.

use tauri::plugin::{Builder, TauriPlugin};
use tauri::Wry;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{SC_KEYMENU, WM_NCDESTROY, WM_SYSCOMMAND};

const SUBCLASS_ID: usize = 0x5357_4b4d;

/// Guards every window the app builds, as it is built.
pub(crate) fn plugin() -> TauriPlugin<Wry> {
  Builder::new("keyboard-menu")
    .on_window_ready(|window| {
      let Ok(hwnd) = window.hwnd() else {
        return;
      };
      let installed =
        unsafe { SetWindowSubclass(HWND(hwnd.0), Some(subclass_proc), SUBCLASS_ID, 0) };
      if !installed.as_bool() {
        eprintln!(
          "Could not guard {} against the keyboard menu: {}",
          window.label(),
          std::io::Error::last_os_error()
        );
      }
    })
    .build()
}

/// Whether a `WM_SYSCOMMAND` is the menu bar asked for by Alt alone: the
/// command is `SC_KEYMENU` and no character came with it. Alt+Space carries
/// the space and opens the window menu.
const fn is_bare_alt_menu(command: usize, character: isize) -> bool {
  command & 0xFFF0 == SC_KEYMENU as usize && character == 0
}

unsafe extern "system" fn subclass_proc(
  hwnd: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
  subclass_id: usize,
  _ref_data: usize,
) -> LRESULT {
  if message == WM_SYSCOMMAND && is_bare_alt_menu(wparam.0, lparam.0) {
    return LRESULT(0);
  }
  if message == WM_NCDESTROY {
    let _ = RemoveWindowSubclass(hwnd, Some(subclass_proc), subclass_id);
  }
  DefSubclassProc(hwnd, message, wparam, lparam)
}

#[cfg(test)]
mod tests {
  use super::is_bare_alt_menu;
  use windows::Win32::UI::WindowsAndMessaging::{SC_CLOSE, SC_KEYMENU};

  #[test]
  fn only_alt_alone_is_kept_from_the_keyboard_menu() {
    // Alt pressed and released by itself.
    assert!(is_bare_alt_menu(SC_KEYMENU as usize, 0));
    // The low four bits are the system's own and say nothing of the command.
    assert!(is_bare_alt_menu(SC_KEYMENU as usize | 0x2, 0));
    // Alt+Space opens the window menu, as on every other window.
    assert!(!is_bare_alt_menu(SC_KEYMENU as usize, b' ' as isize));
    assert!(!is_bare_alt_menu(SC_CLOSE as usize, 0));
  }
}
