// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::layout::PreviewPane;
use crate::editor::preview_platform::workspace_editor::{
  apply_layer_gesture, fit_canvas_to_layers, GestureOperation, NormalizedRect,
};
use crate::editor::preview_workspace_model::{apply_output_geometry, output_geometry};
use crate::screenshots::ScreenshotOutputSettings;

#[allow(clippy::too_many_arguments)]
pub(super) fn apply(
  start: &ScreenshotOutputSettings,
  output: &mut ScreenshotOutputSettings,
  source: Option<&PreviewPane>,
  recenter_mode: bool,
  operation: GestureOperation,
  edges: u32,
  scale: f64,
  delta: (f64, f64),
  auto_fit_edge: u32,
) -> bool {
  let start_geometry = output_geometry(start);
  let mut geometry = if recenter_mode && operation == GestureOperation::Resize {
    let Some(source) = source else {
      return false;
    };
    let source_crop = start.source_crop;
    if source_crop.validate().is_err() {
      return false;
    }
    super::super::screenshot_preview::recenter::resized_recenter_geometry(
      start_geometry,
      (
        f64::from(start.width.max(1)),
        f64::from(start.height.max(1)),
      ),
      (
        f64::from(source.source_width),
        f64::from(source.source_height),
      ),
      NormalizedRect {
        x: source_crop.x,
        y: source_crop.y,
        width: source_crop.width,
        height: source_crop.height,
      },
      edges,
      scale,
    )
  } else {
    apply_layer_gesture(start_geometry, operation, delta, scale)
  };
  if operation == GestureOperation::Move && !recenter_mode && edges & auto_fit_edge != 0 {
    let ((width, height), fitted) = fit_canvas_to_layers((start.width, start.height), &[geometry]);
    geometry = fitted[0];
    output.width = width;
    output.height = height;
  }
  apply_output_geometry(output, geometry);
  true
}
