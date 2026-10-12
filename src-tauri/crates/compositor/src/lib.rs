// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shared GPU compositor: everything the preview and the export draw a
//! frame with, on the one wgpu device. The canvas and its background, the
//! source, the camera, the cursor, the keyboard overlay and every kind of
//! annotation, plus the workspace geometry the editor lays panes out with.
//! The app presents what it draws, in a window or into a file.

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod annotation_gpu;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod background_image;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod compositor;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod keyboard_artwork;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod object_picking;
#[cfg(target_os = "macos")]
pub mod platform_macos;
#[cfg(target_os = "windows")]
pub mod platform_windows;
pub mod workspace_editor;
pub mod workspace_transform;

#[cfg(target_os = "macos")]
pub use platform_macos as platform;
#[cfg(target_os = "windows")]
pub use platform_windows as platform;

/// A pane or viewport rectangle in webview points, relative to the window.
///
/// Shared rather than per-backend: the webview reports the same geometry
/// whatever composites it.
#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub struct PreviewSurfaceRect {
  pub height: f64,
  pub width: f64,
  pub x: f64,
  pub y: f64,
}
