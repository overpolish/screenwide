// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text/OCR recognition and native overlay session state.

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::{capture_overlays, screenshots, windows::WindowLabel};
mod adapter;
mod input;
mod interaction;
#[cfg(target_os = "macos")]
mod native_overlay_macos;
#[cfg(target_os = "macos")]
mod platform_macos;
#[cfg(target_os = "windows")]
mod platform_windows;
mod qr;
pub(crate) mod qr_details;
pub(crate) mod snapshot;
mod text_selection;
pub(crate) mod toolbar;
pub(crate) mod visual;

pub(crate) use input::{
  dispatch_control, dispatch_text_input as native_text_input,
  selection_finished as native_selection_finished, selection_started as native_selection_started,
  text_interaction_started as native_text_interaction_started,
};
pub(crate) use interaction::{copy_all_and_dismiss, copy_selection_and_dismiss};
pub use snapshot::TextRecognitionState;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRect {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecognizedCharacter {
  pub start: usize,
  pub end: usize,
  pub bounds: TextRect,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecognizedLine {
  pub text: String,
  pub confidence: f32,
  pub bounds: TextRect,
  pub characters: Vec<RecognizedCharacter>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecognizedQrCode {
  pub bounds: TextRect,
  pub content: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub decode_error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRecognitionResult {
  pub lines: Vec<RecognizedLine>,
  pub qr_codes: Vec<RecognizedQrCode>,
  pub text: String,
}

fn recognition_windows(app: &AppHandle) -> Vec<tauri::WebviewWindow> {
  app
    .get_webview_window(WindowLabel::TextRecognition.as_str())
    .into_iter()
    .collect()
}

pub(crate) fn is_active(app: &AppHandle) -> bool {
  app.state::<TextRecognitionState>().is_active()
}

pub(crate) fn restart_after_topology_change(app: &AppHandle) {
  let state = app.state::<TextRecognitionState>();
  let Some(generation) = state.active_generation() else {
    return;
  };
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    let state = app.state::<TextRecognitionState>();
    if state.active_generation() != Some(generation) {
      return;
    }
    if let Err(error) = start(&app).await {
      eprintln!("Could not rebuild text recognition after a display change: {error}");
    }
  });
}

fn close_recognition_windows(app: &AppHandle, except: Option<&str>) {
  // Native surfaces must be concealed before their owner webviews close:
  // peers are compositor panels rather than Tauri windows and otherwise
  // outlive the first monitor's visual teardown.
  adapter::close(app, except);
}

pub fn dismiss(app: &AppHandle) {
  qr_details::hide_without_resume(app);
  let had_windows = !recognition_windows(app).is_empty();
  close_recognition_windows(app, None);
  let had_capture = app.state::<TextRecognitionState>().cancel();
  if had_windows || had_capture {
    capture_overlays::emit_lifecycle(app, false);
  }
  crate::windows::sync_recording_ui_escape(app, crate::ruler::is_active(app));
}

pub async fn start(app: &AppHandle) -> Result<(), String> {
  dismiss(app);
  capture_overlays::dismiss_except(app, Some(capture_overlays::CaptureOverlay::TextRecognition));
  let generation = app.state::<TextRecognitionState>().begin();

  let monitors = capture_overlays::monitor_layout(app)?;
  let mut snapshots = Vec::with_capacity(monitors.len());
  for (monitor_id, scale, _) in &monitors {
    let image = screenshots::capture_text_recognition_snapshot(*monitor_id).await?;
    snapshots.push((*monitor_id, *scale, image));
  }
  let native_snapshots = snapshots
    .iter()
    .map(|(id, _, image)| (*id, image.clone()))
    .collect::<Vec<_>>();
  if !app
    .state::<TextRecognitionState>()
    .install(generation, snapshots)
  {
    return Ok(());
  }
  let (anchor_id, anchor_scale, anchor_monitor) = monitors
    .first()
    .ok_or_else(|| "No monitor is available for text recognition".to_owned())?;
  let anchor_id = *anchor_id;
  let position = anchor_monitor.position().to_logical::<f64>(*anchor_scale);
  let size = anchor_monitor.size().to_logical::<f64>(*anchor_scale);
  let window = WebviewWindowBuilder::new(
    app,
    WindowLabel::TextRecognition.as_str(),
    WebviewUrl::App("/text-recognition".into()),
  )
  .accept_first_mouse(true)
  .always_on_top(true)
  .decorations(false)
  // On macOS the global shortcut's Command key-up can arrive after this
  // window is presented. Do not make the overlay key until the user begins
  // a selection, otherwise Tao forwards that key-up into the new window.
  .focused(!cfg!(target_os = "macos"))
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
  #[cfg(not(target_os = "windows"))]
  window
    .set_content_protected(true)
    .map_err(|error| error.to_string())?;
  capture_overlays::set_level(&window, capture_overlays::FOREGROUND_LEVEL)?;

  let native_installed = adapter::install(&window, anchor_id, &native_snapshots)?;
  if native_installed {
    adapter::show_without_activation(&window)?;
    adapter::present(&window)?;
  } else {
    crate::windows::show(&window, true).map_err(|error| error.to_string())?;
  }

  capture_overlays::emit_lifecycle(app, true);
  crate::windows::sync_recording_ui_escape(app, crate::ruler::is_active(app));

  Ok(())
}

pub fn start_detached(app: &AppHandle) {
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    if let Err(error) = start(&app).await {
      eprintln!("Could not start text recognition: {error}");
    }
  });
}

