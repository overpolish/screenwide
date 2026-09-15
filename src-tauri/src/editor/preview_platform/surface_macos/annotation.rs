// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The annotation tool's half of the native interaction view.
//!
//! Only two things cross here: the grips of every arrow on the selected
//! layer, so the OSC can draw them and hit-test them, and the pointer samples
//! that come back. Both speak the layer's image-normalised space, never
//! source pixels, so the native side needs nothing but the transform it
//! already applies to the selection chrome.

use super::super::{AnnotationGestureCallback, AnnotationHoverCallback};
use super::callbacks::{
  annotation_gesture_callback, annotation_hover_callback, release_callback_on_main,
};
use super::ffi::{
  screenwide_preview_surface_set_annotation_gesture_callback,
  screenwide_preview_surface_set_annotation_hover_callback,
  screenwide_preview_surface_set_annotations,
};
use super::RecordingPreviewSurface;
use crate::editor::annotations::handles::NativeAnnotationHandles;

impl RecordingPreviewSurface {
  /// Publishes the selected layer's arrow grips. `selected_index` is the
  /// arrow whose three handles are drawn, or -1 for none; `mode` is what the
  /// pointer does over the picture: nothing (0), hit-test the arrows that are
  /// there and otherwise fall through to the layer (1), or also draw a new
  /// arrow on empty picture (2).
  pub(crate) fn set_annotations(
    &self,
    handles: &[NativeAnnotationHandles],
    selected_index: i32,
    mode: u32,
  ) {
    self.set_annotation_layer(handles, selected_index, mode, -1);
  }

  pub(crate) fn set_annotation_layer(
    &self,
    handles: &[NativeAnnotationHandles],
    selected_index: i32,
    mode: u32,
    active_layer: i32,
  ) {
    unsafe {
      screenwide_preview_surface_set_annotations(
        self.handle,
        handles.as_ptr(),
        handles.len(),
        selected_index,
        mode as i32,
        active_layer,
      );
    }
  }

  /// Installs the hover callback that drives the halo. It is invoked on the
  /// main thread, on hover change and on every frame of the pulse.
  pub(crate) fn set_annotation_hover_callback(&mut self, callback: AnnotationHoverCallback) {
    let mut callback = Box::new(callback);
    let context = (&mut *callback) as *mut AnnotationHoverCallback as *mut std::ffi::c_void;
    unsafe {
      screenwide_preview_surface_set_annotation_hover_callback(
        self.handle,
        Some(annotation_hover_callback),
        context,
      );
    }
    release_callback_on_main(self.annotation_hover_callback.replace(callback));
  }

  /// Installs the arrow tool's pointer callback. It is invoked on the main
  /// thread with the point in the layer's image-normalised space.
  pub(crate) fn set_annotation_gesture_callback(&mut self, callback: AnnotationGestureCallback) {
    let mut callback = Box::new(callback);
    let context = (&mut *callback) as *mut AnnotationGestureCallback as *mut std::ffi::c_void;
    unsafe {
      screenwide_preview_surface_set_annotation_gesture_callback(
        self.handle,
        Some(annotation_gesture_callback),
        context,
      );
    }
    release_callback_on_main(self.annotation_gesture_callback.replace(callback));
  }
}
