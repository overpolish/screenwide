// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
fn maps() -> GradientMaps {
  GradientMaps {
    soft_edges: None,
    gx: vec![0; 80 * 80],
    gy: vec![0; 80 * 80],
    width: 80,
    height: 80,
  }
}

fn paint_corner(
  maps: &mut GradientMaps,
  bounds: ComponentBox,
  corner: Corner,
  radius: u32,
  strength: u8,
) {
  for step in 0..=90 {
    let angle = f64::from(step) / 90.0 * std::f64::consts::FRAC_PI_2;
    let point = LocalPoint {
      u: (f64::from(radius) - f64::from(radius) * angle.cos()).round() as u32,
      v: (f64::from(radius) - f64::from(radius) * angle.sin()).round() as u32,
    };
    let (x, y) = pixel_at(bounds, corner, point);
    let index = (y * maps.width + x) as usize;
    maps.gx[index] = (f64::from(strength) * angle.cos()).round() as u8;
    maps.gy[index] = (f64::from(strength) * angle.sin()).round() as u8;
  }
  for v in radius..=bounds.height {
    let (x, y) = pixel_at(bounds, corner, LocalPoint { u: 0, v });
    maps.gx[(y * maps.width + x) as usize] = strength;
  }
  for u in radius..=bounds.width {
    let (x, y) = pixel_at(bounds, corner, LocalPoint { u, v: 0 });
    maps.gy[(y * maps.width + x) as usize] = strength;
  }
}

#[test]
fn fits_all_corner_orientations() {
  let bounds = ComponentBox {
    x: 10,
    y: 10,
    width: 50,
    height: 40,
  };
  for corner in [
    Corner::TopLeft,
    Corner::TopRight,
    Corner::BottomLeft,
    Corner::BottomRight,
  ] {
    let mut field = maps();
    paint_corner(&mut field, bounds, corner, 8, 30);
    let origin = corner_origin(bounds, corner);
    let cursor = Point {
      x: origin.x + if corner.right() { -2.0 } else { 2.0 },
      y: origin.y + if corner.bottom() { -2.0 } else { 2.0 },
    };
    assert_eq!(
      corner_radius_at(&[bounds], cursor, &field, 24, 1.0, 1.0)
        .map(|value| (value.corner, value.radius)),
      Some((corner, 8))
    );
  }
}

#[test]
fn sensitivity_and_cursor_distance_are_respected() {
  let bounds = ComponentBox {
    x: 10,
    y: 10,
    width: 50,
    height: 40,
  };
  let mut field = maps();
  paint_corner(&mut field, bounds, Corner::TopLeft, 8, 10);
  assert!(corner_radius_at(&[bounds], Point { x: 12.0, y: 12.0 }, &field, 24, 1.0, 1.0).is_none());
  assert_eq!(
    corner_radius_at(&[bounds], Point { x: 12.0, y: 12.0 }, &field, 5, 1.0, 1.0)
      .map(|value| value.radius),
    Some(8)
  );
  assert!(corner_radius_at(&[bounds], Point { x: 70.0, y: 70.0 }, &field, 5, 1.0, 1.0).is_none());
}

fn rounded_card(radius: f64, softness: f64, foreground: u8) -> GradientMaps {
  let size = 120u32;
  let mut rgba = vec![255; (size * size * 4) as usize];
  for y in 0..size {
    for x in 0..size {
      let qx = (f64::from(x) + 0.5 - 60.0).abs() - (40.0 - radius);
      let qy = (f64::from(y) + 0.5 - 60.0).abs() - (35.0 - radius);
      let distance = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;
      let coverage = (0.5 - distance / softness).clamp(0.0, 1.0);
      let coverage = coverage * coverage * (3.0 - 2.0 * coverage);
      let value = (255.0 - f64::from(255 - foreground) * coverage).round() as u8;
      let index = ((y * size + x) * 4) as usize;
      rgba[index..index + 3].fill(value);
    }
  }
  crate::ruler::analysis::compute_gradients(&rgba, size, size)
}

#[test]
fn detects_soft_low_contrast_corners_from_image_pixels() {
  for radius in [8u32, 14, 24] {
    for foreground in [0x30, 0xE5] {
      for softness in [1.0, 3.0, 5.0] {
        let field = rounded_card(f64::from(radius), softness, foreground);
        let boxes = crate::ruler::analysis::detect_boxes(&field, 24);
        for (corner, x, y) in [
          (Corner::TopLeft, 24.0, 29.0),
          (Corner::TopRight, 96.0, 29.0),
          (Corner::BottomLeft, 24.0, 91.0),
          (Corner::BottomRight, 96.0, 91.0),
        ] {
          let result = corner_radius_at(&boxes, Point { x, y }, &field, 24, 1.0, 1.0);
          let estimate = result
            .unwrap_or_else(|| panic!("missing {corner:?}, softness {softness}, boxes {boxes:?}"));
          assert_eq!(estimate.corner, corner);
          assert!(
            estimate.radius.abs_diff(radius) <= 2,
            "{estimate:?}, expected radius {radius}, softness {softness}, colour {foreground}"
          );
        }
      }
    }
  }
}

#[test]
fn square_cards_do_not_create_radius_candidates() {
  for softness in [1.0, 3.0, 5.0] {
    let field = rounded_card(0.0, softness, 0xE5);
    let boxes = crate::ruler::analysis::detect_boxes(&field, 24);
    assert!(
      corner_radius_at(&boxes, Point { x: 22.0, y: 27.0 }, &field, 24, 1.0, 1.0).is_none(),
      "softness {softness}"
    );
  }
}
