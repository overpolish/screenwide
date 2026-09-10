// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{edge_mass_x, edge_mass_y, ComponentBox, GradientMaps};

/// Trim a one-sided protrusion only when all four replacement sides have
/// sustained evidence. Sparse icon strokes should not extend a card's bounds.
/// Nine samples per candidate side keep the search linear in its perimeter.
pub(super) fn refine(maps: &GradientMaps, bounds: ComponentBox, threshold: u8) -> ComponentBox {
  if bounds.width < 12 || bounds.height < 12 {
    return bounds;
  }
  let supported = |position: u32, vertical: bool| {
    let (start, length) = if vertical {
      (bounds.y, bounds.height)
    } else {
      (bounds.x, bounds.width)
    };
    (0..9)
      .filter(|sample| {
        let across = start + length * (2 + sample) / 12;
        let (x, y) = if vertical {
          (position, across)
        } else {
          (across, position)
        };
        if x >= maps.width || y >= maps.height {
          return false;
        }
        let index = (y * maps.width + x) as usize;
        let w = maps.width as usize;
        let hard = if vertical {
          maps.gx[index] > 0 && edge_mass_x(&maps.gx, index, w) >= u16::from(threshold) * 2
        } else {
          maps.gy[index] > 0
            && edge_mass_y(&maps.gy, index, w, maps.height as usize) >= u16::from(threshold) * 2
        };
        hard
          || maps
            .soft_edges
            .as_ref()
            .is_some_and(|(gx, gy)| (if vertical { gx[index] } else { gy[index] }) >= threshold)
      })
      .count()
      >= 7
  };
  let right = bounds.x + bounds.width;
  let bottom = bounds.y + bounds.height;
  let Some(left_edge) = (bounds.x..=right).find(|x| supported(*x, true)) else {
    return bounds;
  };
  let Some(right_edge) = (bounds.x..=right).rev().find(|x| supported(*x, true)) else {
    return bounds;
  };
  let Some(top_edge) = (bounds.y..=bottom).find(|y| supported(*y, false)) else {
    return bounds;
  };
  let Some(bottom_edge) = (bounds.y..=bottom).rev().find(|y| supported(*y, false)) else {
    return bounds;
  };
  let moved = [
    left_edge - bounds.x,
    right - right_edge,
    top_edge - bounds.y,
    bottom - bottom_edge,
  ];
  if moved.iter().filter(|distance| **distance > 2).count() != 1
    || right_edge - left_edge < bounds.width / 2
    || bottom_edge - top_edge < bounds.height / 2
  {
    return bounds;
  }
  ComponentBox {
    x: left_edge,
    y: top_edge,
    width: right_edge - left_edge,
    height: bottom_edge - top_edge,
  }
}
