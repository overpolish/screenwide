// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
  editor::{cursor_effects::GpuCursor, keyboard_effects::KeyboardOverlay},
  screenshots::{CapturedImage, ScreenshotOutputSettings, StillOverlay},
};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct NativeWorkspacePlacement {
  pub(super) x: i32,
  pub(super) y: i32,
  pub(super) width: u32,
  pub(super) height: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct NativeWorkspacePaneRect {
  pub(super) index: u32,
  pub(super) x: f64,
  pub(super) y: f64,
  pub(super) width: f64,
  pub(super) height: f64,
}

/// Input for one layer in the retained recording workspace. A decoded RGBA
/// image or a native CVPixelBuffer may be supplied, and a camera frame
/// composed into it beside the cursor and the shortcut strip.
pub(crate) struct RecordingWorkspaceLayer<'a> {
  pub pane_index: u32,
  pub source_token: u64,
  pub source: Option<&'a CapturedImage>,
  pub source_pixels: Option<(*mut std::ffi::c_void, (u32, u32))>,
  pub settings: ScreenshotOutputSettings,
  pub placement: NativeWorkspacePlacement,
  pub seconds: f64,
  pub cursor: Option<GpuCursor>,
  pub keyboard: Option<KeyboardOverlay>,
  pub camera: Option<&'a CapturedImage>,
  pub camera_pixels: Option<(*mut std::ffi::c_void, (u32, u32))>,
  pub overlay: Option<&'a StillOverlay>,
  /// In One video, the camera's own settings, carrying the annotations drawn
  /// into its frame, and the camera's source size, which they are placed in.
  pub camera_settings: Option<(&'a ScreenshotOutputSettings, (u32, u32))>,
  pub clip_cursor_at_video_edge: bool,
  pub foreground_only: bool,
}
