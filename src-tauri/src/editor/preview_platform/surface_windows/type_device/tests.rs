// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::TypeDevice;

/// GDI drew this face with hard, one-bit edges above about 116 pixels to the
/// em; type that size must still come out with partial coverage along its
/// edges, and inside the width it was measured at.
#[test]
fn large_type_is_antialiased_within_its_measured_width() {
  let device = TypeDevice::new(240.0).unwrap();
  let width = device.advance("World");
  let (ascent, descent) = device.vertical_metrics();
  let cell = (
    width.ceil() as u32 + 4,
    (ascent + descent).ceil() as u32 + 2,
  );
  let coverage = device.draw(cell, &[((2.0, 1.0), "World")]).unwrap();

  let inked = coverage.iter().filter(|&&covered| covered > 0).count();
  let partial = coverage
    .iter()
    .filter(|&&covered| covered > 0 && covered < 255)
    .count();
  assert!(inked > 0);
  assert!(
    partial * 20 > inked,
    "{partial} of {inked} inked pixels are partial"
  );

  let columns = (0..cell.0 as usize)
    .filter(|&column| (0..cell.1 as usize).any(|row| coverage[row * cell.0 as usize + column] > 0));
  let right = columns.max().unwrap() as f64;
  assert!(
    right <= 2.0 + width + 1.0,
    "ink reaches {right}, measured {width}"
  );
}
