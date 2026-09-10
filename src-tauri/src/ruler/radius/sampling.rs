// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{pixel_at, ComponentBox, Corner, GradientMaps, LocalPoint};

fn gradient_at(maps: &GradientMaps, horizontal: bool, position: u32, across: u32) -> u8 {
  let (x, y, plane) = if horizontal {
    (position, across, &maps.gx)
  } else {
    (across, position, &maps.gy)
  };
  if x >= maps.width || y >= maps.height {
    return 0;
  }
  plane[(y * maps.width + x) as usize]
}

fn edge_mass(maps: &GradientMaps, horizontal: bool, position: u32, across: u32) -> u16 {
  let center = u16::from(gradient_at(maps, horizontal, position, across));
  let before = position.checked_sub(1).map_or(0, |value| {
    u16::from(gradient_at(maps, horizontal, value, across))
  });
  let after = position.checked_add(1).map_or(0, |value| {
    u16::from(gradient_at(maps, horizontal, value, across))
  });
  center * 2 + before + after
}

fn clears_threshold(maps: &GradientMaps, horizontal: bool, x: u32, y: u32, threshold: u8) -> bool {
  let (position, across) = if horizontal { (x, y) } else { (y, x) };
  let mass = edge_mass(maps, horizontal, position, across);
  let peak = gradient_at(maps, horizontal, position, across) > 0
    && position
      .checked_sub(1)
      .is_none_or(|before| edge_mass(maps, horizontal, before, across) <= mass)
    && position
      .checked_add(1)
      .is_none_or(|after| edge_mass(maps, horizontal, after, across) <= mass);
  (peak && mass >= u16::from(threshold))
    || (x < maps.width
      && y < maps.height
      && maps.soft_strength((y * maps.width + x) as usize, horizontal) >= threshold)
}

pub(super) fn curve_points(
  bounds: ComponentBox,
  corner: Corner,
  maps: &GradientMaps,
  limit: u32,
  threshold: u8,
) -> Vec<LocalPoint> {
  let mut points = Vec::new();
  let mut add = |point: LocalPoint| {
    if point.u > 0
      && point.v > 0
      && !points
        .iter()
        .any(|item: &LocalPoint| item.u == point.u && item.v == point.v)
    {
      points.push(point);
    }
  };
  for v in 0..=limit {
    for u in 0..=limit {
      let (x, y) = pixel_at(bounds, corner, LocalPoint { u, v });
      if clears_threshold(maps, true, x, y, threshold) {
        add(LocalPoint { u, v });
        break;
      }
    }
  }
  for u in 0..=limit {
    for v in 0..=limit {
      let (x, y) = pixel_at(bounds, corner, LocalPoint { u, v });
      if clears_threshold(maps, false, x, y, threshold) {
        add(LocalPoint { u, v });
        break;
      }
    }
  }
  points
}

/// Component extents can include the shoulder of a softened transition.
/// Anchor the fit to the sustained straight-edge peak inside those extents.
pub(super) fn aligned_bounds(maps: &GradientMaps, bounds: ComponentBox) -> ComponentBox {
  let align = |position: u32, end: u32, across: u32, length: u32, horizontal: bool| {
    (position..=end)
      .max_by_key(|candidate| {
        (3..=5)
          .map(|sample| {
            u32::from(edge_mass(
              maps,
              horizontal,
              *candidate,
              across + length * sample / 8,
            ))
          })
          .sum::<u32>()
      })
      .unwrap_or(position)
  };
  let right = bounds.x + bounds.width;
  let bottom = bounds.y + bounds.height;
  let x = align(
    bounds.x,
    (bounds.x + 2).min(right),
    bounds.y,
    bounds.height,
    true,
  );
  let y = align(
    bounds.y,
    (bounds.y + 2).min(bottom),
    bounds.x,
    bounds.width,
    false,
  );
  let right = align(
    right.saturating_sub(2).max(x),
    right,
    bounds.y,
    bounds.height,
    true,
  );
  let bottom = align(
    bottom.saturating_sub(2).max(y),
    bottom,
    bounds.x,
    bounds.width,
    false,
  );
  ComponentBox {
    x,
    y,
    width: right - x,
    height: bottom - y,
  }
}
