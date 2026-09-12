// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{workspace_editor, PreviewSelection};

#[derive(Clone, Copy)]
pub(super) struct CropMagnifier {
  pub(super) bounds: [f32; 4],
  pub(super) display_box: [f32; 4],
  pub(super) geometry: [f32; 4],
  pub(super) options: [f32; 4],
}

pub(super) fn magnifier_bounds(selection: PreviewSelection) -> [f32; 4] {
  if selection.recenter_width <= 0.0 || selection.recenter_height <= 0.0 {
    return [0.0, 0.0, 1.0, 1.0];
  }
  [
    ((selection.image_x - selection.recenter_x) / selection.recenter_width) as f32,
    ((selection.image_y - selection.recenter_y) / selection.recenter_height) as f32,
    (selection.image_width / selection.recenter_width) as f32,
    (selection.image_height / selection.recenter_height) as f32,
  ]
}

pub(super) fn selection_resize(
  start: PreviewSelection,
  edges: u32,
  delta: (f64, f64),
  pane: (f64, f64),
  zoom: f64,
  centered: bool,
) -> workspace_editor::SelectionResize {
  let minimum = if start.layer_id == u32::MAX - 1 {
    0.01
  } else {
    (36.0 / (pane.0 * zoom * start.width).max(1.0))
      .max(36.0 / (pane.1 * zoom * start.height).max(1.0))
  };
  let minimum = if start.minimum_scale > 0.0 {
    minimum.max(start.minimum_scale)
  } else {
    minimum
  };
  let mut resize = workspace_editor::selection_resize(
    workspace_editor::NormalizedRect {
      x: start.x,
      y: start.y,
      width: start.width,
      height: start.height,
    },
    edges,
    delta,
    minimum,
    centered,
  );
  super::keyboard_hit::resize_limit(start, &mut resize);
  if start.maximum_scale > 0.0 {
    resize.maximum_scale = resize.maximum_scale.min(start.maximum_scale);
    resize.minimum_scale = resize.minimum_scale.min(resize.maximum_scale);
    resize.scale = resize
      .scale
      .clamp(resize.minimum_scale, resize.maximum_scale);
  }
  resize
}
