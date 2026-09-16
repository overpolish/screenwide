// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The gesture a press over the picture becomes.
//!
//! The twin of `annotation_mouse_down` / `_dragged` / `_up` in
//! `recording_preview_surface_macos+annotation.m`. A press the arrow chrome
//! declines carries on to the layer underneath, which is what the `bool` each
//! entry point returns says.
//!
//! Nothing here emits while the surface state is locked. A gesture callback
//! runs on this thread and comes straight back in through `set_annotations`
//! to publish the edit, so holding the state across the call would deadlock
//! against ourselves - `std::sync::Mutex` is not reentrant. Every entry point
//! therefore resolves its samples under the lock, drops it, and only then
//! reports them, exactly as the selection path in `input/down.rs` does.

use super::*;

/// One resolved pointer sample, in the layer's image-normalised space and
/// addressed the way everything above the facade expects: by layer and by the
/// arrow's index within it. Resolved under the state lock, reported without.
#[derive(Clone, Copy)]
pub(super) struct Sample {
  phase: SelectionGesturePhase,
  layer: u32,
  target_kind: u32,
  index: u32,
  handle: u32,
  x: f64,
  y: f64,
}

/// Resolves a sample against the published chrome. `None` when the point has
/// no picture to be normalised against, which is nothing to report.
fn resolve(
  state: &SurfaceState,
  phase: SelectionGesturePhase,
  target_kind: u32,
  index: u32,
  handle: u32,
  point: (f64, f64),
) -> Option<Sample> {
  let (x, y) = normalised_point(state, point)?;
  // Windows draws every layer into its own pane, but the gesture addresses
  // the layer, which is the identity a selection gesture reports too.
  let mut layer = state.selection.map_or(0, |selection| selection.layer_id);
  let mut index = index;
  if matches!(target_kind, TARGET_EXISTING | TARGET_SELECT) {
    if let Some(item) = state.annotation.handles.get(index as usize) {
      if item.layer_id >= 0 {
        layer = item.layer_id as u32;
      }
      index = item.index;
    }
  }
  Some(Sample {
    phase,
    layer,
    target_kind,
    index,
    handle,
    x,
    y,
  })
}

/// Reports resolved samples. MUST be called with no surface state held: the
/// callback re-enters the surface to publish what it changed.
fn report(inner: &SurfaceInner, samples: &[Sample]) {
  if samples.is_empty() {
    return;
  }
  let Ok(mut callbacks) = inner.callbacks.lock() else {
    return;
  };
  let Some(callback) = callbacks.annotation_gesture.as_mut() else {
    return;
  };
  for sample in samples {
    callback(
      sample.phase,
      sample.layer,
      sample.target_kind,
      sample.index,
      sample.handle,
      sample.x,
      sample.y,
    );
  }
}

/// Takes the press if the arrow chrome owns it. `false` lets it carry on to
/// the layer underneath, exactly as `annotation_mouse_down` returning `NO`
/// does on macOS.
pub(crate) fn down(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  let mut samples = Vec::new();
  let taken = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    if state.annotation.mode == MODE_NONE {
      return false;
    }
    let handle = handle_at_point(&state, point);
    let shaft = match handle {
      Some(_) => None,
      None => shaft_at_point(&state, point),
    };
    match (handle, shaft) {
      (None, None) if state.annotation.mode != MODE_ARROW => {
        // Empty picture with only the select tool in hand: the arrow chrome
        // lets go, and the press carries on to the layer underneath. A live
        // mark - one with no layer of its own - has its choice cleared first.
        if selected_item(&state).is_some_and(|item| item.layer_id < 0) {
          state.annotation.selected = -1;
          samples.extend(resolve(
            &state,
            SelectionGesturePhase::Begin,
            TARGET_NONE,
            0,
            HANDLE_BODY,
            point,
          ));
        }
        false
      }
      (None, None) => {
        // Empty picture: a new arrow, once the press proves to be a drag. The
        // end grip is the one the drag carries, so the arrow grows from where
        // it started towards the pointer.
        state.annotation.drag = Some(Drag::pending(TARGET_NEW, 0, HANDLE_END, point));
        true
      }
      // A grip and a chosen shaft both wait for the press to travel before
      // they begin: a click must not nudge the arrow by the few points
      // between the grip's centre and the pointer, nor leave an edit in the
      // history for it.
      (Some(handle), _) => {
        let index = state.annotation.selected.max(0) as u32;
        state.annotation.drag = Some(Drag::pending(TARGET_EXISTING, index, handle, point));
        true
      }
      (None, Some(shaft)) => {
        // Choosing an arrow is complete on the press: the manager commits the
        // choice and the chrome moves to it.
        state.annotation.selected = shaft as i32;
        if let Some(target) = state
          .annotation
          .handles
          .get(shaft)
          .and_then(|item| layer_selection(&state, item.layer_id))
        {
          state.selection = Some(target);
        }
        state.annotation.drag = Some(Drag::pending(
          TARGET_EXISTING,
          shaft as u32,
          HANDLE_BODY,
          point,
        ));
        samples.extend(resolve(
          &state,
          SelectionGesturePhase::Begin,
          TARGET_SELECT,
          shaft as u32,
          HANDLE_BODY,
          point,
        ));
        true
      }
    }
  };
  report(inner, &samples);
  taken
}

pub(crate) fn pointer_move(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  let mut samples = Vec::new();
  {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let Some(mut drag) = state.annotation.drag else {
      return false;
    };
    let began = drag.sample(point);
    state.annotation.drag = Some(drag);
    if began {
      // The gesture begins where the press did, not where it has reached, so
      // the shape it edits is the one the hand took hold of.
      samples.extend(resolve(
        &state,
        SelectionGesturePhase::Begin,
        drag.target_kind,
        drag.index,
        drag.handle,
        drag.origin,
      ));
    }
    if drag.reports() {
      samples.extend(resolve(
        &state,
        SelectionGesturePhase::Update,
        drag.target_kind,
        drag.index,
        drag.handle,
        point,
      ));
    }
  }
  report(inner, &samples);
  true
}

pub(crate) fn up(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  let mut samples = Vec::new();
  {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let Some(drag) = state.annotation.drag.take() else {
      return false;
    };
    if drag.begun {
      samples.extend(resolve(
        &state,
        SelectionGesturePhase::End,
        drag.target_kind,
        drag.index,
        drag.handle,
        point,
      ));
    }
  }
  report(inner, &samples);
  true
}

pub(crate) fn cancel(state: &mut SurfaceState) {
  state.annotation.drag = None;
}
