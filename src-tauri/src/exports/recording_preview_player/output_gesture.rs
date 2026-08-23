// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::layout::PreviewPane;
use crate::exports::preview_platform::workspace_editor::{
  apply_layer_gesture, fit_canvas_to_layers, GestureOperation, LayerGeometry, NormalizedRect,
};
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
  let start_geometry = LayerGeometry {
    crop: NormalizedRect {
      x: start.screenshot_crop_x_percent / 100.0,
      y: start.screenshot_crop_y_percent / 100.0,
      width: start.screenshot_crop_width_percent / 100.0,
      height: start.screenshot_crop_height_percent / 100.0,
    },
    image_center_x: start.screenshot_image_x_percent / 100.0,
    image_center_y: start.screenshot_image_y_percent / 100.0,
    image_width: start.screenshot_image_width_percent / 100.0,
    radius_percent: start.radius_percent,
  };
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
  output.screenshot_crop_x_percent = geometry.crop.x * 100.0;
  output.screenshot_crop_y_percent = geometry.crop.y * 100.0;
  output.screenshot_crop_width_percent = geometry.crop.width * 100.0;
  output.screenshot_crop_height_percent = geometry.crop.height * 100.0;
  output.screenshot_image_x_percent = geometry.image_center_x * 100.0;
  output.screenshot_image_y_percent = geometry.image_center_y * 100.0;
  output.screenshot_image_width_percent = geometry.image_width * 100.0;
  output.radius_percent = geometry.radius_percent;
  true
}
