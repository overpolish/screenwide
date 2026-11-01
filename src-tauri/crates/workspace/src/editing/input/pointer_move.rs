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
  let mut began = None;
  let mut update = None;
  // A move or resize sample's chrome is drawn with the picture the gesture
  // redraws for it, when the batch closes after the report below, rather
  // than ahead of it: drawn at once, the grips ran a frame or more ahead of
  // the picture through a drag and only met it again at mouse-up.
  let mut batched = false;
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
        Some(ActiveGesture::Selection(gesture))
          if gesture.operation == SelectionGestureOperation::CropDraw =>
        {
          (began, update) = super::crop_draw::drag(inner, &mut state, gesture, point, centered);
        }
        Some(ActiveGesture::Selection(mut gesture)) => {
          if gesture.operation != SelectionGestureOperation::FrameResize {
            begin_present_batch(inner);
            batched = true;
          }
          let owns_geometry = state.frame_resize.is_some();
          let pane = state
            .panes
            .get(gesture.selection_start.pane_index as usize)
            .and_then(Option::as_ref)
            .map(|pane| pane_canvas_rect(pane, owns_geometry));
          if let Some(pane) = pane {
            // A turned layer's grips are dragged across the layer as its turn
            // stood at the press, so the press and `point` are carried back
            // for them; a move travels the canvas, as the picture does.
            let (pointer_start, point) = if gesture.operation == SelectionGestureOperation::Resize
              && gesture.selection_start.framed != crate::editing::FRAMED_FIXED
            {
              (
                turn::unturned_point(&state, gesture.selection_start, gesture.pointer_start),
                turn::unturned_point(&state, gesture.selection_start, point),
              )
            } else {
              (gesture.pointer_start, point)
            };
            let dx =
              (point.0 - pointer_start.0) / (pane.width * state.workspace_transform.zoom).max(1.0);
            let dy =
              (point.1 - pointer_start.1) / (pane.height * state.workspace_transform.zoom).max(1.0);
            let mut selection = gesture.selection_start;
            if gesture.selection_start.framed == crate::editing::FRAMED_FIXED
              && matches!(
                gesture.operation,
                SelectionGestureOperation::Move
                  | SelectionGestureOperation::Resize
                  | SelectionGestureOperation::CropResize
              )
            {
              super::framed::apply(&mut state, &mut gesture, &mut selection, pane, (dx, dy));
            } else if gesture.operation == SelectionGestureOperation::FrameResize {
              super::frame_resize::apply(inner, &mut state, &mut gesture, point, centered);
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
              super::crop_resize::apply(&mut state, &mut gesture, &mut selection, pane, (dx, dy));
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
            state.selection = Some(selection);
            state.gesture = Some(ActiveGesture::Selection(gesture));
            update_magnifier(&mut state);
            draw_workspace(inner, &mut state);
            draw_selection(inner, &state);
            inner.host.commit(false);
            update = Some(gesture);
          }
        }
        None => {}
      }
    } else {
      inner
        .host
        .set_cursor(cursor_for_state(inner, &state, point));
    }
  }
  if let Some(gesture) = began {
    emit_gesture(inner, SelectionGesturePhase::Begin, gesture);
  }
  if let Some(gesture) = update {
    emit_gesture(inner, SelectionGesturePhase::Update, gesture);
  }
  if batched {
    end_present_batch(inner);
  }
}
