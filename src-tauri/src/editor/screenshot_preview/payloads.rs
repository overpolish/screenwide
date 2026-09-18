// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::preview_platform::PreviewSurfaceRect;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotSurfacePane {
  #[allow(dead_code)]
  pub(super) index: u32,
  pub(super) rect: PreviewSurfaceRect,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotSelectionOverlay {
  #[serde(default)]
  pub(super) crop_mode: bool,
  #[serde(default)]
  pub(super) image: Option<PreviewSurfaceRect>,
  pub(super) layer_id: Option<u32>,
  pub(super) pane_index: u32,
  pub(super) radius_percent: f64,
  #[serde(default)]
  pub(super) recenter_bounds: Option<PreviewSurfaceRect>,
  pub(super) rect: PreviewSurfaceRect,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScreenshotPreviewTransformEvent {
  pub(super) session_id: u64,
  pub(super) zoom_percent: f64,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScreenshotSelectionGestureEvent {
  pub(super) delta_x: f64,
  pub(super) delta_y: f64,
  pub(super) edges: u32,
  pub(super) operation: u32,
  pub(super) pane_index: u32,
  pub(super) phase: &'static str,
  pub(super) scale: f64,
  pub(super) session_id: u64,
}

/// The layer's annotations after a pointer gesture, for React to commit into
/// the document and its edit history.
#[cfg(any(target_os = "macos", target_os = "windows"))]
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScreenshotAnnotationChangeEvent {
  pub(super) annotations: Vec<crate::editor::annotations::Annotation>,
  pub(super) pane_index: u32,
  pub(super) selected_annotation_id: Option<String>,
  pub(super) session_id: u64,
}

/// Which annotation the pointer is resting on, so the keyboard - which belongs
/// to the webview - can act on what the halo is showing.
#[cfg(any(target_os = "macos", target_os = "windows"))]
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScreenshotAnnotationHoverEvent {
  pub(super) annotation_id: Option<String>,
  pub(super) session_id: u64,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScreenshotSelectionChangeEvent {
  pub(super) pane_index: Option<u32>,
  pub(super) session_id: u64,
}
