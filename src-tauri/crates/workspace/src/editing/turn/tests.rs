// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// A small upright layer, 60 points square with its middle at (70, 70),
/// chosen with the select tool, so its rings, 24 points round, reach past
/// its edge midpoints' grips.
fn small_layer() -> SurfaceState {
  let selection = PreviewSelection {
    x: 0.1,
    y: 0.1,
    width: 0.15,
    height: 0.15,
    image_width: 1.0,
    image_height: 1.0,
    turnable: 1,
    ..Default::default()
  };
  SurfaceState {
    editor_active: true,
    panes: vec![Some(crate::editing::Pane {
      base_rect: PreviewSurfaceRect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 400.0,
      },
      seen: true,
      ..Default::default()
    })],
    selection: Some(selection),
    ..Default::default()
  }
}

#[test]
fn a_grip_wins_a_press_where_a_ring_meets_it() {
  let state = small_layer();
  // The X ring, seen edge on, runs up the middle to 46: just below the top
  // grip at (70, 40), the press is the grip's.
  assert!(ring_hit(&state, (70.0, 47.0)).is_none());
  assert!(grip_over_rings(&state, (70.0, 47.0)));
  // Away from every grip the rings still take a press: the Z ring a
  // quarter turn round from the top-right diagonal.
  let reach = 24.0 * std::f64::consts::FRAC_1_SQRT_2;
  let on_z = (70.0 + reach, 70.0 - reach);
  assert!(!grip_over_rings(&state, on_z));
  assert_eq!(ring_hit(&state, on_z).map(|(hit, _)| hit.axis), Some(2));
}
