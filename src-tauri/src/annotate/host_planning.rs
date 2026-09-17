// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the overlay is about to be drawn on, and opening the windows for it.

use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::{capture_overlays, windows::WindowLabel};

/// One host's geometry, read from the monitor layout before any window is
/// built: xcap's display handles are not `Send`, so nothing here may await.
pub(crate) struct HostPlan {
  /// The capture display this host covers, which is how the toolbar
  /// recognises the screen it was last dropped on.
  pub(crate) display_id: u32,
  pub(crate) position: tauri::LogicalPosition<f64>,
  pub(crate) size: tauri::LogicalSize<f64>,
  /// The display less the menu bar, the notch and the Dock. The hosts cover
  /// the whole screen, but anything the user has to read or press - the
  /// toolbar - belongs inside this.
  pub(crate) work_position: tauri::LogicalPosition<f64>,
  pub(crate) work_size: tauri::LogicalSize<f64>,
}

/// The anchor keeps the plain label so the window is recognisable; the peers
/// are numbered after it.
fn label(index: usize) -> String {
  let anchor = WindowLabel::Annotate.as_str();
  if index == 0 {
    anchor.to_owned()
  } else {
    format!("{anchor}-{index}")
  }
}

/// Reads the layout and tells the native side what it will be drawing on.
pub(crate) fn plan(app: &AppHandle) -> Result<Vec<HostPlan>, String> {
  let monitors = capture_overlays::monitor_layout(app)?;
  if monitors.is_empty() {
    return Err("No monitor is available for Annotate".to_owned());
  }
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let mut displays = Vec::with_capacity(monitors.len());
  let mut plans = Vec::with_capacity(monitors.len());
  for (display_id, scale, monitor) in &monitors {
    let position = monitor.position().to_logical::<f64>(*scale);
    let size = monitor.size().to_logical::<f64>(*scale);
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    displays.push(crate::annotate::native_overlay::Display {
      origin: (position.x, position.y),
      scale: *scale,
    });
    let work_area = monitor.work_area();
    plans.push(HostPlan {
      display_id: *display_id,
      position,
      size,
      work_position: work_area.position.to_logical::<f64>(*scale),
      work_size: work_area.size.to_logical::<f64>(*scale),
    });
  }
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  crate::annotate::native_overlay::set_displays(displays);
  Ok(plans)
}

/// Opens one display's host, hidden. Presentation is a separate step so every
/// window is in place before any of them takes the foreground.
pub(crate) fn build(
  app: &AppHandle,
  index: usize,
  host: &HostPlan,
) -> Result<WebviewWindow, String> {
  let window = WebviewWindowBuilder::new(app, label(index), WebviewUrl::App("/annotate".into()))
    .accept_first_mouse(true)
    .always_on_top(true)
    .decorations(false)
    .focused(false)
    .inner_size(host.size.width, host.size.height)
    .position(host.position.x, host.position.y)
    .resizable(false)
    .shadow(false)
    .skip_taskbar(true)
    .transparent(true)
    .visible(false)
    .visible_on_all_workspaces(true)
    .build()
    .map_err(|error| error.to_string())?;
  capture_overlays::set_level(&window, capture_overlays::FOREGROUND_LEVEL)?;
  // macOS capture excludes this process's own windows through its content
  // filter, so the host is out of every Screenwide capture and visible to
  // everything else without being told.
  #[cfg(not(target_os = "windows"))]
  crate::windows::exclude_from_capture(&window).map_err(|error| error.to_string())?;
  #[cfg(target_os = "windows")]
  super::apply_capture_affinity(&window)?;
  Ok(window)
}
