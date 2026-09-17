// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Who holds the keyboard while the overlay is up.
//!
//! The anchor host owns it, which is what keeps the letters that pick up a tool
//! working while the toolbar is used. A field on the toolbar is the exception:
//! there is nothing to type into until its window is key, so the page says when
//! one is focused and the panel - which becomes key only when a view asks - is
//! given key status for exactly that long.
//!
//! Giving it back cannot be left to AppKit. The overlay's event monitor
//! swallows every press on a host, so none of them reaches AppKit and the host
//! can never win key status back on its own: every way out of a field names the
//! anchor host explicitly.

#[cfg(target_os = "macos")]
use objc2_app_kit::NSWindow;
use tauri::AppHandle;

/// The toolbar has a field focused and needs the keystrokes.
#[cfg(target_os = "macos")]
pub(crate) fn take(app: &AppHandle) -> Result<(), String> {
  let Some(window) = super::toolbar(app) else {
    return Ok(());
  };
  crate::windows::platform::take_key_focus(&window).map_err(|error| error.to_string())
}

/// The field is done with, or a press landed on the picture: the keyboard goes
/// back to the anchor host.
///
/// Asked on every press, and deliberately not remembered: AppKit gives the
/// panel key status of its own accord when a press lands in a field, so what
/// this page asked for is not what the window server is doing. The check that
/// decides whether there is anything to hand back therefore reads AppKit, and
/// every press re-asserts the handover rather than trusting that an earlier one
/// took. A press is once per stroke, so the lookups it costs are not on any
/// path that repeats.
#[cfg(target_os = "macos")]
pub(crate) fn give_back(app: &AppHandle) {
  let Some(toolbar) = super::toolbar(app) else {
    return;
  };
  let Some(anchor) = super::super::host::anchor(app) else {
    return;
  };
  let result = app.run_on_main_thread(move || {
    if !crate::windows::platform::is_key_panel(&toolbar).unwrap_or(false) {
      return;
    }
    // The host is already on screen and the application already active - the
    // overlay's own lease saw to both - so this moves key status and nothing
    // else.
    if let Ok(raw) = anchor.ns_window() {
      let native: &NSWindow = unsafe { &*raw.cast() };
      native.makeKeyWindow();
    }
  });
  if let Err(error) = result {
    eprintln!("Could not reach the main thread to move keyboard focus: {error}");
  }
}

/// Windows has no panel that can refuse key status: the toolbar is a
/// `WS_EX_NOACTIVATE` window, which is a different model for the same problem
/// and is left to the platform.
#[cfg(not(target_os = "macos"))]
pub(crate) fn take(_app: &AppHandle) -> Result<(), String> {
  Ok(())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn give_back(_app: &AppHandle) {}
