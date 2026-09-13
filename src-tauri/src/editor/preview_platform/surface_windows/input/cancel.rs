// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn cancel(inner: &std::sync::Arc<SurfaceInner>) {
  let mut cancelled = None;
  let mut cancelled_zoom = None;
  if let Ok(mut state) = inner.state.lock() {
    if let Some(ActiveGesture::Selection(gesture)) = state.gesture.take() {
      state.selection = Some(gesture.selection_start);
      redraw_keyboard_transform(
        inner,
        &mut state,
        gesture.keyboard_start,
        gesture.selection_start,
        1.0,
      );
      state.move_auto_fit = None;
      // An auto-fit Move owns the workspace exactly like a Frame resize
      // until it commits, and is unwound the same way.
      if gesture.operation == SelectionGestureOperation::FrameResize || state.frame_resize.is_some()
      {
        // Restore the whole workspace the drag re-flowed and rebased, not
        // just the pane under the pointer.
        if let Some(start) = state.frame_resize.take() {
          for (index, rect) in &start.pane_rects {
            if let Some(pane) = state.panes.get_mut(*index).and_then(Option::as_mut) {
              pane.base_rect = *rect;
            }
          }
          state.workspace_transform = start.transform;
          state.workspace_natural_size = start.natural_size;
          cancelled_zoom = Some(start.transform.zoom);
        } else if let Some(pane) = state
          .panes
          .get_mut(gesture.selection_start.pane_index as usize)
          .and_then(Option::as_mut)
        {
          pane.base_rect = gesture.pane_start;
        }
        state.frame_resize_committed = false;
        // The pane still holds the canvas the drag composed, so the
        // restored boxes wait for the cancel gesture's re-composition of
        // the restored composition (and, failing that, the interactive
        // still the manager restarts) rather than letterboxing the
        // dragged canvas into them for a frame.
        apply_workspace_transform(inner, &mut state, true);
      }
      clear_selection_snap_guides(&mut state);
      update_magnifier(&mut state);
      redraw_magnifier(inner, &mut state);
      draw_selection(inner, &state);
      cancelled = Some(gesture);
    } else {
      state.gesture = None;
      clear_selection_snap_guides(&mut state);
    }
  }
  if let Some(zoom) = cancelled_zoom {
    emit_transform(inner, zoom);
  }
  if let Some(gesture) = cancelled {
    emit_gesture(inner, SelectionGesturePhase::Cancel, gesture);
  }
}
