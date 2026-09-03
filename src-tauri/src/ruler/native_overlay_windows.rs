// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows twin of `native_overlay_macos.rs`. The compositor calls are the
//! same in the same order; only the handle differs — the Windows module keys
//! its contexts by the Tauri window rather than by `ns_view()`, so the window
//! itself is passed straight through.

use tauri::Manager;

use crate::{
  osc::geometry::Rect, screenshots::CapturedImage,
  windows::screenshot_region::native_osc_windows as native_region,
};

pub(super) fn install(
  window: &tauri::WebviewWindow,
  anchor_id: u32,
  snapshots: &[(u32, CapturedImage)],
) -> Result<native_region::DesktopBinding, String> {
  let window = window.clone();
  let snapshots = snapshots.to_vec();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result = (|| -> Result<native_region::DesktopBinding, String> {
        let size = window.inner_size().map_err(|error| error.to_string())?;
        let scale = window.scale_factor().map_err(|error| error.to_string())?;
        if !native_region::ensure_ruler_attached(
          &window,
          f64::from(size.width) / scale,
          f64::from(size.height) / scale,
        ) {
          return Err("Could not attach the native Ruler surface".to_owned());
        }
        let binding = native_region::configure_desktop_window(&window, anchor_id)?;
        let desktop = Rect::from_xywh(0.0, 0.0, binding.size.width, binding.size.height);
        if !native_region::configure_desktop(&window, binding.clone(), None)
          || !native_region::set_allow_drawing(&window, false)
          || !native_region::set_aspect(&window, None)
          || !native_region::set_show_frame(&window, false)
          || !native_region::set_show_handles(&window, false)
          || !native_region::set_snapshot_composited(&window, true)
          || !native_region::set_input_enabled(&window, true)
        {
          return Err("Could not configure the native Ruler surface".to_owned());
        }
        for (display_id, image) in &snapshots {
          if !native_region::set_snapshot(
            &window,
            *display_id,
            &image.rgba,
            image.width,
            image.height,
          ) {
            return Err(format!(
              "Could not install frozen Ruler display {display_id}"
            ));
          }
        }
        if !native_region::set_snapshot_presented(&window, true)
          || !native_region::present_region(&window, Some(desktop))
        {
          return Err("Could not present the native Ruler surface".to_owned());
        }
        let keyboard_target = native_region::input_hwnd(&window)
          .ok_or_else(|| "Could not locate the native Ruler input surface".to_owned())?;
        crate::osc::keyboard_windows::start(
          keyboard_target,
          crate::osc::keyboard_windows::Overlay::Ruler,
        )?;
        Ok(binding)
      })();
      let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?
}

/// Windows needs the two AppKit operations separately: show the host, then
/// activate it so the compositor child can take keyboard focus in `present`.
pub(super) fn show_interactive(window: &tauri::WebviewWindow) -> Result<(), String> {
  crate::windows::show(window, true).map_err(|error| error.to_string())?;
  window.set_focus().map_err(|error| error.to_string())
}

pub(super) fn present(window: &tauri::WebviewWindow) -> Result<(), String> {
  let window = window.clone();
  let app = window.app_handle().clone();
  app
    .run_on_main_thread(move || {
      let _ = native_region::set_desktop_presented(&window, true);
      let _ = native_region::claim_pointer_surface(&window);
      let _ = native_region::refresh_ruler_pointer(&window);
      let _ = native_region::focus_ruler_input(&window);
    })
    .map_err(|error| error.to_string())
}

/// While a screenshot is taken through the frozen desktop the overlay stops
/// taking input and hides its transient chrome, then resumes only once the
/// window is hit-testable again — claiming earlier leaves the readout dormant
/// until the first click.
pub(super) fn set_screenshot_mode(
  window: &tauri::WebviewWindow,
  active: bool,
) -> Result<(), String> {
  // Ruler's anchor compositor is a child of the Tauri host, while every
  // additional display is a top-level peer. Temporarily make both halves of
  // that window graph capturable so a cross-display shutter sees every
  // stamped artifact, regardless of the persistent app-window preference.
  let capturable =
    active || crate::settings::current(window.app_handle()).record_screenwide_windows;
  crate::windows::set_window_capture_affinity(window, capturable)
    .map_err(|error| error.to_string())?;
  if !native_region::set_capture_affinity(window, capturable) {
    return Err("Could not update the native Ruler capture affinity".to_owned());
  }
  if !active {
    window
      .set_ignore_cursor_events(false)
      .map_err(|error| error.to_string())?;
  }
  let window = window.clone();
  let target = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let _ = native_region::set_ruler_transient_chrome(&target, !active);
      let _ = native_region::set_input_enabled(&target, !active);
      if !active {
        let _ = crate::windows::show(&target, true);
        let _ = native_region::claim_pointer_surface(&target);
        let _ = native_region::refresh_ruler_pointer(&target);
      }
      let _ = sender.send(());
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?;
  if active {
    window
      .set_ignore_cursor_events(true)
      .map_err(|error| error.to_string())?;
  }
  Ok(())
}

/// Teardown order is load-bearing: the native surfaces must be concealed
/// before the webview that owns them closes, because the peers outlive it.
fn close_windows(windows: Vec<tauri::WebviewWindow>) {
  crate::osc::keyboard_windows::stop(crate::osc::keyboard_windows::Overlay::Ruler);
  for window in windows {
    let _ = native_region::set_input_enabled(&window, false);
    let _ = native_region::set_snapshot_presented(&window, false);
    let _ = native_region::clear_region(&window);
    let _ = native_region::set_desktop_presented(&window, false);
    let _ = window.close();
  }
}

/// True when this call is already on the thread that owns `window`. Blocking
/// on `run_on_main_thread` from that thread would deadlock, which is the
/// deadlock the macOS port avoided with `MainThreadMarker`.
fn on_owning_thread(window: &tauri::WebviewWindow) -> bool {
  use windows::Win32::{
    Foundation::HWND, System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::GetWindowThreadProcessId,
  };
  window.hwnd().is_ok_and(|hwnd| unsafe {
    GetWindowThreadProcessId(HWND(hwnd.0), None) == GetCurrentThreadId()
  })
}

pub(super) fn close(app: &tauri::AppHandle) {
  let windows = super::ruler_windows(app);
  if windows.first().is_some_and(on_owning_thread) {
    close_windows(windows);
    return;
  }
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  if app
    .run_on_main_thread(move || {
      close_windows(windows);
      let _ = sender.send(());
    })
    .is_ok()
  {
    let _ = receiver.recv();
  }
}
