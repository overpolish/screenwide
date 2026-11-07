// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The windows built at launch and how they come up: the island, which must
//! appear the moment a shortcut asks. The rest are built when wanted
//! (`super::on_demand`).
//!
//! Held here rather than in the application's setup closure, so that adding a
//! window is a change to the window layer alone.

use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri::Manager;

use super::{hide_instead_of_close, WindowLabel};

pub fn initialize_predefined_windows(app: &AppHandle) -> tauri::Result<()> {
  // The island is only ever hidden, never closed.
  hide_instead_of_close(app, WindowLabel::Island);

  // Predefined pages start hidden, including panels converted lazily later.
  #[cfg(target_os = "macos")]
  for window in app.webview_windows().into_values() {
    if !window.is_visible()? {
      super::webview_visibility::hide_webview(&window)?;
    }
  }

  Ok(())
}
