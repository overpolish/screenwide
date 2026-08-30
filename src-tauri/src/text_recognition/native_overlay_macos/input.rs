// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CString;

use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::{osc::geometry::Point, windows::screenshot_region::native_osc_macos as native_region};

use super::{native_rects, TextAction, TextRecognitionState, VisualPhase};

pub(crate) fn text_input(
  window: &tauri::WebviewWindow,
  phase: u32,
  point: Point,
  modifiers: u8,
  display_id: Option<u32>,
) -> native_region::NativeOscResult {
  let action = match phase {
    1 => TextAction::Hover,
    2 => TextAction::Down {
      additive: modifiers & 2 != 0,
      double: modifiers & 4 != 0,
    },
    3 => TextAction::Drag,
    4 => TextAction::Up,
    6 => TextAction::SelectAll,
    7 => TextAction::Copy,
    _ => return native_region::invalid_result(),
  };
  let app = window.app_handle();
  let Some(update) = app
    .state::<TextRecognitionState>()
    .text_input(action, point)
  else {
    return native_region::invalid_result();
  };
  if let Some(snapshot) = update.snapshot {
    let empty = CString::new("").expect("empty OCR status");
    if let Ok(view) = window.ns_view() {
      let _ = native_region::set_ocr(
        view.cast(),
        VisualPhase::Ready as u32,
        &native_rects(&snapshot),
        &empty,
      );
    }
  }
  if let Some(text) = update.copy_text {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
      let _ = app.clipboard().write_text(text);
      crate::text_recognition::dismiss(&app);
    });
  }
  if let Some(code) = update.qr_code {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
      if let Err(error) = crate::text_recognition::qr_details::show(&app, code, display_id) {
        eprintln!("Could not show QR details: {error}");
      }
    });
  }
  native_region::text_recognition_ready_result(update.text_cursor, update.qr_cursor)
}
