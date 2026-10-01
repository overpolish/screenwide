// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Regression coverage for arrowheads on strongly curved and very short paths.

use super::*;

fn trim_parameter(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
  tip: AnnotationPoint,
  head_length: f64,
  at_end: bool,
) -> f64 {
  let mut low = if at_end { 0.5 } else { 0.0 };
  let mut high = if at_end { 1.0 } else { 0.5 };
  for _ in 0..10 {
    let middle = (low + high) / 2.0;
    let at = curve_point(start, control, end, middle);
    let inside = (at.x - tip.x).hypot(at.y - tip.y) <= head_length;
    if inside == at_end {
      high = middle;
    } else {
      low = middle;
    }
  }
  if at_end {
    high
  } else {
    low
  }
}

fn ink_components(image: &crate::screenshots::CapturedImage) -> usize {
  let mut visited = vec![false; image.width as usize * image.height as usize];
  let mut components = 0;
  for y in 0..image.height as usize {
    for x in 0..image.width as usize {
      let index = y * image.width as usize + x;
      if visited[index] || image.rgba[index * 4] <= 128 {
        continue;
      }
      components += 1;
      let mut pending = vec![(x, y)];
      visited[index] = true;
      while let Some((cx, cy)) = pending.pop() {
        for ny in cy.saturating_sub(1)..=(cy + 1).min(image.height as usize - 1) {
          for nx in cx.saturating_sub(1)..=(cx + 1).min(image.width as usize - 1) {
            let neighbour = ny * image.width as usize + nx;
            if !visited[neighbour] && image.rgba[neighbour * 4] > 128 {
              visited[neighbour] = true;
              pending.push((nx, ny));
            }
          }
        }
      }
    }
  }
  components
}

/// The head cross-section must be symmetric about the line from the trimmed
/// shaft endpoint to the tip, even when the endpoint tangent points elsewhere.
#[test]
fn large_curved_head_base_is_centred_on_the_trimmed_curve() {
  let (start, control, end) = (point(287.0, 701.0), point(666.5, 247.0), point(100.0, 77.0));
  let width = 60.0;
  for (name, head) in [("end", AnnotationHead::End), ("both", AnnotationHead::Both)] {
    let image = composed((538, 756), arrow(start, control, end, width, head));
    write_png(&format!("regression-large-curved-{name}"), &image);
    for (tip, at_end) in [(end, true), (start, false)] {
      if !at_end && name == "end" {
        continue;
      }
      let trim = trim_parameter(start, control, end, tip, width * 4.0, at_end);
      let join = curve_point(start, control, end, trim);
      let length = (tip.x - join.x).hypot(tip.y - join.y);
      let normal = point(-(tip.y - join.y) / length, (tip.x - join.x) / length);
      let centre = point(join.x * 0.6 + tip.x * 0.4, join.y * 0.6 + tip.y * 0.4);
      let ink: Vec<f64> = (-720..=720)
        .filter_map(|step| {
          let offset = f64::from(step) * 0.25;
          let x = (centre.x + normal.x * offset).round();
          let y = (centre.y + normal.y * offset).round();
          if x < 0.0 || y < 0.0 || x >= f64::from(image.width) || y >= f64::from(image.height) {
            return None;
          }
          (image.rgba[(y as usize * image.width as usize + x as usize) * 4] > 128).then_some(offset)
        })
        .collect();
      let first = *ink.first().expect("missing head");
      let last = *ink.last().unwrap();
      assert!(
        (first + last).abs() < 3.0,
        "{name} head is off-centre: {first}..{last}"
      );
      assert!(last - first > 100.0, "{name} head lost its requested size");
    }
  }
}

/// A short, thick curve cannot fit full-size heads. It must reduce the visual
/// scale as a unit so the heads overlap the shaft and leave one connected
/// annotation.
#[test]
fn short_thick_curved_arrow_stays_connected() {
  let (start, control, end) = (point(106.0, 299.0), point(312.5, 186.5), point(129.0, 40.0));
  for (name, head) in [("end", AnnotationHead::End), ("both", AnnotationHead::Both)] {
    let image = composed((432, 330), arrow(start, control, end, 60.0, head));
    write_png(&format!("regression-short-curved-{name}"), &image);
    assert_eq!(
      ink_components(&image),
      1,
      "short {name} arrow ink became disconnected"
    );
  }
}

/// A folded curve whose midpoint nearly reaches its end tip still has a
/// usable shaft. Fit room must come from the rest of that half, rather than
/// treating the midpoint as the whole arrow's available length.
#[test]
fn folded_short_curved_arrow_stays_connected() {
  let (start, control, end) = (point(100.0, 300.0), point(95.0, 0.0), point(130.0, 100.0));
  let image = composed(
    (240, 340),
    arrow(start, control, end, 60.0, AnnotationHead::End),
  );
  write_png("regression-folded-short-curved", &image);
  assert_eq!(
    ink_components(&image),
    1,
    "folded short arrow ink became disconnected"
  );
}
