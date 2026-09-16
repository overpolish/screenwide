// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// A sticky panel stands with the window it belongs to instead of floating
/// over every other application: it drops to the ordinary window level and
/// becomes a native child, the way the confirm sheet does.
#[cfg(target_os = "macos")]
pub(super) fn attach_to_parent(
  app: &AppHandle,
  parent: &tauri::WebviewWindow,
  panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  platform::set_normal_level(panel)?;
  crate::editor::export_window::presentation::attach(app, parent, panel)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn attach_to_parent(
  _app: &AppHandle,
  _parent: &tauri::WebviewWindow,
  _panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  Ok(())
}

/// Ordering a still-attached child out drags its parent with it, so this runs
/// before the panel is hidden.
#[cfg(target_os = "macos")]
pub(super) fn detach_from_parent(
  app: &AppHandle,
  parent: &tauri::WebviewWindow,
  panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  crate::editor::export_window::presentation::detach(app, parent, panel)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn detach_from_parent(
  _app: &AppHandle,
  _parent: &tauri::WebviewWindow,
  _panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  Ok(())
}

/// Owned top-level windows stay above their owner and minimize with it.
/// Unlike a modal export sheet, a tool panel never disables the editor.
#[cfg(target_os = "windows")]
pub(super) fn attach_to_parent(
  _app: &AppHandle,
  parent: &tauri::WebviewWindow,
  panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  platform::set_normal_level(panel)?;
  set_owner(panel, parent.hwnd()?.0 as isize)
}

#[cfg(target_os = "windows")]
pub(super) fn detach_from_parent(
  _app: &AppHandle,
  _parent: &tauri::WebviewWindow,
  panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  set_owner(panel, 0)
}

#[cfg(target_os = "windows")]
fn set_owner(panel: &tauri::WebviewWindow, owner: isize) -> tauri::Result<()> {
  use windows::Win32::{
    Foundation::{GetLastError, SetLastError, HWND, WIN32_ERROR},
    UI::WindowsAndMessaging::{SetWindowLongPtrW, GWLP_HWNDPARENT},
  };
  unsafe {
    SetLastError(WIN32_ERROR(0));
    let previous = SetWindowLongPtrW(HWND(panel.hwnd()?.0), GWLP_HWNDPARENT, owner);
    let error = GetLastError();
    if previous == 0 && error.0 != 0 {
      return Err(std::io::Error::from_raw_os_error(error.0 as i32).into());
    }
  }
  Ok(())
}
