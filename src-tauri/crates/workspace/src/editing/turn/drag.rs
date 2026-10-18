// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A press, drag and release of a gizmo ring. Each answers whether the event
//! was the gizmo's.

use super::*;
fn same_layer(a: PreviewSelection, b: PreviewSelection) -> bool {
  a.pane_index == b.pane_index && a.layer_id == b.layer_id
}

/// The sample a ring drag reports of the turn `selection` has reached: about
/// X and Y as its travel, about Z as its scale.
fn report(state: &SurfaceState, held: &TurnDrag, selection: PreviewSelection) -> EditorGesture {
  EditorGesture {
    edges: 0,
    last_delta: (selection.rotation_x, selection.rotation_y),
    last_scale: selection.rotation_z,
    operation: SelectionGestureOperation::Rotate,
    pane_start: selection_pane_rect(state, held.start),
    pointer_start: held.origin,
    selection_start: held.start,
  }
}

pub(in super::super) fn down(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  // The chosen image's rings sit over the image, inside the layer's chrome.
  if annotation::image_turn_down(inner, point) {
    return true;
  }
  if !inner
    .callbacks
    .lock()
    .is_ok_and(|callbacks| callbacks.gesture.is_some())
  {
    return false;
  }
  let began = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let (Some(selection), Some((hit, radius))) = (state.selection, ring_hit(&state, point)) else {
      return false;
    };
    let held = TurnDrag {
      drag: RingDrag::begin(rotation(selection), radius, hit.axis, hit.angle),
      start: selection,
      origin: point,
      edges: hit.edges,
    };
    state.turn = TurnState {
      drag: Some(held),
      image: None,
      hovered: None,
    };
    state.gesture = None;
    state.last_pointer = point;
    clear_selection_snap_guides(&mut state);
    draw_selection(inner, &state);
    report(&state, &held, selection)
  };
  emit_gesture(inner, SelectionGesturePhase::Begin, began);
  refresh_cursor_for(inner);
  true
}

pub(in super::super) fn pointer_move(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  if annotation::image_turn_move(inner, point) {
    return true;
  }
  let Some((phase, sample)) = turn_to(inner, point) else {
    return false;
  };
  emit_gesture(inner, phase, sample);
  true
}

/// Turns the held ring as far as `point` has carried it from the press.
fn turn_to(
  inner: &SurfaceInner,
  point: (f64, f64),
) -> Option<(SelectionGesturePhase, EditorGesture)> {
  let mut state = inner.state.lock().ok()?;
  let mut held = state.turn.drag?;
  state.last_pointer = point;
  let Some(mut selection) = state
    .selection
    .filter(|selection| same_layer(*selection, held.start))
  else {
    // The layer went away under the ring: the turn it reached goes with it.
    state.turn.drag = None;
    draw_selection(inner, &state);
    return Some((
      SelectionGesturePhase::Cancel,
      report(&state, &held, held.start),
    ));
  };
  let turned = held.drag.turned(
    (point.0 - held.origin.0, point.1 - held.origin.1),
    state.modifiers.shift,
  );
  selection.rotation_x = turned.x;
  selection.rotation_y = turned.y;
  selection.rotation_z = turned.z;
  state.selection = Some(selection);
  state.turn.drag = Some(held);
  draw_selection(inner, &state);
  Some((
    SelectionGesturePhase::Update,
    report(&state, &held, selection),
  ))
}

pub(in super::super) fn up(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  if annotation::image_turn_up(inner, point) {
    return true;
  }
  // A release can land past the last drag sample; it is the turn kept.
  if !pointer_move(inner, point) {
    return false;
  }
  let ended = inner.state.lock().ok().and_then(|mut state| {
    let held = state.turn.drag.take()?;
    let selection = state.selection?;
    update_hover(&mut state, point);
    draw_selection(inner, &state);
    Some(report(&state, &held, selection))
  });
  if let Some(ended) = ended {
    emit_gesture(inner, SelectionGesturePhase::End, ended);
  }
  refresh_cursor_for(inner);
  true
}

/// Lets go of a held ring without keeping the turn it reached.
pub(in super::super) fn cancel(inner: &SurfaceInner) -> bool {
  if annotation::image_turn_cancel(inner) {
    return true;
  }
  let cancelled = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let Some(held) = state.turn.drag.take() else {
      return false;
    };
    if let Some(selection) = state
      .selection
      .as_mut()
      .filter(|selection| same_layer(**selection, held.start))
    {
      selection.rotation_x = held.start.rotation_x;
      selection.rotation_y = held.start.rotation_y;
      selection.rotation_z = held.start.rotation_z;
    }
    draw_selection(inner, &state);
    report(&state, &held, held.start)
  };
  emit_gesture(inner, SelectionGesturePhase::Cancel, cancelled);
  refresh_cursor_for(inner);
  true
}
