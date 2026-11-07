// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, WebviewUrl, WebviewWindow};

use crate::app_windows::{platform, WindowLabel};

use super::{INITIAL_HEIGHT, MAX_WIDTH};

/// The tooltip is drawn in the island's black by its page, as a menu opened
/// from an island is, so the window carries no material under it and no
/// shadow: AppKit draws that with a light rim, which reads as a border on
/// the black.
fn create(app: &AppHandle) -> tauri::Result<WebviewWindow> {
  crate::app_windows::webview_window(
    app,
    WindowLabel::Tooltip.as_str(),
    WebviewUrl::App("/tooltip".into()),
  )
  .inner_size(MAX_WIDTH, INITIAL_HEIGHT)
  .always_on_top(true)
  .decorations(false)
  .focused(false)
  .minimizable(false)
  .maximizable(false)
  .resizable(false)
  .shadow(false)
  .skip_taskbar(true)
  .transparent(true)
  .visible(false)
  .build()
}

fn set_up(window: &WebviewWindow) -> tauri::Result<()> {
  platform::initialize_tooltip(window)?;
  window.set_ignore_cursor_events(true)
}

#[cfg(not(target_os = "windows"))]
pub(super) fn build(app: &AppHandle) -> tauri::Result<WebviewWindow> {
  let window = create(app)?;
  set_up(&window)?;
  Ok(window)
}

/// Builds the window off the main thread, which WebView2 needs, and sets it up
/// on the main thread, which owns it. Asked again while building, it waits for
/// the build already under way.
#[cfg(target_os = "windows")]
pub(super) fn build_in_background(app: &AppHandle) {
  use std::sync::atomic::{AtomicBool, Ordering};

  static BUILDING: AtomicBool = AtomicBool::new(false);
  if BUILDING.swap(true, Ordering::AcqRel) {
    return;
  }
  let app = app.clone();
  std::thread::spawn(move || {
    let built = create(&app);
    BUILDING.store(false, Ordering::Release);
    let window = match built {
      Ok(window) => window,
      Err(error) => return eprintln!("Could not build the tooltip window: {error}"),
    };
    let main_app = app.clone();
    let _ = app.run_on_main_thread(move || {
      if let Err(error) = set_up(&window) {
        return eprintln!("Could not set up the tooltip window: {error}");
      }
      // Shown for the tooltip asked for while it was built, if one still is.
      if super::pending_tooltip(&main_app).is_some() {
        if let Err(error) = super::present_concealed(&main_app, &window) {
          eprintln!("Could not show the tooltip: {error}");
        }
      }
    });
  });
}
