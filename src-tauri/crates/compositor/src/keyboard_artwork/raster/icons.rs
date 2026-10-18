// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The modifier icons the React keycap draws, Lucide's 24-unit outlines
//! stroked two units wide, rasterised as coverage directly so no platform's
//! pen or font metrics change their look.

use super::labels::Icon;

/// An icon's size on its cap, in design points.
pub(super) const ICON_SIZE: f64 = 12.0;

/// Appends the arc round `(x, y)` from `from` to `to` degrees.
fn arc(points: &mut Vec<(f64, f64)>, (x, y): (f64, f64), radius: f64, from: f64, to: f64) {
  for step in 1..=16 {
    let angle = (from + (to - from) * f64::from(step) / 16.0).to_radians();
    points.push((x + radius * angle.cos(), y + radius * angle.sin()));
  }
}

/// Lucide Command.
fn command() -> Vec<(f64, f64)> {
  let mut points = vec![(15.0, 6.0), (15.0, 18.0)];
  arc(&mut points, (18.0, 18.0), 3.0, 180.0, -90.0);
  points.push((6.0, 15.0));
  arc(&mut points, (6.0, 18.0), 3.0, -90.0, -360.0);
  points.push((9.0, 6.0));
  arc(&mut points, (6.0, 6.0), 3.0, 0.0, -270.0);
  points.push((18.0, 9.0));
  arc(&mut points, (18.0, 6.0), 3.0, 90.0, -180.0);
  points
}

/// Lucide ArrowBigUp.
fn shift() -> Vec<(f64, f64)> {
  let mut points = vec![(9.0, 19.0)];
  arc(&mut points, (10.0, 19.0), 1.0, 180.0, 90.0);
  points.push((14.0, 20.0));
  arc(&mut points, (14.0, 19.0), 1.0, 90.0, 0.0);
  points.push((15.0, 13.0));
  arc(&mut points, (16.0, 13.0), 1.0, 180.0, 270.0);
  points.push((19.293, 12.0));
  arc(&mut points, (19.293, 11.293), 0.707, 90.0, -45.0);
  points.push((12.707, 3.707));
  arc(&mut points, (12.0, 4.414), 1.0, -45.0, -135.0);
  points.push((4.207, 10.793));
  arc(&mut points, (4.707, 11.293), 0.707, -135.0, -270.0);
  points.push((8.0, 12.0));
  arc(&mut points, (8.0, 13.0), 1.0, -90.0, 0.0);
  points.push((9.0, 19.0));
  points
}

/// Lucide ChevronUp.
fn control() -> Vec<(f64, f64)> {
  vec![(18.0, 15.0), (12.0, 9.0), (6.0, 15.0)]
}

fn outline(icon: Icon) -> Vec<(f64, f64)> {
  match icon {
    Icon::Command => command(),
    Icon::Shift => shift(),
    Icon::Control => control(),
  }
}

/// Strokes `icon` centred on the cap `key` (its left edge and width in
/// pixels) into `coverage`, a `width` by `height` plane.
pub(super) fn draw(
  coverage: &mut [u8],
  (width, height): (u32, u32),
  key: (u32, u32),
  icon: Icon,
  scale: f64,
) {
  let size = ICON_SIZE * scale;
  // Pixels per icon unit, which is also the stroke's half width.
  let unit = size / 24.0;
  let left = f64::from(key.0) + (f64::from(key.1) - size) * 0.5;
  let top = (f64::from(height) - size) * 0.5;
  let points = outline(icon)
    .into_iter()
    .map(|(x, y)| (left + x * unit, top + y * unit))
    .collect::<Vec<_>>();
  let reach = unit + 1.0;
  let rows = (top - reach).max(0.0) as u32..((top + size + reach).ceil() as u32).min(height);
  for row in rows {
    let columns = (left - reach).max(0.0) as u32..((left + size + reach).ceil() as u32).min(width);
    for column in columns {
      let point = (f64::from(column) + 0.5, f64::from(row) + 0.5);
      let distance = points
        .windows(2)
        .map(|pair| segment_distance(point, pair[0], pair[1]))
        .fold(f64::INFINITY, f64::min);
      let value = ((unit + 0.5 - distance).clamp(0.0, 1.0) * 255.0).round() as u8;
      let pixel = &mut coverage[(row * width + column) as usize];
      *pixel = (*pixel).max(value);
    }
  }
}

fn segment_distance(point: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
  let along = (b.0 - a.0, b.1 - a.1);
  let t = (((point.0 - a.0) * along.0 + (point.1 - a.1) * along.1)
    / (along.0 * along.0 + along.1 * along.1).max(1e-12))
  .clamp(0.0, 1.0);
  (point.0 - a.0 - t * along.0).hypot(point.1 - a.1 - t * along.1)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shift_is_a_visible_hollow_outline_centered_in_its_keycap() {
    let mut coverage = vec![0; 80 * 40];
    draw(&mut coverage, (80, 40), (0, 40), Icon::Shift, 2.0);
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
