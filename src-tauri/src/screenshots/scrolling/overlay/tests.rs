// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn centres_the_overlay_on_the_region() {
  let origin = centred_origin(500.0, 400.0);

  assert!((origin.x - (500.0 - WIDTH / 2.0)).abs() < f64::EPSILON);
  assert!((origin.y - (400.0 - HEIGHT / 2.0)).abs() < f64::EPSILON);
}

/// A region smaller than the overlay still centres it, so the status window
/// stays over the thing being captured rather than jumping to a corner.
#[test]
fn centres_a_region_smaller_than_the_overlay() {
  let origin = centred_origin(50.0, 20.0);

  assert!(origin.x < 0.0);
  assert!(origin.y < 0.0);
}
