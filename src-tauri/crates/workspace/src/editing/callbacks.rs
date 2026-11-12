// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_annotations::gesture::MODE_SELECT;

pub(super) fn emit_selection(inner: &SurfaceInner, selection: Option<u32>) {
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.selection.as_mut() {
      callback(selection);
    }
  }
}

pub(super) fn emit_press(inner: &SurfaceInner, press: PointerPress) {
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.pointer_press.as_mut() {
      callback(press);
    }
  }
}

pub(super) fn emit_gesture(
  inner: &SurfaceInner,
  phase: SelectionGesturePhase,
  gesture: EditorGesture,
) {
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.gesture.as_mut() {
      // A frame resize addresses the pane, not the sentinel frame layer id,
      // matching the macOS surface's `emit_selection_gesture`.
      let layer_id = if gesture.operation == SelectionGestureOperation::FrameResize {
        gesture.selection_start.pane_index
      } else {
        gesture.selection_start.layer_id
      };
      callback(
        phase,
        layer_id,
        gesture.operation,
        gesture.edges,
        gesture.last_scale,
        gesture.last_delta.0,
        gesture.last_delta.1,
      );
    }
  }
}

pub(super) fn refresh_cursor_for(inner: &SurfaceInner) {
  let kind = inner
    .state
    .lock()
    .ok()
    .map(|state| match state.gesture {
      Some(ActiveGesture::Pan { .. }) => CursorKind::Move,
      // A layer moved with Select in hand keeps the arrow, as most tools
      // have it; the crop tool's move shows the hand.
      Some(ActiveGesture::Selection(gesture))
        if matches!(
          gesture.operation,
          SelectionGestureOperation::Move | SelectionGestureOperation::CropMove
        ) =>
      {
        if state.annotation.mode == MODE_SELECT {
          CursorKind::Arrow
        } else {
          CursorKind::Move
        }
      }
      Some(ActiveGesture::Selection(gesture))
        if gesture.operation == SelectionGestureOperation::CropDraw && gesture.edges == 0 =>
      {
        CursorKind::Crosshair
      }
      Some(ActiveGesture::Selection(gesture)) => resize_cursor(gesture.edges),
      _ => cursor_for_state(inner, &state, state.last_pointer),
    })
    .unwrap_or(CursorKind::Arrow);
  inner.host.set_cursor(kind);
}

/// The resize cursor running along `edges`' sides.
pub(super) fn resize_cursor(edges: u32) -> CursorKind {
  let edges = edges & 0b1111;
  if edges == 1 || edges == 2 {
    CursorKind::ResizeHorizontal
  } else if edges == 4 || edges == 8 {
    CursorKind::ResizeVertical
  } else if edges == (1 | 4) || edges == (2 | 8) {
    CursorKind::ResizeNwse
  } else {
    CursorKind::ResizeNesw
  }
}
