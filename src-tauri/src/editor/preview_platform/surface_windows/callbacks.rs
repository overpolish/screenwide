// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn emit_transform(inner: &SurfaceInner, zoom: f64) {
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.transform.as_mut() {
      callback(zoom * 100.0);
    }
  }
}

pub(super) fn emit_selection(inner: &SurfaceInner, selection: Option<u32>) {
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.selection.as_mut() {
      callback(selection);
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
      // Frame gestures address the pane, not the sentinel frame layer id,
      // matching the Metal backend's `emit_selection_gesture`.
      let layer_id = if matches!(
        gesture.operation,
        SelectionGestureOperation::FrameResize | SelectionGestureOperation::FrameRadius
      ) {
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
      Some(ActiveGesture::Pan { .. }) => editor::CursorKind::Move,
      Some(ActiveGesture::Selection(gesture))
        if matches!(
          gesture.operation,
          SelectionGestureOperation::Move | SelectionGestureOperation::CropMove
        ) =>
      {
        editor::CursorKind::Move
      }
      Some(ActiveGesture::Selection(gesture))
        if gesture.operation == SelectionGestureOperation::Radius =>
      {
        editor::CursorKind::ResizeNwse
      }
      Some(ActiveGesture::Selection(gesture)) => {
        let edges = gesture.edges;
        if edges == 1 || edges == 2 {
          editor::CursorKind::ResizeHorizontal
        } else if edges == 4 || edges == 8 {
          editor::CursorKind::ResizeVertical
        } else if edges == (1 | 4) || edges == (2 | 8) {
          editor::CursorKind::ResizeNwse
        } else {
          editor::CursorKind::ResizeNesw
        }
      }
      _ => cursor_for_state(inner, &state, state.last_pointer),
    })
    .unwrap_or(editor::CursorKind::Arrow);
  editor::EditorWindow::set_cursor(kind);
}
