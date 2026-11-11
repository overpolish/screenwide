// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use crate::glide::core::{GlideAction, GlideEffects, GlidePhase, GlideRuntime, GlideSample};

fn ready(effects: GlideEffects) -> bool {
  effects.detection.phase == GlidePhase::Ready
}

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
fn a_push_ends_once_the_hand_stops() {
  let mut runtime = runtime();
  runtime.update(sample(-45.0, 0.0, false, 0.0));

  assert!(!ready(runtime.settle(31.0)));
  assert!(ready(runtime.settle(32.0)));
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
fn an_immediate_next_fold_is_held_until_its_own_push_ends() {
  let mut runtime = runtime();
  runtime.update(sample(-45.0, 0.0, false, 0.0));
  let next = runtime.update(sample(45.0, 0.0, false, 100.0));
  assert!(next.move_to.is_some());
  assert!(!ready(next));
  assert!(!ready(runtime.settle(131.0)));
  assert!(ready(runtime.settle(132.0)));
}

#[test]
fn continuous_drag_holds_the_push_but_small_jitter_does_not() {
  let mut runtime = runtime();
  runtime.update(sample(45.0, 0.0, false, 0.0));
  for timestamp in [16.0, 32.0, 48.0, 64.0] {
    let dragging = runtime.update(sample(20.0, 0.0, false, timestamp));
    assert!(!ready(dragging));
    assert!(dragging.move_to.is_none());
  }
  assert!(!ready(runtime.update(sample(1.0, 0.0, false, 80.0))));
  assert!(!ready(runtime.settle(95.0)));
  assert!(ready(runtime.settle(96.0)));
  let up = runtime.update(sample(0.0, -36.0, false, 98.0));
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
fn a_monitor_push_steps_once_however_far_it_goes() {
  let mut runtime = runtime();
  runtime.begin_monitor_navigation();
  let steps = (0..40)
    .filter_map(|at| {
      runtime
        .update(sample(-10.0, -10.0, false, f64::from(at) * 8.0))
        .monitor_step
    })
    .collect::<Vec<_>>();
  assert_eq!(steps, [(-1, -1)]);
}

#[test]
fn the_next_monitor_push_steps_once_the_last_has_stopped() {
  let mut runtime = runtime();
  runtime.begin_monitor_navigation();
  assert_eq!(
    runtime
      .update(sample(-45.0, -45.0, false, 0.0))
      .monitor_step,
    Some((-1, -1))
  );
  assert_eq!(
    runtime
      .update(sample(-20.0, -20.0, false, 10.0))
      .monitor_step,
    None
  );
  // Stopped for two polls, the push is over.
  assert!(ready(runtime.settle(42.0)));
  assert_eq!(
    runtime
      .update(sample(-45.0, -45.0, false, 50.0))
      .monitor_step,
    Some((-1, -1))
  );
}

#[test]
fn turning_back_picks_the_monitor_back_without_stopping() {
  let mut runtime = runtime();
  runtime.begin_monitor_navigation();
  assert_eq!(
    runtime.update(sample(45.0, 0.0, false, 0.0)).monitor_step,
    Some((1, 0))
  );
  assert_eq!(
    runtime.update(sample(-15.0, 0.0, false, 8.0)).monitor_step,
    None
  );
  assert_eq!(
    runtime.update(sample(-45.0, 0.0, false, 16.0)).monitor_step,
    Some((-1, 0))
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
  runtime.update(sample(1.0, -1.0, false, 20.0));
  assert!(!ready(runtime.settle(32.0)));
  assert!(!ready(runtime.settle(51.0)));
  assert!(ready(runtime.settle(52.0)));
}
