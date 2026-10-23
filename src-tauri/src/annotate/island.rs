// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while the overlay draws: it carries the annotate tools, on an
//! island of their own or under the recording's row, and stands above the
//! overlay's hosts, so a stroke can never be drawn over the controls.
//!
//! The island is a non-activating panel, so a press on its controls leaves
//! the keyboard with the anchor host and the keys that draw keep working.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter};

use crate::app_windows::{self, WindowLabel};

/// The island is told when this changes (`annotate://drawing`), and asks for
/// it (`annotate_drawing`) when its page loads after the overlay opened.
static DRAWING: AtomicBool = AtomicBool::new(false);

/// The overlay is drawing: the island comes out with the tools, above the
/// hosts. Called after the hosts are presented, so on Windows, where there
/// are no window levels, raising the island puts it above them.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub(super) fn present(app: &AppHandle) {
  DRAWING.store(true, Ordering::Release);
  let _ = app.emit_to(WindowLabel::Island.as_str(), "annotate://drawing", true);
  if let Err(error) = app_windows::island::set_above_overlays(app, true) {
    eprintln!("Could not raise the island over the overlay: {error}");
  }
  #[cfg(target_os = "windows")]
  own_by_anchor(app, true);
  if let Err(error) = app_windows::island::show(app) {
    eprintln!("Could not show the island for Annotate: {error}");
  }
  // The pointer is an arrow over the island: the crosshair belongs to the
  // canvas, and the guard that keeps it there exempts this window.
  #[cfg(target_os = "macos")]
  if let Some(window) = app_windows::island::window(app) {
    let _ = app.run_on_main_thread(move || super::cursor::exempt_window(&window));
  }
}

/// The overlay stopped drawing. The island drops the tools and its place
/// above the hosts, and goes unless the recording or its controls keep it.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub(super) fn withdraw(app: &AppHandle) {
  if !DRAWING.swap(false, Ordering::AcqRel) {
    return;
  }
  let _ = app.emit_to(WindowLabel::Island.as_str(), "annotate://drawing", false);
  #[cfg(target_os = "macos")]
  let _ = app.run_on_main_thread(super::cursor::forget_exempt_window);
  // Released before the hosts close: a window takes the windows it owns
  // with it when it is destroyed.
  #[cfg(target_os = "windows")]
  own_by_anchor(app, false);
  if let Err(error) = app_windows::island::set_above_overlays(app, false) {
    eprintln!("Could not return the island to its level: {error}");
  }
  if crate::recording::is_idle(app) && !app_windows::is_recording_ui_visible() {
    if let Err(error) = app_windows::island::hide(app) {
      eprintln!("Could not hide the island after Annotate: {error}");
    }
  }
}

/// Windows has no window levels, and a press on a host raises it over every
/// other topmost window, so the island stands above the hosts by being owned
/// by the anchor host: an owned window always stands above its owner.
#[cfg(target_os = "windows")]
fn own_by_anchor(app: &AppHandle, owned: bool) {
  let Some(island) = app_windows::island::window(app) else {
    return;
  };
  let anchor = if owned {
    super::host::anchor(app)
  } else {
    None
  };
  if owned && anchor.is_none() {
    return;
  }
  if let Err(error) = app_windows::platform::set_owner(&island, anchor.as_ref()) {
    eprintln!("Could not set the island's owner: {error}");
  }
}

#[tauri::command]
pub fn annotate_drawing() -> bool {
  DRAWING.load(Ordering::Acquire)
}

/// A press on the island is done with: the keyboard goes back to the anchor
/// host, where the letters pick tools up.
///
/// Asked on every release rather than remembered: the island holds key
/// status only if a view on it took it, and AppKit decides that, so the check
/// reads AppKit. The overlay's event monitor swallows every press on a host,
/// so the host can never win key status back on its own.
#[cfg(target_os = "macos")]
pub(super) fn return_keyboard(app: &AppHandle) {
  use objc2_app_kit::NSWindow;

  let Some(island) = app_windows::island::window(app) else {
    return;
  };
  let Some(anchor) = super::host::anchor(app) else {
    return;
  };
  let result = app.run_on_main_thread(move || {
    if !app_windows::platform::is_key_panel(&island).unwrap_or(false) {
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

/// `WS_EX_NOACTIVATE` refuses activation, not focus: a press anywhere on the
/// island moves the keyboard into its web view, where the overlay's keys go
/// unheard. Giving it back focuses the anchor's overlay child again.
#[cfg(target_os = "windows")]
pub(super) fn return_keyboard(app: &AppHandle) {
  let result = app.run_on_main_thread(super::native_overlay::focus_input);
  if let Err(error) = result {
    eprintln!("Could not reach the main thread to move keyboard focus: {error}");
  }
}
