// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::Rect;
use super::luma::Plane;

/// The weakest corner kept, as the smaller eigenvalue of the gradients'
/// structure over a 3 by 3 window, in squared brightness steps. Below it a
/// point sits on flat colour or on compression noise and has nothing to lock
/// onto.
const MIN_STRENGTH: f32 = 60.0;
/// A corner must also be this share of the region's strongest, so a region
/// full of sharp text does not spend its points on faint texture.
const QUALITY: f32 = 0.02;
/// How far from the plane's edge a corner must be for its flow window to
/// start inside the frame.
const EDGE: f32 = 8.0;

/// Up to `max` corners in `region`, strongest first and at least
/// `min_distance` from each other and from every point in `existing`.
///
/// With `centre_weighted`, a corner's strength is discounted by how far it
/// sits from the region's centre. A redaction is drawn with room to spare
/// around what it hides, and on a still background that margin would
/// otherwise outvote the moving thing inside it.
pub(crate) fn corners(
  plane: &Plane,
  region: &Rect,
  max: usize,
  min_distance: f32,
  centre_weighted: bool,
  existing: &[[f32; 2]],
) -> Vec<[f32; 2]> {
  let bounds = Rect {
    x0: EDGE,
    y0: EDGE,
    x1: plane.width as f32 - EDGE,
    y1: plane.height as f32 - EDGE,
  };
  let area = region.intersect(&bounds);
  if area.width() < 3.0 || area.height() < 3.0 || max == 0 {
    return Vec::new();
  }
  let (x0, y0) = (area.x0.floor() as usize, area.y0.floor() as usize);
  let (x1, y1) = (area.x1.ceil() as usize, area.y1.ceil() as usize);
  let (width, height) = (x1 - x0, y1 - y0);

  // Central-difference gradients over the area and a one pixel border, so the
  // structure window of every pixel inside has its neighbours.
  let (gw, gh) = (width + 2, height + 2);
  let mut gxx = vec![0.0_f32; gw * gh];
  let mut gyy = vec![0.0_f32; gw * gh];
  let mut gxy = vec![0.0_f32; gw * gh];
  for gy in 0..gh {
    let y = (y0 + gy) as isize - 1;
    for gx in 0..gw {
      let x = (x0 + gx) as isize - 1;
      let dx = 0.5 * (plane.at_clamped(x + 1, y) - plane.at_clamped(x - 1, y));
      let dy = 0.5 * (plane.at_clamped(x, y + 1) - plane.at_clamped(x, y - 1));
      let index = gy * gw + gx;
      gxx[index] = dx * dx;
      gyy[index] = dy * dy;
      gxy[index] = dx * dy;
    }
  }

  let mut strength = vec![0.0_f32; width * height];
  let mut strongest = 0.0_f32;
  for y in 0..height {
    for x in 0..width {
      let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
      for wy in 0..3 {
        let row = (y + wy) * gw + x;
        for wx in 0..3 {
          sxx += gxx[row + wx];
          syy += gyy[row + wx];
          sxy += gxy[row + wx];
        }
      }
      let half_trace = 0.5 * (sxx + syy);
      let spread = (0.25 * (sxx - syy) * (sxx - syy) + sxy * sxy).sqrt();
      let value = half_trace - spread;
      strength[y * width + x] = value;
      strongest = strongest.max(value);
    }
  }
  let floor = MIN_STRENGTH.max(QUALITY * strongest);

  let [cx, cy] = region.centre();
  let (half_width, half_height) = (
    0.5 * region.width().max(1.0),
    0.5 * region.height().max(1.0),
  );
  let mut candidates = Vec::new();
  for y in 1..height.saturating_sub(1) {
    for x in 1..width.saturating_sub(1) {
      let value = strength[y * width + x];
      if value < floor {
        continue;
      }
      // Only local maxima, so a corner is one point and not its blur.
      let is_peak = (0..3).all(|wy| {
        (0..3)
          .all(|wx| (wy == 1 && wx == 1) || strength[(y + wy - 1) * width + x + wx - 1] <= value)
      });
      if !is_peak {
        continue;
      }
      let point = [(x0 + x) as f32, (y0 + y) as f32];
      let score = if centre_weighted {
        let off = ((point[0] - cx) / half_width)
          .abs()
          .max(((point[1] - cy) / half_height).abs());
        value * (1.0 - 0.75 * off.min(1.0).powi(2))
      } else {
        value
      };
      candidates.push((score, point));
    }
  }
  candidates.sort_by(|a, b| b.0.total_cmp(&a.0));

  let min_squared = min_distance * min_distance;
  let far_enough = |point: [f32; 2], others: &[[f32; 2]]| {
    others
      .iter()
      .all(|other| (other[0] - point[0]).powi(2) + (other[1] - point[1]).powi(2) >= min_squared)
  };
  let mut chosen: Vec<[f32; 2]> = Vec::with_capacity(max);
  for (_, point) in candidates {
    if chosen.len() == max {
      break;
    }
    if far_enough(point, &chosen) && far_enough(point, existing) {
      chosen.push(point);
    }
  }
  chosen
}
