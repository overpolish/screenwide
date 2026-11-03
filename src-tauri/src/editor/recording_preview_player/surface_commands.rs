// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native preview surface layout and visibility commands.

mod geometry_helpers;

#[cfg(test)]
mod tests;

mod layout_command;
pub use layout_command::layout_recording_preview_surface;
pub use layout_command::{
  __cmd__layout_recording_preview_surface, __tauri_command_name_layout_recording_preview_surface,
};

use geometry_helpers::{
  clear_inactive_pane_targets, recording_workspace_geometry, workspace_topology,
};

use super::surface_selection::RecordingPreviewSelection;
use super::*;

use crate::editor::preview_workspace_model::WorkspacePane;
use screenwide_compositor::workspace_editor::WorldRect;
use screenwide_compositor::{PreviewSurfaceCorners, PreviewSurfaceRect};
use screenwide_scenes::CameraOverlaySettings;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSurfacePane {
  index: u32,
  rect: PreviewSurfaceRect,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingPreviewSurfaceLayout {
  /// The annotation tool in hand, when one is. It rides with the layout
  /// because it decides which chrome owns the screen and the selection it
  /// has to agree with travels in this same payload.
  #[serde(default)]
  annotation_tool: Option<String>,
  backdrop: Option<[f64; 4]>,
  bake_camera: bool,
  camera_overlay: CameraOverlaySettings,
  native_editor: bool,
  panes: Vec<PreviewSurfacePane>,
  recording_output: RecordingOutputSettings,
  request_id: u64,
  scale: f64,
  selection: Option<RecordingPreviewSelection>,
  selection_targets: Option<Vec<RecordingPreviewSelection>>,
  session_id: u64,
  viewport: PreviewSurfaceRect,
  /// How round the viewport's corners are where it meets the chrome.
  viewport_corners: PreviewSurfaceCorners,
  /// Whether the picture stays where it is on screen as the viewport changes:
  /// sent when the page moved only the viewport's bottom edge, as the strip
  /// under it grows or shrinks, and not when the window was resized.
  #[serde(default)]
  keep_view: bool,
}
