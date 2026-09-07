// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::glide::core::{GlideAction, GlideRuntime, GlideSample};

// These tests isolate runtime effects from the opening direction grace.
fn runtime() -> GlideRuntime {
  GlideRuntime::new(crate::glide::core::GlideDetectorOptions {
    opening_grace_ms: 0.0,
    ..Default::default()
  })
}

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
  let mut runtime = runtime();

  let first = runtime.update(sample(-45.0, 0.0, false, 0.0));
  assert!(first.reveal);
  assert_eq!(first.move_to, first.detection.region);

  let settling = runtime.update(sample(0.0, 0.0, false, 10.0));
  assert!(!settling.reveal);
  assert_eq!(settling.move_to, None);
}

#[test]
fn pending_minimize_reveals_without_moving_and_only_commits_on_a_lift() {
  let mut runtime = runtime();

  let armed = runtime.update(sample(0.0, 45.0, false, 0.0));
  assert_eq!(armed.detection.pending, Some(GlideAction::Minimize));
  assert!(armed.reveal);
  assert_eq!(armed.move_to, None);
  assert!(runtime.should_minimize(false));
  assert!(!runtime.should_minimize(true));
  assert!(runtime.commits_terminal_action(false));
  assert!(!runtime.commits_terminal_action(true));
}

#[test]
fn only_full_screen_region_commits_are_terminal_for_cursor_placement() {
  let mut full_screen = runtime();
  full_screen.update(sample(0.0, -45.0, false, 0.0));
  assert!(full_screen.commits_terminal_action(false));
  assert!(!full_screen.commits_terminal_action(true));

  let mut half_screen = runtime();
  half_screen.update(sample(-45.0, 0.0, false, 0.0));
  assert!(!half_screen.commits_terminal_action(false));
}

#[test]
fn rest_reports_one_ready_effect() {
  let mut runtime = runtime();
  runtime.update(sample(-45.0, 0.0, false, 0.0));

  assert!(!runtime.settle(99.0).ready);
  assert!(runtime.settle(100.0).ready);
  assert!(!runtime.settle(101.0).ready);
}

#[test]
fn disarming_over_an_applied_region_does_not_move_it_again() {
  let mut runtime = runtime();
  runtime.update(sample(-45.0, 0.0, false, 0.0));
  runtime.settle(100.0);
  runtime.update(sample(0.0, 50.0, false, 101.0));
  runtime.settle(201.0);
  runtime.update(sample(0.0, 50.0, false, 202.0));
  runtime.settle(302.0);

  let disarmed = runtime.update(sample(0.0, -50.0, false, 303.0));
  assert_eq!(disarmed.detection.pending, None);
  assert_eq!(disarmed.move_to, None);
}

#[test]
fn immediate_next_fold_does_not_send_a_stale_haptic_or_preview_pulse() {
  let mut runtime = runtime();
  runtime.update(sample(-45.0, 0.0, false, 0.0));
  let next = runtime.update(sample(45.0, 0.0, false, 100.0));
  assert!(next.move_to.is_some());
  assert!(!next.ready);
  assert!(!next.detection.became_ready);
  assert!(!runtime.settle(199.0).ready);
  let ready = runtime.settle(200.0);
  assert!(ready.ready);
  assert!(ready.detection.became_ready);
  assert!(!runtime.settle(201.0).ready);
}

#[test]
fn continuous_drag_delays_the_tick_but_small_jitter_does_not() {
  let mut runtime = runtime();
  runtime.update(sample(45.0, 0.0, false, 0.0));
  for timestamp in [40.0, 80.0, 120.0, 160.0] {
    let dragging = runtime.update(sample(20.0, 0.0, false, timestamp));
    assert!(!dragging.ready);
    assert!(dragging.move_to.is_none());
  }
  assert!(!runtime.update(sample(1.0, 0.0, false, 200.0)).ready);
  assert!(!runtime.settle(259.0).ready);
  assert!(runtime.settle(260.0).ready);
  assert!(!runtime.settle(261.0).ready);
  let up = runtime.update(sample(0.0, -36.0, false, 262.0));
  assert_eq!(up.move_to.unwrap().row_span, 1);
}

#[test]
fn monitor_navigation_starts_with_a_direct_cardinal_step() {
  let mut runtime = runtime();
  runtime.begin_monitor_navigation();
  let step = runtime.update(sample(0.0, -45.0, false, 0.0));
  assert_eq!(step.monitor_step, Some((0, -1)));
  assert_eq!(step.detection.region, None);
  assert_eq!(step.detection.pending, None);
  assert!(runtime.monitor_navigation_active());
  assert_eq!(step.move_to, None);
  assert!(!runtime.should_minimize(false));
  assert!(!runtime.commits_terminal_action(false));
}

#[test]
fn monitor_navigation_does_not_resize_or_minimize() {
  let mut runtime = runtime();
  runtime.begin_monitor_navigation();
  let step = runtime.update(sample(0.0, 45.0, false, 0.0));
  assert_eq!(step.monitor_step, Some((0, 1)));
  assert_eq!(step.detection.region, None);
  assert_eq!(step.detection.pending, None);
  assert_eq!(step.move_to, None);
  assert!(!runtime.should_minimize(false));
}

#[test]
fn navigation_waits_for_stillness_and_accepts_a_diagonal_corner_repeat() {
  let mut runtime = runtime();
  runtime.begin_monitor_navigation();

  let mut dragging = runtime.update(sample(-45.0, -45.0, false, 0.0));
  assert_eq!(dragging.monitor_step, Some((-1, -1)));
  dragging = runtime.update(sample(-20.0, -20.0, false, 10.0));
  assert_eq!(dragging.monitor_step, None);
  assert!(runtime.settle(110.0).ready);
  assert_eq!(
    runtime
      .update(sample(-45.0, -45.0, false, 111.0))
      .monitor_step,
    Some((-1, -1))
  );
}

#[test]
fn side_regions_keep_corner_refinement_and_two_thirds_keep_the_ladder() {
  let mut side = runtime();
  side.update(sample(-45.0, 0.0, false, 0.0));
  side.settle(100.0);
  let corner = side.update(sample(0.0, -45.0, false, 101.0));
  assert_eq!(corner.monitor_step, None);
  assert!(corner.move_to.is_some());

  let mut two_thirds = runtime();
  two_thirds.update(sample(45.0, 0.0, true, 0.0));
  two_thirds.settle(100.0);
  two_thirds.update(sample(-45.0, 0.0, true, 101.0));
  two_thirds.settle(201.0);
  let narrowed = two_thirds.update(sample(45.0, 0.0, true, 202.0));
  assert_eq!(narrowed.monitor_step, None);
  assert!(narrowed.move_to.is_some());
}

#[test]
fn noise_floor_counts_both_axes_and_includes_the_boundary() {
  let mut runtime = runtime();
  runtime.update(sample(45.0, 0.0, false, 0.0));
  runtime.update(sample(1.0, -1.0, false, 80.0));
  assert!(!runtime.settle(100.0).ready);
  assert!(!runtime.settle(179.0).ready);
  assert!(runtime.settle(180.0).ready);
}
