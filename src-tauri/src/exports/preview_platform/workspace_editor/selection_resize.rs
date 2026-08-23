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
  constraint: Option<NormalizedRect>,
  edges: u32,
  delta: (f64, f64),
  minimum_scale: f64,
  centered: bool,
) -> SelectionResize {
  let start_center = (start.x + start.width / 2.0, start.y + start.height / 2.0);
  let pivot = constraint.map_or_else(
    || start_center,
    |bounds| {
      (
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
      )
    },
  );
  let anchor = if constraint.is_some() || centered {
    pivot
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
  let maximum_scale = constraint.map_or(8.0, |bounds| {
    (bounds.width / start.width.max(0.000_001))
      .min(bounds.height / start.height.max(0.000_001))
      .max(0.01)
  });
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
  fn constraint_centres_and_clamps_the_live_selection() {
    let resize = selection_resize(
      NormalizedRect {
        height: 0.2,
        width: 0.4,
        x: 0.3,
        y: 0.4,
      },
      Some(NormalizedRect {
        height: 0.4,
        width: 0.8,
        x: 0.1,
        y: 0.3,
      }),
      2 | 8,
      (2.0, 2.0),
      0.1,
      false,
    );
    assert_eq!(resize.anchor, (0.5, 0.5));
    assert_eq!(resize.maximum_scale, 2.0);
    assert_eq!(resize.minimum_scale, 0.1);
    assert_eq!(resize.scale, 2.0);
    assert!((resize.vector.0 - 0.2).abs() < 1e-9);
    assert!((resize.vector.1 - 0.1).abs() < 1e-9);
  }
}
