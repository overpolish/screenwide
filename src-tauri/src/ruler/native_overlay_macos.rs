// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::Manager;

use crate::{
  osc::geometry::Rect, screenshots::CapturedImage,
  windows::screenshot_region::native_osc_macos as native_region,
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
      let result = window
        .ns_view()
        .map_err(|error| error.to_string())
        .and_then(|view| {
          let view = view.cast();
          let size = window.inner_size().map_err(|error| error.to_string())?;
          let scale = window.scale_factor().map_err(|error| error.to_string())?;
          if !native_region::ensure_ruler_attached(
            view,
            window.clone(),
            f64::from(size.width) / scale,
            f64::from(size.height) / scale,
          ) {
            return Err("Could not attach the native Ruler surface".to_owned());
          }
          let binding = native_region::configure_desktop_window(view, anchor_id)?;
          let desktop = Rect::from_xywh(0.0, 0.0, binding.size.width, binding.size.height);
          if !native_region::configure_desktop(view, binding.clone(), None)
            || !native_region::set_allow_drawing(view, false)
            || !native_region::set_aspect(view, None)
            || !native_region::set_show_frame(view, false)
            || !native_region::set_show_handles(view, false)
            || !native_region::set_snapshot_composited(view, true)
            || !native_region::set_input_enabled(view, true)
          {
            return Err("Could not configure the native Ruler surface".to_owned());
          }
          for (display_id, image) in &snapshots {
            if !native_region::set_snapshot(
              view,
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
          if !native_region::set_snapshot_presented(view, true)
            || !native_region::present_region(view, Some(desktop))
          {
            return Err("Could not present the native Ruler surface".to_owned());
          }
          Ok(binding)
        });
      let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?
}

pub(super) fn show_interactive(window: &tauri::WebviewWindow) -> Result<(), String> {
  let window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result =
        crate::osc::cursor::macos::present_window(&window).map_err(|error| error.to_string());
      let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?
}

pub(super) fn present(window: &tauri::WebviewWindow) -> Result<(), String> {
  let window = window.clone();
  let app = window.app_handle().clone();
  app
    .run_on_main_thread(move || {
      if let Ok(view) = window.ns_view() {
        let view = view.cast();
        let _ = native_region::set_desktop_presented(view, true);
        let _ = native_region::claim_pointer_surface(view);
        let _ = native_region::refresh_ruler_pointer(view);
      }
    })
    .map_err(|error| error.to_string())
}

pub(super) fn set_screenshot_mode(
  window: &tauri::WebviewWindow,
  active: bool,
) -> Result<(), String> {
  if !active {
    // Resume the native surface only after WindowServer routes pointer events
    // to it again. Claiming while the Tauri window is still passthrough leaves
    // transient Ruler chrome dormant until the first click.
    window
      .set_ignore_cursor_events(false)
      .map_err(|error| error.to_string())?;
  }
  let window = window.clone();
  let native_window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result = native_window
        .ns_view()
        .map_err(|error| error.to_string())
        .map(|view| {
          let view = view.cast();
          let _ = native_region::set_ruler_transient_chrome(view, !active);
          let _ = native_region::set_input_enabled(view, !active);
          if let Ok(raw_window) = native_window.ns_window() {
            let panel: &objc2_app_kit::NSWindow = unsafe { &*raw_window.cast() };
            if active {
              panel.resignKeyWindow();
            } else {
              panel.makeKeyAndOrderFront(None);
            }
          }
          if !active {
            let _ = native_region::claim_pointer_surface(view);
            let _ = native_region::refresh_ruler_pointer(view);
          }
        });
      let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())??;
  if active {
    window
      .set_ignore_cursor_events(true)
      .map_err(|error| error.to_string())?;
  }
  Ok(())
}

fn close_windows(windows: Vec<tauri::WebviewWindow>) {
  for window in windows {
    crate::osc::cursor::macos::prepare_window_close(&window);
    if let Ok(view) = window.ns_view() {
      let view = view.cast();
      let _ = native_region::set_input_enabled(view, false);
      let _ = native_region::set_snapshot_presented(view, false);
      let _ = native_region::clear_region(view);
      let _ = native_region::set_desktop_presented(view, false);
    }
    let _ = window.close();
  }
}

pub(super) fn close(app: &tauri::AppHandle) {
  let windows = super::ruler_windows(app);
  if objc2::MainThreadMarker::new().is_some() {
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
