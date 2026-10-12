// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Desktop-wide Region geometry. Interaction uses the point coordinate space
//! shared by AppKit screens; persisted regions remain relative to one stable
//! anchor display and may extend beyond its bounds.

use super::geometry::{Monitor, Point, Rect, Size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopDisplay {
  pub id: u32,
  pub origin: Point,
  pub size: Size,
  pub scale: f64,
}

/// A snapshot of the virtual desktop used by an OSC session. Platform code is
/// responsible only for discovering displays and constructing this binding.
#[derive(Clone, Debug)]
pub struct DesktopBinding {
  pub displays: Vec<DesktopDisplay>,
  pub anchor_id: u32,
  pub size: Size,
  pub layout_changed: bool,
}

impl DesktopBinding {
  pub fn anchor(&self) -> Option<DesktopDisplay> {
    self
      .displays
      .iter()
      .copied()
      .find(|display| display.id == self.anchor_id)
  }

  pub fn virtual_monitor(&self) -> Monitor {
    Monitor { size: self.size }
  }

  pub fn project_local(&self, local: Rect) -> Option<Rect> {
    self.anchor().map(|anchor| global_region(anchor, local))
  }

  pub fn reconcile_local(&self, local: Rect) -> Option<DesktopRegion> {
    let global = self.project_local(local)?;
    let (global, owner) = reconcile_region(&self.displays, Some(self.anchor_id), global)?;
    Some(DesktopRegion {
      anchor_local: local_projection(self.anchor()?, global),
      owner_local: local_projection(owner, global),
      global,
      owner_id: owner.id,
    })
  }

  pub fn display_at(&self, point: Point) -> Option<u32> {
    self.displays.iter().find_map(|display| {
      Rect::from_xywh(
        display.origin.x,
        display.origin.y,
        display.size.width,
        display.size.height,
      )
      .contains(point)
      .then_some(display.id)
    })
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopRegion {
  pub anchor_local: Rect,
  pub owner_local: Rect,
  pub global: Rect,
  pub owner_id: u32,
}

impl DesktopDisplay {
  pub fn valid(self) -> bool {
    self.origin.finite()
      && self.size.valid()
      && self.size.width > 0.0
      && self.size.height > 0.0
      && self.scale.is_finite()
      && self.scale > 0.0
  }

  pub fn logical_monitor(self) -> Monitor {
    Monitor {
      size: Size {
        width: self.size.width,
        height: self.size.height,
      },
    }
  }
}

pub fn global_region(display: DesktopDisplay, local: Rect) -> Rect {
  Rect {
    origin: Point {
      x: display.origin.x + local.origin.x,
      y: display.origin.y + local.origin.y,
    },
    size: Size {
      width: local.size.width,
      height: local.size.height,
    },
  }
}

/// Projects a desktop region into one display without clamping it. Keeping
/// off-display coordinates lets separate windows render one continuous frame
/// while AppKit clips each projection to its own monitor.
pub fn local_projection(display: DesktopDisplay, global: Rect) -> Rect {
  Rect {
    origin: Point {
      x: global.origin.x - display.origin.x,
      y: global.origin.y - display.origin.y,
    },
    size: Size {
      width: global.size.width,
      height: global.size.height,
    },
  }
}

pub fn overlap_area(display: DesktopDisplay, region: Rect) -> f64 {
  let left = display.origin.x.max(region.origin.x);
  let top = display.origin.y.max(region.origin.y);
  let right = (display.origin.x + display.size.width).min(region.right());
  let bottom = (display.origin.y + display.size.height).min(region.bottom());
  (right - left).max(0.0) * (bottom - top).max(0.0)
}

/// Chooses the display containing most of the region. An exact tie retains the
/// current owner, preventing a selector centred on a seam from oscillating.
pub fn owner_for_region(
  displays: &[DesktopDisplay],
  current_owner: Option<u32>,
  region: Rect,
) -> Option<DesktopDisplay> {
  let mut best = None;
  for display in displays.iter().copied().filter(|display| display.valid()) {
    let area = overlap_area(display, region);
    let retains_owner = current_owner == Some(display.id);
    match best {
      None => best = Some((display, area, retains_owner)),
      Some((_, best_area, best_retains))
        if area > best_area || (area == best_area && retains_owner && !best_retains) =>
      {
        best = Some((display, area, retains_owner));
      }
      _ => {}
    }
  }
  best.map(|(display, _, _)| display)
}

fn nearest_display(
  displays: &[DesktopDisplay],
  current_owner: Option<u32>,
  region: Rect,
) -> Option<DesktopDisplay> {
  displays
    .iter()
    .copied()
    .filter(|display| display.valid())
    .min_by(|a, b| {
      let distance = |display: DesktopDisplay| {
        let left = display.origin.x;
        let top = display.origin.y;
        let right = left + display.size.width;
        let bottom = top + display.size.height;
        let dx = if region.right() < left {
          left - region.right()
        } else if region.origin.x > right {
          region.origin.x - right
        } else {
          0.0
        };
        let dy = if region.bottom() < top {
          top - region.bottom()
        } else if region.origin.y > bottom {
          region.origin.y - bottom
        } else {
          0.0
        };
        dx * dx + dy * dy
      };
      distance(*a).total_cmp(&distance(*b)).then_with(|| {
        let a_owner = current_owner == Some(a.id);
        let b_owner = current_owner == Some(b.id);
        b_owner.cmp(&a_owner)
      })
    })
}

/// Reconciles a desktop-space region after the display topology changes.
/// Regions that remain completely covered by the surviving displays keep
/// their cross-display geometry. Stranded regions move to the closest display
/// and only shrink when that display cannot contain their current size.
pub fn reconcile_region(
  displays: &[DesktopDisplay],
  current_owner: Option<u32>,
  region: Rect,
) -> Option<(Rect, DesktopDisplay)> {
  if !region.committed() {
    return owner_for_region(displays, current_owner, region).map(|owner| (region, owner));
  }
  let area = region.size.width * region.size.height;
  let covered = displays
    .iter()
    .copied()
    .filter(|display| display.valid())
    .map(|display| overlap_area(display, region))
    .sum::<f64>();
  let owner = if covered > 0.0 {
    owner_for_region(displays, current_owner, region)
  } else {
    nearest_display(displays, current_owner, region)
  }?;
  if covered + 0.5 >= area {
    return Some((region, owner));
  }
  let local = local_projection(owner, region).clamp(owner.logical_monitor());
  Some((global_region(owner, local), owner))
}

#[cfg(test)]
mod tests;
