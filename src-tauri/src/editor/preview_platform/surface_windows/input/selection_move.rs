// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn apply(
  inner: &std::sync::Arc<SurfaceInner>,
  state: &mut SurfaceState,
  gesture: &mut EditorGesture,
  selection: &mut PreviewSelection,
  pane: PreviewSurfaceRect,
  point: (f64, f64),
  centered: bool,
  snapping: bool,
  zoom: &mut Option<f64>,
) {
  let auto_fit_active = state.move_auto_fit.as_ref().is_some_and(|fit| fit.active);
  if auto_fit_active && !centered {
    // Rebase the remaining drag onto the accepted grown canvas
    // while its one edit-history transaction remains open.
    state.frame_resize = None;
    state.frame_resize_committed = true;
    // The committed canvas becomes the move's starting point,
    // so Alt can grow it again later in this same gesture:
    // re-express the mouse-down targets and canvas size in it.
    if let Some(fit) = state.move_auto_fit.as_mut() {
      fit.active = false;
      if let Some(bounds) = fit.last_bounds.take() {
        for target in &mut fit.targets_start {
          target.x = (target.x - bounds.x) / bounds.width;
          target.y = (target.y - bounds.y) / bounds.height;
          target.width /= bounds.width;
          target.height /= bounds.height;
        }
        fit.natural_size = fit.natural_size.map(|(width, height)| {
          (
            (width * bounds.width).round().max(1.0),
            (height * bounds.height).round().max(1.0),
          )
        });
      }
    }
    clear_selection_snap_guides(state);
    // The displayed selection is already expressed in the
    // grown canvas; it becomes the new gesture origin, and this
    // sample's pointer travel is absorbed by the checkpoint.
    *selection = state.selection.unwrap_or(gesture.selection_start);
    gesture.selection_start = *selection;
    gesture.pane_start = selection_pane_rect(state, *selection);
    gesture.pointer_start = point;
    gesture.last_delta = (0.0, 0.0);
    gesture.last_scale = 1.0;
    // Cleared again by the next sample (see below).
    gesture.edges = AUTO_FIT_COMMIT_EDGE;
  } else {
    let auto_fit = centered && state.move_auto_fit.is_some();
    if auto_fit && state.frame_resize.is_none() {
      // The first Alt sample takes over the current pane box.
      if let Some(size) = state.workspace_natural_size {
        let transform = state.workspace_transform;
        state.workspace_transforms.insert(size, transform);
      }
      gesture.pane_start = selection_pane_rect(state, gesture.selection_start);
      state.frame_resize = Some(frame_resize_start(state));
    }
    // Re-derive grown samples from mouse-down geometry.
    let (move_pane, move_zoom) = state
      .frame_resize
      .as_ref()
      .map_or((pane, state.workspace_transform.zoom), |start| {
        (gesture.pane_start, start.transform.zoom)
      });
    let dx = (point.0 - gesture.pointer_start.0) / (move_pane.width * move_zoom).max(1.0);
    let dy = (point.1 - gesture.pointer_start.1) / (move_pane.height * move_zoom).max(1.0);
    selection.x += dx;
    selection.y += dy;
    gesture.last_delta = (dx, dy);
    if state.selection_snapping_enabled && snapping {
      let targets_x = selection_snap_targets(state, gesture.selection_start, true);
      let targets_y = selection_snap_targets(state, gesture.selection_start, false);
      let horizontal = snapping::move_axis(
        selection.x,
        selection.width,
        pane.width,
        pane.height,
        state.workspace_transform.zoom,
        &targets_x,
        selection.layer_id,
      );
      let vertical = snapping::move_axis(
        selection.y,
        selection.height,
        pane.height,
        pane.width,
        state.workspace_transform.zoom,
        &targets_y,
        selection.layer_id,
      );
      if horizontal.found {
        selection.x += horizontal.adjustment;
      }
      if vertical.found {
        selection.y += vertical.adjustment;
      }
      state.selection_snap_guide_x = horizontal.found.then_some(horizontal);
      state.selection_snap_guide_y = vertical.found.then_some(vertical);
      gesture.last_delta = (
        selection.x - gesture.selection_start.x,
        selection.y - gesture.selection_start.y,
      );
    } else {
      clear_selection_snap_guides(state);
    }
    keyboard_hit::clamp_move(selection, gesture, state);
    gesture.edges = if auto_fit { AUTO_FIT_MOVE_EDGE } else { 0 };
    if auto_fit {
      // Grow the canvas around the move: the pane box follows
      // the whole-pixel bounds of every layer, its siblings
      // re-flow around it and one rebase keeps the pixels still
      // (all from the gesture's starts, as for a Frame resize).
      // The gesture emitted below re-composes the fitted canvas
      // synchronously and that present publishes this box.
      let bounds = state.move_auto_fit.as_ref().map_or(
        PreviewSurfaceRect {
          x: 0.0,
          y: 0.0,
          width: 1.0,
          height: 1.0,
        },
        |fit| auto_fit_selection_bounds(fit, *selection),
      );
      let start = gesture.pane_start;
      let resized = PreviewSurfaceRect {
        x: start.x + bounds.x * start.width,
        y: start.y + bounds.y * start.height,
        width: bounds.width * start.width,
        height: bounds.height * start.height,
      };
      let selected = gesture.selection_start.pane_index as usize;
      if let Some(start_state) = state.frame_resize.take() {
        let reflowed = reflow_workspace_panes(&start_state.pane_rects, selected, resized);
        for (index, rect) in reflowed {
          if let Some(pane) = state.panes.get_mut(index).and_then(Option::as_mut) {
            pane.base_rect = rect;
          }
        }
        rebase_workspace_fit(state, &start_state);
        state.frame_resize = Some(start_state);
        *zoom = Some(state.workspace_transform.zoom);
      }
      if let Some(fit) = state.move_auto_fit.as_mut() {
        fit.active = true;
        fit.last_bounds = Some(bounds);
      }
      // The gesture keeps reporting mouse-down canvas units; only
      // the displayed selection is renormalised into the grown
      // canvas, matching the layers the managers fit into it.
      selection.x = (selection.x - bounds.x) / bounds.width;
      selection.y = (selection.y - bounds.y) / bounds.height;
      selection.width /= bounds.width;
      selection.height /= bounds.height;
      apply_workspace_transform(inner, state, true);
    }
  }
}
