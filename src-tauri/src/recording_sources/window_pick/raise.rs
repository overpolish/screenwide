// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window picked comes to the front, so whoever is recording sees it and
//! can work in it without bringing it forward themselves.

use super::targets::Target;

#[cfg(target_os = "macos")]
pub(super) fn raise(target: &Target) {
  match accessible_window(target) {
    Some(window) => crate::window_front::raise(&window),
    // Without Accessibility, or when the app doesn't list the window, the app
    // still comes forward, with its own front window.
    None => crate::window_front::activate(target.pid as i32),
  }
}

/// The picked window as Accessibility knows it: the window of its app at the
/// same frame. Both are in points from the top left of the main display.
#[cfg(target_os = "macos")]
fn accessible_window(target: &Target) -> Option<cidre::arc::R<cidre::ax::UiElement>> {
  use cidre::{arc, ax, cf};
  if !ax::is_process_trusted() {
    return None;
  }
  let app = ax::UiElement::with_app_pid(target.pid as i32);
  let _ = app.set_messaging_timeout_secs(0.25);
  let value = app.attr_value(ax::attr::windows()).ok()?;
  // SAFETY: `AXWindows` holds an array of window elements.
  let windows: arc::R<cf::ArrayOf<ax::UiElement>> = unsafe { cf::Type::retain(&value) };
  let wanted = [
    f64::from(target.position.x),
    f64::from(target.position.y),
    f64::from(target.size.width),
    f64::from(target.size.height),
  ];
  windows
    .iter()
    .find(|window| {
      window
        .frame()
        .ok()
        .and_then(|frame| frame.cg_rect())
        .is_some_and(|frame| {
          let found = [
            frame.origin.x,
            frame.origin.y,
            frame.size.width,
            frame.size.height,
          ];
          found
            .iter()
            .zip(wanted)
            .all(|(found, wanted)| (found - wanted).abs() <= 1.0)
        })
    })
    .map(|window| window.retained())
}

/// Raising is best effort: an elevated window, such as Task Manager, may turn
/// it down from a process that isn't, as User Interface Privilege Isolation
/// lets it.
#[cfg(target_os = "windows")]
pub(super) fn raise(target: &Target) {
  use windows::Win32::Foundation::HWND;
  // `xcap` lists a window by its handle cut to 32 bits, which Windows keeps
  // meaningful: a handle is safe to truncate and sign-extend back.
  let hwnd = HWND(target.id as i32 as isize as *mut core::ffi::c_void);
  crate::window_front::raise(hwnd);
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn raise(_target: &Target) {}
