// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drags on a selection a scene has placed. The scene fixes where its panes
//! sit, so the outline stays where it is: a move reports how far the pointer
//! has travelled, which pans the picture inside the pane, and a corner reports
//! how far it has been pulled from the pane's middle, which zooms it. A crop
//! window the crop tool lays over a scene's picture keeps the box's shape as
//! it is resized. Nothing snaps or grows the canvas. The twin of
//! `recording_preview_surface_macos+framed.m`.

use super::*;

/// The zoom one corner drag reaches at most each way, matching the camera
/// framing's own limits.
const SCALE_LIMIT: f64 = 8.0;

/// The shortest a framed crop window's side may be pulled, in display points.
const CROP_MINIMUM_POINTS: f64 = 36.0;

/// Records the drag `delta`, as shares of `pane`, as the gesture's report: a
/// framed crop window's new place in `selection`, reported as its corner's
/// travel and how far it grew.
pub(super) fn apply(
  state: &mut SurfaceState,
  gesture: &mut EditorGesture,
  selection: &mut PreviewSelection,
  pane: PreviewSurfaceRect,
  (dx, dy): (f64, f64),
) {
  clear_selection_snap_guides(state);
  if gesture.operation == SelectionGestureOperation::Move {
    gesture.last_delta = (dx, dy);
    return;
  }
  if gesture.operation == SelectionGestureOperation::CropResize {
    let start = gesture.selection_start;
    let zoom = state.workspace_transform.zoom;
    let next = apply_framed_crop_resize(
      NormalizedRect {
        x: start.x,
        y: start.y,
        width: start.width,
        height: start.height,
      },
      NormalizedRect {
        x: start.image_x,
        y: start.image_y,
        width: start.image_width,
        height: start.image_height,
      },
      gesture.edges,
      (dx, dy),
      (
        CROP_MINIMUM_POINTS / (pane.width * zoom).max(1.0),
        CROP_MINIMUM_POINTS / (pane.height * zoom).max(1.0),
      ),
    );
    selection.x = next.x;
    selection.y = next.y;
    selection.width = next.width;
    selection.height = next.height;
    gesture.last_delta = (next.x - start.x, next.y - start.y);
    gesture.last_scale = next.width / start.width.max(f64::EPSILON);
    return;
  }
  let start = gesture.selection_start;
  let edges = gesture.edges;
  let middle = (start.x + start.width / 2.0, start.y + start.height / 2.0);
  let handle_x = if edges & 1 != 0 {
    start.x
  } else if edges & 2 != 0 {
    start.x + start.width
  } else {
    middle.0
  };
  let handle_y = if edges & 4 != 0 {
    start.y
  } else if edges & 8 != 0 {
    start.y + start.height
  } else {
    middle.1
  };
  let vector = (
    (handle_x - middle.0) * pane.width,
    (handle_y - middle.1) * pane.height,
  );
  let length = vector.0 * vector.0 + vector.1 * vector.1;
  let scale = if length > 0.0 {
    ((vector.0 + dx * pane.width) * vector.0 + (vector.1 + dy * pane.height) * vector.1) / length
  } else {
    1.0
  };
  gesture.last_delta = (0.0, 0.0);
  gesture.last_scale = scale.clamp(1.0 / SCALE_LIMIT, SCALE_LIMIT);
}
