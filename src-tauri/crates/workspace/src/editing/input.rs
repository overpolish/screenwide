// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
mod cancel;
mod context_menu;
mod crop_draw;
mod crop_resize;
mod down;
mod framed;
mod pointer_move;
mod up;
mod view_motion;

/// Hands one pointer event to the gesture it belongs to, and tells the editor
/// above of every press going down and being let go.
pub(super) fn handle_editor_input(inner: &Arc<SurfaceInner>, input: PointerInput) {
  // Gestures read the modifiers as of the event being handled, so either can
  // be taken and let go part way through a drag. A shortcut modifier spent on
  // holding Select reads as up until it is let go, and a press under the hold
  // brings the layer's chrome back.
  let mut modifiers = inner.host.modifiers();
  let pressing = matches!(input, PointerInput::Down { .. });
  let (scale, spent) = inner.state.lock().ok().map_or((1.0, false), |mut state| {
    state.shortcut_spent &= modifiers.shortcut;
    state.select_held &= !pressing;
    let spent = state.shortcut_spent;
    modifiers.shortcut &= !spent;
    state.modifiers = modifiers;
    (state.scale.max(0.1), spent)
  });
  let logical = |x: f64, y: f64| (x / scale, y / scale);
  match input {
    PointerInput::ContextMenu { x, y } => {
      // A right press holds nothing once its menu is up.
      emit_press(inner, PointerPress::Down { took: false });
      context_menu::open(inner, logical(x, y));
      emit_press(inner, PointerPress::Up);
    }
    PointerInput::Down {
      centered,
      x,
      y,
      snapping: _,
    } => {
      let took = press(inner, scale, centered, x, y);
      emit_press(inner, PointerPress::Down { took });
    }
    PointerInput::Move {
      centered,
      x,
      y,
      pressed,
      snapping,
    } => {
      // A layout under way has the viewport and the panes half moved. Every
      // drag is measured from its press, so skipping the sample loses
      // nothing: the next one, once the layout lands, reads it all afresh.
      if pressed
        && inner
          .state
          .lock()
          .is_ok_and(|state| state.workspace.laying_out)
      {
        return;
      }
      if pressed && turn::pointer_move(inner, logical(x, y)) {
        return;
      }
      // A sample past a turned layer's horizon reads nowhere on it, so what
      // the press holds stays where the last sample on it put it.
      let reading = if pressed {
        turn::layer_reading(inner, logical(x, y))
      } else {
        Some(turn::layer_point(inner, logical(x, y)))
      };
      if let (true, Some(layer)) = (pressed, reading) {
        if let Ok(mut state) = inner.state.lock() {
          state.drag_reading = layer;
        }
      }
      if pressed
        && (object_picking::pointer_move(inner, logical(x, y))
          || reading.is_some_and(|layer| annotation::typing_move(inner, layer)))
      {
        return;
      }
      if !pressed {
        turn::hover(inner, logical(x, y));
      }
      if let Some(layer) = reading {
        annotation::update_hover(inner, layer, pressed);
        if pressed && annotation::pointer_move(inner, layer) {
          if let Ok(mut state) = inner.state.lock() {
            state.last_pointer = logical(x, y);
          }
          return;
        }
      }
      let snapping = snapping && !spent;
      pointer_move::pointer_move(inner, scale, centered, x, y, pressed, snapping);
    }
    PointerInput::PanDown { x, y } => {
      let point = logical(x, y);
      if let Ok(mut state) = inner.state.lock() {
        state.last_pointer = point;
        // A selection drag already in flight on the primary button keeps
        // going; the middle button only pans from rest.
        if state.gesture.is_none() && state.turn.drag.is_none() {
          state.gesture = Some(ActiveGesture::Pan {
            pointer_start: point,
            transform_start: state.workspace_transform,
          });
        }
      }
      refresh_cursor_for(inner);
      emit_press(inner, PointerPress::Down { took: false });
    }
    PointerInput::PanUp { x, y } => {
      let point = logical(x, y);
      if let Ok(mut state) = inner.state.lock() {
        state.last_pointer = point;
        if matches!(state.gesture, Some(ActiveGesture::Pan { .. })) {
          state.gesture = None;
        }
        inner
          .host
          .set_cursor(cursor_for_state(inner, &state, point));
      }
      emit_press(inner, PointerPress::Up);
    }
    PointerInput::Up { x, y } => {
      release(inner, scale, x, y);
      emit_press(inner, PointerPress::Up);
    }
    PointerInput::Cancel => {
      if !turn::cancel(inner) {
        if let Ok(mut state) = inner.state.lock() {
          annotation::cancel(&mut state);
        }
        cancel::cancel(inner);
      }
      emit_press(inner, PointerPress::Up);
    }
    PointerInput::Wheel { x, y, delta } => {
      view_motion::zoom_at(inner, logical(x, y), (delta * 0.12).exp());
    }
    PointerInput::Magnify { x, y, factor } => view_motion::zoom_at(inner, logical(x, y), factor),
    PointerInput::Scroll { dx, dy } => view_motion::pan_by(inner, dx, dy),
    PointerInput::DoubleClick { x, y } => {
      emit_press(inner, PointerPress::Down { took: false });
      // A platform reports a double-click in place of its second press. Over
      // a text box or the caption it opens it for typing, and in what is typed
      // into it selects a word; only elsewhere does it fit the view.
      if object_picking::double_click(inner, logical(x, y))
        || annotation::typing_press(inner, turn::layer_point(inner, logical(x, y)), true)
        || annotation::open_text(inner, turn::layer_point(inner, logical(x, y)))
        || annotation::open_caption(
          inner,
          logical(x, y),
          turn::layer_point(inner, logical(x, y)),
        )
      {
        refresh_cursor_for(inner);
        return;
      }
      // Back to 100%, centred in the whole viewport.
      if let Ok(mut state) = inner.state.lock() {
        state.workspace_transform = WorkspaceTransform::default();
        apply_workspace_transform(inner, &mut state, false);
      }
    }
  }
}

