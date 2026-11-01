// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;

/// Makes a panel an ordinary window, as one attached to a parent window has
/// to be: at the ordinary level, so it sits with its parent in the window
/// order instead of floating over every other application, and managed in
/// Spaces, so AppKit moves it with its parent. A panel that joins every
/// Space stays put through a Spaces swipe while its parent slides away, and
/// only vanishes once AppKit orders the stray child out after the swipe.
/// `restore_recording_level` puts the floating level and Spaces back.
pub fn set_normal_level(window: &WebviewWindow) -> tauri::Result<()> {
  set_level(window, PanelLevel::Normal.value())?;
  set_collection_behavior(
    window,
    CollectionBehavior::new().managed().full_screen_auxiliary(),
  )
}

pub fn restore_recording_level(window: &WebviewWindow) -> tauri::Result<()> {
  let Some(level) = recording_panel_level(window) else {
    return Ok(());
  };
  set_level(window, PanelLevel::Custom(level).value())?;
  set_collection_behavior(window, recording_collection_behavior())
}

/// Raises a floating panel over the capture overlays, or puts the ordinary
/// floating level back. The island takes it while Annotate draws, so no
/// stroke covers its tools, and the tooltip while the overlay is up, so it
/// is not hidden under what is drawn.
pub fn set_above_capture_overlays(window: &WebviewWindow, above: bool) -> tauri::Result<()> {
  let level = if above {
    i64::from(crate::capture_overlays::FOREGROUND_LEVEL as i32 + 2)
  } else {
    PanelLevel::Floating.value()
  };
  set_level(window, level)
}

fn set_level(window: &WebviewWindow, level: i64) -> tauri::Result<()> {
  let panel = ensure_recording_panel(window)?;
  // Export cleanup can close tool panels from an async command. AppKit's
  // window-level transaction must share the main queue with detach/hide.
  window
    .app_handle()
    .run_on_main_thread(move || panel.set_level(level))
}

fn set_collection_behavior(
  window: &WebviewWindow,
  behavior: CollectionBehavior,
) -> tauri::Result<()> {
  let panel = ensure_recording_panel(window)?;
  let behavior = behavior.into();
  window
    .app_handle()
    .run_on_main_thread(move || panel.set_collection_behavior(behavior))
}

pub fn set_opacity(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  let panel = ensure_recording_panel(window)?;
  let app = window.app_handle().clone();
  app.run_on_main_thread(move || panel.set_alpha_value(opacity))
}

/// Whether this panel is the one WindowServer is sending keystrokes to. Main
/// thread only.
pub fn is_key_panel(window: &WebviewWindow) -> tauri::Result<bool> {
  Ok(registered_panel(window)?.as_panel().isKeyWindow())
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
