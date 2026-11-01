// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn up(inner: &std::sync::Arc<SurfaceInner>, scale: f64, x: f64, y: f64) {
  let point = (x / scale, y / scale);
  let mut ended = None;
  let mut chose_nothing = false;
  if let Ok(mut state) = inner.state.lock() {
    state.last_pointer = point;
    let gesture = state.gesture.take();
    if let Some(ActiveGesture::Selection(gesture)) = gesture {
      if gesture.operation == SelectionGestureOperation::FrameResize {
        state.frame_resize = None;
        // The committed layout that follows carries the canvas size the
        // drag produced. It must adopt the rebased transform rather than
        // restore a remembered one, and record it as the transform that
        // belongs to that size.
        state.frame_resize_committed = true;
      } else if gesture.operation == SelectionGestureOperation::Move {
        // Mouse-up with Alt still held commits the grown canvas the same
        // way; a plain move never took the geometry over.
        if state.frame_resize.take().is_some() {
          state.frame_resize_committed = true;
        }
        state.move_auto_fit = None;
      }
      ended = super::crop_draw::finish(&mut state, &gesture).then_some(gesture);
    } else if let Some(ActiveGesture::Pan { pointer_start, .. }) = gesture {
      // A click on nothing, a press let go where it went down, chooses
      // nothing: the editor lets the layer go and, with it, the layer's mode.
      // A drag from nothing pans the view and keeps the choice. The frame's
      // own selection belongs to the Frame tool, which a click never puts
      // down, and a click beside the camera while its objects are picked
      // keeps the camera.
      chose_nothing = (point.0 - pointer_start.0).hypot(point.1 - pointer_start.1)
        <= CLICK_SLOP_POINTS
        && !state.object_picking.is_active()
        && state
          .selection
          .is_some_and(|selection| selection.layer_id != FRAME_LAYER_ID);
    }
    clear_selection_snap_guides(&mut state);
    update_magnifier(&mut state);
    draw_workspace(inner, &mut state);
    draw_selection(inner, &state);
    inner
      .host
      .set_cursor(cursor_for_state(inner, &state, point));
  }
  if let Some(gesture) = ended {
    emit_gesture(inner, SelectionGesturePhase::End, gesture);
  }
  if chose_nothing {
    emit_selection(inner, None);
  }
}

/// How far a press may wander, in points, and still be a click.
const CLICK_SLOP_POINTS: f64 = 3.0;
