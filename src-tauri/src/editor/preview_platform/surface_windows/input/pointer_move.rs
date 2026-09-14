// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn pointer_move(
  inner: &std::sync::Arc<SurfaceInner>,
  scale: f64,
  centered: bool,
  x: f64,
  y: f64,
  pressed: bool,
  snapping: bool,
) {
  let point = (x / scale, y / scale);
  let mut update = None;
  let mut zoom = None;
  if let Ok(mut state) = inner.state.lock() {
    state.last_pointer = point;
    if pressed {
      match state.gesture {
        Some(ActiveGesture::Pan {
          pointer_start,
          transform_start,
        }) => {
          state.workspace_transform.pan_x = transform_start.pan_x + point.0 - pointer_start.0;
          state.workspace_transform.pan_y = transform_start.pan_y + point.1 - pointer_start.1;
          apply_workspace_transform(inner, &mut state, false);
        }
        Some(ActiveGesture::Selection(mut gesture)) => {
          let owns_geometry = state.frame_resize.is_some();
          let pane = state
            .panes
            .get(gesture.selection_start.pane_index as usize)
            .and_then(Option::as_ref)
            .map(|pane| pane_canvas_rect(pane, owns_geometry));
          if let Some(pane) = pane {
            let dx = (point.0 - gesture.pointer_start.0)
              / (pane.width * state.workspace_transform.zoom).max(1.0);
            let dy = (point.1 - gesture.pointer_start.1)
              / (pane.height * state.workspace_transform.zoom).max(1.0);
            let mut selection = gesture.selection_start;
            if gesture.operation == SelectionGestureOperation::FrameResize {
              super::frame_resize::apply(
                inner,
                &mut state,
                &mut gesture,
                point,
                centered,
                &mut zoom,
              );
            } else if matches!(
              gesture.operation,
              SelectionGestureOperation::Radius | SelectionGestureOperation::FrameRadius
            ) {
              clear_selection_snap_guides(&mut state);
              let frame =
                display_selection(&state, gesture.selection_start).unwrap_or(PreviewSurfaceRect {
                  height: 1.0,
                  width: 1.0,
                  x: 0.0,
                  y: 0.0,
                });
              let shortest = frame.width.min(frame.height).max(1.0);
              let radius = (((point.0 - frame.x) + (point.1 - frame.y)) / 2.0 - 10.0) / 0.55;
              selection.radius_percent = (radius * 100.0 / shortest).clamp(0.0, 50.0);
              gesture.last_scale = selection.radius_percent;
            } else if gesture.operation == SelectionGestureOperation::Move {
              super::selection_move::apply(
                inner,
                &mut state,
                &mut gesture,
                &mut selection,
                super::selection_move::MoveSample {
                  pane,
                  point,
                  centered,
                  snapping,
                },
                &mut zoom,
              );
            } else if gesture.operation == SelectionGestureOperation::CropMove {
              clear_selection_snap_guides(&mut state);
              let crop = NormalizedRect {
                x: selection.x,
                y: selection.y,
                width: selection.width,
                height: selection.height,
              };
              let image = NormalizedRect {
                x: gesture.selection_start.image_x,
                y: gesture.selection_start.image_y,
                width: gesture.selection_start.image_width,
                height: gesture.selection_start.image_height,
              };
              let next = apply_crop_move(crop, image, (dx, dy));
              selection.x = next.x;
              selection.y = next.y;
              gesture.last_delta = (
                selection.x - gesture.selection_start.x,
                selection.y - gesture.selection_start.y,
              );
            } else if gesture.operation == SelectionGestureOperation::CropResize {
              clear_selection_snap_guides(&mut state);
              let crop = NormalizedRect {
                x: gesture.selection_start.x,
                y: gesture.selection_start.y,
                width: gesture.selection_start.width,
                height: gesture.selection_start.height,
              };
              let image = NormalizedRect {
                x: gesture.selection_start.image_x,
                y: gesture.selection_start.image_y,
                width: gesture.selection_start.image_width,
                height: gesture.selection_start.image_height,
              };
              let next = apply_crop_resize(crop, image, gesture.edges, (dx, dy), false);
              selection.x = next.x;
              selection.y = next.y;
              selection.width = next.width;
              selection.height = next.height;
              gesture.last_delta = (
                if gesture.edges & 1 != 0 {
                  selection.x - gesture.selection_start.x
                } else if gesture.edges & 2 != 0 {
                  selection.x + selection.width
                    - gesture.selection_start.x
                    - gesture.selection_start.width
                } else {
                  0.0
                },
                if gesture.edges & 4 != 0 {
                  selection.y - gesture.selection_start.y
                } else if gesture.edges & 8 != 0 {
                  selection.y + selection.height
                    - gesture.selection_start.y
                    - gesture.selection_start.height
                } else {
                  0.0
                },
              );
              gesture.last_scale = if gesture.selection_start.width.abs() > f64::EPSILON {
                selection.width / gesture.selection_start.width
              } else {
                1.0
              };
            } else {
              let edges = gesture.edges;
              let start = gesture.selection_start;
              let resize = recenter::selection_resize(
                start,
                edges,
                (dx, dy),
                (pane.width, pane.height),
                state.workspace_transform.zoom,
                centered,
              );
              let (anchor_x, anchor_y) = resize.anchor;
              let (vx, vy) = resize.vector;
              let maximum = resize.maximum_scale;
              let mut factor = resize.scale;
              if state.selection_snapping_enabled && snapping {
                let targets_x = selection_snap_targets(&state, start, true);
                let targets_y = selection_snap_targets(&state, start, false);
                let horizontal = snapping::resize_axis(snapping::ResizeAxis {
                  anchor: anchor_x,
                  vector: vx,
                  raw_scale: factor,
                  pane_width: pane.width,
                  pane_height: pane.height,
                  zoom: state.workspace_transform.zoom,
                  minimum: resize.minimum_scale,
                  maximum,
                  targets: &targets_x,
                  layer_id: start.layer_id,
                });
                let vertical = snapping::resize_axis(snapping::ResizeAxis {
                  anchor: anchor_y,
                  vector: vy,
                  raw_scale: factor,
                  pane_width: pane.height,
                  pane_height: pane.width,
                  zoom: state.workspace_transform.zoom,
                  minimum: resize.minimum_scale,
                  maximum,
                  targets: &targets_y,
                  layer_id: start.layer_id,
                });
                let chosen = if horizontal.found
                  && (!vertical.found || horizontal.distance <= vertical.distance)
                {
                  horizontal
                } else {
                  vertical
                };
                if chosen.found {
                  factor = chosen.adjustment;
                }
                let x_difference = if horizontal.found {
                  (horizontal.adjustment - factor).abs()
                    * vx.abs()
                    * pane.width
                    * state.workspace_transform.zoom
                } else {
                  f64::INFINITY
                };
                let y_difference = if vertical.found {
                  (vertical.adjustment - factor).abs()
                    * vy.abs()
                    * pane.height
                    * state.workspace_transform.zoom
                } else {
                  f64::INFINITY
                };
                state.selection_snap_guide_x =
                  (horizontal.found && x_difference <= 0.5).then_some(horizontal);
                state.selection_snap_guide_y =
                  (vertical.found && y_difference <= 0.5).then_some(vertical);
              } else {
                clear_selection_snap_guides(&mut state);
              }
              selection.x = anchor_x + (start.x - anchor_x) * factor;
              selection.y = anchor_y + (start.y - anchor_y) * factor;
              selection.width = start.width * factor;
              selection.height = start.height * factor;
              gesture.last_delta = (selection.x - start.x, selection.y - start.y);
              gesture.last_scale = factor;
            }
            redraw_keyboard_transform(
              inner,
              &mut state,
              gesture.keyboard_start,
              selection,
              gesture.last_scale,
            );
            state.selection = Some(selection);
            state.gesture = Some(ActiveGesture::Selection(gesture));
            update_magnifier(&mut state);
            redraw_magnifier(inner, &mut state);
            draw_selection(inner, &state);
            let _ = unsafe { inner.gpu.composition.Commit() };
            update = Some(gesture);
          }
        }
        None => {}
      }
    } else {
      editor::EditorWindow::set_cursor(cursor_for_state(inner, &state, point));
    }
  }
  // The toolbar percentage follows a live canvas resize: the rebase keeps
  // the pixels still by changing the zoom the workspace is expressed in.
  if let Some(zoom) = zoom {
    emit_transform(inner, zoom);
  }
  if let Some(gesture) = update {
    emit_gesture(inner, SelectionGesturePhase::Update, gesture);
  }
}
