// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[no_mangle]
pub unsafe extern "C" fn native_osc_input(
  context: *mut c_void,
  phase: u32,
  x: f64,
  y: f64,
  modifiers: u8,
  out: *mut NativeOscResult,
) {
  if out.is_null() {
    return;
  }
  let result = catch_unwind(|| {
    if context.is_null() {
      invalid_result()
    } else {
      (&*context.cast::<Context>()).input(phase, Point { x, y }, modifiers)
    }
  })
  .unwrap_or_else(|_| invalid_result());
  *out = result;
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_viewport_input(
  context: *mut c_void,
  display_id: u32,
  operation: u32,
  anchor_x: f64,
  anchor_y: f64,
  delta_x: f64,
  delta_y: f64,
  out: *mut NativeOscResult,
) -> i32 {
  if out.is_null() {
    return 0;
  }
  let result = catch_unwind(|| {
    if context.is_null() {
      invalid_result()
    } else {
      (&*context.cast::<Context>()).ruler_viewport_input(
        display_id,
        operation,
        Point {
          x: anchor_x,
          y: anchor_y,
        },
        Point {
          x: delta_x,
          y: delta_y,
        },
      )
    }
  })
  .unwrap_or_else(|_| invalid_result());
  let handled = result.status != super::super::ResultStatus::Invalid as u8;
  *out = result;
  i32::from(handled)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_label_input(
  context: *mut c_void,
  operation: u32,
  kind: u8,
  id: u64,
  pointer_x: f64,
  pointer_y: f64,
  label_center_x: f64,
  label_center_y: f64,
  out: *mut NativeOscResult,
) {
  if out.is_null() {
    return;
  }
  *out = catch_unwind(|| {
    if context.is_null() {
      invalid_result()
    } else {
      (&*context.cast::<Context>()).ruler_label_input(
        operation,
        kind,
        id,
        Point {
          x: pointer_x,
          y: pointer_y,
        },
        Point {
          x: label_center_x,
          y: label_center_y,
        },
      )
    }
  })
  .unwrap_or_else(|_| invalid_result());
}

pub unsafe extern "C" fn native_osc_layout_changed(context: *mut c_void) {
  let _ = catch_unwind(|| {
    if context.is_null() {
      return;
    }
    let context = &*context.cast::<Context>();
    if context.purpose == super::super::Purpose::TextRecognition {
      crate::text_recognition::restart_after_topology_change(context.window.app_handle());
      return;
    }
    if context.purpose == super::super::Purpose::Ruler {
      crate::ruler::restart_after_topology_change(context.window.app_handle());
      return;
    }
    let _ = context.window.emit_to(
      tauri::EventTarget::webview_window(context.window.label()),
      super::super::NATIVE_OSC_LAYOUT_EVENT,
      (),
    );
  });
}
