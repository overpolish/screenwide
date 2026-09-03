// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(target_os = "macos")]
use tauri::Manager;
use tauri::{AppHandle, Emitter};

use crate::windows::WindowLabel;

/// Sits immediately above the recording Dock's native macOS level (32).
pub const FOREGROUND_LEVEL: isize = 33;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureOverlay {
  Ruler,
  TextRecognition,
}

/// Gives capture overlays a stable order above the recording controls on
/// macOS. Other platforms retain their existing always-on-top behavior.
pub fn set_level(window: &tauri::WebviewWindow, level: isize) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  crate::windows::initialize_capture_overlay(window).map_err(|error| error.to_string())?;

  #[cfg(target_os = "macos")]
  {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let window = window.clone();
    let app = window.app_handle().clone();
    app
      .run_on_main_thread(move || {
        let result = window
          .ns_window()
          .map_err(|error| error.to_string())
          .map(|raw_window| {
            let native_window: &objc2_app_kit::NSWindow = unsafe { &*raw_window.cast() };
            native_window.setLevel(level);
          });
        let _ = sender.send(result);
      })
      .map_err(|error| error.to_string())?;
    receiver.recv().map_err(|error| error.to_string())?
  }

  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = (window, level);
    Ok(())
  }

  #[cfg(target_os = "windows")]
  {
    let _ = level;
    Ok(())
  }
}

/// Extension point for capture tools that must be mutually exclusive while
/// still allowing one overlay to survive a handoff such as ruler screenshots.
pub fn dismiss_except(app: &AppHandle, preserved: Option<CaptureOverlay>) {
  if preserved != Some(CaptureOverlay::TextRecognition) {
    crate::text_recognition::dismiss(app);
  }
  if preserved != Some(CaptureOverlay::Ruler) {
    crate::ruler::dismiss(app);
  }
}

pub fn dismiss_all(app: &AppHandle) {
  dismiss_except(app, None);
}

pub fn emit_lifecycle(app: &AppHandle, active: bool) {
  let event = if active {
    "capture-overlay://started"
  } else {
    "capture-overlay://ended"
  };
  let _ = app.emit_to(WindowLabel::RecordingBar.as_str(), event, ());
}

// xcap::Monitor wraps a raw display handle that is not Send on every
// platform, so the command future may never hold one across an await.
// Enumerating synchronously drops the handles before the first snapshot.
pub fn monitor_layout(app: &AppHandle) -> Result<Vec<(u32, f64, tauri::Monitor)>, String> {
  let capture_monitors = xcap::Monitor::all().map_err(|error| error.to_string())?;
  let tauri_monitors = app
    .available_monitors()
    .map_err(|error| error.to_string())?;
  if capture_monitors.len() != tauri_monitors.len() {
    return Err("Tauri and xcap returned different monitor counts".to_owned());
  }

  capture_monitors
    .into_iter()
    .zip(tauri_monitors)
    .map(|(capture_monitor, monitor)| {
      let monitor_id = capture_monitor.id().map_err(|error| error.to_string())?;
      Ok((monitor_id, monitor.scale_factor(), monitor))
    })
    .collect()
}

/// Resolves a capture display ID without assuming that xcap and Tauri return
/// monitors in the same order. Both APIs accept the OS desktop coordinate
/// space, so a point inside the xcap display gives Tauri the matching monitor
/// on macOS and Windows alike.
pub fn monitor_by_capture_id(
  app: &AppHandle,
  display_id: u32,
) -> Result<Option<(f64, tauri::Monitor)>, String> {
  let monitor = xcap::Monitor::all()
    .map_err(|error| error.to_string())?
    .into_iter()
    .find(|monitor| monitor.id().ok() == Some(display_id));
  let Some(monitor) = monitor else {
    return Ok(None);
  };
  let x = f64::from(monitor.x().map_err(|error| error.to_string())?);
  let y = f64::from(monitor.y().map_err(|error| error.to_string())?);
  let width = f64::from(monitor.width().map_err(|error| error.to_string())?);
  let height = f64::from(monitor.height().map_err(|error| error.to_string())?);
  let scale = f64::from(monitor.scale_factor().map_err(|error| error.to_string())?);
  let matched = app
    .monitor_from_point(x + width / 2.0, y + height / 2.0)
    .map_err(|error| error.to_string())?;
  Ok(matched.map(|monitor| (scale, monitor)))
}
