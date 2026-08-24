// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pixel-aligned geometry shared by the Windows DirectComposition surface.

/// Rounds both edges instead of rounding an origin and length independently.
/// This prevents one-pixel seams at fractional DPI and preview zoom factors.
pub(super) fn scaled_edges(origin: f64, length: f64, scale: f64) -> (i32, i32) {
  (
    (origin * scale).round() as i32,
    ((origin + length) * scale).round() as i32,
  )
}

/// Places a one-pixel primitive on a physical pixel centre. D3D pixel shader
/// positions are half-integers, so leaving guides and controls at an arbitrary
/// fractional coordinate spreads their coverage across neighbouring pixels.
pub(super) fn pixel_center(value: f64) -> f32 {
  (value.floor() + 0.5) as f32
}

#[cfg(test)]
mod tests {
  use super::{pixel_center, scaled_edges};

  #[test]
  fn fractional_geometry_does_not_lose_the_far_edge() {
    let edges = scaled_edges(282.4, 908.4, 1.0);
    assert_eq!(edges, (282, 1_191));
    assert_ne!(edges.0 + 908, edges.1);
  }

  #[test]
  fn one_pixel_primitives_land_on_pixel_centres() {
    assert_eq!(pixel_center(27.0), 27.5);
    assert_eq!(pixel_center(27.9), 27.5);
  }
}
