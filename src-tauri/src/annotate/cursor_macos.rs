// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeping the crosshair over the overlay's transparent host.
//!
//! The cursor lease sets the crosshair once, and that is enough for a native
//! surface. This host is a webview: WebKit asks for the arrow cursor again on
//! every pointer move over it, and the arrow wins. The region overlay settled
//! this once - turn the window's cursor rectangles off and let the shared
//! `NSCursor set` guard substitute the crosshair for any arrow request - and
//! this is the same claim for a host that has no native view of its own yet.

use std::ffi::c_void;

use objc2::rc::Retained;
use objc2_app_kit::{NSCursor, NSWindow};
use tauri::WebviewWindow;

unsafe extern "C" {
  /// `src/ruler/cursor_guard_macos.m`. Installs the `NSCursor set` guard and
  /// names the cursor an arrow request is replaced with; null clears it.
  fn screenwide_set_region_expected_cursor(cursor: *mut c_void);
}

fn with_window(window: &WebviewWindow, work: impl FnOnce(&NSWindow)) {
  if let Ok(raw) = window.ns_window() {
    work(unsafe { &*raw.cast() });
  }
}

/// Main thread only, while the host window still exists.
pub(super) fn claim(window: &WebviewWindow) {
  with_window(window, |native| native.disableCursorRects());
  let crosshair = NSCursor::crosshairCursor();
  unsafe {
    screenwide_set_region_expected_cursor(Retained::as_ptr(&crosshair).cast_mut().cast());
  }
  crosshair.set();
}

/// Main thread only, before the host window is closed. The lease's own release
/// puts the arrow back once the overlay is gone.
pub(super) fn release(window: &WebviewWindow) {
  unsafe { screenwide_set_region_expected_cursor(std::ptr::null_mut()) };
  with_window(window, |native| {
    native.enableCursorRects();
    native.resetCursorRects();
  });
}
