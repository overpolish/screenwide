// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod adapter;
pub(crate) mod magnifier;
#[cfg(target_os = "macos")]
pub(crate) mod native_osc_macos;
#[cfg(target_os = "windows")]
pub(crate) mod native_osc_windows;
pub(crate) mod osc_command;
pub(crate) mod presentation;

pub(super) fn acquire_quick_screenshot_cursor(app: &tauri::AppHandle) -> Result<(), String> {
  adapter::acquire_quick_screenshot_cursor(app)
}

pub(super) fn release_quick_screenshot_cursor(app: &tauri::AppHandle) -> Result<(), String> {
  adapter::release_quick_screenshot_cursor(app)
}

pub(super) fn set_recording_overlay_desktop_presented(
  window: &tauri::WebviewWindow,
  presented: bool,
) -> tauri::Result<()> {
  adapter::set_desktop_presented(window, presented)
}

#[cfg(target_os = "windows")]
pub(super) fn set_recording_overlay_capture_affinity(
  window: &tauri::WebviewWindow,
  capturable: bool,
) -> tauri::Result<()> {
  adapter::set_capture_affinity(window, capturable)
}

/// Stops Quick Screenshot input immediately while retaining its last visual
/// frame until the normal Region scene atomically replaces it.
pub(super) fn prepare_recording_overlay_for_region_restore(
  window: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  adapter::prepare_for_region_restore(window)
}

/// Atomically hides the shared native surfaces and clears their transient
/// region before Quick Screenshot is allowed to re-present the window.
pub(super) fn prepare_recording_overlay_for_screenshot(
  window: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  adapter::prepare_for_screenshot(window)
}
