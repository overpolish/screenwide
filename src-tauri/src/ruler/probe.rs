// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::analysis::GradientMaps;

const NEIGHBOUR_HITS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProbeAxis {
  Horizontal,
  Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PixelProbe {
  pub axis: ProbeAxis,
  pub start: u32,
  pub end: u32,
  pub position: u32,
}

pub(crate) struct ProbeIndex {
  width: u32,
  height: u32,
  horizontal: Vec<Vec<u32>>,
  vertical: Vec<Vec<u32>>,
}

impl ProbeIndex {
  pub(crate) fn new(maps: &GradientMaps, threshold: u8) -> Self {
    let horizontal = (0..maps.height)
      .map(|y| {
        (1..maps.width)
          .filter(|x| is_edge(maps, ProbeAxis::Horizontal, *x, y, threshold))
          .collect()
      })
      .collect();
    let vertical = (0..maps.width)
      .map(|x| {
        (1..maps.height)
          .filter(|y| is_edge(maps, ProbeAxis::Vertical, *y, x, threshold))
          .collect()
      })
      .collect();
    Self {
      width: maps.width,
      height: maps.height,
      horizontal,
      vertical,
    }
  }

  pub(crate) fn probes_at(&self, x: u32, y: u32) -> [PixelProbe; 2] {
    [
      indexed_probe(
        ProbeAxis::Horizontal,
        x,
        y,
        self.width.saturating_sub(1),
        self.horizontal.get(y as usize).map_or(&[], Vec::as_slice),
      ),
      indexed_probe(
        ProbeAxis::Vertical,
        y,
        x,
        self.height.saturating_sub(1),
        self.vertical.get(x as usize).map_or(&[], Vec::as_slice),
      ),
    ]
  }
}

pub(crate) fn probes_at_threshold(
  maps: &GradientMaps,
  x: u32,
  y: u32,
  threshold: u8,
) -> [PixelProbe; 2] {
  [
    scanned_probe(
      maps,
      ProbeAxis::Horizontal,
      x,
      y,
      maps.width.saturating_sub(1),
      threshold,
    ),
    scanned_probe(
      maps,
      ProbeAxis::Vertical,
      y,
      x,
      maps.height.saturating_sub(1),
      threshold,
    ),
  ]
}

fn scanned_probe(
  maps: &GradientMaps,
  axis: ProbeAxis,
  target: u32,
  across: u32,
  limit: u32,
  threshold: u8,
) -> PixelProbe {
  let start = (1..=target.min(limit))
    .rev()
    .find(|position| is_edge(maps, axis, *position, across, threshold))
    .unwrap_or(0);
  let end = (target < limit)
    .then(|| (target.saturating_add(1).max(1))..=limit)
    .and_then(|positions| {
      positions
        .into_iter()
        .find(|position| is_edge(maps, axis, *position, across, threshold))
    })
    .unwrap_or(limit);
  PixelProbe {
    axis,
    start,
    end,
    position: across,
  }
}

fn indexed_probe(
  axis: ProbeAxis,
  target: u32,
  across: u32,
  limit: u32,
  edges: &[u32],
) -> PixelProbe {
  let split = edges.partition_point(|edge| *edge <= target);
  PixelProbe {
    axis,
    start: split
      .checked_sub(1)
      .and_then(|index| edges.get(index))
      .copied()
      .unwrap_or(0),
    end: edges.get(split).copied().unwrap_or(limit),
    position: across,
  }
}

fn is_edge(
  maps: &GradientMaps,
  axis: ProbeAxis,
  position: u32,
  across: u32,
  threshold: u8,
) -> bool {
  let sustained = (-1..=1)
    .filter(|offset| {
      let Some(across) = across.checked_add_signed(*offset) else {
        return false;
      };
      clears_threshold(maps, axis, position, across, threshold)
    })
    .take(NEIGHBOUR_HITS)
    .count()
    >= NEIGHBOUR_HITS;
  sustained || thin_stroke_edge(maps, axis, position, across, threshold)
}

fn clears_threshold(
  maps: &GradientMaps,
  axis: ProbeAxis,
  position: u32,
  across: u32,
  threshold: u8,
) -> bool {
  gradient_at(maps, axis, position, across) > 0
    && edge_mass(maps, axis, position, across) >= u16::from(threshold) * 2
}

/// A one-pixel stroke has only one hit across its endpoint, so the normal
/// neighbour rule deliberately mistakes it for a speckle. A real stroke has
/// a perpendicular edge continuing along at least two nearby pixels; an
/// isolated pixel has only one. Use that local corroboration without relaxing
/// the speckle filter for arbitrary single-pixel gradients.
fn thin_stroke_edge(
  maps: &GradientMaps,
  axis: ProbeAxis,
  position: u32,
  across: u32,
  threshold: u8,
) -> bool {
  if !clears_threshold(maps, axis, position, across, threshold) {
    return false;
  }
  (-2..=2)
    .filter_map(|offset| position.checked_add_signed(offset))
    .filter(|along| match axis {
      ProbeAxis::Horizontal => {
        clears_threshold(maps, ProbeAxis::Vertical, across, *along, threshold)
      }
      ProbeAxis::Vertical => {
        clears_threshold(maps, ProbeAxis::Horizontal, across, *along, threshold)
      }
    })
    .take(2)
    .count()
    >= 2
}

fn edge_mass(maps: &GradientMaps, axis: ProbeAxis, position: u32, across: u32) -> u16 {
  let center = u16::from(gradient_at(maps, axis, position, across));
  let before = position
    .checked_sub(1)
    .map_or(0, |value| u16::from(gradient_at(maps, axis, value, across)));
  let after = position
    .checked_add(1)
    .map_or(0, |value| u16::from(gradient_at(maps, axis, value, across)));
  center * 2 + before + after
}

fn gradient_at(maps: &GradientMaps, axis: ProbeAxis, position: u32, across: u32) -> u8 {
  let (x, y, plane) = match axis {
    ProbeAxis::Horizontal => (position, across, &maps.gx),
    ProbeAxis::Vertical => (across, position, &maps.gy),
  };
  if x >= maps.width || y >= maps.height {
    return 0;
  }
  plane[(y * maps.width + x) as usize]
}

#[cfg(test)]
mod tests {
  use super::*;

  fn maps(width: u32, height: u32) -> GradientMaps {
    GradientMaps {
      gx: vec![0; (width * height) as usize],
      gy: vec![0; (width * height) as usize],
      width,
      height,
    }
  }

  #[test]
  fn probes_stop_at_sustained_edges_on_both_axes() {
    let mut maps = maps(12, 10);
    for y in 3..=5 {
      maps.gx[(y * maps.width + 2) as usize] = 30;
      maps.gx[(y * maps.width + 9) as usize] = 30;
    }
    for x in 5..=7 {
      maps.gy[(2 * maps.width + x) as usize] = 30;
      maps.gy[(8 * maps.width + x) as usize] = 30;
    }
    assert_eq!(
      ProbeIndex::new(&maps, 24).probes_at(6, 4),
      [
        PixelProbe {
          axis: ProbeAxis::Horizontal,
          start: 2,
          end: 9,
          position: 4,
        },
        PixelProbe {
          axis: ProbeAxis::Vertical,
          start: 2,
          end: 8,
          position: 6,
        },
      ]
    );
  }

  #[test]
  fn isolated_speckle_does_not_stop_a_probe() {
    let mut maps = maps(10, 5);
    maps.gx[(2 * maps.width + 7) as usize] = 255;
    let horizontal = ProbeIndex::new(&maps, 24).probes_at(4, 2)[0];
    assert_eq!((horizontal.start, horizontal.end), (0, 9));
  }

  #[test]
  fn a_bright_one_pixel_stroke_still_has_detectable_endpoints() {
    let (width, height) = (37, 26);
    let mut rgba = vec![51; width * height * 4];
    for pixel in rgba.chunks_exact_mut(4) {
      pixel[3] = 255;
    }
    for x in 14..=23 {
      let value = if x == 14 || x == 23 { 120 } else { 125 };
      let offset = (13 * width + x) * 4;
      rgba[offset..offset + 4].copy_from_slice(&[value, value, value, 255]);
    }
    let maps = super::super::analysis::compute_gradients(
      &rgba,
      width.try_into().unwrap(),
      height.try_into().unwrap(),
    );
    let horizontal = probes_at_threshold(&maps, 18, 13, 5)[0];
    assert!(horizontal.start >= 14 && horizontal.start <= 15);
    assert!(horizontal.end >= 23 && horizontal.end <= 24);
  }

  #[test]
  fn split_antialiasing_mass_reaches_the_threshold() {
    let mut maps = maps(10, 5);
    for y in 1..=3 {
      maps.gx[(y * maps.width + 7) as usize] = 12;
      maps.gx[(y * maps.width + 8) as usize] = 12;
    }
    let horizontal = ProbeIndex::new(&maps, 18).probes_at(4, 2)[0];
    assert_eq!(horizontal.end, 7);
  }

  #[test]
  fn cursor_local_scan_applies_sensitivity_without_rebuilding_the_index() {
    let mut maps = maps(12, 7);
    for y in 2..=4 {
      maps.gx[(y * maps.width + 3) as usize] = 7;
      maps.gx[(y * maps.width + 9) as usize] = 7;
    }
    let balanced = probes_at_threshold(&maps, 6, 3, 24)[0];
    assert_eq!((balanced.start, balanced.end), (0, 11));
    let subtle = probes_at_threshold(&maps, 6, 3, 5)[0];
    assert_eq!((subtle.start, subtle.end), (3, 9));
  }

  #[test]
  fn indexed_lookup_uses_the_edge_under_the_pointer_only_as_the_start() {
    let mut maps = maps(10, 5);
    for y in 1..=3 {
      maps.gx[(y * maps.width + 4) as usize] = 30;
      maps.gx[(y * maps.width + 8) as usize] = 30;
    }
    let horizontal = ProbeIndex::new(&maps, 24).probes_at(4, 2)[0];
    assert_eq!((horizontal.start, horizontal.end), (4, 8));
  }
}
