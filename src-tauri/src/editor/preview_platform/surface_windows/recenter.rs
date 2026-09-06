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
    .is_some_and(|selection| selection.recenter_mode != 0 || selection.layer_id == u32::MAX - 1)
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
    .is_some_and(|mut overlay| overlay.action.down(physical).consumed);
  if hit {
    redraw_action(inner);
  }
  hit
}

pub(super) fn animate_action(inner: &SurfaceInner) -> bool {
  redraw_action(inner);
  inner
    .gpu
    .selection
    .lock()
    .is_ok_and(|overlay| overlay.action.is_animating())
}

pub(super) fn action_hit(inner: &SurfaceInner, physical: (f64, f64)) -> bool {
  inner
    .gpu
    .selection
    .lock()
    .ok()
    .is_some_and(|overlay| overlay.action.hit_index(physical) != 0)
}

pub(super) fn update_action(inner: &SurfaceInner, physical: (f64, f64)) -> (bool, bool) {
  let (hovered, changed) = inner
    .gpu
    .selection
    .lock()
    .ok()
    .map_or((false, false), |mut overlay| {
      let update = overlay.action.move_to(physical);
      (update.consumed, update.changed)
    });
  if changed {
    redraw_action(inner);
  }
  (hovered, changed)
}

pub(super) fn release_action(
  inner: &SurfaceInner,
  physical: (f64, f64),
  logical: (f64, f64),
) -> bool {
  let (activated, changed) = inner
    .gpu
    .selection
    .lock()
    .ok()
    .map_or((0, false), |mut overlay| {
      let update = overlay.action.up(physical);
      (update.activated, update.changed)
    });
  if changed {
    redraw_action(inner);
  }
  let action = (activated != 0)
    .then(|| inner.state.lock().ok())
    .flatten()
    .and_then(|state| {
      let selection = state
        .selection
        .filter(|selection| selection.recenter_mode != 0 || selection.layer_id == u32::MAX - 1)?;
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
        operation: if selection.layer_id == u32::MAX - 1 {
          if activated == 2 {
            SelectionGestureOperation::ApplyToAllAction
          } else {
            SelectionGestureOperation::ResetAction
          }
        } else {
          SelectionGestureOperation::RecenterAction
        },
        pane_start,
        pointer_start: logical,
        selection_start: selection,
        keyboard_start: None,
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
  let constraint =
    (start.recenter_mode != 0 && start.recenter_width > 0.0 && start.recenter_height > 0.0)
      .then_some(workspace_editor::NormalizedRect {
        x: start.recenter_x,
        y: start.recenter_y,
        width: start.recenter_width,
        height: start.recenter_height,
      });
  let mut resize = workspace_editor::selection_resize(
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
  );
  if start.maximum_scale > 0.0 {
    resize.maximum_scale = resize.maximum_scale.min(start.maximum_scale);
    resize.minimum_scale = resize.minimum_scale.min(resize.maximum_scale);
    resize.scale = resize
      .scale
      .clamp(resize.minimum_scale, resize.maximum_scale);
  }
  resize
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
