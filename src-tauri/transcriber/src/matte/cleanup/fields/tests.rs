// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{blur, distance, label};

#[test]
fn distances_are_exact_to_the_nearest_pixel_of_the_other_side() {
  let size = (23, 17);
  // An irregular shape: a disc and a bar, with a pinhole in the disc.
  let mask = (0..size.0 * size.1)
    .map(|index| {
      let (x, y) = ((index % size.0) as i32, (index / size.0) as i32);
      let disc = (x - 8).pow(2) + (y - 8).pow(2) <= 36 && !(x == 9 && y == 7);
      disc || (y == 14 && (3..20).contains(&x))
    })
    .collect::<Vec<_>>();
  let mut scratch = Vec::new();
  for value in [true, false] {
    let mut found = vec![0.0; mask.len()];
    distance(&mask, value, size, &mut found, &mut scratch);
    for (index, &got) in found.iter().enumerate() {
      let expected = if mask[index] != value {
        0.0
      } else {
        let (x, y) = ((index % size.0) as f32, (index / size.0) as f32);
        mask
          .iter()
          .enumerate()
          .filter(|(_, &other)| other != value)
          .map(|(other, _)| {
            let (ox, oy) = ((other % size.0) as f32, (other / size.0) as f32);
            ((x - ox).powi(2) + (y - oy).powi(2)).sqrt()
          })
          .fold(f32::INFINITY, f32::min)
      };
      assert!(
        (got - expected).abs() < 1e-3,
        "pixel {index}: {got} against {expected}"
      );
    }
  }
}

#[test]
fn areas_are_counted_by_side_by_side_neighbours_only() {
  // Two pixels touching only at a corner are two areas.
  let mask = [true, false, false, true];
  let (mut seen, mut order) = (vec![false; 4], Vec::new());
  let areas = label(&mask, true, (2, 2), &mut seen, &mut order);
  assert_eq!(
    areas.iter().map(|area| area.size).collect::<Vec<_>>(),
    [1, 1]
  );
}

#[test]
fn a_blur_keeps_the_total_and_spreads_a_point() {
  let size = (41, 41);
  let mut field = vec![0.0; size.0 * size.1];
  field[20 * 41 + 20] = 1.0;
  blur(&mut field, &mut Vec::new(), size, 4.0);
  let total = field.iter().sum::<f32>();
  assert!((total - 1.0).abs() < 1e-3, "{total}");
  let at = |x: usize, y: usize| field[y * 41 + x];
  assert!(
    at(20, 20) > at(24, 20) && at(24, 20) > at(28, 20),
    "falls away from the centre"
  );
  assert!(
    (at(24, 20) - at(20, 24)).abs() < 1e-4,
    "the same across as down"
  );
}
