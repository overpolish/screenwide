// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::glide::{core::GlideFrame, region_rect::RegionGravity};

pub(super) fn travel(
  start: GlideFrame,
  requested: GlideFrame,
  resizable: bool,
  gravity: RegionGravity,
) -> (GlideFrame, bool) {
  let changes_size = start.width != requested.width || start.height != requested.height;
  if !changes_size || resizable {
    return (requested, changes_size);
  }
  let (x, y) = crate::glide::core::corrected_origin(requested, start, gravity);
  (
    GlideFrame {
      x,
      y,
      width: start.width,
      height: start.height,
    },
    false,
  )
}

pub(super) fn interpolate(start: GlideFrame, destination: GlideFrame, fraction: f64) -> GlideFrame {
  GlideFrame {
    x: lerp(start.x, destination.x, fraction),
    y: lerp(start.y, destination.y, fraction),
    width: lerp(start.width, destination.width, fraction),
    height: lerp(start.height, destination.height, fraction),
  }
}

fn lerp(from: f64, to: f64, fraction: f64) -> f64 {
  from + (to - from) * fraction
}

pub(super) fn eased(fraction: f64) -> f64 {
  let remaining = 1.0 - fraction.clamp(0.0, 1.0);
  1.0 - remaining * remaining * remaining
}

#[cfg(test)]
mod motion_tests {
  use super::*;

  #[test]
  fn ease_starts_and_ends_on_endpoints() {
    assert_eq!(eased(0.0), 0.0);
    assert_eq!(eased(1.0), 1.0);
  }

  #[test]
  fn interpolation_lands_on_destination() {
    let start = GlideFrame {
      x: 0.0,
      y: 0.0,
      width: 100.0,
      height: 100.0,
    };
    let destination = GlideFrame {
      x: 40.0,
      y: 20.0,
      width: 300.0,
      height: 200.0,
    };
    assert_eq!(interpolate(start, destination, 1.0), destination);
    assert_eq!(interpolate(start, destination, 0.0), start);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::glide::region_rect::{Gravity, RegionGravity};

  #[test]
  fn fixed_size_windows_move_to_the_regions_gravity_without_resizing() {
    let start = GlideFrame {
      x: 100.0,
      y: 100.0,
      width: 200.0,
      height: 44.0,
    };
    let requested = GlideFrame {
      x: 500.0,
      y: 0.0,
      width: 500.0,
      height: 800.0,
    };
    let (destination, resizes) = travel(
      start,
      requested,
      false,
      RegionGravity {
        horizontal: Gravity::End,
        vertical: Gravity::Center,
      },
    );
    assert!(!resizes);
    assert_eq!(
      destination,
      GlideFrame {
        x: 800.0,
        y: 378.0,
        width: 200.0,
        height: 44.0
      }
    );
  }
}
