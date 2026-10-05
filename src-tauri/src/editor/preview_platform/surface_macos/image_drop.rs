// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures dropped on the workspace, from the interaction view's drop
//! target in `recording_preview_surface_macos+drop.m`.

use std::path::PathBuf;

use super::callbacks::release_callback_on_main;
use super::RecordingPreviewSurface;
use crate::editor::images::dropped::{DroppedPicture, ImagePoint};
use crate::editor::preview_platform::ImageDropCallback;

/// What the drop target hands over: a file's path, a path the dragging app
/// wrote a file to for the drop, or the picture's own file data. The twin of
/// `ScreenwideImageDrop` in `recording_preview_surface_macos_private.h`.
const DROP_FILE: u32 = 0;
const DROP_WRITTEN: u32 = 1;
const DROP_DATA: u32 = 2;

pub(super) type ImageDropTrampoline =
  unsafe extern "C" fn(u32, i32, f64, f64, *const u8, usize, *mut std::ffi::c_void);

unsafe extern "C" {
  pub(super) fn screenwide_preview_surface_set_image_drop_callback(
    handle: *mut std::ffi::c_void,
    callback: Option<ImageDropTrampoline>,
    context: *mut std::ffi::c_void,
  );
}

/// # Safety
/// `data` must point at `length` readable bytes, and `context` at the
/// callback the surface was given.
unsafe extern "C" fn image_drop_callback(
  kind: u32,
  layer: i32,
  x: f64,
  y: f64,
  data: *const u8,
  length: usize,
  context: *mut std::ffi::c_void,
) {
  let Some(callback) = (unsafe { (context as *mut ImageDropCallback).as_mut() }) else {
    return;
  };
  if data.is_null() || length == 0 {
    return;
  }
  let bytes = unsafe { std::slice::from_raw_parts(data, length) };
  let path = || std::str::from_utf8(bytes).ok().map(PathBuf::from);
  let picture = match kind {
    DROP_FILE => path().map(DroppedPicture::File),
    DROP_WRITTEN => path().map(DroppedPicture::Written),
    DROP_DATA => Some(DroppedPicture::Data(bytes.to_vec())),
    _ => None,
  };
  let at = u32::try_from(layer)
    .ok()
    .map(|layer| ImagePoint { layer, x, y });
  if let Some(picture) = picture {
    callback(picture, at);
  }
}

impl RecordingPreviewSurface {
  /// Installs the callback a picture dropped on the workspace is handed to,
  /// on the main thread. Without one, nothing can be dropped there.
  pub(crate) fn set_image_drop_callback(&mut self, callback: ImageDropCallback) {
    let mut callback = Box::new(callback);
    let context = (&mut *callback) as *mut ImageDropCallback as *mut std::ffi::c_void;
    unsafe {
      screenwide_preview_surface_set_image_drop_callback(
        self.handle,
        Some(image_drop_callback),
        context,
      );
    }
    release_callback_on_main(self.image_drop_callback.replace(callback));
  }
}
