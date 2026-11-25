// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The boxes' grips the rotation gizmo's rings give way to. The boxes are
//! drawn over the rings, so where a grip and a ring meet, as they do round a
//! small layer or image, a press is the grip's.

use super::*;

/// Whether a grip of a box is under `point`: the selected layer's, or the
/// chosen annotation's.
pub(in super::super) fn grip_over_rings(state: &SurfaceState, point: (f64, f64)) -> bool {
  selected_grip_at(state, point) || annotation::grip_at_display(state, point)
}

/// Whether one of the selected layer's grips is under the display `point`,
/// measured as `shared_selection_hit` measures them: on the layer standing
/// upright where a turn carries it. None show while ⌘ holds Select, and a
/// blank canvas's own layer has none.
fn selected_grip_at(state: &SurfaceState, point: (f64, f64)) -> bool {
  let Some(selection) = state.selection.filter(|selection| selection.blank == 0) else {
    return false;
  };
  if state.select_held {
    return false;
  }
  let Some(rect) = display_selection(state, selection) else {
    return false;
  };
  let point = if selection_is_turned(state, selection) {
    unturned_point(state, selection, point)
  } else {
    point
  };
  let target = DisplayTarget {
    id: 0,
    rect: DisplayRect {
      x: rect.x,
      y: rect.y,
      width: rect.width,
      height: rect.height,
    },
    z_order: 0,
    selected: 1,
    visible: 1,
  };
  hit_test_display(&[target], point, 8.0).is_some_and(|hit| hit.handle != 0)
}
