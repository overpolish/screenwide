// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// A press the layers get. Answers whether it took hold of something: a
/// layer or one of its grips, or Select's band, rather than drawing a crop or
/// panning.
pub(super) fn down(inner: &std::sync::Arc<SurfaceInner>, scale: f64, x: f64, y: f64) -> bool {
  let point = (x / scale, y / scale);
  let mut selected = None;
  let mut began = None;
  let mut took = false;
  if let Ok(mut state) = inner.state.lock() {
    took = true;
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
    let handle = shared
      .filter(|(_, handle)| (1..=8).contains(handle))
      .map(|(selection, handle)| (selection, shared_handle_edges(handle)));
    // A press on an uncropped crop's body draws a new crop rather than
    // moving one that cannot move; it falls through to `crop_draw::start`.
    let draws_crop = crop_press_draws(&state, shared);
    let target = shared
      .filter(|(selection, handle)| {
        *handle == 0 && selection.layer_id != FRAME_LAYER_ID && !draws_crop
      })
      .map(|hit| hit.0);
    let frame_target = shared
      .filter(|(selection, handle)| *handle == 0 && selection.layer_id == FRAME_LAYER_ID)
      .map(|hit| hit.0);
    let overlay = shared
      .filter(|(selection, handle)| *handle == 0 && crate::is_overlay_layer(selection.layer_id))
      .map(|hit| hit.0);
    if let Some(overlay) = overlay {
      // An overlay takes its size and place from its tool, so the press only
      // reports it, for the editor to open that tool.
      state.gesture = None;
      selected = Some(Some(overlay.layer_id));
    } else if let Some(target) = inactive_frame_target {
      state.selection = Some(target);
      state.annotation.selected = -1;
      state.gesture = None;
      clear_selection_snap_guides(&mut state);
      selected = Some(Some(target.pane_index));
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
        // A scene's fixed window is resized by `framed`, on the canvas.
        start_turn: (selection.layer_id != FRAME_LAYER_ID
          && selection.crop_mode == 0
          && selection.framed != crate::editing::FRAMED_FIXED)
          .then(|| super::turn::turned_start(&state, selection))
          .flatten(),
        follows_frame: false,
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
        || annotation::has_group(&state)
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
      let changed = annotation::press_changes_choice(&state, target);
      // React updates target hit regions asynchronously. When this is already
      // the selected pane, its native selection is the freshest geometry (for
      // example immediately after a resize), so a stale target rectangle
      // must not replace it at the start of the next move.
      let selection = if layer_changed {
        target
      } else {
        state.selection.unwrap_or(target)
      };
      // Only the layer that is the whole choice moves under Select. A press
      // on any other draws a band once it travels. Let go first, it chooses
      // the layer, or, with annotations chosen, lets them go as a click
      // beside them does.
      let holds_annotations = state.annotation.selected != -1 || annotation::has_group(&state);
      let click = if holds_annotations {
        annotation::BandClick::ChoosesNothing
      } else {
        annotation::BandClick::Chooses(selection)
      };
      if annotation::bands_over(&state, target) && annotation::arm_band(&mut state, point, click) {
        // A band is Select's own work, so the press counts as taken: under
        // the ⌘ hold it keeps Select in hand once ⌘ is let go.
        state.gesture = None;
      } else {
        began = Some(start_move(&mut state, selection, point));
        if changed {
          selected = Some(Some(target.layer_id));
        }
      }
      draw_selection(inner, &state);
    } else if let Some(gesture) = super::crop_draw::start(&mut state, point) {
      // A press on nothing while cropping draws a new crop window. It
      // begins only once the press becomes a drag, from `pointer_move`.
      state.gesture = Some(ActiveGesture::Selection(gesture));
      took = false;
    } else if annotation::arm_band(&mut state, point, annotation::BandClick::ChoosesNothing) {
      // Empty picture under Select draws a band once the press travels; a
      // click there still chooses nothing. Taken, as a band on a layer is.
      state.gesture = None;
      draw_selection(inner, &state);
    } else {
      state.gesture = Some(ActiveGesture::Pan {
        pointer_start: point,
        transform_start: state.workspace_transform,
      });
      took = false;
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
  took
}

/// Starts moving `selection`, the layer a press took hold of, from `point`.
fn start_move(
  state: &mut SurfaceState,
  selection: PreviewSelection,
  point: (f64, f64),
) -> EditorGesture {
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
    pane_start: selection_pane_rect(state, selection),
    pointer_start: point,
    selection_start: selection,
    start_turn: None,
    follows_frame: false,
  };
  // A pane a scene places never grows the canvas: the scene owns where it
  // sits on a canvas of a fixed size.
  state.move_auto_fit = (gesture.operation == SelectionGestureOperation::Move
    && selection.framed == 0)
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
  gesture
}
