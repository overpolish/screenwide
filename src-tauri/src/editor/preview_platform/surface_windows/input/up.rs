// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn up(inner: &std::sync::Arc<SurfaceInner>, scale: f64, x: f64, y: f64) {
  let point = (x / scale, y / scale);
  let mut ended = None;
  if let Ok(mut state) = inner.state.lock() {
    state.last_pointer = point;
    if let Some(ActiveGesture::Selection(gesture)) = state.gesture.take() {
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
      ended = Some(gesture);
    } else {
      state.gesture = None;
    }
    clear_selection_snap_guides(&mut state);
    update_magnifier(&mut state);
    redraw_magnifier(inner, &mut state);
    draw_selection(inner, &state);
    editor::EditorWindow::set_cursor(cursor_for_state(inner, &state, point));
  }
  if let Some(gesture) = ended {
    emit_gesture(inner, SelectionGesturePhase::End, gesture);
  }
}
