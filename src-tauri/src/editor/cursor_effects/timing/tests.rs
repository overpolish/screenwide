// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::recording::cursor::CursorSourceKind;
fn compositor(positions: Vec<Position>, button_events: Vec<ButtonEvent>) -> CursorCompositor {
  let raw_positions = positions.clone();
  CursorCompositor {
    visibility: Vec::new(),
    appearances: Vec::new(),
    button_events,
    dwell_anchors: Vec::new(),
    raw_positions,
    positions,
    source: CursorSource {
      height: 1_000.0,
      kind: CursorSourceKind::Screen,
      platform_id: "test".to_owned(),
      video_height: 1_000,
      video_width: 1_000,
      width: 1_000.0,
      x: 0.0,
      y: 0.0,
    },
  }
}

#[test]
fn click_holds_down_until_release_then_settles() {
  let compositor = compositor(
    Vec::new(),
    vec![
      ButtonEvent {
        state: ButtonState::Down,
        timestamp_us: 100_000,
      },
      ButtonEvent {
        state: ButtonState::Up,
        timestamp_us: 1_000_000,
      },
    ],
  );
  assert!((compositor.click_scale(500_000) - 0.86).abs() < 0.001);
  assert!((compositor.click_scale(1_000_000) - 0.86).abs() < 0.001);
  assert_ne!(compositor.click_scale(1_100_000), 1.0);
  assert_eq!(compositor.click_scale(1_500_000), 1.0);
}

#[test]
fn repeated_click_does_not_jump_back_to_full_size() {
  let compositor = compositor(
    Vec::new(),
    vec![
      ButtonEvent {
        state: ButtonState::Down,
        timestamp_us: 0,
      },
      ButtonEvent {
        state: ButtonState::Up,
        timestamp_us: 100_000,
      },
      ButtonEvent {
        state: ButtonState::Down,
        timestamp_us: 200_000,
      },
    ],
  );
  let before = compositor.click_scale(199_999);
  let after = compositor.click_scale(200_000);
  assert!((before - after).abs() < 0.001);
}

#[test]
fn motion_lean_carries_through_connected_fast_and_slow_motion() {
  let compositor = compositor(
    [
      (0, 0.0),
      (50_000, 150.0),
      (100_000, 300.0),
      (150_000, 450.0),
      (200_000, 500.0),
      (300_000, 550.0),
      (400_000, 600.0),
      (500_000, 650.0),
    ]
    .into_iter()
    .map(|(timestamp_us, x)| Position {
      segment: 0,
      timestamp_us,
      x,
      y: 0.0,
    })
    .collect(),
    Vec::new(),
  );
  let fast_lean = compositor.motion_lean_degrees(100_000, 32.0);
  let transition_lean = compositor.motion_lean_degrees(190_000, 32.0);
  let slow_lean = compositor.motion_lean_degrees(320_000, 32.0);
  assert!(fast_lean > 2.0);
  assert!(
    transition_lean > fast_lean * 0.6,
    "momentum collapsed at the speed transition: {transition_lean}"
  );
  assert!(
    slow_lean > 0.5,
    "the connected slow movement lost all momentum: {slow_lean}"
  );
  assert!(compositor.motion_lean_degrees(1_500_000, 32.0).abs() < 0.1);
}

#[test]
fn motion_lean_eases_in_before_reaching_its_full_angle() {
  let compositor = compositor(
    vec![
      Position {
        segment: 0,
        timestamp_us: 0,
        x: 0.0,
        y: 0.0,
      },
      Position {
        segment: 0,
        timestamp_us: 250_000,
        x: 800.0,
        y: 0.0,
      },
    ],
    Vec::new(),
  );
  let initial = compositor.motion_lean_degrees(0, 32.0);
  let entering = compositor.motion_lean_degrees(35_000, 32.0);
  let established = compositor.motion_lean_degrees(160_000, 32.0);
  assert_eq!(initial, 0.0);
  assert!(entering > 0.0);
  assert!(
    entering < established * 0.3,
    "lean skipped its warmup: entering={entering}, established={established}"
  );
}

#[test]
fn short_fast_flick_rotates_less_than_a_long_fast_move() {
  let movement = |distance| {
    compositor(
      vec![
        Position {
          segment: 0,
          timestamp_us: 0,
          x: 0.0,
          y: 0.0,
        },
        Position {
          segment: 0,
          timestamp_us: 20_000,
          x: distance,
          y: 0.0,
        },
      ],
      Vec::new(),
    )
  };
  let short = movement(20.0).motion_lean_degrees(10_000, 32.0);
  let long = movement(300.0).motion_lean_degrees(10_000, 32.0);
  assert!(short < 2.0, "short flick leaned too far: {short}");
  assert!(
    long > short * 3.0,
    "distance did not shape lean: {short} vs {long}"
  );
}

#[test]
fn giant_cursor_has_more_rotational_inertia() {
  let compositor = compositor(
    vec![
      Position {
        segment: 0,
        timestamp_us: 0,
        x: 0.0,
        y: 0.0,
      },
      Position {
        segment: 0,
        timestamp_us: 150_000,
        x: 500.0,
        y: 0.0,
      },
    ],
    Vec::new(),
  );
  let normal = compositor.motion_lean_degrees(75_000, 32.0);
  let giant = compositor.motion_lean_degrees(75_000, 160.0);
  assert!(giant < normal * 0.5, "giant={giant}, normal={normal}");
}

