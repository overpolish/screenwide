// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What DWM draws round a window that draws its own look: its corner and
//! the rim Windows 11 gives it.

use tauri::WebviewWindow;
use windows::Win32::{
  Foundation::HWND,
  Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_WINDOW_CORNER_PREFERENCE,
    DWMWCP_ROUND,
  },
};

/// Shapes a floating panel (a pop-up list, the tooltip) as a Fluent flyout:
/// the 8px overlay corner, which DWM gives every framed window on Windows 11
/// but not an undecorated one, and is the only corner that clips the
/// window's own material and shadow. The light rim DWM draws round a rounded
/// window goes: the panel paints its own faint ring.
pub(super) fn shape_as_panel(window: &WebviewWindow) -> tauri::Result<()> {
  let preference = DWMWCP_ROUND;
  unsafe {
    DwmSetWindowAttribute(
      HWND(window.hwnd()?.0),
      DWMWA_WINDOW_CORNER_PREFERENCE,
      (&raw const preference).cast(),
      std::mem::size_of_val(&preference) as u32,
    )
  }
  .map_err(std::io::Error::other)?;
  remove_border(window)
}

/// Takes away the 1px rim Windows 11 draws round a window.
pub(super) fn remove_border(window: &WebviewWindow) -> tauri::Result<()> {
  let color = DWMWA_COLOR_NONE;
  unsafe {
    DwmSetWindowAttribute(
      HWND(window.hwnd()?.0),
      DWMWA_BORDER_COLOR,
      (&raw const color).cast(),
      std::mem::size_of_val(&color) as u32,
    )
  }
  .map_err(std::io::Error::other)?;
  Ok(())
}
