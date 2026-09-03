// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Compile-time native host boundary for Text Recognition.
//!
//! Recognition state, selection geometry and visual snapshots stay in shared
//! Rust. Platform implementations own only native surfaces, compositor upload,
//! focus/cursor integration and presentation.

use tauri::Manager;

#[cfg(target_os = "macos")]
#[path = "adapter/macos.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "adapter/windows.rs"]
mod platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "adapter/unavailable.rs"]
mod platform;

pub(super) use platform::{close, install, present, render, render_window, show_interactive};

pub(super) fn show_ready(app: &tauri::AppHandle, generation: u64) {
  let Some(snapshot) = app
    .state::<super::TextRecognitionState>()
    .visual_snapshot(generation)
  else {
    return;
  };
  render(app, super::visual::RenderPacket::ready(&snapshot));
}

pub(super) fn show_error(app: &tauri::AppHandle, message: &str) {
  render(app, super::visual::RenderPacket::error(message));
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::screenshots::CapturedImage;

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
