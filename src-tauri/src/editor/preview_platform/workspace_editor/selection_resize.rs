// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::NormalizedRect;

pub struct SelectionResize {
  pub anchor: (f64, f64),
  pub maximum_scale: f64,
  pub minimum_scale: f64,
  pub scale: f64,
  pub vector: (f64, f64),
}

pub fn selection_resize(
  start: NormalizedRect,
  edges: u32,
  delta: (f64, f64),
  minimum_scale: f64,
  centered: bool,
) -> SelectionResize {
  let start_center = (start.x + start.width / 2.0, start.y + start.height / 2.0);
  let anchor = if centered {
    start_center
  } else {
    (
      if edges & 1 != 0 {
        start.x + start.width
      } else if edges & 2 != 0 {
        start.x
      } else {
        start_center.0
      },
      if edges & 4 != 0 {
        start.y + start.height
      } else if edges & 8 != 0 {
        start.y
      } else {
        start_center.1
      },
    )
  };
  let handle = (
    if edges & 1 != 0 {
      start.x
    } else if edges & 2 != 0 {
      start.x + start.width
    } else {
      start_center.0
    },
    if edges & 4 != 0 {
      start.y
    } else if edges & 8 != 0 {
      start.y + start.height
    } else {
      start_center.1
    },
  );
  let vector = (handle.0 - anchor.0, handle.1 - anchor.1);
  let denominator = vector.0 * vector.0 + vector.1 * vector.1;
  let requested = if denominator > 0.0 {
    ((delta.0 + vector.0) * vector.0 + (delta.1 + vector.1) * vector.1) / denominator
  } else {
    1.0
  };
  let maximum_scale = 8.0;
  SelectionResize {
    anchor,
    maximum_scale,
    minimum_scale: minimum_scale.min(maximum_scale),
    scale: requested.clamp(minimum_scale.min(maximum_scale), maximum_scale),
    vector,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_opposite_corner_anchors_a_handle_drag() {
    let resize = selection_resize(
      NormalizedRect {
        height: 0.2,
        width: 0.4,
        x: 0.3,
        y: 0.4,
      },
      2 | 8,
      (0.2, 0.1),
      0.1,
      false,
    );
    assert_eq!(resize.anchor, (0.3, 0.4));
    assert_eq!(resize.maximum_scale, 8.0);
    assert_eq!(resize.minimum_scale, 0.1);
    assert!((resize.scale - 1.5).abs() < 1e-9);
    assert!((resize.vector.0 - 0.4).abs() < 1e-9);
    assert!((resize.vector.1 - 0.2).abs() < 1e-9);
  }

  #[test]
  fn a_centred_drag_pivots_on_the_selection_centre() {
    let resize = selection_resize(
      NormalizedRect {
        height: 0.2,
        width: 0.4,
        x: 0.3,
        y: 0.4,
      },
      2 | 8,
      (0.0, 0.0),
      0.1,
      true,
    );
    assert_eq!(resize.anchor, (0.5, 0.5));
    assert_eq!(resize.scale, 1.0);
  }
}
