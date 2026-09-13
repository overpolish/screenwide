// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn hides_private_aggregate_devices_from_microphone_selection() {
  let physical = cpal::DeviceDescriptionBuilder::new("Built-in Microphone").build();
  let aggregate = cpal::DeviceDescriptionBuilder::new("Cpal loopback aggregate")
    .interface_type(InterfaceType::Aggregate)
    .build();

  assert!(is_user_selectable_microphone(&physical));
  assert!(!is_user_selectable_microphone(&aggregate));
}

#[test]
fn sorts_camera_modes_by_size_then_orientation() {
  let mut modes = vec![
    (640, 480, 60),
    (1080, 1920, 60),
    (1552, 1552, 60),
    (1760, 1328, 60),
    (1328, 1760, 60),
    (1920, 1080, 60),
  ];

  sort_camera_modes(&mut modes, 60);

  assert_eq!(
    modes,
    vec![
      (1920, 1080, 60),
      (1080, 1920, 60),
      (1760, 1328, 60),
      (1328, 1760, 60),
      (1552, 1552, 60),
      (640, 480, 60),
    ]
  );
}

#[test]
fn falls_back_to_the_next_preference_rather_than_the_nearest_rate() {
  // A 30 fps-only camera under PAL lighting must land on 25, not on the
  // numerically closer 30, which flickers.
  assert_eq!(choose_fps(&[(1.0, 30.0)], &[50, 25]), 25);
  assert_eq!(choose_fps(&[(1.0, 60.0)], &[50, 25]), 50);
  assert_eq!(choose_fps(&[(1.0, 60.0)], &[60, 30]), 60);
  assert_eq!(choose_fps(&[(15.0, 30.0)], &[60, 30]), 30);
}

#[test]
fn clamps_to_the_closest_rate_when_no_preference_is_supported() {
  assert_eq!(choose_fps(&[(30.0, 30.0)], &[25]), 30);
}
