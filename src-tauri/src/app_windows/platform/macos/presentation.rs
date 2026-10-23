// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{Manager, WebviewWindow};
use tauri_nspanel::StyleMask;

use super::super::{ensure_recording_panel, registered_panel, restore_recording_level};
use crate::app_windows::webview_visibility::{hide_window, show_webview};

pub fn raise_without_activation(window: &WebviewWindow) -> tauri::Result<()> {
  let panel = ensure_recording_panel(window)?;
  show_webview(window)?;
  panel.show();
  restore_recording_level(window)
}

/// Presents a normally nonactivating overlay as the key window for one
/// explicit interactive-tool lease.
///
/// Recording UI must continue to use [`show`]. Only a cursor lease that has
/// already captured foreground ownership may enter this path, and it must call
/// [`restore_nonactivating_overlay`] before returning foreground ownership.
pub fn show_interactive_overlay(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  window.set_ignore_cursor_events(false)?;
  let panel = ensure_recording_panel(window)?;
  let app = window.app_handle().clone();
  let window = window.clone();
  app.run_on_main_thread(move || {
    let _ = show_webview(&window);
    panel.set_style_mask(StyleMask::empty().into());
    panel.set_alpha_value(opacity);
    if let Err(error) = crate::osc::cursor_macos::present_window(&window) {
      eprintln!("Could not present interactive overlay: {error}");
    }
  })
}

/// Returns a leased interactive overlay to the recording UI's nonactivating
/// presentation before the cursor lease restores the prior application.
pub fn restore_nonactivating_overlay(window: &WebviewWindow) -> tauri::Result<()> {
  let panel = ensure_recording_panel(window)?;
  let app = window.app_handle().clone();
  app.run_on_main_thread(move || {
    panel.resign_key_window();
    panel.resign_main_window();
    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
  })
}

pub fn hide(window: &WebviewWindow) -> tauri::Result<()> {
  let _ = window.eval("globalThis.__SCREENWIDE_INACTIVE_HOVER__?.clear()");
  window.set_ignore_cursor_events(true)?;
  let Ok(panel) = registered_panel(window) else {
    return hide_window(window);
  };
  let window = window.clone();
  let app = window.app_handle().clone();
  app.run_on_main_thread(move || {
    panel.set_alpha_value(0.0);
    let _ = hide_window(&window);
    panel.hide();
  })
}

/// Orders a recording panel onscreen without disturbing keyboard focus.
/// Tauri's `WebviewWindow::show` is `makeKeyAndOrderFront:` underneath. On a
/// non-activating panel that is the worst of both worlds: the app never
/// activates, but WindowServer still moves keyboard focus off whatever the user
/// was working in - Final Cut, a browser - every time recording starts. So this
/// never calls it. `Panel::show` is `orderFrontRegardless`, which puts the
/// panel on screen and leaves key status where it is.
///
/// Tao's `is_visible` asks the NSWindow, so callers see the panel as soon as it
/// is ordered front.
pub fn show(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  window.set_ignore_cursor_events(false)?;
  let panel = ensure_recording_panel(window)?;
  let app = window.app_handle().clone();
  let window = window.clone();
  app.run_on_main_thread(move || {
    let _ = show_webview(&window);
    panel.set_alpha_value(opacity);
    panel.show();
  })
}
