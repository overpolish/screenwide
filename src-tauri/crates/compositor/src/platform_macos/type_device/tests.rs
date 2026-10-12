// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::TypeDevice;

/// Type comes out with partial coverage along its edges, inside the width it
/// was measured at, and standing on its baseline the ascent below the line's
/// top: the atlas lays a line out from those three numbers.
#[test]
fn type_is_antialiased_inside_its_measured_box_and_sits_on_its_baseline() {
  let device = TypeDevice::new(120.0).unwrap();
  let width = device.advance("World");
  let (ascent, descent) = device.vertical_metrics();
  assert!(width > 0.0 && ascent > 0.0 && descent > 0.0);
  let top = 4.0;
  let cell = (
    width.ceil() as u32 + 4,
    (top + ascent + descent).ceil() as u32 + 2,
  );
  let coverage = device.draw(cell, &[((2.0, top), "World")]).unwrap();
  let inked = |column: usize, row: usize| coverage[row * cell.0 as usize + column] > 0;

  let ink = coverage.iter().filter(|&&covered| covered > 0).count();
  let partial = coverage
    .iter()
    .filter(|&&covered| covered > 0 && covered < 255)
    .count();
  assert!(ink > 0);
  assert!(
    partial * 20 > ink,
    "{partial} of {ink} inked pixels are partial"
  );

  let columns =
    (0..cell.0 as usize).filter(|&column| (0..cell.1 as usize).any(|row| inked(column, row)));
  let right = columns.max().unwrap() as f64;
  assert!(
    right <= 2.0 + width + 1.0,
    "ink reaches {right}, measured {width}"
  );

  // "World" has no descenders, so its lowest ink is the baseline.
  let rows =
    (0..cell.1 as usize).filter(|&row| (0..cell.0 as usize).any(|column| inked(column, row)));
  let bottom = rows.max().unwrap() as f64;
  let baseline = top + ascent;
  assert!(
    (bottom - baseline).abs() <= 2.0,
    "ink ends at row {bottom}, the baseline is {baseline}"
  );
}

/// Tabular figures give every digit the same advance, so a counter's number
/// does not shift as it grows.
#[test]
fn numbers_set_every_digit_at_one_width() {
  let device = TypeDevice::numbers(48.0).unwrap();
  let one = device.advance("1");
  assert!(one > 0.0);
  assert!((device.advance("8") - one).abs() < 0.01);
}
