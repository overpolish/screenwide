// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CString;

use tauri::Manager;

use crate::{screenshots, windows::screenshot_region::native_osc_macos as native_region};

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
      let result = window
        .ns_view()
        .map_err(|error| error.to_string())
        .and_then(|view| {
          let view = view.cast();
          let size = window.inner_size().map_err(|error| error.to_string())?;
          let scale = window.scale_factor().map_err(|error| error.to_string())?;
          if !native_region::ensure_text_recognition_attached(
            view,
            window.clone(),
            f64::from(size.width) / scale,
            f64::from(size.height) / scale,
          ) {
            return Err("Could not attach the native text selection surface".to_owned());
          }
          let binding = native_region::configure_desktop_window(view, anchor_id)?;
          if !native_region::configure_desktop(view, binding, None)
            || !native_region::set_allow_drawing(view, true)
            || !native_region::set_aspect(view, None)
            || !native_region::set_show_frame(view, true)
            || !native_region::set_show_handles(view, false)
            || !native_region::set_input_enabled(view, true)
            || !native_region::set_ocr_cancel_visible(view, true)
          {
            return Err("Could not configure the native text selection surface".to_owned());
          }
          for (display_id, image) in &snapshots {
            if !native_region::set_snapshot(
              view,
              *display_id,
              &image.rgba,
              image.width,
              image.height,
            ) {
              return Err(format!("Could not install frozen display {display_id}"));
            }
          }
          let _ = native_region::set_snapshot_presented(view, true);
          native_region::present_region(view, None)
            .then_some(())
            .ok_or_else(|| "Could not present the native text selection surface".to_owned())
        });
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
        let _ = native_region::set_desktop_presented(view.cast(), true);
        let _ = native_region::claim_pointer_surface(view.cast());
      }
    })
    .map_err(|error| error.to_string())
}

pub(super) fn show_interactive(window: &tauri::WebviewWindow) -> Result<(), String> {
  let window = window.clone();
  let app = window.app_handle().clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result = window
        .ns_window()
        .map_err(|error| error.to_string())
        .map(|raw_window| {
          let native_window: &objc2_app_kit::NSWindow = unsafe { &*raw_window.cast() };
          native_window.makeKeyAndOrderFront(None);
        });
      let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?
}

fn close_windows(windows: Vec<tauri::WebviewWindow>) {
  let empty = CString::new("").expect("empty OCR status");
  for window in windows {
    if let Ok(view) = window.ns_view() {
      let view = view.cast();
      let _ = native_region::set_input_enabled(view, false);
      let _ = native_region::set_ocr(view, VisualPhase::Idle as u32, &[], &empty);
      let _ = native_region::set_snapshot_presented(view, false);
      let _ = native_region::clear_region(view);
      let _ = native_region::set_desktop_presented(view, false);
    }
    let _ = window.close();
  }
}

pub(super) fn close(app: &tauri::AppHandle, except: Option<&str>) {
  let windows = super::recognition_windows(app)
    .into_iter()
    .filter(|window| Some(window.label()) != except)
    .collect::<Vec<_>>();
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

pub(super) fn render(app: &tauri::AppHandle, packet: RenderPacket) {
  update_surfaces(app, packet);
}

pub(super) fn render_window(window: &tauri::WebviewWindow, packet: RenderPacket) {
  if let Ok(view) = window.ns_view() {
    let _ = apply_packet(view.cast(), packet);
  }
}

fn apply_packet(view: *mut std::ffi::c_void, packet: RenderPacket) -> bool {
  let message = CString::new(packet.message.replace('\0', " ")).expect("sanitized OCR status");
  if !native_region::set_ocr(view, packet.phase as u32, &packet.rects, &message) {
    return false;
  }
  let presentation = packet.presentation;
  if let Some(frame) = presentation.frame {
    let _ = native_region::set_show_frame(view, frame);
  }
  if presentation.reset {
    let _ = native_region::reset_text_recognition_input(view);
  }
  if let Some(input) = presentation.input {
    let _ = native_region::set_input_enabled(view, input);
  }
  if presentation.claim_crosshair {
    let _ = native_region::claim_pointer_surface(view);
  }
  true
}

fn update_surfaces(app: &tauri::AppHandle, packet: RenderPacket) {
  let windows = super::recognition_windows(app);
  let app = app.clone();
  let _ = app.clone().run_on_main_thread(move || {
    for window in windows {
      let Ok(view) = window.ns_view() else { continue };
      if apply_packet(view.cast(), packet.clone()) {
        break;
      }
    }
  });
}
