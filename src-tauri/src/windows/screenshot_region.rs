// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(target_os = "macos")]
use tauri::Manager;

pub(crate) mod magnifier;
#[cfg(target_os = "macos")]
pub(crate) mod native_osc_macos;
pub(crate) mod osc_command;
pub(crate) mod presentation;

#[cfg(target_os = "macos")]
pub(super) fn set_recording_overlay_desktop_presented(
  window: &tauri::WebviewWindow,
  presented: bool,
) -> tauri::Result<()> {
  let window = window.clone();
  let app = window.app_handle().clone();
  app.run_on_main_thread(move || {
    if let Ok(view) = window.ns_view() {
      let _ = native_osc_macos::set_desktop_presented(view.cast(), presented);
    }
  })
}

/// Atomically hides the shared native surfaces and clears their transient
/// region before Quick Screenshot is allowed to re-present the window.
#[cfg(target_os = "macos")]
pub(super) fn prepare_recording_overlay_for_screenshot(
  window: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  let window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app.run_on_main_thread(move || {
    let result = window.ns_view().map(|view| {
      let _ = native_osc_macos::set_desktop_presented(view.cast(), false);
      let _ = native_osc_macos::clear_region(view.cast());
    });
    let _ = sender.send(result);
  })?;
  receiver.recv().unwrap_or(Err(tauri::Error::WindowNotFound))
}
