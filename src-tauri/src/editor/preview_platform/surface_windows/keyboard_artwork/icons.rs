// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Lucide ArrowBigUp's 24-unit outline. Rasterize coverage directly so GDI's
// default pen, brush and font metrics cannot change the modifier appearance.
fn shift_outline() -> Vec<(f64, f64)> {
  let mut points = vec![(9.0, 19.0)];
  fn arc(points: &mut Vec<(f64, f64)>, x: f64, y: f64, radius: f64, from: f64, to: f64) {
    for step in 1..=16 {
      let angle = (from + (to - from) * f64::from(step) / 16.0).to_radians();
      points.push((x + radius * angle.cos(), y + radius * angle.sin()));
    }
  }
  arc(&mut points, 10.0, 19.0, 1.0, 180.0, 90.0);
  points.push((14.0, 20.0));
  arc(&mut points, 14.0, 19.0, 1.0, 90.0, 0.0);
  points.push((15.0, 13.0));
  arc(&mut points, 16.0, 13.0, 1.0, 180.0, 270.0);
  points.push((19.293, 12.0));
  arc(&mut points, 19.293, 11.293, 0.707, 90.0, -45.0);
  points.push((12.707, 3.707));
  arc(&mut points, 12.0, 4.414, 1.0, -45.0, -135.0);
  points.push((4.207, 10.793));
  arc(&mut points, 4.707, 11.293, 0.707, -135.0, -270.0);
  points.push((8.0, 12.0));
  arc(&mut points, 8.0, 13.0, 1.0, -90.0, 0.0);
  points.push((9.0, 19.0));
  points
}

pub(super) fn draw_shift(
  coverage: &mut [u8],
  width: u32,
  height: u32,
  key: (u32, u32),
  scale: f64,
) {
  let unit = scale * 0.5; // 24-unit icon displayed at 12 design points.
  let left = f64::from(key.0) + (f64::from(key.1) - 12.0 * scale) * 0.5;
  let top = (f64::from(height) - 12.0 * scale) * 0.5;
  let points = shift_outline()
    .into_iter()
    .map(|(x, y)| (left + x * unit, top + y * unit))
    .collect::<Vec<_>>();
  for row in top.max(0.0) as u32..((top + 12.0 * scale).ceil() as u32).min(height) {
    for column in left.max(0.0) as u32..((left + 12.0 * scale).ceil() as u32).min(width) {
      let p = (f64::from(column) + 0.5, f64::from(row) + 0.5);
      let distance = points
        .windows(2)
        .map(|pair| {
          let (a, b) = (pair[0], pair[1]);
          let v = (b.0 - a.0, b.1 - a.1);
          let t = (((p.0 - a.0) * v.0 + (p.1 - a.1) * v.1) / (v.0 * v.0 + v.1 * v.1).max(1e-12))
            .clamp(0.0, 1.0);
          (p.0 - a.0 - t * v.0).hypot(p.1 - a.1 - t * v.1)
        })
        .fold(f64::INFINITY, f64::min);
      coverage[(row * width + column) as usize] =
        ((unit + 0.5 - distance).clamp(0.0, 1.0) * 255.0).round() as u8;
    }
  }
}

pub(super) fn draw_modifiers(
  coverage: &mut [u8],
  width: u32,
  height: u32,
  keys: &[(u32, u32)],
  labels: &[String],
  scale: f64,
) {
  for (key, label) in keys.iter().zip(labels) {
    if label == "⇧" {
      draw_shift(coverage, width, height, *key, scale);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shift_is_a_visible_hollow_outline_centered_in_its_keycap() {
    let mut coverage = vec![0; 80 * 40];
    draw_shift(&mut coverage, 80, 40, (0, 40), 2.0);
    assert!(coverage.iter().any(|value| *value > 200));
    assert_eq!(
      coverage[24 * 80 + 20],
      0,
      "the arrow stem must remain hollow"
    );
    for row in coverage.chunks_exact(80) {
      assert!(row[40..].iter().all(|value| *value == 0));
    }
  }
}
