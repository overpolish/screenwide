// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::editor::preview_platform::workspace_editor::{
  apply_layer_gesture, fit_canvas_to_layers, GestureOperation,
};
use crate::editor::preview_workspace_model::{apply_output_geometry, output_geometry};
use crate::screenshots::ScreenshotOutputSettings;

pub(super) fn apply(
  start: &ScreenshotOutputSettings,
  output: &mut ScreenshotOutputSettings,
  operation: GestureOperation,
  edges: u32,
  scale: f64,
  delta: (f64, f64),
  auto_fit_edge: u32,
) {
  let mut geometry = apply_layer_gesture(output_geometry(start), operation, delta, scale);
  if operation == GestureOperation::Move && edges & auto_fit_edge != 0 {
    let ((width, height), fitted) = fit_canvas_to_layers((start.width, start.height), &[geometry]);
    geometry = fitted[0];
    output.width = width;
    output.height = height;
  }
  apply_output_geometry(output, geometry);
}
