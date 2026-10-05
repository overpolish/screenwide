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
use crate::editor::annotations::handles::HANDLE_FLAG_GROUPED;

/// Resolving pointer samples and reporting them to the manager.
mod sample;
use sample::{report, resolve, Sample};

/// Presses that change the choice: a group carry, a toggle and a marquee.
mod group;

/// Takes the press if the arrow chrome owns it. `false` lets it carry on to
/// the layer underneath, exactly as `annotation_mouse_down` returning `NO`
/// does on macOS.
pub(crate) fn down(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  let mut samples = Vec::new();
  let mut taken_layer = None;
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
    // Read before the match: the arms take the state by mutable reference,
    // which a borrow held by the scrutinee would forbid.
    let drawing = drawing_kind(state.annotation.mode);
    let empty = handle.is_none() && shaft.is_none();
    let placing_image = empty && drawing == Some(AnnotationKind::Image);
    if let Some(sample) = placing_image
      .then(|| group::image_lets_go(&mut state, point))
      .flatten()
    {
      drop(state);
      report(inner, sample.as_slice());
      return true;
    }
    // A fresh annotation joins the picture under the press, or the nearest.
    if empty && drawing.is_some() {
      taken_layer = take_drawing_layer(inner, &mut state, point);
    }
    match (handle, shaft) {
      (None, None) if drawing.is_none() && group::arm_empty(&mut state, point) => true,
      (None, None) => match drawing {
        None => {
          // Empty picture with only the select tool in hand: the arrow chrome
          // lets go, and the press carries on to the layer underneath. A
          // still's annotation - one on the selected layer, with no layer of
          // its own - has its choice cleared first, and so has a group of them.
          let still = match selected_item(&state) {
            Some(item) => item.layer_id < 0,
            None => {
              has_group(&state)
                && state
                  .annotation
                  .handles
                  .iter()
                  .any(|item| item.layer_id < 0 && item.flags & HANDLE_FLAG_GROUPED != 0)
            }
          };
          if still {
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
        Some(AnnotationKind::Counter | AnnotationKind::Text | AnnotationKind::Image) => {
          // Empty picture under the counter, text or image tool: the
          // annotation is dropped whole where the press lands, so it begins
          // at once and a click alone makes it; the drag that may follow
          // carries it. A fresh text box goes straight on to being typed into.
          state.annotation.drag = Some(Drag::begun(TARGET_NEW, 0, HANDLE_BODY, point));
          samples.extend(resolve(
            &state,
            SelectionGesturePhase::Begin,
            TARGET_NEW,
            0,
            HANDLE_BODY,
            point,
          ));
          true
        }
        Some(
          AnnotationKind::Arrow
          | AnnotationKind::Redact
          | AnnotationKind::Highlight
          | AnnotationKind::Shape
          | AnnotationKind::Spotlight
          | AnnotationKind::Draw
          | AnnotationKind::Magnify,
        ) => {
          // Empty picture: a new arrow, redaction, highlight, shape,
          // spotlight, stroke or magnifier's zoom area, once the press
          // proves to be a drag. The end grip is the one the drag carries,
          // so it grows from where it started towards the pointer.
          state.annotation.drag = Some(Drag::pending(TARGET_NEW, 0, HANDLE_END, point));
          true
        }
      },
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
        let on_loupe = state.annotation.handles.get(shaft).is_some_and(|item| {
          item.shape_kind() == AnnotationKind::Magnify
            && super::picking::item_image_frame(&state, shaft as i32)
              .is_some_and(|image| super::magnify_chrome::on_loupe(image, item, point))
        });
        let handle = if on_loupe { HANDLE_MIDDLE } else { HANDLE_BODY };
        // One of several chosen together carries them all, and the toggle
        // modifier waits for the release to add or remove the annotation.
        // Otherwise choosing an arrow is complete on the press: the manager
        // commits the choice and the chrome moves to it. A press inside a
        // magnifier's loupe carries the loupe on its own.
        if !group::arm_shaft(&mut state, shaft, handle, point) {
          state.annotation.drag = Some(Drag::pending(TARGET_EXISTING, shaft as u32, handle, point));
          samples.extend(choose(&mut state, shaft, point));
        }
        true
      }
    }
  };
  // The new layer is reported before the gesture, as a press with the select
  // tool reports its layer before the move it starts.
  if let Some(layer) = taken_layer {
    emit_selection(inner, Some(layer));
  }
  report(inner, &samples);
  taken
}

