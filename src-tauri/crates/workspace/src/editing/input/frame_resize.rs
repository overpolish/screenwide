// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_output::{narrowest_in_reach, CanvasAnchor};

/// The smallest a canvas is drawn while dragged, in display points, where
/// nothing it holds asks for more.
const MIN_DRAWN: f64 = 36.0;

/// Which of a resize start's reach floors applies on an axis: 0 with the far
/// edge, 1 about the middle, 2 with the near edge.
fn reach_index(edges: u32, near: u32, centered: bool) -> usize {
  if centered {
    1
  } else if edges & near != 0 {
    2
  } else {
    0
  }
}

/// The narrowest and shortest the canvas of the pane at `index`, drawn at
/// `start`, can be dragged to while what it holds stays within reach of it,
/// in display points, for each way its corner can move (`FrameResizeStart::
/// reach`). A recording's panes are canvases of their own; a screenshot's
/// share the one canvas, so every one of them holds it.
pub(super) fn reach_floors(
  state: &SurfaceState,
  index: usize,
  start: PreviewSurfaceRect,
) -> [(f64, f64); 3] {
  let held: Vec<&ScreenshotOutputSettings> = state
    .panes
    .iter()
    .enumerate()
    .filter(|(each, _)| !state.workspace.recording || *each == index)
    .filter_map(|(_, pane)| pane.as_ref()?.settings.as_ref())
    .collect();
  [0.0, 0.5, 1.0].map(|share| {
    held.iter().fold((0.0_f64, 0.0_f64), |floor, settings| {
      let (width, height) = narrowest_in_reach(settings, CanvasAnchor { x: share, y: share });
      (
        floor
          .0
          .max(width * start.width / f64::from(settings.width.max(1))),
        floor
          .1
          .max(height * start.height / f64::from(settings.height.max(1))),
      )
    })
  })
}

pub(super) fn apply(
  inner: &std::sync::Arc<SurfaceInner>,
  state: &mut SurfaceState,
  gesture: &mut EditorGesture,
  point: (f64, f64),
  centered: bool,
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
  // A canvas already smaller than its floor is let resize freely, so it
  // can be grown back.
  let reach = state.frame_resize.as_ref().and_then(|start| start.reach);
  let floor = |size: f64, near: u32, axis: fn(&(f64, f64)) -> f64| {
    let needed = reach.map_or(0.0, |floors| {
      axis(&floors[reach_index(edges, near, centered)])
    });
    MIN_DRAWN.max(if needed <= size { needed } else { 0.0 })
  };
  let min_width = floor(start.width, 1, |floor| floor.0);
  let min_height = floor(start.height, 4, |floor| floor.1);
  let mut left = start.x;
  let mut right = start.x + start.width;
  let mut top = start.y;
  let mut bottom = start.y + start.height;
  if edges & 1 != 0 {
    let movement = raw_x.min(if centered {
      (start.width - min_width) / 2.0
    } else {
      start.width - min_width
    });
    left += movement;
    if centered {
      right -= movement;
    }
  } else if edges & 2 != 0 {
    let movement = raw_x.max(if centered {
      -(start.width - min_width) / 2.0
    } else {
      min_width - start.width
    });
    right += movement;
    if centered {
      left -= movement;
    }
  }
  if edges & 4 != 0 {
    let movement = raw_y.min(if centered {
      (start.height - min_height) / 2.0
    } else {
      start.height - min_height
    });
    top += movement;
    if centered {
      bottom -= movement;
    }
  } else if edges & 8 != 0 {
    let movement = raw_y.max(if centered {
      -(start.height - min_height) / 2.0
    } else {
      min_height - start.height
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
