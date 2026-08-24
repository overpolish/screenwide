// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows host-window behavior specific to the ruler overlay.

use tauri::{Manager, WebviewWindow};
use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, WPARAM},
  UI::{
    Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
    WindowsAndMessaging::{SC_KEYMENU, WM_NCDESTROY, WM_SYSCOMMAND},
  },
};

const RULER_SUBCLASS_ID: usize = 0x5357_5255;
const SYSTEM_COMMAND_MASK: usize = 0xfff0;

/// A bare Alt or F10 release is passed to `DefWindowProc`, which turns it into
/// `SC_KEYMENU` with no mnemonic character. On a frameless ruler that enters a
/// menu mode with no menu to display, withholding pointer movement until the
/// user clicks back into the webview. Alt+letter commands carry the letter in
/// `lparam` and are deliberately left alone.
fn is_bare_menu_key_command(message: u32, wparam: usize, lparam: isize) -> bool {
  message == WM_SYSCOMMAND && wparam & SYSTEM_COMMAND_MASK == SC_KEYMENU as usize && lparam == 0
}

unsafe extern "system" fn ruler_subclass_proc(
  hwnd: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
  _subclass_id: usize,
  _reference_data: usize,
) -> LRESULT {
  if is_bare_menu_key_command(message, wparam.0, lparam.0) {
    return LRESULT(0);
  }
  if message == WM_NCDESTROY {
    let _ = unsafe { RemoveWindowSubclass(hwnd, Some(ruler_subclass_proc), RULER_SUBCLASS_ID) };
  }
  unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

/// Prevents Win32's menu-key mode without consuming Alt key events themselves,
/// so WebView2 still reports the modifier to the ruler's Option-key hook.
pub(super) fn suppress_menu_key_mode(window: &WebviewWindow) -> Result<(), String> {
  let hwnd = window.hwnd().map_err(|error| error.to_string())?.0 as usize;
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  window
    .app_handle()
    .run_on_main_thread(move || {
      let hwnd = HWND(hwnd as *mut std::ffi::c_void);
      let installed =
        unsafe { SetWindowSubclass(hwnd, Some(ruler_subclass_proc), RULER_SUBCLASS_ID, 0) }
          .as_bool();
      let _ = sender.send(installed);
    })
    .map_err(|error| error.to_string())?;
  match receiver.recv().map_err(|error| error.to_string())? {
    true => Ok(()),
    false => Err("Windows could not install the ruler menu-key handler".to_owned()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use windows::Win32::UI::WindowsAndMessaging::SC_CLOSE;

  #[test]
  fn suppresses_only_menu_mode_without_a_mnemonic() {
    assert!(is_bare_menu_key_command(
      WM_SYSCOMMAND,
      SC_KEYMENU as usize,
      0
    ));
    assert!(!is_bare_menu_key_command(
      WM_SYSCOMMAND,
      SC_KEYMENU as usize,
      isize::from(b'F')
    ));
    assert!(!is_bare_menu_key_command(
      WM_SYSCOMMAND,
      SC_CLOSE as usize,
      0
    ));
    assert!(!is_bare_menu_key_command(
      WM_SYSCOMMAND - 1,
      SC_KEYMENU as usize,
      0
    ));
  }
}
