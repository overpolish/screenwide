// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
  osc::geometry::Point,
  ruler::analysis::{ComponentBox, GradientMaps},
};

const MAXIMUM_RADIUS: u32 = 128;
const MAXIMUM_CURSOR_DISTANCE: f64 = 96.0;
const MAXIMUM_ARC_DISTANCE: f64 = 24.0;
const MAXIMUM_CANDIDATES: usize = 16;
const FIT_TOLERANCE: f64 = 2.0;
const ANGLE_BINS: usize = 8;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Corner {
  TopLeft = 1,
  TopRight = 2,
  BottomLeft = 3,
  BottomRight = 4,
}

impl Corner {
  pub(crate) const fn right(self) -> bool {
    matches!(self, Self::TopRight | Self::BottomRight)
  }

  pub(crate) const fn bottom(self) -> bool {
    matches!(self, Self::BottomLeft | Self::BottomRight)
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RadiusEstimate {
  pub bounds: ComponentBox,
  pub corner: Corner,
  pub radius: u32,
  pub low_confidence: bool,
}

#[derive(Clone, Copy)]
struct LocalPoint {
  u: u32,
  v: u32,
}

#[derive(Clone, Copy)]
struct Fit {
  radius: u32,
  score: f64,
  low_confidence: bool,
}

fn pixel_at(bounds: ComponentBox, corner: Corner, point: LocalPoint) -> (u32, u32) {
  (
    if corner.right() {
      bounds.x + bounds.width - point.u
    } else {
      bounds.x + point.u
    },
    if corner.bottom() {
      bounds.y + bounds.height - point.v
    } else {
      bounds.y + point.v
    },
  )
}

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
  gradient_at(maps, horizontal, position, across) > 0
    && edge_mass(maps, horizontal, position, across) >= u16::from(threshold)
}

fn curve_points(
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

fn median(mut values: Vec<f64>) -> f64 {
  values.sort_by(f64::total_cmp);
  let middle = values.len() / 2;
  if values.len() % 2 == 0 {
    (values[middle - 1] + values[middle]) * 0.5
  } else {
    values[middle]
  }
}

fn fit_score(coverage: f64, inlier_share: f64, residual: f64) -> f64 {
  residual * 3.0 + (1.0 - coverage) * 0.5 + (1.0 - inlier_share) * 0.5
}

fn fit_circle(points: &[LocalPoint], limit: u32) -> Option<Fit> {
  if points.len() < 4 {
    return None;
  }
  let mut best: Option<(u32, f64, f64, f64, f64)> = None;
  for radius in 2..=limit {
    let residuals = points
      .iter()
      .map(|point| {
        ((f64::from(point.u) - f64::from(radius)).hypot(f64::from(point.v) - f64::from(radius))
          - f64::from(radius))
        .abs()
      })
      .collect::<Vec<_>>();
    let mut bins = [false; ANGLE_BINS];
    let mut inliers = 0usize;
    for (point, residual) in points.iter().zip(&residuals) {
      if point.u <= radius + FIT_TOLERANCE as u32
        && point.v <= radius + FIT_TOLERANCE as u32
        && *residual <= FIT_TOLERANCE
      {
        inliers += 1;
        let angle =
          (f64::from(radius) - f64::from(point.v)).atan2(f64::from(radius) - f64::from(point.u));
        let bin = ((angle / (std::f64::consts::PI * 0.5)) * ANGLE_BINS as f64)
          .floor()
          .clamp(0.0, (ANGLE_BINS - 1) as f64) as usize;
        bins[bin] = true;
      }
    }
    let coverage = bins.into_iter().filter(|value| *value).count() as f64 / ANGLE_BINS as f64;
    let inlier_share = inliers as f64 / points.len() as f64;
    let residual = median(residuals);
    let score = fit_score(coverage, inlier_share, residual);
    if best.is_none_or(|(_, _, _, _, best_score)| score < best_score) {
      best = Some((radius, coverage, inlier_share, residual, score));
    }
  }
  let (radius, coverage, inlier_share, residual, score) = best?;
  if coverage < 0.25 || inlier_share < 0.4 {
    return None;
  }
  let radius = (radius + u32::from(radius < 10)).min(limit);
  Some(Fit {
    radius,
    score,
    low_confidence: !(residual <= 1.25
      && coverage >= if radius < 10 { 0.375 } else { 0.5 }
      && inlier_share >= 0.6),
  })
}

fn corner_origin(bounds: ComponentBox, corner: Corner) -> Point {
  Point {
    x: f64::from(if corner.right() {
      bounds.x + bounds.width
    } else {
      bounds.x
    }),
    y: f64::from(if corner.bottom() {
      bounds.y + bounds.height
    } else {
      bounds.y
    }),
  }
}

fn cursor_arc_distance(bounds: ComponentBox, corner: Corner, cursor: Point, radius: u32) -> f64 {
  let origin = corner_origin(bounds, corner);
  let u = if corner.right() {
    origin.x - cursor.x
  } else {
    cursor.x - origin.x
  };
  let v = if corner.bottom() {
    origin.y - cursor.y
  } else {
    cursor.y - origin.y
  };
  let radius = f64::from(radius);
  ((u - radius).hypot(v - radius) - radius).abs()
    + 0.0f64.max(-u).max(-v).max(u - radius).max(v - radius)
}

pub(crate) fn corner_radius_at(
  boxes: &[ComponentBox],
  cursor: Point,
  maps: &GradientMaps,
  threshold: u8,
  scale_x: f64,
  scale_y: f64,
) -> Option<RadiusEstimate> {
  let scale = (scale_x + scale_y) * 0.5;
  let corners = [
    Corner::TopLeft,
    Corner::TopRight,
    Corner::BottomLeft,
    Corner::BottomRight,
  ];
  let mut candidates = boxes
    .iter()
    .flat_map(|bounds| {
      corners.into_iter().map(move |corner| {
        let origin = corner_origin(*bounds, corner);
        let distance = ((origin.x - cursor.x) * scale_x).hypot((origin.y - cursor.y) * scale_y);
        (*bounds, corner, distance)
      })
    })
    .filter(|(_, _, distance)| *distance <= MAXIMUM_CURSOR_DISTANCE)
    .collect::<Vec<_>>();
  candidates.sort_by(|left, right| left.2.total_cmp(&right.2));
  candidates.truncate(MAXIMUM_CANDIDATES);
  let mut best: Option<(RadiusEstimate, f64)> = None;
  for (bounds, corner, _) in candidates {
    let limit = MAXIMUM_RADIUS.min(bounds.width.min(bounds.height) / 2);
    if limit < 2 {
      continue;
    }
    let Some(fit) = fit_circle(&curve_points(bounds, corner, maps, limit, threshold), limit) else {
      continue;
    };
    let arc_distance = cursor_arc_distance(bounds, corner, cursor, fit.radius) * scale;
    if arc_distance > MAXIMUM_ARC_DISTANCE {
      continue;
    }
    let score = arc_distance + fit.score * scale * 2.0 + if fit.low_confidence { 4.0 } else { 0.0 };
    let estimate = RadiusEstimate {
      bounds,
      corner,
      radius: fit.radius,
      low_confidence: fit.low_confidence,
    };
    if best.is_none_or(|(_, best_score)| score < best_score) {
      best = Some((estimate, score));
    }
  }
  best.map(|(estimate, _)| estimate)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn maps() -> GradientMaps {
    GradientMaps {
      gx: vec![0; 80 * 80],
      gy: vec![0; 80 * 80],
      width: 80,
      height: 80,
    }
  }

  fn paint_corner(
    maps: &mut GradientMaps,
    bounds: ComponentBox,
    corner: Corner,
    radius: u32,
    strength: u8,
  ) {
    for step in 0..=90 {
      let angle = f64::from(step) / 90.0 * std::f64::consts::FRAC_PI_2;
      let point = LocalPoint {
        u: (f64::from(radius) - f64::from(radius) * angle.cos()).round() as u32,
        v: (f64::from(radius) - f64::from(radius) * angle.sin()).round() as u32,
      };
      let (x, y) = pixel_at(bounds, corner, point);
      let index = (y * maps.width + x) as usize;
      maps.gx[index] = (f64::from(strength) * angle.cos()).round() as u8;
      maps.gy[index] = (f64::from(strength) * angle.sin()).round() as u8;
    }
    for v in radius..=bounds.height {
      let (x, y) = pixel_at(bounds, corner, LocalPoint { u: 0, v });
      maps.gx[(y * maps.width + x) as usize] = strength;
    }
    for u in radius..=bounds.width {
      let (x, y) = pixel_at(bounds, corner, LocalPoint { u, v: 0 });
      maps.gy[(y * maps.width + x) as usize] = strength;
    }
  }

  #[test]
  fn fits_all_corner_orientations() {
    let bounds = ComponentBox {
      x: 10,
      y: 10,
      width: 50,
      height: 40,
    };
    for corner in [
      Corner::TopLeft,
      Corner::TopRight,
      Corner::BottomLeft,
      Corner::BottomRight,
    ] {
      let mut field = maps();
      paint_corner(&mut field, bounds, corner, 8, 30);
      let origin = corner_origin(bounds, corner);
      let cursor = Point {
        x: origin.x + if corner.right() { -2.0 } else { 2.0 },
        y: origin.y + if corner.bottom() { -2.0 } else { 2.0 },
      };
      assert_eq!(
        corner_radius_at(&[bounds], cursor, &field, 24, 1.0, 1.0)
          .map(|value| (value.corner, value.radius)),
        Some((corner, 8))
      );
    }
  }

  #[test]
  fn sensitivity_and_cursor_distance_are_respected() {
    let bounds = ComponentBox {
      x: 10,
      y: 10,
      width: 50,
      height: 40,
    };
    let mut field = maps();
    paint_corner(&mut field, bounds, Corner::TopLeft, 8, 10);
    assert!(
      corner_radius_at(&[bounds], Point { x: 12.0, y: 12.0 }, &field, 24, 1.0, 1.0).is_none()
    );
    assert_eq!(
      corner_radius_at(&[bounds], Point { x: 12.0, y: 12.0 }, &field, 5, 1.0, 1.0)
        .map(|value| value.radius),
      Some(8)
    );
    assert!(corner_radius_at(&[bounds], Point { x: 70.0, y: 70.0 }, &field, 5, 1.0, 1.0).is_none());
  }
}
