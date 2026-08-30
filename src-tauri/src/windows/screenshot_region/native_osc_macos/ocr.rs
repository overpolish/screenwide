// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{ffi::c_void, sync::atomic::Ordering};

use tauri::Manager;

use super::{
  ffi, state::with_context, Context, InputPhase, NativeOscResult, Purpose, ResultStatus,
};

pub(super) fn control_input(context: &Context, phase: u32) -> Option<NativeOscResult> {
  if context.purpose != Purpose::TextRecognition {
    return None;
  }
  if !matches!(phase, 8..=12) {
    return None;
  }
  let app = context.window.app_handle().clone();
  crate::text_recognition::qr_details::hide_without_resume(&app);
  match phase {
    x if x == InputPhase::OcrCancel as u32 => {
      tauri::async_runtime::spawn(async move { crate::text_recognition::dismiss(&app) });
    }
    x if x == InputPhase::OcrCopyAll as u32 => {
      tauri::async_runtime::spawn(async move {
        let _ = crate::text_recognition::copy_all_and_dismiss(&app);
      });
    }
    x if x == InputPhase::OcrCopyParagraph as u32 => {
      tauri::async_runtime::spawn(async move {
        let _ = crate::text_recognition::copy_selection_and_dismiss(&app, true, true);
      });
    }
    x if x == InputPhase::OcrReset as u32 => {
      tauri::async_runtime::spawn(async move {
        if let Err(error) = crate::text_recognition::start(&app).await {
          eprintln!("Could not reset text recognition: {error}");
        }
      });
    }
    x if x == InputPhase::OcrClose as u32 => {
      tauri::async_runtime::spawn(async move { crate::text_recognition::dismiss(&app) });
    }
    _ => unreachable!("OCR control phases are filtered above"),
  }
  Some(NativeOscResult::default())
}

pub fn ready_result(text_cursor: bool, qr_cursor: bool) -> NativeOscResult {
  NativeOscResult {
    cursor: if qr_cursor {
      ffi::CURSOR_POINTING_HAND
    } else if text_cursor {
      ffi::CURSOR_IBEAM
    } else {
      ffi::CURSOR_ARROW
    },
    ..Default::default()
  }
}

pub(super) fn dismiss_on_idle_cancel(
  context: *mut c_void,
  phase: u32,
  result: &mut NativeOscResult,
) {
  if context.is_null()
    || phase != InputPhase::Cancel as u32
    || result.status != ResultStatus::None as u8
  {
    return;
  }
  let context = unsafe { &*context.cast::<Context>() };
  if context.purpose != Purpose::TextRecognition {
    return;
  }
  result.cursor = ffi::CURSOR_ARROW;
  let app = context.window.app_handle().clone();
  tauri::async_runtime::spawn(async move { crate::text_recognition::dismiss(&app) });
}

pub fn set_ocr(
  view: *mut c_void,
  phase: u32,
  rects: &[ffi::NativeOcrRect],
  message: &std::ffi::CStr,
) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe {
    ffi::screenwide_region_osc_set_ocr(view, phase, rects.as_ptr(), rects.len(), message.as_ptr())
      != 0
  }
}

pub fn set_cancel_visible(view: *mut c_void, visible: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe {
    ffi::screenwide_region_osc_ocr_set_cancel_visible(view, i32::from(visible));
  }
  true
}

pub fn reset_input(view: *mut c_void) -> bool {
  with_context(view, |context| {
    if context.purpose != Purpose::TextRecognition {
      return false;
    }
    context.completed.store(false, Ordering::Release);
    if let Ok(mut controller) = context.controller.lock() {
      let _ = controller.set_committed(None);
    }
    true
  })
  .unwrap_or(false)
}
