// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::preview_platform::{PreviewSelection, PreviewSurfaceRect};

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

impl ScreenshotSelectionOverlay {
  /// The overlay as the native surface reads a layer: on macOS every layer
  /// is drawn into the one workspace pane, so its pane is always 0 there.
  pub(super) fn preview_selection(self) -> PreviewSelection {
    PreviewSelection {
      recenter_height: self.recenter_bounds.map_or(0.0, |bounds| bounds.height),
      recenter_width: self.recenter_bounds.map_or(0.0, |bounds| bounds.width),
      recenter_x: self.recenter_bounds.map_or(0.0, |bounds| bounds.x),
      recenter_y: self.recenter_bounds.map_or(0.0, |bounds| bounds.y),
      crop_mode: u32::from(self.crop_mode),
      image_height: self.image.map_or(0.0, |image| image.height),
      image_width: self.image.map_or(0.0, |image| image.width),
      image_x: self.image.map_or(0.0, |image| image.x),
      image_y: self.image.map_or(0.0, |image| image.y),
      layer_id: self.layer_id.unwrap_or(self.pane_index),
      #[cfg(target_os = "macos")]
      pane_index: 0,
      #[cfg(not(target_os = "macos"))]
      pane_index: self.pane_index,
      x: self.rect.x,
      y: self.rect.y,
      width: self.rect.width,
      height: self.rect.height,
      radius_percent: self.radius_percent,
      minimum_scale: 0.0,
      maximum_scale: 0.0,
      ..PreviewSelection::default()
    }
  }
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScreenshotPreviewTransformEvent {
  pub(super) session_id: u64,
  pub(super) zoom_percent: f64,
}

#[derive(Clone, serde::Serialize, ts_rs::TS)]
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

/// The layer's annotations after a pointer gesture, or as a text box is typed
/// into, for React to commit into the document and its edit history.
/// `text_edit` is where in the typing the commit falls; React groups a
/// typing's commits into one edit.
#[cfg(any(target_os = "macos", target_os = "windows"))]
#[derive(Clone, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScreenshotAnnotationChangeEvent {
  pub(super) annotations: Vec<crate::editor::annotations::Annotation>,
  pub(super) pane_index: u32,
  pub(super) selected_annotation_ids: Vec<String>,
  pub(super) session_id: u64,
  pub(super) text_edit: Option<crate::editor::annotations::text::edit::TextEditPhase>,
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

/// A right press on an annotation, for React to open its menu at `x`, `y` in
/// the window's content. `pane_index` is the layer the annotation is drawn on.
#[cfg(any(target_os = "macos", target_os = "windows"))]
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScreenshotAnnotationMenuEvent {
  pub(super) annotation_id: String,
  pub(super) pane_index: u32,
  pub(super) session_id: u64,
  pub(super) x: f64,
  pub(super) y: f64,
}

/// A right press on a bare layer, for React to open the layer's menu at `x`,
/// `y` in the window's content. `pane_index` is the layer's workspace order.
#[cfg(any(target_os = "macos", target_os = "windows"))]
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScreenshotLayerMenuEvent {
  pub(super) pane_index: u32,
  pub(super) session_id: u64,
  pub(super) x: f64,
  pub(super) y: f64,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ScreenshotSelectionChangeEvent {
  pub(super) pane_index: Option<u32>,
  pub(super) session_id: u64,
}