#[test]
fn idle_gap_starts_a_fresh_motion_segment() {
  let compositor = compositor(
    vec![
      Position {
        segment: 0,
        timestamp_us: 0,
        x: 10.0,
        y: 10.0,
      },
      Position {
        segment: 0,
        timestamp_us: 50_000,
        x: 20.0,
        y: 10.0,
      },
      Position {
        segment: 1,
        timestamp_us: 500_000,
        x: 800.0,
        y: 600.0,
      },
    ],
    Vec::new(),
  );

  assert!(compositor.smoothed_position(450_000, true).unwrap().x < 30.0);
  let resumed = compositor.smoothed_position(500_000, true).unwrap();
  assert_eq!(resumed.segment, 1);
  assert!((resumed.x - 800.0).abs() < 0.001);
  assert_eq!(compositor.motion_lean_degrees(500_000, 32.0), 0.0);
}

#[test]
fn tiny_cursor_corrections_become_one_stationary_average() {
  let positions = [
    (0, 500.0, 500.0),
    (100_000, 505.0, 493.0),
    (200_000, 510.0, 500.0),
  ]
  .into_iter()
  .map(|(timestamp_us, x, y)| Position {
    segment: 0,
    timestamp_us,
    x,
    y,
  })
  .collect::<Vec<_>>();
  let compositor = compositor(stabilise_positions(&positions, 1_000.0), Vec::new());
  let midpoint = compositor.smoothed_position(100_000, true).unwrap();
  assert!((midpoint.x - 505.0).abs() < 0.001);
  assert!((midpoint.y - 500.0).abs() < 0.001);
  let endpoint = compositor.smoothed_position(200_000, true).unwrap();
  assert!((endpoint.x - 505.0).abs() < 0.001);
  assert!((endpoint.y - 500.0).abs() < 0.001);
}

#[test]
fn disabling_smoothing_uses_uncollapsed_cursor_samples() {
  let raw_positions = [(0, 0.0), (100_000, 5.0), (200_000, 10.0)]
    .into_iter()
    .map(|(timestamp_us, x)| Position {
      segment: 0,
      timestamp_us,
      x,
      y: 0.0,
    })
    .collect::<Vec<_>>();
  let mut compositor = compositor(raw_positions.clone(), Vec::new());
  compositor.positions = stabilise_positions(&raw_positions, compositor.source.width);

  assert_eq!(compositor.smoothed_position(0, false).unwrap().x, 0.0);
  assert_eq!(compositor.smoothed_position(100_000, false).unwrap().x, 5.0);
  assert_eq!(
    compositor.smoothed_position(200_000, false).unwrap().x,
    10.0
  );
  assert!((compositor.smoothed_position(0, true).unwrap().x - 5.0).abs() < 0.001);
}

#[test]
fn windows_dwell_is_an_exact_anchor_with_a_smooth_approach() {
  let positions = vec![
    Position {
      segment: 0,
      timestamp_us: 0,
      x: 0.0,
      y: 0.0,
    },
    Position {
      segment: 0,
      timestamp_us: 100_000,
      x: 100.0,
      y: 50.0,
    },
    Position {
      segment: 0,
      timestamp_us: 500_000,
      x: 200.0,
      y: 80.0,
    },
  ];
  let mut compositor = compositor(positions.clone(), Vec::new());
  compositor.dwell_anchors = dwell_anchors(&positions);

  let held = compositor.smoothed_position(300_000, true).unwrap();
  assert_eq!((held.x, held.y), (100.0, 50.0));
  let arrival = compositor.smoothed_position(100_000, true).unwrap();
  assert_eq!((arrival.x, arrival.y), (100.0, 50.0));
  let approaching = compositor.smoothed_position(75_000, true).unwrap();
  assert!(approaching.x < 100.0);
  assert!(approaching.x > 75.0);
}

#[test]
fn sparse_windows_stationary_drift_is_held_still() {
  let positions = [
    (2_397_216, 1_384.0, 457.0),
    (3_029_907, 1_403.0, 459.0),
    (3_186_238, 1_402.0, 459.0),
    (3_724_213, 1_401.0, 459.0),
    (4_394_084, 1_402.0, 459.0),
    (5_035_643, 1_403.0, 457.0),
    (5_601_508, 1_400.0, 457.0),
    (6_394_144, 1_403.0, 458.0),
    (6_598_040, 1_404.0, 460.0),
  ]
  .into_iter()
  .map(|(timestamp_us, x, y)| Position {
    segment: 0,
    timestamp_us,
    x,
    y,
  })
  .collect::<Vec<_>>();
  let compositor = compositor(stabilise_positions(&positions, 1_920.0), Vec::new());
  let early = compositor.smoothed_position(3_500_000, true).unwrap();
  let late = compositor.smoothed_position(6_200_000, true).unwrap();

  assert!((early.x - late.x).abs() < 0.001);
  assert!((early.y - late.y).abs() < 0.001);
  assert_eq!(compositor.motion_lean_degrees(5_000_000, 32.0), 0.0);
}
