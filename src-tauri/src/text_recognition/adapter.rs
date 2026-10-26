// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Compile-time native host boundary for Text Recognition.
//!
//! Recognition state, selection geometry and visual snapshots stay in shared
//! Rust. Platform implementations own only native surfaces, compositor upload,
//! focus/cursor integration and presentation.

use tauri::Manager;

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod native;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use self::native as platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unavailable;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use self::unavailable as platform;

pub(super) use platform::{close, install, present, render, render_window, show_interactive};

pub(super) fn show_ready(app: &tauri::AppHandle, generation: u64) {
  let Some(snapshot) = app
    .state::<super::TextRecognitionState>()
    .visual_snapshot(generation)
  else {
    return;
  };
  render(app, super::visual::RenderPacket::ready(&snapshot));
  super::island::sync_ready(app);
}

/// Reading failed. The detail is the log's; the island says so in words.
pub(super) fn show_error(app: &tauri::AppHandle, message: &str) {
  eprintln!("Text recognition failed: {message}");
  render(app, super::visual::RenderPacket::choosing());
  super::island::choosing(app, true);
}

/// Another area is to be read: drawn over the desktop already frozen, so
/// the screen is not captured again.
pub(super) fn choose_again(app: &tauri::AppHandle) {
  app.state::<super::TextRecognitionState>().clear_selection();
  render(app, super::visual::RenderPacket::choosing());
  super::island::choosing(app, false);
}

#[cfg(test)]
mod tests {
  use super::*;
  use screenwide_imaging::CapturedImage;

  type Install = fn(&tauri::WebviewWindow, u32, &[(u32, CapturedImage)]) -> Result<bool, String>;

  #[test]
  fn selected_platform_satisfies_the_ocr_surface_contract() {
    let _: Install = platform::install;
    let _: fn(&tauri::AppHandle, super::super::visual::RenderPacket) = platform::render;
    let _: fn(&tauri::WebviewWindow, super::super::visual::RenderPacket) = platform::render_window;
    let _: fn(&tauri::WebviewWindow) -> Result<(), String> = platform::present;
    let _: fn(&tauri::AppHandle, Option<&str>) = platform::close;
  }
}