/// A press going down. Answers whether it took hold of something already
/// there, rather than drawing, panning or picking.
fn press(inner: &Arc<SurfaceInner>, scale: f64, centered: bool, x: f64, y: f64) -> bool {
  let point = (x / scale, y / scale);
  // While objects are picked in the camera every press is theirs, Alt
  // removing what it picks.
  if object_picking::down(inner, point, centered) {
    refresh_cursor_for(inner);
    return false;
  }
  // A box being typed into sees the press first: in the box it places the
  // caret, anywhere else it ends the typing and may carry on below.
  if annotation::typing_press(inner, turn::layer_point(inner, point), false) {
    refresh_cursor_for(inner);
    return false;
  }
  // The rotation gizmo's rings sit over the layer they turn.
  if turn::down(inner, point) {
    return true;
  }
  // A turned layer's annotations are pressed where its turn draws them.
  let layer = turn::layer_point(inner, point);
  if let Ok(mut state) = inner.state.lock() {
    state.drag_reading = layer;
  }
  // A press is never a hover: the halo goes out before anything moves.
  annotation::update_hover(inner, layer, true);
  // The arrow chrome gets the press first and keeps it if it lands on a
  // grip, on a shaft, or on empty picture with the arrow tool in hand.
  // Anything it declines belongs to the layer underneath.
  if annotation::down(inner, point) {
    let took = inner.state.lock().ok().is_some_and(|mut state| {
      state.last_pointer = point;
      annotation::holds_existing(&state)
    });
    refresh_cursor_for(inner);
    return took;
  }
  down::down(inner, scale, x, y)
}

/// A press being let go.
fn release(inner: &Arc<SurfaceInner>, scale: f64, x: f64, y: f64) {
  let point = (x / scale, y / scale);
  if turn::up(inner, point) {
    return;
  }
  if object_picking::up(inner, point) {
    refresh_cursor_for(inner);
    return;
  }
  // Let go past a turned layer's horizon, what the press holds ends where
  // the last sample on the layer put it.
  let layer = turn::layer_reading(inner, point)
    .unwrap_or_else(|| inner.state.lock().map_or(point, |state| state.drag_reading));
  if annotation::typing_up(inner) || annotation::up(inner, layer) {
    if let Ok(mut state) = inner.state.lock() {
      state.last_pointer = point;
    }
    // What now lies under the resting pointer - an annotation just placed
    // or carried there - is haloed, so it can be deleted without the
    // pointer moving first; the press had put the halo out.
    annotation::update_hover(inner, layer, false);
    refresh_cursor_for(inner);
    return;
  }
  up::up(inner, scale, x, y);
}

mod frame_resize;

mod selection_move;
