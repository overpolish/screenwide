// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn apply(
  inner: &std::sync::Arc<SurfaceInner>,
  state: &mut SurfaceState,
  gesture: &mut EditorGesture,
  point: (f64, f64),
  centered: bool,
  zoom: &mut Option<f64>,
) {
  clear_selection_snap_guides(state);
  let start = gesture.pane_start;
  // Convert display travel with the immutable starting zoom.
  let start_zoom = state
    .frame_resize
    .as_ref()
    .map_or(state.workspace_transform.zoom, |start| start.transform.zoom);
  let raw_x = (point.0 - gesture.pointer_start.0) / start_zoom.max(0.0001);
  let raw_y = (point.1 - gesture.pointer_start.1) / start_zoom.max(0.0001);
  let edges = gesture.edges & !CENTERED_RESIZE_EDGE;
  let mut left = start.x;
  let mut right = start.x + start.width;
  let mut top = start.y;
  let mut bottom = start.y + start.height;
  if edges & 1 != 0 {
    let movement = raw_x.min(if centered {
      (start.width - 36.0) / 2.0
    } else {
      start.width - 36.0
    });
    left += movement;
    if centered {
      right -= movement;
    }
  } else if edges & 2 != 0 {
    let movement = raw_x.max(if centered {
      -(start.width - 36.0) / 2.0
    } else {
      36.0 - start.width
    });
    right += movement;
    if centered {
      left -= movement;
    }
  }
  if edges & 4 != 0 {
    let movement = raw_y.min(if centered {
      (start.height - 36.0) / 2.0
    } else {
      start.height - 36.0
    });
    top += movement;
    if centered {
      bottom -= movement;
    }
  } else if edges & 8 != 0 {
    let movement = raw_y.max(if centered {
      -(start.height - 36.0) / 2.0
    } else {
      36.0 - start.height
    });
    bottom += movement;
    if centered {
      top -= movement;
    }
  }
  let resized = PreviewSurfaceRect {
    x: left,
    y: top,
    width: right - left,
    height: bottom - top,
  };
  let selected = gesture.selection_start.pane_index as usize;
  // Re-derive the whole workspace from the gesture's starts:
  // the dragged canvas, then its siblings re-flowed around
  // it, then one rebase that re-expresses zoom/pan so none of
  // it moves on screen. Without the starts the row would
  // accumulate its own re-flow each move.
  if let Some(start_state) = state.frame_resize.take() {
    let reflowed = reflow_workspace_panes(&start_state.pane_rects, selected, resized);
    for (index, rect) in reflowed {
      if let Some(pane) = state.panes.get_mut(index).and_then(Option::as_mut) {
        pane.base_rect = rect;
      }
    }
    rebase_workspace_fit(state, &start_state);
    *zoom = Some(state.workspace_transform.zoom);
    state.frame_resize = Some(start_state);
  } else if let Some(pane) = state.panes.get_mut(selected).and_then(Option::as_mut) {
    pane.base_rect = resized;
  }
  gesture.edges = edges | if centered { CENTERED_RESIZE_EDGE } else { 0 };
  gesture.last_delta = (raw_x / start.width.max(1.0), raw_y / start.height.max(1.0));
  gesture.last_scale = 1.0;
  // The synchronous present publishes pixels and boxes from
  // the same pointer sample; uniform fit handles model limits.
  apply_workspace_transform(inner, state, true);
}
