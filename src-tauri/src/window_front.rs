// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bringing another application's window to the front: Glide raises the
//! window it picks up, and window picking the window picked to record.

#[cfg(target_os = "macos")]
use cidre::{ax, cf};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
#[cfg(target_os = "windows")]
use windows::Win32::{
  Foundation::HWND,
  System::Threading::{AttachThreadInput, GetCurrentThreadId},
  UI::WindowsAndMessaging::{
    BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, IsWindow, SetForegroundWindow,
  },
};

/// Activates the application, which brings its main window forward.
#[cfg(target_os = "macos")]
pub(crate) fn activate(pid: i32) {
  if let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) {
    #[allow(deprecated)]
    app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
  }
}

/// Raises the window above its application's other windows, then activates
/// the application. The window is made the application's main and key window
/// first: activating brings those forward, so another of its windows, behind
/// or on another display, would otherwise come forward as well.
#[cfg(target_os = "macos")]
pub(crate) fn raise(window: &ax::UiElement) {
  let mut chosen = window.retained();
  for attr in [ax::attr::main(), ax::attr::focused()] {
    if let Err(error) = chosen.set_attr(attr, cf::Boolean::value_true()) {
      eprintln!("Could not make the window its application's front window: {error}");
    }
  }
  if let Err(error) = window.perform_action(ax::action::raise()) {
    eprintln!("Could not raise the window: {error}");
  }
  if let Ok(pid) = window.pid() {
    activate(pid);
  }
}

/// Brings the window to the front. A background process may not take the
/// foreground on its own, so this thread briefly attaches to the thread that
/// owns it and hands it over.
#[cfg(target_os = "windows")]
pub(crate) fn raise(hwnd: HWND) {
  if !unsafe { IsWindow(Some(hwnd)) }.as_bool() {
    return;
  }
  let foreground = unsafe { GetForegroundWindow() };
  if foreground == hwnd {
    return;
  }
  let current = unsafe { GetCurrentThreadId() };
  let owner = if foreground.0.is_null() {
    0
  } else {
    unsafe { GetWindowThreadProcessId(foreground, None) }
  };
  let attached =
    owner != 0 && owner != current && unsafe { AttachThreadInput(current, owner, true) }.as_bool();
  let _ = unsafe { BringWindowToTop(hwnd) };
  let _ = unsafe { SetForegroundWindow(hwnd) };
  if attached {
    let _ = unsafe { AttachThreadInput(current, owner, false) };
  }
}
