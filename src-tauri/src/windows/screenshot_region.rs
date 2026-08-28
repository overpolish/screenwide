// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(target_os = "macos")]
use tauri::Manager;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

use crate::capture_overlays;

const WINDOW_PREFIX: &str = "screenshot-region-";

#[cfg(target_os = "macos")]
fn set_system_cursor(app: &AppHandle, crosshair: bool) -> Result<(), String> {
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      if crosshair {
        objc2_app_kit::NSCursor::crosshairCursor().set();
        unsafe extern "C" {
          fn screenwide_arm_screenshot_initial_crosshair_guard();
        }
        unsafe {
          screenwide_arm_screenshot_initial_crosshair_guard();
        }
      } else {
        unsafe extern "C" {
          fn screenwide_disarm_screenshot_initial_crosshair_guard();
        }
        unsafe {
          screenwide_disarm_screenshot_initial_crosshair_guard();
        }
        objc2_app_kit::NSCursor::arrowCursor().set();
      }
      let _ = sender.send(());
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())
}

pub(super) fn set_opacity(window: &tauri::WebviewWindow, opacity: f64) -> Result<(), String> {
  #[cfg(target_os = "macos")]
  {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let window = window.clone();
    let app = window.app_handle().clone();
    app
      .run_on_main_thread(move || {
        let result = window.ns_window().map(|raw_window| {
          let native_window: &objc2_app_kit::NSWindow = unsafe { &*raw_window.cast() };
          native_window.setAlphaValue(opacity);
        });
        let _ = sender.send(result);
      })
      .map_err(|error| error.to_string())?;
    receiver
      .recv()
      .map_err(|error| error.to_string())?
      .map_err(|error| error.to_string())
  }

  #[cfg(not(target_os = "macos"))]
  super::platform::set_opacity(window, opacity).map_err(|error| error.to_string())
}

fn release_overlay_input(app: &AppHandle) {
  for window in capture_overlays::windows(app, WINDOW_PREFIX) {
    let _ = window.set_ignore_cursor_events(true);
  }
}

fn dispose_overlays(app: &AppHandle) {
  release_overlay_input(app);
  capture_overlays::close_windows(app, WINDOW_PREFIX, None);
}

/// Opens a disposable full-monitor region selector for every connected
/// monitor. Unlike the recording region selector, these windows deliberately
/// do not use the selected recording monitor: a screenshot shortcut is a
/// desktop-wide operation.
#[tauri::command]
pub fn open_screenshot_region_overlays(app: AppHandle, destination: String) -> Result<(), String> {
  if !matches!(destination.as_str(), "export" | "clipboard") {
    return Err(format!("Unsupported screenshot destination: {destination}"));
  }

  dispose_overlays(&app);
  let monitors = capture_overlays::monitor_layout(&app)?;

  for (index, (monitor_id, scale, monitor)) in monitors.into_iter().enumerate() {
    let position = monitor.position().to_logical::<f64>(scale);
    let size = monitor.size().to_logical::<f64>(scale);
    let label = format!("{WINDOW_PREFIX}{index}");
    let window = WebviewWindowBuilder::new(
      &app,
      label,
      WebviewUrl::App(
        format!("/region-selector?monitorId={monitor_id}&destination={destination}").into(),
      ),
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

    // Web cursor styles are not resolved until the first pointer update. Set
    // the native window default before showing.
    window
      .set_cursor_icon(tauri::CursorIcon::Crosshair)
      .map_err(|error| error.to_string())?;

    // The overlay is an interactive selection surface, and must remain out
    // of recordings even when the user's preference includes app windows.
    window
      .set_ignore_cursor_events(false)
      .map_err(|error| error.to_string())?;
    crate::windows::exclude_from_capture(&window).map_err(|error| error.to_string())?;
    #[cfg(not(target_os = "windows"))]
    window
      .set_content_protected(true)
      .map_err(|error| error.to_string())?;
    capture_overlays::set_level(&window, capture_overlays::FOREGROUND_LEVEL)?;
    crate::windows::show(&window, index == 0).map_err(|error| error.to_string())?;
    // AppKit/WebKit replaces a cursor assigned during window construction as
    // presentation completes. Queue this after `show` so the crosshair wins
    // that initial cursor-rect update without waiting for physical movement.
    #[cfg(target_os = "macos")]
    set_system_cursor(&app, true)?;
  }

  Ok(())
}

#[tauri::command]
pub fn close_screenshot_region_overlays(app: AppHandle) {
  #[cfg(target_os = "macos")]
  let _ = set_system_cursor(&app, false);
  // The invoking WebView still owns the rest of the screenshot-session
  // cleanup. Release pointer input synchronously, return the IPC response,
  // then dispose the now-pass-through windows after that cleanup can finish.
  release_overlay_input(&app);
  tauri::async_runtime::spawn(async move {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    capture_overlays::close_windows(&app, WINDOW_PREFIX, None);
  });
}
