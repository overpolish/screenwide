// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Choosing an annotation, as a press on it does, and as a right press does
//! before the menu opens.

use super::*;

/// Chooses the annotation at `shaft` in the published list, as a press on it
/// does: its layer becomes the selection and the manager commits the choice.
pub(super) fn choose(state: &mut SurfaceState, shaft: usize, point: (f64, f64)) -> Option<Sample> {
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
