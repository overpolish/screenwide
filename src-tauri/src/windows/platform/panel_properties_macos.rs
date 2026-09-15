// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Drops a panel to the ordinary window level, so one attached to a parent
/// window sits with it in the window order instead of floating over every
/// other application. `restore_recording_level` puts the floating level back.
pub fn set_normal_level(window: &WebviewWindow) -> tauri::Result<()> {
  set_level(window, PanelLevel::Normal.value())
}

pub fn restore_recording_level(window: &WebviewWindow) -> tauri::Result<()> {
  let Some(level) = recording_panel_level(window) else {
    return Ok(());
  };
  set_level(window, PanelLevel::Custom(level).value())
}

fn set_level(window: &WebviewWindow, level: i64) -> tauri::Result<()> {
  let panel = ensure_recording_panel(window)?;
  // Export cleanup can close tool panels from an async command. AppKit's
  // window-level transaction must share the main queue with detach/hide.
  window
    .app_handle()
    .run_on_main_thread(move || panel.set_level(level))
}

pub fn set_opacity(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  let panel = ensure_recording_panel(window)?;
  let app = window.app_handle().clone();
  app.run_on_main_thread(move || panel.set_alpha_value(opacity))
}

/// Hands keyboard focus back to whatever app owned it before this panel took
/// key status, without hiding the overlay.
///
/// `resignKeyWindow` is a notification AppKit sends itself; calling it directly
/// tells the window it lost focus while WindowServer still routes keystrokes
/// here. For a non-activating panel the only public way to actually give focus
/// up is to leave the window list and come back: ordering the key panel out
/// makes AppKit pick a new key window - the frontmost app's, since this process
/// is not active - and `orderFrontRegardless` then puts the overlay back on
/// screen without asking for key again.
///
/// A panel that is not key is left alone; ordering it out and in would be a
/// pointless flicker.
pub fn release_key_focus(window: &WebviewWindow) -> tauri::Result<()> {
  let panel = ensure_recording_panel(window)?;
  let app = window.app_handle().clone();
  app.run_on_main_thread(move || {
    if !panel.as_panel().isKeyWindow() {
      return;
    }
    // `Panel::hide` is `orderOut:nil` and `Panel::show` is
    // `orderFrontRegardless`; neither touches alpha, so the overlay stays as
    // visible as it was.
    panel.hide();
    panel.show();
  })
}
