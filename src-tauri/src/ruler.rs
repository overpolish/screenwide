// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::{capture_overlays, screenshots};

pub(crate) mod analysis;
pub(crate) mod focus;
#[cfg(target_os = "windows")]
mod platform_windows;
mod screenshot_mode;
pub(crate) mod snapshot;
use focus::{follow_cursor_focus, watch_focus, FocusRegion};
pub use snapshot::RulerState;

const WINDOW_PREFIX: &str = "ruler-";

#[cfg(target_os = "macos")]
fn set_system_ruler_cursor_visible(app: &AppHandle, visible: bool) -> Result<(), String> {
  app
    .run_on_main_thread(move || unsafe {
      unsafe extern "C" {
        fn screenwide_set_ruler_cursor_visible(visible: std::ffi::c_int);
      }
      screenwide_set_ruler_cursor_visible(i32::from(visible));
    })
    .map_err(|error| error.to_string())
}

#[cfg(not(target_os = "macos"))]
fn set_system_ruler_cursor_visible(_app: &AppHandle, _visible: bool) -> Result<(), String> {
  Ok(())
}

/// Returns the cursor in this ruler window's CSS-pixel coordinate space. The
/// webview has not received a pointer event when it first opens, so seeding its
/// cursor state here lets the transient rulers paint on the initial frame.
#[tauri::command]
pub fn get_ruler_cursor_position(app: AppHandle, window: tauri::WebviewWindow) -> Option<[f64; 2]> {
  let cursor = app.cursor_position().ok()?;
  let origin = window.outer_position().ok()?;
  let scale = window.scale_factor().ok()?;

  #[cfg(target_os = "macos")]
  let (x, y) = {
    // macOS reports the global cursor in primary-monitor physical pixels, but
    // window origins use their own monitor scale. Convert both to the shared
    // logical desktop space before subtracting.
    let primary_scale = app
      .primary_monitor()
      .ok()
      .flatten()
      .map_or(1.0, |monitor| monitor.scale_factor());
    let origin = origin.to_logical::<f64>(scale);
    (
      cursor.x / primary_scale - origin.x,
      cursor.y / primary_scale - origin.y,
    )
  };

  #[cfg(not(target_os = "macos"))]
  let (x, y) = (
    (cursor.x - f64::from(origin.x)) / scale,
    (cursor.y - f64::from(origin.y)) / scale,
  );

  let size = window.inner_size().ok()?.to_logical::<f64>(scale);
  (x >= 0.0 && x < size.width && y >= 0.0 && y < size.height).then_some([x, y])
}

#[tauri::command]
pub fn set_ruler_cursor_visible(
  app: AppHandle,
  window: tauri::WebviewWindow,
  visible: bool,
) -> Result<(), String> {
  set_system_ruler_cursor_visible(&app, visible)?;
  window
    .set_cursor_visible(visible)
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_ruler_cursor_range_active(app: AppHandle, active: bool) -> Result<(), String> {
  #[cfg(target_os = "macos")]
  app
    .run_on_main_thread(move || unsafe {
      unsafe extern "C" {
        fn screenwide_set_ruler_cursor_range_active(active: std::ffi::c_int);
      }
      screenwide_set_ruler_cursor_range_active(i32::from(active));
    })
    .map_err(|error| error.to_string())?;
  #[cfg(not(target_os = "macos"))]
  let _ = (app, active);
  Ok(())
}

fn ruler_windows(app: &AppHandle) -> Vec<tauri::WebviewWindow> {
  capture_overlays::windows(app, WINDOW_PREFIX)
}

fn close_ruler_windows(app: &AppHandle) {
  capture_overlays::close_windows(app, WINDOW_PREFIX, None);
}

pub fn dismiss(app: &AppHandle) {
  let _ = set_system_ruler_cursor_visible(app, true);
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
  !ruler_windows(app).is_empty()
}

#[tauri::command]
pub async fn set_ruler_screenshot_mode(app: AppHandle, active: bool) -> Result<(), String> {
  screenshot_mode::set(&app, active).await
}

pub async fn start(app: &AppHandle) -> Result<(), String> {
  dismiss(app);
  capture_overlays::dismiss_except(app, Some(capture_overlays::CaptureOverlay::Ruler));
  let generation = app.state::<RulerState>().begin();

  let monitors = capture_overlays::monitor_layout(app)?;
  let mut snapshots = Vec::with_capacity(monitors.len());
  for (monitor_id, scale, _) in &monitors {
    let image = screenshots::capture_overlay_snapshot(*monitor_id).await?;
    snapshots.push((*monitor_id, *scale, image));
  }
  if !app.state::<RulerState>().install(generation, snapshots) {
    return Ok(());
  }

  let mut regions = Vec::new();
  for (index, (monitor_id, scale, monitor)) in monitors.into_iter().enumerate() {
    let position = monitor.position().to_logical::<f64>(scale);
    let size = monitor.size().to_logical::<f64>(scale);
    // Logical coordinates on macOS (mixed-DPI physical rects share no global
    // space); physical elsewhere - matching `focus::poll_cursor`.
    #[cfg(target_os = "macos")]
    regions.push(FocusRegion {
      height: size.height,
      label: format!("{WINDOW_PREFIX}{index}"),
      width: size.width,
      x: position.x,
      y: position.y,
    });
    #[cfg(not(target_os = "macos"))]
    regions.push(FocusRegion {
      height: f64::from(monitor.size().height),
      label: format!("{WINDOW_PREFIX}{index}"),
      width: f64::from(monitor.size().width),
      x: f64::from(monitor.position().x),
      y: f64::from(monitor.position().y),
    });
    let window = WebviewWindowBuilder::new(
      app,
      format!("{WINDOW_PREFIX}{index}"),
      WebviewUrl::App(format!("/ruler?monitorId={monitor_id}").into()),
    )
    .accept_first_mouse(true)
    .always_on_top(true)
    .decorations(false)
    .focused(index == 0)
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
    // CSS cursor changes are not applied by WebKit until its first pointer
    // update. Hide natively before showing so invocation has no arrow flash.
    window
      .set_cursor_visible(false)
      .map_err(|error| error.to_string())?;
    set_system_ruler_cursor_visible(app, false)?;
    #[cfg(target_os = "windows")]
    platform_windows::suppress_menu_key_mode(&window)?;
    // Deliberately shareable: the screenshot shortcut can preserve the ruler
    // and capture its annotations as part of the selected desktop region.
    capture_overlays::set_level(&window, capture_overlays::FOREGROUND_LEVEL)?;
    // Switching to any other app - or any other window of ours - ends the
    // session, exactly as the cancel command does.
    watch_focus(&window);
    crate::windows::show(&window, index == 0).map_err(|error| error.to_string())?;
  }
  follow_cursor_focus(app, regions);

  capture_overlays::emit_lifecycle(app, true);
  crate::windows::sync_recording_ui_escape(app, true);

  Ok(())
}

pub fn start_detached(app: &AppHandle) {
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    if let Err(error) = start(&app).await {
      eprintln!("Could not start ruler overlay: {error}");
    }
  });
}

#[tauri::command]
pub async fn start_ruler(app: AppHandle) -> Result<(), String> {
  start(&app).await
}

#[tauri::command]
pub fn cancel_ruler(app: AppHandle) {
  dismiss(&app);
}

#[tauri::command]
pub fn copy_ruler_value(app: AppHandle, value: String) -> Result<(), String> {
  app
    .clipboard()
    .write_text(value)
    .map_err(|error| error.to_string())
}
