// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Live keyboard transforms stay inside their normalized output canvas.

#[no_mangle]
pub extern "C" fn screenwide_keyboard_clamp_origin(origin: f64, extent: f64) -> f64 {
  origin.clamp(0.0, (1.0 - extent).max(0.0))
}

/// Largest uniform scale that preserves the resize anchor and all four edges.
#[no_mangle]
pub extern "C" fn screenwide_keyboard_resize_limit(
  x: f64,
  y: f64,
  width: f64,
  height: f64,
  anchor_x: f64,
  anchor_y: f64,
) -> f64 {
  let mut limit = f64::INFINITY;
  for (edge, anchor) in [
    (x, anchor_x),
    (x + width, anchor_x),
    (y, anchor_y),
    (y + height, anchor_y),
  ] {
    let offset = edge - anchor;
    if offset < 0.0 {
      limit = limit.min(anchor / -offset);
    } else if offset > 0.0 {
      limit = limit.min((1.0 - anchor) / offset);
    }
  }
  limit.max(0.0)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dragging_stops_at_each_edge_before_release_and_reenters_without_drift() {
    for extent in [0.1, 0.6, 1.0] {
      assert_eq!(screenwide_keyboard_clamp_origin(-2.0, extent), 0.0);
      assert_eq!(screenwide_keyboard_clamp_origin(2.0, extent), 1.0 - extent);
      assert_eq!(screenwide_keyboard_clamp_origin(0.0, extent), 0.0);
    }
    assert_eq!(screenwide_keyboard_clamp_origin(0.25, 0.2), 0.25);
  }

  #[test]
  fn resize_stops_at_the_first_canvas_edge_for_corners_sides_and_center() {
    let (x, y, width, height) = (0.1, 0.3, 0.4, 0.2);
    for anchor_x in [x, x + width / 2.0, x + width] {
      for anchor_y in [y, y + height / 2.0, y + height] {
        let scale = screenwide_keyboard_resize_limit(x, y, width, height, anchor_x, anchor_y);
        let left = anchor_x + (x - anchor_x) * scale;
        let top = anchor_y + (y - anchor_y) * scale;
        let right = left + width * scale;
        let bottom = top + height * scale;
        assert!(left >= -1e-9 && top >= -1e-9 && right <= 1.0 + 1e-9 && bottom <= 1.0 + 1e-9);
        assert!(
          left
            .abs()
            .min(top.abs())
            .min((1.0 - right).abs())
            .min((1.0 - bottom).abs())
            < 1e-9
        );
      }
    }
  }
}
