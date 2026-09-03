// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::magnifier_for;
use crate::osc::{
  geometry::{Point, Rect, Size},
  protocol::OscResult,
};

use crate::windows::screenshot_region::native_osc_windows::{
  renderer::magnifier_anchor, state::SurfaceSet, surface::MagnifierAnchor,
};

pub(super) fn route(
  set: &mut SurfaceSet,
  desktop_point: Point,
  result: &OscResult,
  phase: u32,
) -> bool {
  let magnifier =
    magnifier_for(phase, result.gesture, result.has_region, result.handle).map(|edges| {
      (
        magnifier_anchor(
          desktop_point,
          Rect::from_xywh(result.x, result.y, result.width, result.height),
          edges,
        ),
        edges,
      )
    });
  let target_display = magnifier.and_then(|_| {
    set
      .all_mut()
      .find(|surface| {
        let offset = surface.desktop_offset();
        contains(desktop_point, offset, surface.logical_size())
      })
      .map(|surface| surface.display_id)
  });

  let mut changed = false;
  for surface in set.all_mut() {
    let next = magnifier.and_then(|(anchor, edges)| {
      (target_display == Some(surface.display_id) && surface.has_magnifier_source()).then(|| {
        let offset = surface.desktop_offset();
        MagnifierAnchor {
          point: Point {
            x: anchor.x - offset.x,
            y: anchor.y - offset.y,
          },
          edges,
        }
      })
    });
    changed |= next != surface.magnifier;
    surface.magnifier = next;
  }
  changed
}

fn contains(point: Point, offset: Point, size: Size) -> bool {
  point.x >= offset.x
    && point.x < offset.x + size.width
    && point.y >= offset.y
    && point.y < offset.y + size.height
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn an_exact_seam_belongs_only_to_the_destination_surface() {
    let point = Point {
      x: 1000.0,
      y: 200.0,
    };
    let size = Size {
      width: 1000.0,
      height: 600.0,
    };
    assert!(!contains(point, Point::default(), size));
    assert!(contains(point, Point { x: 1000.0, y: 0.0 }, size));
  }
}
