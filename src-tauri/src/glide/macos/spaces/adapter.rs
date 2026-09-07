// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::glide::{
  core::desktops::{DesktopAdapter, MoveError, Snapshot},
  platform::tween::WindowTarget,
};
use core_graphics::geometry::CGPoint;
use std::{
  ffi::{c_char, c_void, CStr, CString},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

unsafe extern "C" {
  fn free(pointer: *mut c_void);
  fn sw_glide_space_target(ax: *const c_void, own: *mut c_void) -> *mut c_void;
  fn sw_glide_space_release(target: *mut c_void);
  fn sw_glide_space_snapshot(target: *mut c_void) -> *mut c_char;
  fn sw_glide_space_carry(
    target: *mut c_void,
    group: *const c_char,
    destination: u64,
    x: f64,
    y: f64,
    cancelled: extern "C" fn(*const c_void) -> bool,
    context: *const c_void,
  ) -> i32;
}

pub(super) struct MacDesktopAdapter {
  target: *mut c_void,
  anchor: CGPoint,
  cancelled: Arc<AtomicBool>,
}
impl MacDesktopAdapter {
  pub fn new(
    window: &WindowTarget,
    anchor: CGPoint,
    cancelled: Arc<AtomicBool>,
  ) -> Result<Self, MoveError> {
    let raw = match window {
      WindowTarget::Ax(element) => unsafe {
        sw_glide_space_target(
          (&**element as *const cidre::ax::UiElement).cast(),
          std::ptr::null_mut(),
        )
      },
      WindowTarget::Own(window) => unsafe {
        sw_glide_space_target(
          std::ptr::null(),
          window
            .ns_window()
            .map_err(|error| MoveError::Unavailable(error.to_string()))?,
        )
      },
    };
    if raw.is_null() {
      return Err(MoveError::Unsupported);
    }
    Ok(Self {
      target: raw,
      anchor,
      cancelled,
    })
  }
}
impl Drop for MacDesktopAdapter {
  fn drop(&mut self) {
    unsafe { sw_glide_space_release(self.target) };
  }
}
extern "C" fn cancelled(context: *const c_void) -> bool {
  // The adapter retains this AtomicBool throughout the synchronous FFI call.
  unsafe { &*context.cast::<AtomicBool>() }.load(Ordering::Acquire)
}
impl DesktopAdapter for MacDesktopAdapter {
  fn snapshot(&self) -> Result<Snapshot, MoveError> {
    let raw = unsafe { sw_glide_space_snapshot(self.target) };
    if raw.is_null() {
      return Err(MoveError::Unavailable("Could not read Spaces".into()));
    }
    let parsed = serde_json::from_slice(unsafe { CStr::from_ptr(raw) }.to_bytes());
    unsafe { free(raw.cast()) };
    parsed.map_err(|error| MoveError::Unavailable(error.to_string()))
  }
  fn carry_window(&self, group: &str, destination: &str) -> Result<(), MoveError> {
    let group = CString::new(group).map_err(|_| MoveError::Unsupported)?;
    let id = destination.parse().map_err(|_| MoveError::Unsupported)?;
    match unsafe {
      sw_glide_space_carry(
        self.target,
        group.as_ptr(),
        id,
        self.anchor.x,
        self.anchor.y,
        cancelled,
        Arc::as_ptr(&self.cancelled).cast(),
      )
    } {
      0 => Ok(()),
      1 => Err(MoveError::Unavailable(
        "The titlebar grip is interactive or input is already held".into(),
      )),
      _ => Err(MoveError::FollowUnconfirmed),
    }
  }
}
