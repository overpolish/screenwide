// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::c_void;

use super::{
  state::{native_osc_input, native_osc_layout_changed},
  NativeOscResult,
};

const _: () = assert!(std::mem::size_of::<NativeOscResult>() == 48);
const _: () = assert!(std::mem::offset_of!(NativeOscResult, x) == 8);

pub type ReleaseContext = unsafe extern "C" fn(*mut c_void);
pub const CURSOR_CROSSHAIR: u8 = 1;
pub const CURSOR_OPEN_HAND: u8 = 2;
pub const CURSOR_CLOSED_HAND: u8 = 3;
pub const CURSOR_HORIZONTAL: u8 = 4;
pub const CURSOR_VERTICAL: u8 = 5;
pub const CURSOR_DIAGONAL: u8 = 6;
pub const CURSOR_ARROW: u8 = 7;
pub const CURSOR_IBEAM: u8 = 8;
pub const CURSOR_POINTING_HAND: u8 = 9;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeDesktopDisplay {
  pub id: u32,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub scale: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeOcrRect {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub kind: u8,
  pub padding: [u8; 7],
}

const _: () = assert!(std::mem::size_of::<NativeOcrRect>() == 40);
const _: () = assert!(std::mem::offset_of!(NativeOcrRect, kind) == 32);

unsafe extern "C" {
  pub fn screenwide_region_osc_attach(
    view: *mut c_void,
    context: *mut c_void,
    release: ReleaseContext,
    input: unsafe extern "C" fn(*mut c_void, u32, f64, f64, u8, *mut NativeOscResult),
    layout_changed: unsafe extern "C" fn(*mut c_void),
  ) -> *mut c_void;
  pub fn screenwide_region_osc_context(view: *mut c_void) -> *mut c_void;
  pub fn screenwide_region_osc_set(
    view: *mut c_void,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    visible: i32,
  ) -> i32;
  pub fn screenwide_region_osc_set_magnifier_source(
    view: *mut c_void,
    rgba: *const u8,
    length: usize,
    width: u32,
    height: u32,
  ) -> i32;
  pub fn screenwide_region_osc_set_input_enabled(view: *mut c_void, enabled: i32);
  pub fn screenwide_region_osc_set_show_frame(view: *mut c_void, show_frame: i32);
  pub fn screenwide_region_osc_set_show_handles(view: *mut c_void, show_handles: i32);
  pub fn screenwide_region_osc_set_exclusion_rect(
    view: *mut c_void,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
  );
  pub fn screenwide_region_osc_configure_desktop(
    view: *mut c_void,
    anchor_id: u32,
    displays: *mut NativeDesktopDisplay,
    capacity: usize,
    desktop_width: *mut f64,
    desktop_height: *mut f64,
    resolved_anchor_id: *mut u32,
    layout_changed: *mut i32,
  ) -> usize;
  pub fn screenwide_region_osc_set_desktop_presented(view: *mut c_void, presented: i32);
  pub fn screenwide_region_osc_claim_pointer_surface(view: *mut c_void);
  pub fn screenwide_region_osc_ruler_refresh_pointer(view: *mut c_void);
  pub fn screenwide_region_osc_ruler_set_transient_chrome(view: *mut c_void, visible: i32);
  pub fn screenwide_region_osc_set_snapshot(
    view: *mut c_void,
    display_id: u32,
    rgba: *const u8,
    length: usize,
    width: u32,
    height: u32,
  ) -> i32;
  pub fn screenwide_region_osc_set_snapshot_presented(view: *mut c_void, presented: i32);
  pub fn screenwide_region_osc_set_snapshot_composited(view: *mut c_void, composited: i32);
  pub fn screenwide_region_osc_set_ocr(
    view: *mut c_void,
    phase: u32,
    rects: *const NativeOcrRect,
    count: usize,
    message: *const std::ffi::c_char,
  ) -> i32;
  pub fn screenwide_region_osc_ocr_set_cancel_visible(view: *mut c_void, visible: i32);
}

pub unsafe extern "C" fn release_context(ptr: *mut c_void) {
  if !ptr.is_null() {
    drop(Box::from_raw(ptr.cast::<super::Context>()));
  }
}

pub fn attach(view: *mut c_void, context: *mut super::Context) -> *mut c_void {
  unsafe {
    screenwide_region_osc_attach(
      view,
      context.cast(),
      release_context,
      native_osc_input,
      native_osc_layout_changed,
    )
  }
}
