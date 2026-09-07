// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::WebviewWindow;
use windows::Win32::{
  Foundation::{COLORREF, HWND},
  Graphics::Gdi::{RedrawWindow, RDW_ALLCHILDREN, RDW_ERASE, RDW_INVALIDATE, RDW_UPDATENOW},
  UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE,
    LWA_ALPHA, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    WS_EX_LAYERED,
  },
};

pub(super) fn set_opacity(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  let hwnd = HWND(window.hwnd()?.0);
  unsafe {
    let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED.0 as isize);
    SetLayeredWindowAttributes(
      hwnd,
      COLORREF(0),
      (opacity.clamp(0.0, 1.0) * 255.0).round() as u8,
      LWA_ALPHA,
    )
    .map_err(std::io::Error::other)?;
  }
  Ok(())
}

/// Rebuilds the first transparent WebView/Mica composition after layered
/// alpha and capture affinity have been applied. Without the frame refresh,
/// Windows can present the initial WebView surface as an opaque white shell.
pub(super) fn refresh(window: &WebviewWindow) -> tauri::Result<()> {
  let hwnd = HWND(window.hwnd()?.0);
  unsafe {
    SetWindowPos(
      hwnd,
      None,
      0,
      0,
      0,
      0,
      SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
    )
    .map_err(std::io::Error::other)?;
    RedrawWindow(
      Some(hwnd),
      None,
      None,
      RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_UPDATENOW,
    )
    .ok()
    .map_err(std::io::Error::other)?;
  }
  Ok(())
}
