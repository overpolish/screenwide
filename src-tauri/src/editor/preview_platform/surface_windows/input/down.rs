// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn down(inner: &std::sync::Arc<SurfaceInner>, scale: f64, x: f64, y: f64) {
  let point = (x / scale, y / scale);
  let mut selected = None;
  let mut began = None;
  if let Ok(mut state) = inner.state.lock() {
    state.last_pointer = point;
    let shared = shared_selection_hit(inner, &state, point);
    let inactive_frame_target = shared
      .filter(|(selection, _)| {
        selection.layer_id == FRAME_LAYER_ID
          && state.selection.is_none_or(|current| {
            current.pane_index != selection.pane_index || current.layer_id != selection.layer_id
          })
      })
      .map(|hit| hit.0);
    let radius = shared.filter(|(_, handle)| *handle == 9).map(|hit| hit.0);
    let handle = shared
      .filter(|(_, handle)| (1..=8).contains(handle))
      .map(|(selection, handle)| (selection, shared_handle_edges(handle)));
    let target = shared
      .filter(|(selection, handle)| *handle == 0 && selection.layer_id != FRAME_LAYER_ID)
      .map(|hit| hit.0);
    let frame_target = shared
      .filter(|(selection, handle)| *handle == 0 && selection.layer_id == FRAME_LAYER_ID)
      .map(|hit| hit.0);
    if let Some(target) = inactive_frame_target {
      state.selection = Some(target);
      state.annotation.selected = -1;
      state.gesture = None;
      clear_selection_snap_guides(&mut state);
      selected = Some(Some(target.pane_index));
      draw_selection(inner, &state);
    } else if let Some(selection) = radius {
      let changed = state.selection.is_none_or(|current| {
        current.pane_index != selection.pane_index || current.layer_id != selection.layer_id
      });
      state.selection = Some(selection);
      let gesture = EditorGesture {
        edges: 0,
        last_delta: (0.0, 0.0),
        last_scale: selection.radius_percent,
        operation: if selection.layer_id == FRAME_LAYER_ID {
          SelectionGestureOperation::FrameRadius
        } else {
          SelectionGestureOperation::Radius
        },
        pane_start: selection_pane_rect(&state, selection),
        pointer_start: point,
        selection_start: selection,
        keyboard_start: keyboard_transform_start(&state, selection),
      };
      state.gesture = Some(ActiveGesture::Selection(gesture));
      if changed {
        selected = Some(Some(if selection.layer_id == FRAME_LAYER_ID {
          selection.pane_index
        } else {
          selection.layer_id
        }));
      }
      began = Some(gesture);
      draw_selection(inner, &state);
    } else if let Some((selection, edges)) = handle {
      let changed = state.selection.is_none_or(|current| {
        current.pane_index != selection.pane_index || current.layer_id != selection.layer_id
      });
      state.selection = Some(selection);
      let gesture = EditorGesture {
        edges,
        last_delta: (0.0, 0.0),
        last_scale: 1.0,
        operation: if selection.layer_id == FRAME_LAYER_ID {
          SelectionGestureOperation::FrameResize
        } else if selection.crop_mode != 0 {
          SelectionGestureOperation::CropResize
        } else {
          SelectionGestureOperation::Resize
        },
        pane_start: selection_pane_rect(&state, selection),
        pointer_start: point,
        selection_start: selection,
        keyboard_start: keyboard_transform_start(&state, selection),
      };
      if gesture.operation == SelectionGestureOperation::FrameResize {
        // Remember the transform that belongs to the canvas size being
        // left behind, so undoing back to it restores that zoom.
        if let Some(size) = state.workspace_natural_size {
          let transform = state.workspace_transform;
          state.workspace_transforms.insert(size, transform);
        }
        let start = frame_resize_start(&state);
        state.frame_resize = Some(start);
      }
      state.gesture = Some(ActiveGesture::Selection(gesture));
      if changed {
        selected = Some(Some(if selection.layer_id == FRAME_LAYER_ID {
          selection.pane_index
        } else {
          selection.layer_id
        }));
      }
      began = Some(gesture);
      draw_selection(inner, &state);
    } else if let Some(target) = frame_target {
      let changed = state.annotation.selected != -1
        || state.selection.is_none_or(|current| {
          current.pane_index != target.pane_index || current.layer_id != target.layer_id
        });
      state.selection = Some(target);
      state.annotation.selected = -1;
      state.gesture = None;
      clear_selection_snap_guides(&mut state);
      if changed {
        selected = Some(Some(target.pane_index));
      }
      draw_selection(inner, &state);
    } else if let Some(target) = target {
      let layer_changed = state.selection.is_none_or(|current| {
        current.pane_index != target.pane_index || current.layer_id != target.layer_id
      });
      // A held arrow counts as a change even on the same layer, exactly as
      // the Metal view's `changed` does: the press lets the arrow go, and
      // React has to hear about it to clear its own choice.
      let changed = layer_changed || state.annotation.selected != -1;
      // React updates target hit regions asynchronously. When this is already
      // the selected pane, its native selection is the freshest geometry (for
      // example immediately after a resize), so a stale target rectangle
      // must not replace it at the start of the next move.
      let selection = if layer_changed {
        target
      } else {
        state.selection.unwrap_or(target)
      };
      state.selection = Some(selection);
      state.annotation.selected = -1;
      let gesture = EditorGesture {
        edges: 0,
        last_delta: (0.0, 0.0),
        last_scale: 1.0,
        operation: if selection.crop_mode != 0 {
          SelectionGestureOperation::CropMove
        } else {
          SelectionGestureOperation::Move
        },
        pane_start: selection_pane_rect(&state, selection),
        pointer_start: point,
        selection_start: selection,
        keyboard_start: keyboard_transform_start(&state, selection),
      };
      state.move_auto_fit = (gesture.operation == SelectionGestureOperation::Move
        && selection.layer_id != u32::MAX - 1)
        .then(|| MoveAutoFit {
          active: false,
          last_bounds: None,
          natural_size: state
            .panes
            .get(selection.pane_index as usize)
            .and_then(Option::as_ref)
            .and_then(|pane| pane.settings.as_ref())
            .map(|settings| (f64::from(settings.width), f64::from(settings.height)))
            .or_else(|| {
              state
                .workspace_natural_size
                .map(|(width, height)| (f64::from(width), f64::from(height)))
            }),
          targets_start: state.selection_targets.clone(),
        });
      state.gesture = Some(ActiveGesture::Selection(gesture));
      if changed {
        selected = Some(Some(if target.layer_id == FRAME_LAYER_ID {
          target.pane_index
        } else {
          target.layer_id
        }));
      }
      began = Some(gesture);
      draw_selection(inner, &state);
    } else {
      state.gesture = Some(ActiveGesture::Pan {
        pointer_start: point,
        transform_start: state.workspace_transform,
      });
      draw_selection(inner, &state);
    }
  }
  if let Some(selection) = selected {
    emit_selection(inner, selection);
  }
  if let Some(gesture) = began {
    emit_gesture(inner, SelectionGesturePhase::Begin, gesture);
  }
  refresh_cursor_for(inner);
}
