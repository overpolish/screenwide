// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CString;

use tauri::Manager;

use crate::{
  osc::geometry::Rect, screenshots, windows::screenshot_region::native_osc_macos as native_region,
};

use super::{interaction::TextAction, visual::VisualPhase, TextRecognitionState};

mod input;
pub(crate) use input::text_input;

#[derive(Clone, Copy)]
struct SurfaceMode {
  frame: bool,
  input: bool,
  reset: bool,
  claim_crosshair: bool,
}

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

pub(super) fn show_without_activation(window: &tauri::WebviewWindow) -> Result<(), String> {
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
          native_window.orderFrontRegardless();
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

pub(crate) fn selection_finished(
  window: tauri::WebviewWindow,
  binding: native_region::DesktopBinding,
  monitor_id: u32,
  region: Rect,
) {
  let app = window.app_handle().clone();
  let finishing_window = window.clone();
  let _ = app.run_on_main_thread(move || {
    if let Ok(view) = finishing_window.ns_view() {
      let _ = native_region::set_input_enabled(view.cast(), false);
      let _ = native_region::set_show_frame(view.cast(), false);
      let message = CString::new("Finding text and QR codes…").expect("static OCR status");
      let _ = native_region::set_ocr(view.cast(), VisualPhase::Loading as u32, &[], &message);
    }
  });
  tauri::async_runtime::spawn(async move {
    let capture_app = app.clone();
    let displays = binding.displays.clone();
    let selected = tauri::async_runtime::spawn_blocking(move || {
      capture_app
        .state::<TextRecognitionState>()
        .select_desktop_region(&displays, monitor_id, region)
    })
    .await;
    match selected {
      Ok(Ok(_)) => {}
      Ok(Err(error)) => return emit_error(&app, error),
      Err(error) => return emit_error(&app, error.to_string()),
    }
    if super::recognize_current(&app).await.is_err() {
      return;
    }
    finish_handoff(&app, window);
  });
}

fn emit_error(app: &tauri::AppHandle, error: String) {
  show_error(app, &error);
}

fn finish_handoff(app: &tauri::AppHandle, window: tauri::WebviewWindow) {
  if app.get_webview_window(window.label()).is_none() {
    return;
  }
  let main_app = app.clone();
  let _ = app.run_on_main_thread(move || {
    if let Some(target) = main_app.get_webview_window(window.label()) {
      let _ = target.set_ignore_cursor_events(false);
      let _ = crate::windows::show(&target, true);
    }
  });
}

pub(crate) fn selection_started(window: &tauri::WebviewWindow) {
  // Initial presentation deliberately avoids becoming key while a global
  // shortcut's modifier key is still being released. The first real overlay
  // interaction is the safe point to claim keyboard focus for copy/select-all.
  let _ = window.set_focus();
  if let Ok(view) = window.ns_view() {
    let empty = CString::new("").expect("empty OCR status");
    let _ = native_region::set_ocr(view.cast(), VisualPhase::Idle as u32, &[], &empty);
  }
}

pub(crate) fn text_interaction_started(window: &tauri::WebviewWindow) {
  // Peer panels do not become key themselves. Retain the single Tauri owner
  // for keyboard commands, but avoid reordering it on every text click.
  if !window.is_focused().unwrap_or(false) {
    let _ = window.set_focus();
  }
}

pub(super) fn show_ready(app: &tauri::AppHandle, generation: u64) {
  let Some(snapshot) = app
    .state::<TextRecognitionState>()
    .visual_snapshot(generation)
  else {
    return;
  };
  update_surfaces(
    app,
    VisualPhase::Ready,
    native_rects(&snapshot),
    "",
    SurfaceMode {
      frame: false,
      input: true,
      reset: false,
      claim_crosshair: false,
    },
  );
}

pub(super) fn show_error(app: &tauri::AppHandle, message: &str) {
  update_surfaces(
    app,
    VisualPhase::Error,
    Vec::new(),
    message,
    SurfaceMode {
      frame: true,
      input: true,
      reset: true,
      claim_crosshair: true,
    },
  );
}

fn native_rects(snapshot: &super::visual::VisualSnapshot) -> Vec<native_region::NativeOcrRect> {
  snapshot
    .rects
    .iter()
    .map(|visual| native_region::NativeOcrRect {
      x: visual.rect.origin.x,
      y: visual.rect.origin.y,
      width: visual.rect.size.width,
      height: visual.rect.size.height,
      kind: visual.kind as u8,
      padding: [0; 7],
    })
    .collect()
}

fn update_surfaces(
  app: &tauri::AppHandle,
  phase: VisualPhase,
  rects: Vec<native_region::NativeOcrRect>,
  message: &str,
  mode: SurfaceMode,
) {
  let windows = super::recognition_windows(app);
  let message = CString::new(message.replace('\0', " ")).expect("sanitized OCR status");
  let app = app.clone();
  let _ = app.clone().run_on_main_thread(move || {
    for window in windows {
      let Ok(view) = window.ns_view() else { continue };
      let view = view.cast();
      if native_region::set_ocr(view, phase as u32, &rects, &message) {
        let _ = native_region::set_show_frame(view, mode.frame);
        if mode.reset {
          let _ = native_region::reset_text_recognition_input(view);
        }
        let _ = native_region::set_input_enabled(view, mode.input);
        if mode.claim_crosshair {
          let _ = native_region::claim_pointer_surface(view);
        }
        break;
      }
    }
  });
}
