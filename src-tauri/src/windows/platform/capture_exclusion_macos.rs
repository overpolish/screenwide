// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeping one window's pixels out of every capture on macOS.
//!
//! A recording's ScreenCaptureKit filter is built once, out of the windows
//! this process owned when the capture started, so excluding by owning process
//! covers only the windows that already existed. A capture overlay opened
//! during a recording is not among them and is recorded like any other window.
//!
//! `NSWindowSharingNone` is the per-window answer, and WindowServer honours it
//! whenever the capture began. The region overlay's native peers already carry
//! it for the same reason.

use objc2_app_kit::{NSWindow, NSWindowSharingType};
use tauri::{Manager, WebviewWindow};

pub fn exclude_from_capture(window: &WebviewWindow) -> tauri::Result<()> {
  let window = window.clone();
  let app = window.app_handle().clone();
  // Applied on the next main-loop turn, which still lands before the window is
  // on screen: presentation queues its own turn behind this one.
  app.run_on_main_thread(move || {
    if let Ok(raw) = window.ns_window() {
      let native: &NSWindow = unsafe { &*raw.cast() };
      native.setSharingType(NSWindowSharingType::None);
    }
  })
}
