// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Mutex;

use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::{
  AppHandle, Emitter, LogicalSize, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
  WindowEvent,
};

use crate::windows::WindowLabel;

use super::RecognizedQrCode;

#[derive(Default)]
pub struct QrDetailsState(Mutex<Option<RecognizedQrCode>>);

impl QrDetailsState {
  pub fn current(&self) -> Option<RecognizedQrCode> {
    self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone()
  }

  fn set(&self, code: Option<RecognizedQrCode>) {
    *self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = code;
  }
}

fn place_on_display(
  app: &AppHandle,
  window: &WebviewWindow,
  display_id: Option<u32>,
) -> Result<(), String> {
  let Some((scale, monitor)) = display_id
    .map(|display_id| crate::capture_overlays::monitor_by_capture_id(app, display_id))
    .transpose()?
    .flatten()
  else {
    // Generic topology containment remains the fallback when input did not
    // originate from a native desktop surface or that display disappeared.
    return crate::windows::contain_normal_window(app, window).map_err(|error| error.to_string());
  };
  let size = LogicalSize::new(480.0, 360.0);
  let work_area = monitor.work_area();
  let position = crate::windows::centered_logical_position(
    work_area.position.to_logical::<f64>(scale),
    work_area.size.to_logical::<f64>(scale),
    size,
  );
  window.set_size(size).map_err(|error| error.to_string())?;
  window
    .set_position(position)
    .map_err(|error| error.to_string())
}

pub fn show(
  app: &AppHandle,
  code: RecognizedQrCode,
  display_id: Option<u32>,
) -> Result<(), String> {
  app.state::<QrDetailsState>().set(Some(code.clone()));

  let label = WindowLabel::QrDetails.as_str();
  let window = if let Some(window) = app.get_webview_window(label) {
    window
  } else {
    let effect = if cfg!(target_os = "windows") {
      Effect::Mica
    } else {
      Effect::UnderWindowBackground
    };
    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App("/qr-details".into()))
      .title("QR Details")
      .inner_size(480.0, 360.0)
      .center()
      .always_on_top(true)
      .closable(true)
      .decorations(false)
      .resizable(false)
      .shadow(true)
      .skip_taskbar(true)
      .transparent(true)
      .visible(false)
      .effects(WindowEffectsConfig {
        color: None,
        effects: vec![effect],
        radius: Some(10.0),
        state: Some(EffectState::Active),
      })
      .build()
      .map_err(|error| error.to_string())?;
    let close_app = app.clone();
    window.on_window_event(move |event| {
      if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        hide_and_resume(&close_app);
      }
    });
    window
  };

  window
    .emit("qr-details-updated", code)
    .map_err(|error| error.to_string())?;
  place_on_display(app, &window, display_id)?;
  #[cfg(target_os = "macos")]
  crate::capture_overlays::set_level(&window, crate::capture_overlays::FOREGROUND_LEVEL + 1)?;
  crate::windows::show(&window, true).map_err(|error| error.to_string())?;
  Ok(())
}

pub fn hide_and_resume(app: &AppHandle) {
  hide_without_resume(app);
}

pub fn hide_without_resume(app: &AppHandle) {
  app.state::<QrDetailsState>().set(None);
  if let Some(window) = app.get_webview_window(WindowLabel::QrDetails.as_str()) {
    let _ = window.hide();
  }
}
