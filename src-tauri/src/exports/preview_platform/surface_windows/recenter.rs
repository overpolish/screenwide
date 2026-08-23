// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{
  workspace_editor, PreviewSelection, SelectionGestureOperation, SelectionGesturePhase,
};
use super::{
  draw_selection, emit_gesture, selection_pane_rect, EditorGesture, SurfaceInner, SurfaceState,
};

#[derive(Clone, Copy)]
pub(super) struct CropMagnifier {
  pub(super) bounds: [f32; 4],
  pub(super) display_box: [f32; 4],
  pub(super) geometry: [f32; 4],
  pub(super) options: [f32; 4],
}

pub(super) fn label_origin(
  frame: [f32; 4],
  label_size: (f32, f32),
  viewport: (f32, f32),
  gap: f32,
  action: bool,
) -> (f32, f32) {
  let (width, height) = label_size;
  let mut x = if action {
    frame[0] + (frame[2] - width) * 0.5
  } else {
    frame[0] + frame[2] - width
  };
  let mut y = frame[1] + frame[3] + gap;
  if y + height > viewport.1 {
    y = frame[1] + frame[3] - gap - height;
  }
  x = x.min(viewport.0 - width).max(0.0);
  let maximum_x = frame[0] + frame[2] - width;
  x = if frame[0] <= maximum_x {
    x.clamp(frame[0], maximum_x)
  } else {
    frame[0] + (frame[2] - width) * 0.5
  };
  (x, y.max(0.0).min(frame[1] + frame[3] + gap))
}

pub(super) fn action_visible(state: &SurfaceState) -> bool {
  state
    .selection
    .is_some_and(|selection| selection.recenter_mode != 0)
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

fn redraw_action(inner: &SurfaceInner) {
  if let Ok(state) = inner.state.lock() {
    draw_selection(inner, &state);
  }
}

pub(super) fn begin_action(inner: &SurfaceInner, physical: (f64, f64)) -> bool {
  let hit = inner
    .gpu
    .selection
    .lock()
    .ok()
    .is_some_and(|mut overlay| overlay.action.down(physical));
  if hit {
    redraw_action(inner);
  }
  hit
}

pub(super) fn update_action(inner: &SurfaceInner, physical: (f64, f64)) -> bool {
  let (hovered, changed) = inner
    .gpu
    .selection
    .lock()
    .ok()
    .map_or((false, false), |mut overlay| {
      overlay.action.move_to(physical)
    });
  if changed {
    redraw_action(inner);
  }
  hovered
}

pub(super) fn release_action(
  inner: &SurfaceInner,
  physical: (f64, f64),
  logical: (f64, f64),
) -> bool {
  let (activate, changed) = inner
    .gpu
    .selection
    .lock()
    .ok()
    .map_or((false, false), |mut overlay| overlay.action.up(physical));
  if changed {
    redraw_action(inner);
  }
  let action = activate
    .then(|| inner.state.lock().ok())
    .flatten()
    .and_then(|state| {
      let selection = state
        .selection
        .filter(|selection| selection.recenter_mode != 0)?;
      Some((selection, selection_pane_rect(&state, selection)))
    });
  if let Some((selection, pane_start)) = action {
    emit_gesture(
      inner,
      SelectionGesturePhase::Begin,
      EditorGesture {
        edges: 0,
        last_delta: (0.0, 0.0),
        last_scale: 1.0,
        operation: SelectionGestureOperation::RecenterAction,
        pane_start,
        pointer_start: logical,
        selection_start: selection,
      },
    );
  }
  changed
}

pub(super) fn selection_resize(
  start: PreviewSelection,
  edges: u32,
  delta: (f64, f64),
  pane: (f64, f64),
  zoom: f64,
  centered: bool,
) -> workspace_editor::SelectionResize {
  let minimum = (36.0 / (pane.0 * zoom * start.width).max(1.0))
    .max(36.0 / (pane.1 * zoom * start.height).max(1.0));
  let constraint =
    (start.recenter_mode != 0 && start.recenter_width > 0.0 && start.recenter_height > 0.0)
      .then_some(workspace_editor::NormalizedRect {
        x: start.recenter_x,
        y: start.recenter_y,
        width: start.recenter_width,
        height: start.recenter_height,
      });
  workspace_editor::selection_resize(
    workspace_editor::NormalizedRect {
      x: start.x,
      y: start.y,
      width: start.width,
      height: start.height,
    },
    constraint,
    edges,
    delta,
    minimum,
    centered,
  )
}

pub(super) fn inset_resize(
  start: PreviewSelection,
  edges: u32,
  delta: (f64, f64),
  pane: (f64, f64),
) -> (workspace_editor::NormalizedRect, f64) {
  let resize = workspace_editor::resize_uniform_inset(
    workspace_editor::NormalizedRect {
      x: start.x,
      y: start.y,
      width: start.width,
      height: start.height,
    },
    workspace_editor::NormalizedRect {
      x: start.image_x,
      y: start.image_y,
      width: start.image_width,
      height: start.image_height,
    },
    pane,
    edges,
    delta,
  );
  (resize.rect, resize.scale)
}
