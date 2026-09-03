// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows twin of `native_overlay_macos.rs`. The compositor calls are the
//! same in the same order; only the handle differs — the Windows module keys
//! its contexts by the Tauri window rather than by `ns_view()`, so the window
//! itself is passed straight through.

use tauri::Manager;

use crate::{screenshots, windows::screenshot_region::native_osc_windows as native_region};

use super::visual::{RenderPacket, VisualPhase};

pub(super) fn install(
  window: &tauri::WebviewWindow,
  anchor_id: u32,
  snapshots: &[(u32, screenshots::CapturedImage)],
) -> Result<(), String> {
  let window = window.clone();
  let snapshots = snapshots.to_vec();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result = (|| -> Result<(), String> {
        let size = window.inner_size().map_err(|error| error.to_string())?;
        let scale = window.scale_factor().map_err(|error| error.to_string())?;
        if !native_region::ensure_text_recognition_attached(
          &window,
          f64::from(size.width) / scale,
          f64::from(size.height) / scale,
        ) {
          return Err("Could not attach the native text selection surface".to_owned());
        }
        let binding = native_region::configure_desktop_window(&window, anchor_id)?;
        if !native_region::configure_desktop(&window, binding, None)
          || !native_region::set_allow_drawing(&window, true)
          || !native_region::set_aspect(&window, None)
          || !native_region::set_show_frame(&window, true)
          || !native_region::set_show_handles(&window, false)
          || !native_region::set_input_enabled(&window, true)
        {
          return Err("Could not configure the native text selection surface".to_owned());
        }
        for (display_id, image) in &snapshots {
          if !native_region::set_snapshot(
            &window,
            *display_id,
            &image.rgba,
            image.width,
            image.height,
          ) {
            return Err(format!("Could not install frozen display {display_id}"));
          }
        }
        let _ = native_region::set_snapshot_presented(&window, true);
        native_region::present_region(&window, None)
          .then_some(())
          .ok_or_else(|| "Could not present the native text selection surface".to_owned())?;
        let keyboard_target = native_region::input_hwnd(&window)
          .ok_or_else(|| "Could not locate the native text-recognition input surface".to_owned())?;
        crate::osc::keyboard_windows::start(
          keyboard_target,
          crate::osc::keyboard_windows::Overlay::TextRecognition,
        )
      })();
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
      let _ = native_region::set_desktop_presented(&window, true);
      let _ = native_region::claim_pointer_surface(&window);
    })
    .map_err(|error| error.to_string())
}

/// `makeKeyAndOrderFront` has no Win32 spelling of its own; the shared window
/// helper already shows and focuses a Tauri window the same way.
pub(super) fn show_interactive(window: &tauri::WebviewWindow) -> Result<(), String> {
  crate::windows::show(window, true).map_err(|error| error.to_string())
}

/// Teardown order is load-bearing: the native surfaces must be concealed
/// before the webview that owns them closes, because the peers outlive it.
fn close_windows(windows: Vec<tauri::WebviewWindow>) {
  crate::osc::keyboard_windows::stop(crate::osc::keyboard_windows::Overlay::TextRecognition);
  for window in windows {
    let _ = native_region::set_input_enabled(&window, false);
    let _ = native_region::set_ocr(&window, VisualPhase::Idle as u32, &[], "");
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

pub(super) fn close(app: &tauri::AppHandle, except: Option<&str>) {
  let windows = super::recognition_windows(app)
    .into_iter()
    .filter(|window| Some(window.label()) != except)
    .collect::<Vec<_>>();
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

pub(super) fn render(app: &tauri::AppHandle, packet: RenderPacket) {
  update_surfaces(app, packet);
}

pub(super) fn render_window(window: &tauri::WebviewWindow, packet: RenderPacket) {
  let _ = apply_packet(window, packet);
}

/// The final leg into the compositor. The order is the macOS one
/// (`native_overlay_macos.rs:149`): geometry, then frame, reset, input and the
/// cursor claim.
fn apply_packet(window: &tauri::WebviewWindow, packet: RenderPacket) -> bool {
  let message = packet.message.replace('\0', " ");
  if !native_region::set_ocr(window, packet.phase as u32, &packet.rects, &message) {
    return false;
  }
  let presentation = packet.presentation;
  if let Some(frame) = presentation.frame {
    let _ = native_region::set_show_frame(window, frame);
  }
  if presentation.reset {
    let _ = native_region::reset_text_recognition_input(window);
  }
  if let Some(input) = presentation.input {
    let _ = native_region::set_input_enabled(window, input);
  }
  if presentation.claim_crosshair {
    let _ = native_region::claim_pointer_surface(window);
  }
  true
}

fn update_surfaces(app: &tauri::AppHandle, packet: RenderPacket) {
  let windows = super::recognition_windows(app);
  let _ = app.clone().run_on_main_thread(move || {
    for window in windows {
      if apply_packet(&window, packet.clone()) {
        break;
      }
    }
  });
}
