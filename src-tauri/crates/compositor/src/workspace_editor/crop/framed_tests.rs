// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const IMAGE: NormalizedRect = NormalizedRect {
  x: 0.0,
  y: 0.0,
  width: 1.0,
  height: 1.0,
};

const WINDOW: NormalizedRect = NormalizedRect {
  x: 0.4,
  y: 0.4,
  width: 0.2,
  height: 0.1,
};

fn close(a: f64, b: f64) -> bool {
  (a - b).abs() < 1e-9
}

#[test]
fn a_corner_keeps_the_window_shape_and_its_opposite_corner() {
  let pulled = apply_framed_crop_resize(
    WINDOW,
    IMAGE,
    FRAME_EDGE_RIGHT | FRAME_EDGE_BOTTOM,
    (0.1, 0.01),
    (0.01, 0.01),
  );
  assert!(close(pulled.width / pulled.height, 2.0));
  assert!(close(pulled.width, 0.3));
  assert!(close(pulled.x, 0.4) && close(pulled.y, 0.4));
}

#[test]
fn an_edge_grows_the_window_about_the_middle_of_the_other_axis() {
  let pulled = apply_framed_crop_resize(WINDOW, IMAGE, FRAME_EDGE_LEFT, (-0.2, 0.0), (0.01, 0.01));
  assert!(close(pulled.width, 0.4) && close(pulled.height, 0.2));
  assert!(close(pulled.x + pulled.width, 0.6));
  assert!(close(pulled.y + pulled.height / 2.0, 0.45));
}

#[test]
fn the_window_stays_inside_its_picture_and_above_its_floor() {
  let grown = apply_framed_crop_resize(
    WINDOW,
    IMAGE,
    FRAME_EDGE_RIGHT | FRAME_EDGE_BOTTOM,
    (5.0, 5.0),
    (0.01, 0.01),
  );
  assert!(close(grown.x + grown.width, 1.0));
  assert!(close(grown.width / grown.height, 2.0));
  let shrunk = apply_framed_crop_resize(
    WINDOW,
    IMAGE,
    FRAME_EDGE_RIGHT | FRAME_EDGE_BOTTOM,
    (-5.0, -5.0),
    (0.05, 0.05),
  );
  assert!(close(shrunk.height, 0.05) && close(shrunk.width, 0.1));
}
