// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native screenshot editing preview.
//!
//! The screenshot is a single static image, so the whole editing loop is one
//! GPU composition into the same native pane surface the recording preview
//! uses: the source uploads once (the presenter caches it by token), and each
//! settings change is a uniform-only compute pass. No pixels ever cross IPC.

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod annotation;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod annotation_gesture;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod annotation_hover;
#[cfg(all(any(target_os = "macos", target_os = "windows"), test))]
mod annotation_tests;
/// The halo's growth curve is one thing for both editors.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use annotation::hover_width_points;
mod controls;
mod geometry;
mod gesture;
#[cfg(test)]
mod gesture_tests;
mod layout;
mod payloads;
mod presentation;
mod refresh;
mod start;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod start_callbacks;
mod state;

pub use controls::{
  __cmd__reset_screenshot_preview_view, __cmd__set_screenshot_preview_editor_suspended,
  __cmd__set_screenshot_preview_fit_basis, __cmd__set_screenshot_preview_zoom,
  __cmd__stop_screenshot_preview, __tauri_command_name_reset_screenshot_preview_view,
  __tauri_command_name_set_screenshot_preview_editor_suspended,
  __tauri_command_name_set_screenshot_preview_fit_basis,
  __tauri_command_name_set_screenshot_preview_zoom, __tauri_command_name_stop_screenshot_preview,
  reset_screenshot_preview_view, set_screenshot_preview_editor_suspended,
  set_screenshot_preview_fit_basis, set_screenshot_preview_zoom, stop_screenshot_preview,
};
pub use layout::{
  __cmd__layout_screenshot_preview_surface, __tauri_command_name_layout_screenshot_preview_surface,
  layout_screenshot_preview_surface,
};
#[allow(unused_imports)]
pub use payloads::{ScreenshotSelectionOverlay, ScreenshotSurfacePane};
pub use refresh::{
  __cmd__refresh_screenshot_preview_sources,
  __tauri_command_name_refresh_screenshot_preview_sources, refresh_screenshot_preview_sources,
};
pub use start::{
  __cmd__start_screenshot_preview, __tauri_command_name_start_screenshot_preview,
  start_screenshot_preview,
};
pub use state::ScreenshotPreviewState;

// Native screenshot document extensions enter through presentation; React
// remains the semantic settings, history, and command/event transport layer.
// Only the Metal path reaches back for `run_on_main_queue`.
#[cfg(target_os = "macos")]
pub(super) use super::preview_platform;
pub(super) use super::preview_workspace_model;
