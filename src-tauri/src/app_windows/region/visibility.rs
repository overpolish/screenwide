// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Showing, hiding and focusing the region overlay, and whether it takes
//! clicks. On Windows this goes around tao: tao shows a window with
//! `SW_SHOW`, and any change it makes to a shown window's flags shows it
//! again the same way, each time activating it once Windows lets the app take
//! the foreground, as it does after the overlay's first click. Activated, the
//! overlay's web view takes Win+Shift+S before the system-keys hook sees it,
//! so Snipping Tool opens instead of the session closing. The overlay is
//! shown without activation, its clicks are switched in place, and it is
//! focused only where the keyboard is wanted: a region gesture and a window
//! pick. tao's own record then says the window is hidden while it is shown,
//! which is why hiding also goes through tao, and focusing does not.

use tauri::{AppHandle, Manager, WebviewWindow};

use super::super::{platform, WindowLabel};

#[cfg(not(target_os = "macos"))]
pub(super) fn show(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  #[cfg(target_os = "windows")]
  {
    platform::show_without_activation(window)?;
    platform::set_opacity(window, opacity)
  }
  #[cfg(not(target_os = "windows"))]
  platform::show(window, opacity)
}

pub(super) fn hide(window: &WebviewWindow) -> tauri::Result<()> {
  #[cfg(target_os = "windows")]
  platform::hide_without_activation(window)?;
  platform::hide(window)
}

pub(super) fn set_passthrough(window: &WebviewWindow, passthrough: bool) -> tauri::Result<()> {
  #[cfg(target_os = "windows")]
  return platform::set_pointer_passthrough(window, passthrough);
  #[cfg(not(target_os = "windows"))]
  window.set_ignore_cursor_events(passthrough)
}

pub(crate) fn focus(window: &WebviewWindow) -> tauri::Result<()> {
  #[cfg(target_os = "windows")]
  return platform::activate(window);
  #[cfg(not(target_os = "windows"))]
  window.set_focus()
}

/// Hands the keyboard to the overlay while a window is picked, for the keys
/// that step through the windows and choose one.
#[tauri::command]
pub fn focus_region_selector(app: AppHandle) -> tauri::Result<()> {
  match app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
    Some(region) => focus(&region),
    None => Ok(()),
  }
}
