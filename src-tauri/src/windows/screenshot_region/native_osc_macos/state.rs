// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{ffi::c_void, panic::catch_unwind};

use crate::osc::geometry::{Monitor, Rect, Size};
use tauri::Emitter;

use super::{ffi, Context, DesktopBinding, NativeOscResult, Point};

pub fn invalid_result() -> NativeOscResult {
  NativeOscResult {
    status: super::ResultStatus::Invalid as u8,
    ..Default::default()
  }
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_input(
  context: *mut c_void,
  phase: u32,
  x: f64,
  y: f64,
  shift: u8,
  out: *mut NativeOscResult,
) {
  if out.is_null() {
    return;
  }
  let result = catch_unwind(|| {
    if context.is_null() {
      invalid_result()
    } else {
      (&*context.cast::<Context>()).input(phase, Point { x, y }, shift != 0)
    }
  })
  .unwrap_or_else(|_| invalid_result());
  *out = result;
}

pub unsafe extern "C" fn native_osc_layout_changed(context: *mut c_void) {
  let _ = catch_unwind(|| {
    if context.is_null() {
      return;
    }
    let context = &*context.cast::<Context>();
    let _ = context.window.emit_to(
      tauri::EventTarget::webview_window(context.window.label()),
      super::NATIVE_OSC_LAYOUT_EVENT,
      (),
    );
  });
}

fn attach(view: *mut c_void, window: super::WebviewWindow, width: f64, height: f64) -> bool {
  let context = Box::into_raw(Context::new(window, width, height)).cast();
  !ffi::attach(view, context).is_null()
}

pub fn ensure_attached(
  view: *mut c_void,
  window: super::WebviewWindow,
  width: f64,
  height: f64,
) -> bool {
  with_context(view, |_| ()).is_some() || attach(view, window, width, height)
}

fn with_context<T>(view: *mut c_void, work: impl FnOnce(&Context) -> T) -> Option<T> {
  let ptr = unsafe { ffi::screenwide_region_osc_context(view) };
  (!ptr.is_null()).then(|| work(unsafe { &*ptr.cast::<Context>() }))
}

pub fn set_committed(view: *mut c_void, rect: Option<Rect>) -> bool {
  with_context(view, |context| {
    context
      .controller
      .lock()
      .map(|mut controller| controller.set_committed(rect))
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

/// Clears the borrowed OSC before its window can be presented for a quick
/// screenshot. The recording region remains in frontend storage and will be
/// synchronized back when the normal Region editor resumes.
pub fn clear_region(view: *mut c_void) -> bool {
  if with_context(view, |context| {
    if let Ok(mut controller) = context.controller.lock() {
      let _ = controller.set_committed(None);
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set(view, 0.0, 0.0, 0.0, 0.0, 0) != 0 }
}

pub fn configure_desktop(view: *mut c_void, binding: DesktopBinding, local: Option<Rect>) -> bool {
  with_context(view, |context| {
    if binding.anchor().is_none() {
      return false;
    }
    let committed = super::desktop::global_committed(&binding, local);
    let controller = super::RegionController::new(binding.virtual_monitor(), committed, None);
    let Ok(mut current_controller) = context.controller.lock() else {
      return false;
    };
    let Ok(mut desktop) = context.desktop.lock() else {
      return false;
    };
    *current_controller = controller;
    *desktop = Some(binding);
    true
  })
  .unwrap_or(false)
}

pub fn set_monitor(view: *mut c_void, width: f64, height: f64) -> bool {
  with_context(view, |context| {
    context
      .controller
      .lock()
      .map(|mut controller| {
        controller.set_monitor(Monitor {
          size: Size { width, height },
        })
      })
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

pub fn set_allow_drawing(view: *mut c_void, allow_drawing: bool) -> bool {
  with_context(view, |context| {
    context
      .allow_drawing
      .store(allow_drawing, std::sync::atomic::Ordering::Relaxed);
  })
  .is_some()
}

pub fn set_magnifier_source(view: *mut c_void, rgba: &[u8], width: u32, height: u32) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe {
    ffi::screenwide_region_osc_set_magnifier_source(view, rgba.as_ptr(), rgba.len(), width, height)
      != 0
  }
}

pub fn set_aspect(view: *mut c_void, aspect: Option<f64>) -> bool {
  with_context(view, |context| {
    context
      .controller
      .lock()
      .map(|mut controller| {
        controller.set_aspect(aspect);
        true
      })
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

pub fn set_input_enabled(view: *mut c_void, enabled: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_input_enabled(view, i32::from(enabled)) };
  true
}

pub fn set_show_handles(view: *mut c_void, show_handles: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_show_handles(view, i32::from(show_handles)) };
  true
}

pub fn set_show_frame(view: *mut c_void, show_frame: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_show_frame(view, i32::from(show_frame)) };
  true
}

pub fn set_exclusion_rect(view: *mut c_void, rect: Option<Rect>) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  let rect = rect.unwrap_or_default();
  unsafe {
    ffi::screenwide_region_osc_set_exclusion_rect(
      view,
      rect.origin.x,
      rect.origin.y,
      rect.size.width,
      rect.size.height,
    );
  }
  true
}

pub fn set_desktop_presented(view: *mut c_void, presented: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_desktop_presented(view, i32::from(presented)) };
  true
}

pub fn claim_pointer_surface(view: *mut c_void) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_claim_pointer_surface(view) };
  true
}
