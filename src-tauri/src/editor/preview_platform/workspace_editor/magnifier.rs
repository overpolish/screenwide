// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// Pin a crop magnifier to its live crop frame after snapping the active edge.
pub fn crop_magnifier_anchor(frame: [f64; 4], pointer: (f64, f64), edges: u32) -> (f64, f64) {
  let right = frame[0] + frame[2];
  let bottom = frame[1] + frame[3];
  let x = if edges & 1 != 0 {
    frame[0]
  } else if edges & 2 != 0 {
    right
  } else {
    pointer.0
  };
  let y = if edges & 4 != 0 {
    frame[1]
  } else if edges & 8 != 0 {
    bottom
  } else {
    pointer.1
  };
  (x.clamp(frame[0], right), y.clamp(frame[1], bottom))
}

#[cfg(test)]
mod tests {
  use super::crop_magnifier_anchor;

  const FRAME: [f64; 4] = [100.0, 200.0, 300.0, 150.0];

  #[test]
  fn side_handles_clamp_perpendicular_pointer_travel() {
    assert_eq!(
      crop_magnifier_anchor(FRAME, (250.0, -500.0), 1),
      (100.0, 200.0)
    );
    assert_eq!(
      crop_magnifier_anchor(FRAME, (250.0, 900.0), 2),
      (400.0, 350.0)
    );
    assert_eq!(
      crop_magnifier_anchor(FRAME, (-500.0, 275.0), 4),
      (100.0, 200.0)
    );
    assert_eq!(
      crop_magnifier_anchor(FRAME, (900.0, 275.0), 8),
      (400.0, 350.0)
    );
  }

  #[test]
  fn corner_handles_remain_on_their_crop_corners() {
    assert_eq!(
      crop_magnifier_anchor(FRAME, (900.0, 900.0), 1 | 4),
      (100.0, 200.0)
    );
    assert_eq!(
      crop_magnifier_anchor(FRAME, (-500.0, -500.0), 2 | 8),
      (400.0, 350.0)
    );
  }
}
