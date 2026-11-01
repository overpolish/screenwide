// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One pointer sample of a crop handle pulled outside every scene. A crop
//! given a shape keeps it as a corner or an edge is pulled, as a scene's box
//! does (`framed.rs`); a free one takes whatever shape the pull makes.

use super::*;

/// Resizes the crop window in `selection` by the drag `delta`, as shares of
/// `pane`, and records the gesture's report: the pulled edges' travel and
/// how far the window grew.
pub(super) fn apply(
  state: &mut SurfaceState,
  gesture: &mut EditorGesture,
  selection: &mut PreviewSelection,
  pane: PreviewSurfaceRect,
  (dx, dy): (f64, f64),
) {
  clear_selection_snap_guides(state);
  let start = gesture.selection_start;
  let crop = NormalizedRect {
    x: start.x,
    y: start.y,
    width: start.width,
    height: start.height,
  };
  let image = NormalizedRect {
    x: start.image_x,
    y: start.image_y,
    width: start.image_width,
    height: start.image_height,
  };
  let next = if selection.crop_keeps_shape != 0 {
    let zoom = state.workspace_transform.zoom;
    apply_framed_crop_resize(
      crop,
      image,
      gesture.edges,
      (dx, dy),
      (
        super::framed::CROP_MINIMUM_POINTS / (pane.width * zoom).max(1.0),
        super::framed::CROP_MINIMUM_POINTS / (pane.height * zoom).max(1.0),
      ),
    )
  } else {
    apply_crop_resize(crop, image, gesture.edges, (dx, dy), false)
  };
  selection.x = next.x;
  selection.y = next.y;
  selection.width = next.width;
  selection.height = next.height;
  let edges = gesture.edges;
  gesture.last_delta = (
    if edges & 1 != 0 {
      next.x - start.x
    } else if edges & 2 != 0 {
      next.x + next.width - start.x - start.width
    } else {
      0.0
    },
    if edges & 4 != 0 {
      next.y - start.y
    } else if edges & 8 != 0 {
      next.y + next.height - start.y - start.height
    } else {
      0.0
    },
  );
  gesture.last_scale = if start.width.abs() > f64::EPSILON {
    next.width / start.width
  } else {
    1.0
  };
}
