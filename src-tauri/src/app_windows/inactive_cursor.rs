// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pointer a page asks for while its window is not key. WebKit stops
//! following the pointer in a window that is not key, so the page fakes
//! hover from positions the window's tracking area hands it
//! (`enable_inactive_webview_hover`, `inactive-window-hover.ts`); it also
//! never sets the cursor there, so a tool panel's grab handle or text field
//! kept the arrow until the panel was pressed. The page names the CSS cursor
//! under the pointer and it is set here. A key window is left to WebKit.

use tauri::WebviewWindow;

/// Shows `cursor`, a CSS cursor name, over `window` while it is not key.
#[tauri::command]
#[cfg_attr(not(target_os = "macos"), allow(unused_variables))]
pub(crate) fn show_inactive_cursor(window: WebviewWindow, cursor: String) {
  #[cfg(target_os = "macos")]
  {
    let Ok(native) = window.ns_window().map(|native| native as usize) else {
      return;
    };
    let _ = window.run_on_main_thread(move || {
      use objc2::rc::Retained;
      use objc2_app_kit::{NSCursor, NSWindow};
      unsafe extern "C" {
        /// The frame-resize cursor for `edges` (1 left, 2 right, 4 top, 8
        /// bottom), in `osc/src/gpu/macos/region_cursor_macos.m`.
        fn screenwide_region_resize_cursor(edges: u32) -> *mut NSCursor;
      }
      // Safety: the helper returns an autoreleased cursor or null.
      let resize = |edges: u32| unsafe { Retained::retain(screenwide_region_resize_cursor(edges)) };
      // Safety: on the main thread, where the window lives as long as Tauri
      // keeps it.
      let native: &NSWindow = unsafe { &*(native as *const NSWindow) };
      if native.isKeyWindow() {
        return;
      }
      let shown = match cursor.as_str() {
        "grab" => NSCursor::openHandCursor(),
        "grabbing" => NSCursor::closedHandCursor(),
        "pointer" => NSCursor::pointingHandCursor(),
        "text" => NSCursor::IBeamCursor(),
        "crosshair" => NSCursor::crosshairCursor(),
        "ew-resize" | "col-resize" | "e-resize" | "w-resize" => {
          resize(1).unwrap_or_else(NSCursor::arrowCursor)
        }
        "ns-resize" | "row-resize" | "n-resize" | "s-resize" => {
          resize(4).unwrap_or_else(NSCursor::arrowCursor)
        }
        "not-allowed" => NSCursor::operationNotAllowedCursor(),
        _ => NSCursor::arrowCursor(),
      };
      shown.set();
    });
  }
}
