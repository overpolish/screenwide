// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::glide::core::{GlideAction, GlideRuntime, GlideSample};

fn sample(delta_x: f64, delta_y: f64, thirds: bool, timestamp: f64) -> GlideSample {
  GlideSample {
    delta_x,
    delta_y,
    thirds,
    timestamp,
  }
}

#[test]
fn first_destination_reveals_and_moves_exactly_once() {
  let mut runtime = GlideRuntime::default();

  let first = runtime.update(sample(-45.0, 0.0, false, 0.0));
  assert!(first.reveal);
  assert_eq!(first.move_to, first.detection.region);

  let settling = runtime.update(sample(0.0, 0.0, false, 10.0));
  assert!(!settling.reveal);
  assert_eq!(settling.move_to, None);
}

#[test]
fn pending_minimize_reveals_without_moving_and_only_commits_on_a_lift() {
  let mut runtime = GlideRuntime::default();

  let armed = runtime.update(sample(0.0, 45.0, false, 0.0));
  assert_eq!(armed.detection.pending, Some(GlideAction::Minimize));
  assert!(armed.reveal);
  assert_eq!(armed.move_to, None);
  assert!(runtime.should_minimize(false));
  assert!(!runtime.should_minimize(true));
}

#[test]
fn rest_reports_one_ready_effect() {
  let mut runtime = GlideRuntime::default();
  runtime.update(sample(-45.0, 0.0, false, 0.0));

  assert!(!runtime.settle(59.0).ready);
  assert!(runtime.settle(60.0).ready);
  assert!(!runtime.settle(61.0).ready);
}

#[test]
fn disarming_over_an_applied_region_does_not_move_it_again() {
  let mut runtime = GlideRuntime::default();
  runtime.update(sample(-45.0, 0.0, false, 0.0));
  runtime.settle(60.0);
  runtime.update(sample(0.0, 50.0, false, 61.0));
  runtime.settle(121.0);
  runtime.update(sample(0.0, 50.0, false, 122.0));
  runtime.settle(182.0);

  let disarmed = runtime.update(sample(0.0, -50.0, false, 183.0));
  assert_eq!(disarmed.detection.pending, None);
  assert_eq!(disarmed.move_to, None);
}
