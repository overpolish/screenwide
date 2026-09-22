// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The C boundary of the Metal video compositor: the records it is handed
//! and the callbacks it reports through.

use std::ffi::{c_char, c_void};
use std::sync::atomic::Ordering;

use super::timed_annotations::NativeTimedAnnotation;
use crate::editor::annotations::native::NativeAnnotationData;
use crate::editor::cursor_effects::{NativeGpuArtwork, NativeGpuCursor};

/// The share of the progress bar the GPU pass owns; muxing takes the rest.
pub(super) const GPU_PROGRESS_PERCENT: u64 = 95;

#[repr(C)]
pub(super) struct GpuCallbacks<'a> {
  pub(super) cancelled: &'a std::sync::atomic::AtomicBool,
  pub(super) duration_ms: u64,
  pub(super) on_progress: &'a mut dyn FnMut(u64),
}

#[repr(C)]
#[derive(Default)]
pub(super) struct GpuCameraOverlay {
  pub(super) crop_x: u32,
  pub(super) crop_y: u32,
  pub(super) crop_width: u32,
  pub(super) crop_height: u32,
  pub(super) frame_x: i32,
  pub(super) frame_y: i32,
  pub(super) frame_width: u32,
  pub(super) frame_height: u32,
  pub(super) radius: u32,
  pub(super) drop_shadow: u32,
  pub(super) camera_on_top: u32,
}

pub(super) unsafe extern "C" fn gpu_should_cancel(context: *mut c_void) -> bool {
  let callbacks = unsafe { &*(context.cast::<GpuCallbacks<'_>>()) };
  callbacks.cancelled.load(Ordering::Acquire)
}

pub(super) unsafe extern "C" fn gpu_progress(context: *mut c_void, position_ms: u64) {
  let callbacks = unsafe { &mut *(context.cast::<GpuCallbacks<'_>>()) };
  let position_ms = position_ms.min(callbacks.duration_ms);
  (callbacks.on_progress)(position_ms.saturating_mul(GPU_PROGRESS_PERCENT) / 100);
}

unsafe extern "C" {
  pub(super) fn screenwide_gpu_composite_cursor(
    screen_path: *const c_char,
    cursors: *const NativeGpuCursor,
    cursor_count: u32,
    artworks: *const NativeGpuArtwork,
    artwork_count: u32,
    keyboards: *const crate::editor::keyboard_effects::KeyboardOverlay,
    keyboard_count: u32,
    annotations: *const NativeTimedAnnotation,
    annotation_count: u32,
    annotation_data: *const NativeAnnotationData,
    timeline_ranges: *const crate::editor::timeline_edit::TimelineRange,
    timeline_range_count: u32,
    camera_path: *const c_char,
    camera_overlay: *const GpuCameraOverlay,
    canvas: *const crate::screenshots::NativeCanvas,
    output_path: *const c_char,
    source_width: u32,
    source_height: u32,
    width: u32,
    height: u32,
    bitrate: u64,
    context: *mut c_void,
    should_cancel: unsafe extern "C" fn(*mut c_void) -> bool,
    progress: unsafe extern "C" fn(*mut c_void, u64),
    error_text: *mut c_char,
    error_capacity: usize,
  ) -> i32;
}
