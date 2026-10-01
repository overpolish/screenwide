// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{ffi::c_void, sync::atomic::Ordering};

use super::{ffi, state::with_context, Purpose};

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

#[expect(
  dead_code,
  reason = "the reusable cancel OSC is intentionally not shown by OCR"
)]
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
