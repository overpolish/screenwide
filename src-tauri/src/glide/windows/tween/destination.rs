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
