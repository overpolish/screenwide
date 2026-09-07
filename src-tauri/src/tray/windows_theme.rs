// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{WM_NCDESTROY, WM_SETTINGCHANGE, WM_THEMECHANGED};

const SUBCLASS_ID: usize = 0x5343_5245;

/// Install the native hook that refreshes the taskbar icon when Windows
/// changes its theme. Popup menu icons are recoloured as they open.
pub fn install(tray: &tauri::tray::TrayIcon) -> tauri::Result<()> {
  let app = tray.app_handle().clone();
  tray.with_inner_tray_icon(move |inner| {
    let hwnd = inner.window_handle();
    let data = Box::into_raw(Box::new(app)) as usize;
    let installed = unsafe {
      SetWindowSubclass(
        windows::Win32::Foundation::HWND(hwnd),
        Some(subclass_proc),
        SUBCLASS_ID,
        data,
      )
    };
    if installed.as_bool() {
      Ok(())
    } else {
      unsafe {
        drop(Box::from_raw(data as *mut AppHandle));
      }
      Err(tauri::Error::Io(std::io::Error::last_os_error()))
    }
  })?
}

unsafe extern "system" fn subclass_proc(
  hwnd: HWND,
  message: u32,
  _wparam: WPARAM,
  _lparam: LPARAM,
  subclass_id: usize,
  ref_data: usize,
) -> LRESULT {
  if message == WM_SETTINGCHANGE || message == WM_THEMECHANGED {
    let app = &*(ref_data as *const AppHandle);
    if let Some(tray) = app.tray_by_id(super::TRAY_ID) {
      if let Ok(icon) = super::status_icon(crate::recording::snapshot(app).status) {
        let _ = tray.set_icon(Some(icon));
      }
    }
  }
  if message == windows::Win32::UI::WindowsAndMessaging::WM_INITMENUPOPUP {
    crate::tray::windows_menu::prepare(windows::Win32::UI::WindowsAndMessaging::HMENU(
      _wparam.0 as *mut _,
    ));
  }

  if message == WM_NCDESTROY {
    let _ = RemoveWindowSubclass(hwnd, Some(subclass_proc), subclass_id);
    crate::tray::windows_menu::cleanup();
    drop(Box::from_raw(ref_data as *mut AppHandle));
  }
  DefSubclassProc(hwnd, message, _wparam, _lparam)
}
