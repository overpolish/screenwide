// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
#[path = "input/cancel.rs"]
mod cancel;
#[path = "input/context_menu.rs"]
mod context_menu;
#[path = "input/down.rs"]
mod down;
#[path = "input/pointer_move.rs"]
mod pointer_move;
#[path = "input/up.rs"]
mod up;

pub(super) fn handle_editor_input(editor_hwnd: HWND, input: editor::Input) {
  let Some(inner) = surface_for_editor(editor_hwnd) else {
    return;
  };
  let inner = &inner;
  if matches!(
    input,
    editor::Input::Down { .. } | editor::Input::PanDown { .. } | editor::Input::ContextMenu { .. }
  ) {
    if let Ok(mut callbacks) = inner.callbacks.lock() {
      if let Some(callback) = callbacks.pointer_down.as_mut() {
        callback();
      }
    }
  }
  let scale = inner
    .state
    .lock()
    .ok()
    .map_or(1.0, |state| state.scale.max(0.1));
  let logical = |x: f64, y: f64| (x / scale, y / scale);
  match input {
    editor::Input::ContextMenu { x, y } => context_menu::open(inner, logical(x, y)),
    editor::Input::Down {
      centered: _,
      x,
      y,
      snapping: _,
    } => {
      // A box being typed into sees the press first: in the box it places the
      // caret, anywhere else it ends the typing and may carry on below.
      if annotation::typing_press(inner, logical(x, y), false) {
        refresh_cursor_for(inner);
        return;
      }
      // A press is never a hover: the halo goes out before anything moves.
      annotation::update_hover(inner, logical(x, y), true);
      // The arrow chrome gets the press first and keeps it if it lands on a
      // grip, on a shaft, or on empty picture with the arrow tool in hand.
      // Anything it declines belongs to the layer underneath.
      if annotation::down(inner, logical(x, y)) {
        if let Ok(mut state) = inner.state.lock() {
          state.last_pointer = logical(x, y);
        }
        refresh_cursor_for(inner);
        return;
      }
      down::down(inner, scale, x, y);
    }
    editor::Input::Move {
      centered,
      x,
      y,
      pressed,
      snapping,
    } => {
      if pressed && annotation::typing_move(inner, logical(x, y)) {
        return;
      }
      annotation::update_hover(inner, logical(x, y), pressed);
      if pressed && annotation::pointer_move(inner, logical(x, y)) {
        if let Ok(mut state) = inner.state.lock() {
          state.last_pointer = logical(x, y);
        }
        return;
      }
      pointer_move::pointer_move(inner, scale, centered, x, y, pressed, snapping);
    }
    editor::Input::PanDown { x, y } => {
      let point = logical(x, y);
      if let Ok(mut state) = inner.state.lock() {
        state.last_pointer = point;
        // A selection drag already in flight on the primary button keeps
        // going; the middle button only pans from rest.
        if state.gesture.is_none() {
          state.gesture = Some(ActiveGesture::Pan {
            pointer_start: point,
            transform_start: state.workspace_transform,
          });
        }
      }
      refresh_cursor_for(inner);
    }
    editor::Input::PanUp { x, y } => {
      let point = logical(x, y);
      if let Ok(mut state) = inner.state.lock() {
        state.last_pointer = point;
        if matches!(state.gesture, Some(ActiveGesture::Pan { .. })) {
          state.gesture = None;
        }
        editor::EditorWindow::set_cursor(cursor_for_state(inner, &state, point));
      }
    }
    editor::Input::Up { x, y } => {
      if annotation::typing_up(inner) || annotation::up(inner, logical(x, y)) {
        if let Ok(mut state) = inner.state.lock() {
          state.last_pointer = logical(x, y);
        }
        refresh_cursor_for(inner);
        return;
      }
      up::up(inner, scale, x, y);
    }
    editor::Input::Cancel => {
      if let Ok(mut state) = inner.state.lock() {
        annotation::cancel(&mut state);
      }
      cancel::cancel(inner);
    }
    editor::Input::Wheel { x, y, delta } => {
      let point = logical(x, y);
      let mut zoom = None;
      if let Ok(mut state) = inner.state.lock() {
        state.last_pointer = point;
        let old = state.workspace_transform.zoom;
        let next = (old * (delta * 0.12).exp()).clamp(0.1, maximum_editor_zoom(&state));
        let ratio = next / old;
        let center = (state.viewport.width / 2.0, state.viewport.height / 2.0);
        state.workspace_transform.pan_x =
          point.0 - center.0 - (point.0 - center.0 - state.workspace_transform.pan_x) * ratio;
        state.workspace_transform.pan_y =
          point.1 - center.1 - (point.1 - center.1 - state.workspace_transform.pan_y) * ratio;
        state.workspace_transform.zoom = next;
        apply_workspace_transform(inner, &mut state, false);
        zoom = Some(next);
      }
      if let Some(zoom) = zoom {
        emit_transform(inner, zoom);
      }
    }
    editor::Input::DoubleClick { x, y } => {
      // Windows reports a double-click in place of its second press. Over a
      // text box it opens the box for typing, and in a box being typed into
      // it selects a word; only elsewhere does it fit the view.
      if annotation::typing_press(inner, logical(x, y), true)
        || annotation::open_text(inner, logical(x, y))
      {
        refresh_cursor_for(inner);
        return;
      }
      // Back to the current fit basis: the space beside an open tool panel
      // while one is up, the whole viewport otherwise.
      let mut zoom = 1.0;
      if let Ok(mut state) = inner.state.lock() {
        let transform = fit_basis_transform(&state);
        state.workspace_transform = transform;
        zoom = transform.zoom;
        apply_workspace_transform(inner, &mut state, false);
      }
      emit_transform(inner, zoom);
    }
  }
}

#[path = "input/frame_resize.rs"]
mod frame_resize;

#[path = "input/selection_move.rs"]
mod selection_move;