/// Chooses the annotation at `shaft` in the published list, as a press on it
/// does: its layer becomes the selection and the manager commits the choice.
fn choose(state: &mut SurfaceState, shaft: usize, point: (f64, f64)) -> Option<Sample> {
  state.annotation.selected = shaft as i32;
  if let Some(target) = state
    .annotation
    .handles
    .get(shaft)
    .and_then(|item| layer_selection(state, item.layer_id))
  {
    state.selection = Some(target);
  }
  resolve(
    state,
    SelectionGesturePhase::Begin,
    TARGET_SELECT,
    shaft as u32,
    HANDLE_BODY,
    point,
  )
}

/// Chooses the annotation under a right press, as a left press on it does, so
/// the menu opened there acts on it. One of several chosen together keeps the
/// group, so the menu acts on all of them. Its place in the published list,
/// or `None` when the press is not on one.
pub(crate) fn choose_at(inner: &SurfaceInner, point: (f64, f64)) -> Option<usize> {
  let (shaft, sample) = {
    let mut state = inner.state.lock().ok()?;
    if state.annotation.mode == MODE_NONE {
      return None;
    }
    let shaft = shaft_at_point(&state, point)?;
    let grouped = state
      .annotation
      .handles
      .get(shaft)
      .is_some_and(|item| item.flags & HANDLE_FLAG_GROUPED != 0);
    if grouped {
      return Some(shaft);
    }
    (shaft, choose(&mut state, shaft, point))
  };
  report(inner, sample.as_slice());
  Some(shaft)
}

/// One sample of the drag in hand: a group carry and a marquee band are
/// measured on their layer, anything else on the annotation it holds.
fn drag_sample(
  state: &mut SurfaceState,
  drag: Drag,
  phase: SelectionGesturePhase,
  point: (f64, f64),
) -> Option<Sample> {
  group::layer_sample(state, drag, phase, point).unwrap_or_else(|| {
    resolve(
      state,
      phase,
      drag.target_kind,
      drag.index,
      drag.handle,
      point,
    )
  })
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
    if began {
      drag = group::travelled(&mut state, drag, &mut samples);
    }
    state.annotation.drag = Some(drag);
    if began {
      // The gesture begins where the press did, not where it has reached, so
      // the shape it edits is the one the hand took hold of.
      samples.extend(drag_sample(
        &mut state,
        drag,
        SelectionGesturePhase::Begin,
        drag.origin,
      ));
    }
    if drag.reports() {
      samples.extend(drag_sample(
        &mut state,
        drag,
        SelectionGesturePhase::Update,
        point,
      ));
    }
    if matches!(drag.press, drag::Press::Marquee { .. }) && drag.reports() {
      draw_selection(inner, &state);
    }
  }
  report(inner, &samples);
  true
}

pub(crate) fn up(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  let samples = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let Some(drag) = state.annotation.drag.take() else {
      return false;
    };
    if !drag.begun {
      group::clicked(&mut state, drag, point)
    } else {
      let samples = drag_sample(&mut state, drag, SelectionGesturePhase::End, point);
      if matches!(drag.press, drag::Press::Marquee { .. }) {
        draw_selection(inner, &state);
      }
      samples.into_iter().collect()
    }
  };
  report(inner, &samples);
  true
}

pub(crate) fn cancel(state: &mut SurfaceState) {
  state.annotation.drag = None;
  state.annotation.marquee = None;
}
