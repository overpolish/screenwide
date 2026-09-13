// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;

pub(in crate::ruler::snapshot) fn snap_guide(
  snapshot: &DisplaySnapshot,
  axis: GuideAxis,
  pointer: RulerPointer,
  tolerance: Tolerance,
) -> Option<f64> {
  let display = snapshot.display;
  let (axis_pixels, across_pixels, local_axis, local_across, gradients) = match axis {
    GuideAxis::Vertical => (
      snapshot.image.width,
      snapshot.image.height,
      pointer.world.x - display.origin.x,
      pointer.world.y - display.origin.y,
      &snapshot.gradients.gx,
    ),
    GuideAxis::Horizontal => (
      snapshot.image.height,
      snapshot.image.width,
      pointer.world.y - display.origin.y,
      pointer.world.x - display.origin.x,
      &snapshot.gradients.gy,
    ),
  };
  if axis_pixels < 2 || across_pixels == 0 {
    return None;
  }
  let axis_world = match axis {
    GuideAxis::Vertical => display.size.width,
    GuideAxis::Horizontal => display.size.height,
  };
  let across_world = match axis {
    GuideAxis::Vertical => display.size.height,
    GuideAxis::Horizontal => display.size.width,
  };
  if axis_world <= 0.0 || across_world <= 0.0 {
    return None;
  }
  let pixels_per_world = f64::from(axis_pixels) / axis_world;
  let raw_pixel =
    (local_axis * pixels_per_world).clamp(1.0, f64::from(axis_pixels.saturating_sub(1)));
  let across_pixel = ((local_across / across_world) * f64::from(across_pixels))
    .floor()
    .clamp(0.0, f64::from(across_pixels.saturating_sub(1))) as u32;
  let radius = ((GUIDE_SNAP_RADIUS / pointer.zoom.max(1.0)) * pixels_per_world)
    .ceil()
    .max(1.0) as u32;
  let center = raw_pixel.round() as u32;
  let start = center.saturating_sub(radius).max(1);
  let end = center
    .saturating_add(radius)
    .min(axis_pixels.saturating_sub(1));
  let mut best: Option<(u32, f64, u32)> = None;
  for position in start..=end {
    let score = (-2..=2)
      .filter_map(|offset| across_pixel.checked_add_signed(offset))
      .filter(|across| *across < across_pixels)
      .map(|across| {
        let index = match axis {
          GuideAxis::Vertical => across * snapshot.image.width + position,
          GuideAxis::Horizontal => position * snapshot.image.width + across,
        };
        u32::from(gradients[index as usize])
      })
      .sum::<u32>();
    if score < u32::from(tolerance.threshold()) * 2 {
      continue;
    }
    let distance = (f64::from(position) - raw_pixel).abs();
    if best.is_none_or(|(best_score, best_distance, _)| {
      score > best_score || (score == best_score && distance < best_distance)
    }) {
      best = Some((score, distance, position));
    }
  }
  let position = best?.2;
  let origin = match axis {
    GuideAxis::Vertical => display.origin.x,
    GuideAxis::Horizontal => display.origin.y,
  };
  Some(origin + f64::from(position) / pixels_per_world)
}
