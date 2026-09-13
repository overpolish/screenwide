// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn crop_pixels_uses_the_planned_device_pixel_rect() {
  let image = CapturedImage {
    height: 4,
    rgba: (0_u8..64).collect(),
    width: 4,
  };
  let cropped = crop_pixels(
    &image,
    PixelRect {
      x: 1,
      y: 1,
      width: 2,
      height: 2,
    },
  )
  .unwrap();

  assert_eq!((cropped.width, cropped.height), (2, 2));
  assert_eq!(
    cropped.rgba,
    [&image.rgba[20..28], &image.rgba[36..44]].concat()
  );
}

#[test]
fn desktop_selection_composes_across_the_monitor_boundary() {
  let state = TextRecognitionState::default();
  let generation = state.begin();
  assert!(state.install(
    generation,
    [
      (
        1,
        1.0,
        CapturedImage {
          rgba: [[255, 0, 0, 255], [0, 255, 0, 255]].concat(),
          width: 2,
          height: 1,
        },
      ),
      (
        2,
        1.0,
        CapturedImage {
          rgba: [[0, 0, 255, 255], [255, 255, 0, 255]].concat(),
          width: 2,
          height: 1,
        },
      ),
    ]
  ));
  let displays = [
    OscDesktopDisplay {
      id: 1,
      origin: crate::osc::geometry::Point { x: 0.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 2.0,
        height: 1.0,
      },
      scale: 1.0,
    },
    OscDesktopDisplay {
      id: 2,
      origin: crate::osc::geometry::Point { x: 2.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 2.0,
        height: 1.0,
      },
      scale: 1.0,
    },
  ];
  let selected = state
    .select_desktop_region(&displays, 1, Rect::from_xywh(1.0, 0.0, 2.0, 1.0))
    .unwrap();

  assert_eq!((selected.width, selected.height), (2, 1));
  assert_eq!(selected.rgba, [[0, 255, 0, 255], [0, 0, 255, 255]].concat());
  assert_eq!(state.recognition_input().unwrap().0, generation + 1);
}

#[test]
fn a_new_session_rejects_an_older_recognition_result() {
  let state = TextRecognitionState::default();
  let old_generation = state.begin();
  let current_generation = state.begin();

  assert!(!state.is_current_generation(old_generation));
  assert!(state.is_current_generation(current_generation));
}

#[test]
fn dismissed_sessions_cannot_restart_from_topology_callbacks() {
  let state = TextRecognitionState::default();
  let generation = state.begin();
  assert!(state.install(
    generation,
    [(
      1,
      1.0,
      CapturedImage {
        rgba: vec![0, 0, 0, 255],
        width: 1,
        height: 1,
      }
    )],
  ));
  assert_eq!(state.active_generation(), Some(generation));

  state.cancel();

  assert_eq!(state.active_generation(), None);
  assert!(!state.is_current_generation(generation));
}
