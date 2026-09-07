// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// Returns the axes of a deliberate monitor navigation step.
pub(super) fn deliberate_direction(
  across: f64,
  down: f64,
  horizontal_threshold: f64,
  vertical_threshold: f64,
) -> Option<(i8, i8)> {
  let x = across.abs() / horizontal_threshold;
  let y = down.abs() / vertical_threshold;
  if x.max(y) < 1.0 {
    return None;
  }
  // Use the same diagonal cone as destination selection. Unequal samples in
  // a diagonal stroke must not become a cardinal push just because one axis
  // reaches the threshold a few pixels earlier.
  Some(if x > y * 2.0 {
    (across.signum() as i8, 0)
  } else if y > x * 2.0 {
    (0, down.signum() as i8)
  } else {
    (across.signum() as i8, down.signum() as i8)
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn unequal_diagonal_samples_still_target_the_corner() {
    assert_eq!(
      deliberate_direction(-38.0, -25.0, 36.0, 36.0),
      Some((-1, -1))
    );
  }
}
