// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Compile-time native host boundary for Ruler.
//!
//! The shared Ruler document owns analysis, gestures, viewports, artifacts,
//! labels, history and draw data. Platform adapters own native surfaces,
//! frozen-texture upload, GPU submission, cursors and window presentation.

#[cfg(target_os = "macos")]
#[path = "adapter/macos.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "adapter/windows.rs"]
mod platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "adapter/unavailable.rs"]
mod platform;

pub(super) use platform::{
  available, close, install, present, set_screenshot_mode, show_interactive,
};

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{osc::desktop::DesktopBinding, screenshots::CapturedImage};

  type Install =
    fn(&tauri::WebviewWindow, u32, &[(u32, CapturedImage)]) -> Result<DesktopBinding, String>;

  #[test]
  fn selected_platform_satisfies_the_ruler_surface_contract() {
    let _: Install = platform::install;
    let _: fn(&tauri::WebviewWindow) -> Result<(), String> = platform::present;
    let _: fn(&tauri::WebviewWindow, bool) -> Result<(), String> = platform::set_screenshot_mode;
    let _: fn(&tauri::AppHandle) = platform::close;
  }
}
