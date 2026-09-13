// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native preview surface layout and visibility commands.

#[path = "surface_commands/geometry_helpers.rs"]
mod geometry_helpers;
#[path = "surface_commands/view_controls.rs"]
mod view_controls;
pub use view_controls::reset_recording_preview_view;
pub use view_controls::set_recording_preview_fit_basis;
pub use view_controls::set_recording_preview_zoom;
pub use view_controls::{
  __cmd__reset_recording_preview_view, __tauri_command_name_reset_recording_preview_view,
};
pub use view_controls::{
  __cmd__set_recording_preview_fit_basis, __tauri_command_name_set_recording_preview_fit_basis,
};
pub use view_controls::{
  __cmd__set_recording_preview_zoom, __tauri_command_name_set_recording_preview_zoom,
};

#[cfg(test)]
#[path = "surface_commands/tests.rs"]
mod tests;

#[path = "surface_commands/layout_command.rs"]
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

use crate::editor::preview_platform::workspace_editor::WorldRect;
use crate::editor::preview_platform::PreviewSurfaceRect;
use crate::editor::preview_workspace_model::WorkspacePane;
use crate::editor::CameraOverlaySettings;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSurfacePane {
  index: u32,
  rect: PreviewSurfaceRect,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingPreviewSurfaceLayout {
  backdrop: Option<[f64; 4]>,
  bake_camera: bool,
  #[serde(default)]
  fit_width: Option<f64>,
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
}
