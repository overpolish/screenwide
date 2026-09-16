// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The press-to-drag transition: when a press becomes a gesture, and what a
//! press that never travels leaves behind.

use super::*;

fn drag_from(origin: (f64, f64)) -> Drag {
  Drag::pending(TARGET_NEW, 0, HANDLE_END, origin)
}

#[test]
fn a_press_that_does_not_travel_never_becomes_a_gesture() {
  let mut drag = drag_from((100.0, 50.0));
  assert!(!drag.sample((100.0, 50.0)));
  assert!(!drag.sample((100.0 + DRAG_SLOP - 0.5, 50.0)));
  assert!(!drag.reports(), "a click must report no movement at all");
  assert!(!drag.begun, "and must leave no edit behind");
}

#[test]
fn the_gesture_begins_once_the_press_clears_the_slop() {
  let mut drag = drag_from((100.0, 50.0));
  assert!(drag.sample((100.0 + DRAG_SLOP, 50.0)));
  assert!(drag.begun);
  assert!(drag.reports());
  // Only the first sample past the slop begins the gesture; the rest update.
  assert!(!drag.sample((140.0, 50.0)));
  assert!(drag.reports());
}

#[test]
fn the_slop_is_measured_as_a_distance_not_per_axis() {
  let mut drag = drag_from((0.0, 0.0));
  // Just inside the circle of radius DRAG_SLOP on the diagonal.
  assert!(!drag.sample((2.0, 2.0)));
  assert!(drag.sample((3.0, 3.0)));
}