#[tauri::command]
pub async fn start_text_recognition(app: AppHandle) -> Result<(), String> {
  start(&app).await
}

#[tauri::command]
pub fn cancel_text_recognition(app: AppHandle) {
  dismiss(&app);
}

#[tauri::command]
pub fn copy_recognition_content(app: AppHandle, text: String) -> Result<(), String> {
  app
    .clipboard()
    .write_text(text)
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_qr_details(
  state: tauri::State<'_, qr_details::QrDetailsState>,
) -> Result<RecognizedQrCode, String> {
  state
    .current()
    .ok_or_else(|| "No QR code is selected".to_owned())
}

#[tauri::command]
pub fn close_qr_details(app: AppHandle) {
  qr_details::hide_and_resume(&app);
}

async fn recognize_current(app: &AppHandle) -> Result<TextRecognitionResult, String> {
  let state = app.state::<TextRecognitionState>();
  let (generation, image) = state
    .recognition_input()
    .ok_or_else(|| "The selected image is no longer available".to_owned())?;
  let (lines, qr_codes) = match recognize(image.rgba, image.width, image.height).await {
    Ok(result) => result,
    Err(error) => {
      if state.is_current_generation(generation) {
        adapter::show_error(app, &error);
      }
      return Err(error);
    }
  };
  let text = lines
    .iter()
    .map(|line| line.text.as_str())
    .collect::<Vec<_>>()
    .join("\n");
  let result = TextRecognitionResult {
    lines,
    qr_codes,
    text,
  };
  if state.install_result(generation, result.clone()) {
    adapter::show_ready(app, generation);
  }
  Ok(result)
}

async fn recognize(
  rgba: Vec<u8>,
  width: u32,
  height: u32,
) -> Result<(Vec<RecognizedLine>, Vec<RecognizedQrCode>), String> {
  tauri::async_runtime::spawn_blocking(move || {
    let qr_codes = qr::recognize(&rgba, width, height);
    #[cfg(target_os = "macos")]
    return platform_macos::recognize(&rgba, width, height).map(|lines| (lines, qr_codes));

    #[cfg(target_os = "windows")]
    return platform_windows::recognize(&rgba, width, height).map(|lines| (lines, qr_codes));

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err("Text recognition is not available on this platform".to_owned())
  })
  .await
  .map_err(|error| error.to_string())?
}
