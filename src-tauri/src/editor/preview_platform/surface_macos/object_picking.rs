// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Objects picked in the camera on the workspace itself, from
//! `recording_preview_surface_macos+object_picking.m`: a click on the
//! camera's picture is handed over, and the objects picked in the frame
//! shown are drawn over it.

use super::callbacks::release_callback_on_main;
use super::RecordingPreviewSurface;
use screenwide_compositor::object_picking::{
  ObjectPickCallback, ObjectPress, ObjectsShown, PickedObjectDot,
};

/// The presses `ScreenwideObjectPress…` names.
const PRESS_KEEP: u32 = 0;
const PRESS_REMOVE: u32 = 1;

type ObjectPickTrampoline = unsafe extern "C" fn(u32, *const [f32; 2], u32, *mut std::ffi::c_void);

unsafe extern "C" {
  fn screenwide_preview_surface_set_object_picking(
    handle: *mut std::ffi::c_void,
    active: u32,
    camera_layer: u32,
    objects: *const PickedObjectDot,
    count: u32,
    lines: *const [f32; 2],
    line_points: u32,
  );
  pub(super) fn screenwide_preview_surface_set_object_pick_callback(
    handle: *mut std::ffi::c_void,
    callback: Option<ObjectPickTrampoline>,
    context: *mut std::ffi::c_void,
  );
}

/// # Safety
/// `points` must hold `count` points, and `context` point at the callback
/// the surface was given.
unsafe extern "C" fn object_pick_callback(
  press: u32,
  points: *const [f32; 2],
  count: u32,
  context: *mut std::ffi::c_void,
) {
  let points = if points.is_null() {
    &[][..]
  } else {
    unsafe { std::slice::from_raw_parts(points, count as usize) }
  };
  if press != PRESS_KEEP && press != PRESS_REMOVE {
    return;
  }
  let press = ObjectPress {
    points: points.to_vec(),
    keep: press == PRESS_KEEP,
  };
  if let Some(callback) = unsafe { (context as *mut ObjectPickCallback).as_mut() } {
    callback(press);
  }
}

impl RecordingPreviewSurface {
  /// Picks objects in the camera as `picking` shows them, or stops with
  /// `None`. While picking, a press on the camera is handed to the pick
  /// callback rather than choosing or moving a layer.
  pub(crate) fn set_object_picking(&self, picking: Option<ObjectsShown<'_>>) {
    let (active, layer, objects, lines) = picking
      .map_or((0, 0, &[][..], &[][..]), |(layer, objects, lines)| {
        (1, layer, objects, lines)
      });
    let count = |length: usize| u32::try_from(length).unwrap_or(u32::MAX);
    unsafe {
      screenwide_preview_surface_set_object_picking(
        self.handle,
        active,
        layer,
        objects.as_ptr(),
        count(objects.len()),
        lines.as_ptr(),
        count(lines.len()),
      );
    }
  }

  /// Installs the callback a click on the camera is handed to while objects
  /// are picked, on the main thread.
  pub(crate) fn set_object_pick_callback(&mut self, callback: ObjectPickCallback) {
    let mut callback = Box::new(callback);
    let context = (&mut *callback) as *mut ObjectPickCallback as *mut std::ffi::c_void;
    unsafe {
      screenwide_preview_surface_set_object_pick_callback(
        self.handle,
        Some(object_pick_callback),
        context,
      );
    }
    release_callback_on_main(self.object_pick_callback.replace(callback));
  }
}
