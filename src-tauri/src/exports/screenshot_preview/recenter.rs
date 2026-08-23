// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use super::super::preview_platform::workspace_editor::{
  apply_layer_gesture, resize_uniform_inset_from_scale, GestureOperation, LayerGeometry,
  NormalizedRect,
};
use super::super::ScreenshotWorkspaceOutputSettings;
use crate::screenshots::CapturedImage;

fn resized_recenter_geometry(
  start: LayerGeometry,
  canvas: (f64, f64),
  source: (f64, f64),
  source_crop: NormalizedRect,
  edges: u32,
  scale: f64,
) -> LayerGeometry {
  let image_width = start.image_width * canvas.0;
  let image_height = image_width * source.1 / source.0.max(1.0);
  let image_x = start.image_center_x - image_width / canvas.0 / 2.0;
  let image_y = start.image_center_y - image_height / canvas.1 / 2.0;
  let visible = NormalizedRect {
    x: image_x + image_width * source_crop.x / canvas.0,
    y: image_y + image_height * source_crop.y / canvas.1,
    width: image_width * source_crop.width / canvas.0,
    height: image_height * source_crop.height / canvas.1,
  };
  let crop = resize_uniform_inset_from_scale(start.crop, visible, canvas, edges, scale).rect;
  LayerGeometry { crop, ..start }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_recenter_gesture(
  snapshot: &ScreenshotWorkspaceOutputSettings,
  sources: &[(u64, Arc<CapturedImage>)],
  pane_index: usize,
  operation: GestureOperation,
  edges: u32,
  delta: (f64, f64),
  scale: f64,
  start: LayerGeometry,
) -> LayerGeometry {
  let Some((item, source)) = snapshot.items.get(pane_index).and_then(|item| {
    sources
      .iter()
      .find(|(id, _)| *id == item.id)
      .map(|(_, source)| (item, source))
  }) else {
    return start;
  };
  let mut geometry = if operation == GestureOperation::Move {
    let mut moved = apply_layer_gesture(start, operation, delta, scale);
    moved.crop.x = start.crop.x + delta.0;
    moved.crop.y = start.crop.y + delta.1;
    moved
  } else if operation == GestureOperation::Resize {
    resized_recenter_geometry(
      start,
      (
        f64::from(snapshot.canvas.width.max(1)),
        f64::from(snapshot.canvas.height.max(1)),
      ),
      (f64::from(source.width), f64::from(source.height)),
      {
        let source_crop = item.output.source_crop;
        if source_crop.validate().is_err() {
          return start;
        }
        NormalizedRect {
          x: source_crop.x,
          y: source_crop.y,
          width: source_crop.width,
          height: source_crop.height,
        }
      },
      edges,
      scale,
    )
  } else {
    start
  };
  geometry.radius_percent = start.radius_percent;
  geometry
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inset_is_equal_in_output_pixels_for_a_wide_source() {
    let start = LayerGeometry {
      crop: NormalizedRect {
        height: 0.5,
        width: 0.5,
        x: 0.25,
        y: 0.25,
      },
      image_center_x: 0.5,
      image_center_y: 0.5,
      image_width: 0.8,
      radius_percent: 0.0,
    };
    let geometry = resized_recenter_geometry(
      start,
      (1_000.0, 500.0),
      (1_000.0, 500.0),
      NormalizedRect {
        height: 0.5,
        width: 0.5,
        x: 0.25,
        y: 0.25,
      },
      2,
      1.2,
    );
    let horizontal = (geometry.crop.width * 1_000.0 - 400.0) / 2.0;
    let vertical = (geometry.crop.height * 500.0 - 200.0) / 2.0;
    assert!((horizontal - vertical).abs() < 1e-9);
    assert_eq!(geometry.image_center_x, start.image_center_x);
    assert_eq!(geometry.image_center_y, start.image_center_y);
    assert_eq!(geometry.image_width, start.image_width);
  }
}
