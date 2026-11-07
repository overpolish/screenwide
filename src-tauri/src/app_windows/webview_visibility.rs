// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::WebviewWindow;

/// Ordering an NSWindow out does not reliably stop WebKit's display links.
/// Hide the WKWebView too, and restore it before its next presentation. Use
/// the webview API: WebviewWindow::hide only hides the native window.
#[cfg(target_os = "macos")]
pub(super) fn hide_webview(window: &WebviewWindow) -> tauri::Result<()> {
  let webview: &tauri::Webview = window.as_ref();
  webview.hide()
}

/// Hiding the WKWebView took first responder from it. Hand it back directly
/// before the window becomes key: a window becoming key with no first
/// responder selects its first key view going forward, and WebKit answers a
/// forward selection as Tab into the page, focusing and selecting its first
/// tabbable element - the editor's file name, which then took the space meant
/// for playback.
#[cfg(target_os = "macos")]
pub(super) fn show_webview(window: &WebviewWindow) -> tauri::Result<()> {
  let webview: &tauri::Webview = window.as_ref();
  webview.show()?;
  webview.set_focus()
}

/// WebView2 goes on reading a hidden window's page as on screen, its
/// `visibilityState` "visible", until the WebView is hidden too. A page told
/// lets go of what it holds only while seen, and stops drawing.
#[cfg(target_os = "windows")]
pub(crate) fn set_page_shown(window: &WebviewWindow, shown: bool) -> tauri::Result<()> {
  let webview: &tauri::Webview = window.as_ref();
  if shown {
    webview.show()
  } else {
    webview.hide()
  }
}

pub(crate) fn hide_window(window: &WebviewWindow) -> tauri::Result<()> {
  window.hide()?;
  #[cfg(target_os = "macos")]
  hide_webview(window)?;
  Ok(())
}
