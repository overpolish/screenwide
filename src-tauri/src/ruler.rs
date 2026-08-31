// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::{capture_overlays, screenshots, windows::WindowLabel};

pub(crate) mod analysis;
pub(crate) mod centerlines;
#[cfg(target_os = "macos")]
mod native_overlay_macos;
pub(crate) mod probe;
pub(crate) mod radius;
mod screenshot_mode;
pub(crate) mod snapshot;
pub(crate) mod viewport;
pub use snapshot::RulerState;

fn ruler_windows(app: &AppHandle) -> Vec<tauri::WebviewWindow> {
  app
    .get_webview_window(WindowLabel::Ruler.as_str())
    .into_iter()
    .collect()
}

fn close_ruler_windows(app: &AppHandle) {
  #[cfg(target_os = "macos")]
  native_overlay_macos::close(app);
  #[cfg(not(target_os = "macos"))]
  for window in ruler_windows(app) {
    let _ = window.close();
  }
}

pub fn dismiss(app: &AppHandle) {
  screenshot_mode::reset();
  let had_windows = !ruler_windows(app).is_empty();
  close_ruler_windows(app);
  let had_capture = app.state::<RulerState>().cancel();
  if had_windows || had_capture {
    capture_overlays::emit_lifecycle(app, false);
  }
  crate::windows::sync_recording_ui_escape(app, false);
}

pub fn is_active(app: &AppHandle) -> bool {
  app.state::<RulerState>().active_generation().is_some()
}

#[cfg(target_os = "macos")]
pub(crate) fn restart_after_topology_change(app: &AppHandle) {
  let Some(generation) = app.state::<RulerState>().active_generation() else {
    return;
  };
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    if !app.state::<RulerState>().is_current(generation) {
      return;
    }
    if let Err(error) = start(&app).await {
      eprintln!("Could not rebuild Ruler after a display change: {error}");
    }
  });
}

#[tauri::command]
pub async fn set_ruler_screenshot_mode(app: AppHandle, active: bool) -> Result<(), String> {
  screenshot_mode::set(&app, active).await
}

pub async fn start(app: &AppHandle) -> Result<(), String> {
  #[cfg(not(target_os = "macos"))]
  {
    let _ = app;
    return Ok(());
  }

  #[cfg(target_os = "macos")]
  {
    dismiss(app);
    capture_overlays::dismiss_except(app, Some(capture_overlays::CaptureOverlay::Ruler));
    let generation = app.state::<RulerState>().begin();
    let result = start_macos(app, generation).await;
    if result.is_err() {
      dismiss(app);
    }
    result
  }
}

#[cfg(target_os = "macos")]
async fn start_macos(app: &AppHandle, generation: u64) -> Result<(), String> {
  let monitors = capture_overlays::monitor_layout(app)?;
  let mut snapshots = Vec::with_capacity(monitors.len());
  for (monitor_id, _, _) in &monitors {
    snapshots.push((
      *monitor_id,
      screenshots::capture_overlay_snapshot(*monitor_id).await?,
    ));
  }
  if !app.state::<RulerState>().is_current(generation) {
    return Ok(());
  }
  let (anchor_id, anchor_scale, anchor_monitor) = monitors
    .first()
    .ok_or_else(|| "No monitor is available for Ruler".to_owned())?;
  let position = anchor_monitor.position().to_logical::<f64>(*anchor_scale);
  let size = anchor_monitor.size().to_logical::<f64>(*anchor_scale);
  let window = WebviewWindowBuilder::new(
    app,
    WindowLabel::Ruler.as_str(),
    WebviewUrl::App("/ruler".into()),
  )
  .accept_first_mouse(true)
  .always_on_top(true)
  .decorations(false)
  .focused(false)
  .inner_size(size.width, size.height)
  .position(position.x, position.y)
  .resizable(false)
  .shadow(false)
  .skip_taskbar(true)
  .transparent(true)
  .visible(false)
  .visible_on_all_workspaces(true)
  .build()
  .map_err(|error| error.to_string())?;
  capture_overlays::set_level(&window, capture_overlays::FOREGROUND_LEVEL)?;
  let binding = native_overlay_macos::install(&window, *anchor_id, &snapshots)?;
  if !app
    .state::<RulerState>()
    .install(generation, &binding.displays, &snapshots)
  {
    return Ok(());
  }
  native_overlay_macos::show_without_activation(&window)?;
  native_overlay_macos::present(&window)?;
  capture_overlays::emit_lifecycle(app, true);
  crate::windows::sync_recording_ui_escape(app, true);
  Ok(())
}

#[tauri::command]
pub fn cancel_ruler(app: AppHandle) {
  dismiss(&app);
}

pub fn start_detached(app: &AppHandle) {
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    if let Err(error) = start(&app).await {
      eprintln!("Could not start ruler overlay: {error}");
    }
  });
}
