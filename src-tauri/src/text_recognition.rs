// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text/OCR recognition and native overlay session state.

use std::sync::{Mutex, PoisonError};

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::user_work::UserActivity;
use crate::{app_windows::WindowLabel, capture_overlays};
mod adapter;
mod input;
mod interaction;
pub(crate) mod island;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod native_overlay;
#[cfg(all(test, target_os = "macos"))]
mod perf_tests;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod preview;
pub(crate) mod snapshot;
mod text_selection;
pub(crate) mod visual;
#[cfg(target_os = "windows")]
mod windows;

pub(crate) use input::{
  dispatch_text_input as native_text_input, selection_finished as native_selection_finished,
  selection_started as native_selection_started,
  text_interaction_started as native_text_interaction_started,
};
use interaction::copy_selection_and_dismiss;
pub use snapshot::TextRecognitionState;
mod recognition_worker;
pub(crate) mod settings;
use recognition_worker::recognize;

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

/// Held from the shortcut until the overlay closes: the app is otherwise
/// mostly windowless, and App Nap holds every thread of a napped app to
/// background priority, whatever it asks for.
static ACTIVITY: Mutex<Option<UserActivity>> = Mutex::new(None);

fn hold_activity(activity: Option<UserActivity>) {
  *ACTIVITY.lock().unwrap_or_else(PoisonError::into_inner) = activity;
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
  // Before the overlay closes: on Windows the island is owned by it.
  island::withdraw(app);
  let had_windows = !recognition_windows(app).is_empty();
  close_recognition_windows(app, None);
  #[cfg(target_os = "macos")]
  if let Err(error) = crate::osc::cursor_macos::release_text_recognition(app) {
    eprintln!("Could not release text-recognition cursor ownership: {error}");
  }
  let had_capture = app.state::<TextRecognitionState>().cancel();
  if had_windows || had_capture {
    capture_overlays::emit_lifecycle(app, false);
  }
  crate::app_windows::sync_recording_ui_escape(app, crate::ruler::is_active(app));
  hold_activity(None);
  #[cfg(target_os = "windows")]
  windows::reader::release();
}

pub async fn start(app: &AppHandle) -> Result<(), String> {
  if !settings::enabled() {
    return Ok(());
  }
  // Opens while the desktop freezes, which the overlay waits for anyway.
  screenwide_gpu::open_soon();
  dismiss(app);
  hold_activity(Some(UserActivity::begin("Reading text on screen")));
  capture_overlays::dismiss_except(app, &[capture_overlays::CaptureOverlay::TextRecognition]);
  let generation = app.state::<TextRecognitionState>().begin();
  crate::glide::suspend_for_capture(app);
  // While the desktop freezes and an area is drawn.
  #[cfg(target_os = "windows")]
  windows::reader::warm(app);
  #[cfg(target_os = "macos")]
  if let Err(error) = crate::osc::cursor_macos::acquire_text_recognition(app) {
    dismiss(app);
    return Err(error);
  }
  let result = start_session(app, generation).await;
  if result.is_err() || !is_active(app) {
    dismiss(app);
  }
  result
}

async fn start_session(app: &AppHandle, generation: u64) -> Result<(), String> {
  let monitors = capture_overlays::monitor_layout(app)?;
  let snapshots = snapshot::freeze(app, &monitors).await?;
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
  let window = crate::app_windows::webview_window(
    app,
    WindowLabel::TextRecognition.as_str(),
    WebviewUrl::App("/text-recognition".into()),
  )
  .accept_first_mouse(true)
  .always_on_top(true)
  .decorations(false)
  // Building must not make the window key while the global shortcut event is
  // still unwinding. The cursor lease promotes it only after snapshots and
  // native surfaces are ready for interaction.
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
    adapter::show_interactive(&window)?;
    adapter::present(&window)?;
  } else {
    crate::app_windows::show(&window, true).map_err(|error| error.to_string())?;
  }

  capture_overlays::emit_lifecycle(app, true);
  crate::app_windows::sync_recording_ui_escape(app, crate::ruler::is_active(app));
  island::stand_over(app, &window);

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

async fn recognize_current(app: &AppHandle) -> Result<(), String> {
  let state = app.state::<TextRecognitionState>();
  let Some((generation, image)) = state.recognition_input() else {
    return Err("The selected image is no longer available".to_owned());
  };
  let (lines, qr_codes) = match recognize(image).await {
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
  if state.install_result(generation, result) {
    adapter::show_ready(app, generation);
  }
  Ok(())
}
