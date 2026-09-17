// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The counter mark: a numbered disc with a teardrop tail.
//!
//! The disc is what carries the number, so its diameter is the mark's
//! `style.width` - a counter has no stroke of its own to weigh. The tail is
//! the part that points: it is always there, and only its direction is
//! editable, so a counter can be aimed without being reshaped.
//!
//! The silhouette is the union of the disc and the triangle between the tail
//! tip and the two points where the tangents from that tip touch the disc.
//! Tangency is what makes the join seamless: the triangle's sides meet the
//! circle without crossing it, so no fillet is needed and the outline reads
//! as one teardrop rather than a disc with a spike glued on. Every drawing
//! and picking path - Metal, HLSL and this module - builds it that way.

use super::model::{Annotation, AnnotationPoint, AnnotationShape, AnnotationStyle};

/// The disc diameters the counter control offers, in output pixels. Three
/// sizes far enough apart to be worth choosing between: a handful of pixels
/// either way is no choice at all. The twin of `ANNOTATION_COUNTER_SIZES` in
/// `src/components/shared/annotation-style/widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const COUNTER_SIZES: [f64; 3] = [56.0, 96.0, 160.0];

/// The disc a fresh counter is drawn at, in output pixels: the smallest,
/// which is the size counters were drawn at before the sizes widened.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_COUNTER_WIDTH: f64 = COUNTER_SIZES[0];

/// How far the tail reaches from the disc's centre, in radii, and how round
/// its tip is as a share of the disc's own radius.
///
/// The reach past the edge is what makes it a tail rather than a bump, and
/// keeping both proportional means one direction handle serves every size.
/// A tip with a radius of its own is what keeps the teardrop soft: the
/// silhouette is the hull of two circles, so the sides meet both of them
/// along their tangents and nothing comes to a point.
///
/// Placing and picking a counter's grip is the D3D11 backend's work; the
/// Metal one does both through `geometry.h`, which carries its own twins.
#[cfg(any(target_os = "windows", test))]
pub(crate) const COUNTER_TAIL_REACH: f64 = 1.85;
#[cfg(any(target_os = "windows", test))]
pub(crate) const COUNTER_TIP_SHARE: f64 = 0.17;

/// A fresh counter points its tail right, so a row of them is dropped with
/// the same aim and only the ones that need turning are turned.
pub(crate) const NEW_COUNTER_ANGLE: f64 = 0.0;

/// The dress a fresh counter is drawn in before anything has been chosen:
/// the same first colour an arrow takes, at the middle disc.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_counter_style() -> AnnotationStyle {
  AnnotationStyle {
    color: super::model::NEW_MARK_COLOR.to_owned(),
    head: super::AnnotationHead::None,
    width: NEW_COUNTER_WIDTH,
  }
}

/// A counter numbered `value` at `center`, in `style` or in the tool's own
/// first dress where the editor has not settled on one yet, aimed at `angle`
/// or east where no counter has been turned yet.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn new_counter(
  id: String,
  center: AnnotationPoint,
  value: u32,
  style: Option<&AnnotationStyle>,
  angle: Option<f64>,
) -> Annotation {
  Annotation {
    above_camera: false,
    animated: true,
    id,
    reveal: super::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Counter {
      center,
      value,
      angle: angle
        .filter(|angle| angle.is_finite())
        .unwrap_or(NEW_COUNTER_ANGLE),
    },
    style: style.cloned().unwrap_or_else(default_counter_style),
  }
}

/// The number the next counter dropped on this list takes. The editor keeps
/// the numbers contiguous, so the count is the highest number there is.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn next_counter_value(annotations: &[Annotation]) -> u32 {
  annotations
    .iter()
    .filter(|annotation| matches!(annotation.shape, AnnotationShape::Counter { .. }))
    .count() as u32
    + 1
}

/// Where the tail's tip falls, in the space `center` and `radius` are in.
#[cfg(any(target_os = "windows", test))]
pub(crate) fn counter_tail_tip(
  center: AnnotationPoint,
  radius: f64,
  angle: f64,
) -> AnnotationPoint {
  let reach = radius * COUNTER_TAIL_REACH;
  AnnotationPoint {
    x: center.x + reach * angle.cos(),
    y: center.y + reach * angle.sin(),
  }
}

/// How far apart the angles Shift snaps to are: eight of them, the quarter
/// turns and the diagonals between. The twin of `ANNOTATION_COUNTER_SNAP` in
/// `src/features/editor/annotations.ts`, which the angle control steps by.
pub(crate) const COUNTER_SNAP_RADIANS: f64 = std::f64::consts::FRAC_PI_4;

/// The angle a tail tip dragged to `point` aims the counter at. A press
/// exactly on the centre keeps the aim it had: there is no direction in a
/// zero-length vector to take one from.
///
/// `snap` holds the aim to the nearest eighth of a turn - measured from east
/// rather than from wherever the drag began, so the same eight angles are
/// always the ones on offer however the tail was taken hold of.
pub(crate) fn counter_tail_angle(
  center: AnnotationPoint,
  point: AnnotationPoint,
  current: f64,
  snap: bool,
) -> f64 {
  let dx = point.x - center.x;
  let dy = point.y - center.y;
  if dx == 0.0 && dy == 0.0 {
    return current;
  }
  let angle = dy.atan2(dx);
  if !snap {
    return angle;
  }
  (angle / COUNTER_SNAP_RADIANS).round() * COUNTER_SNAP_RADIANS
}

/// How far `point` falls outside one counter's drawn silhouette, in the
/// space the geometry is in. Zero anywhere the mark is painted, which is
/// what picks it. The twin of `annotation_counter_distance` in `geometry.h`
/// and of the shaders' own copies.
///
/// The silhouette is the hull of two circles - the disc, and the small one
/// the tail ends in - which is one exact distance rather than a union of
/// two shapes. Measured along the tail, `x` across it: inside the disc's
/// cap the distance is the disc's own, inside the tip's cap the tip's, and
/// between them it is the distance to the tangent line that joins the two.
#[cfg(any(target_os = "windows", test))]
pub(crate) fn counter_distance(
  point: (f64, f64),
  center: AnnotationPoint,
  radius: f64,
  angle: f64,
) -> f64 {
  if radius <= 0.0 {
    return (point.0 - center.x).hypot(point.1 - center.y);
  }
  let tip_radius = radius * COUNTER_TIP_SHARE;
  let reach = radius * COUNTER_TAIL_REACH - tip_radius;
  let (axis_x, axis_y) = (angle.cos(), angle.sin());
  let (local_x, local_y) = (point.0 - center.x, point.1 - center.y);
  let along = local_x * axis_x + local_y * axis_y;
  let across = (local_x * -axis_y + local_y * axis_x).abs();
  let slope = (radius - tip_radius) / reach;
  let run = (1.0 - slope * slope).max(0.0).sqrt();
  let side = -slope * across + run * along;
  if side < 0.0 {
    return across.hypot(along) - radius;
  }
  if side > run * reach {
    return across.hypot(along - reach) - tip_radius;
  }
  across * run + along * slope - radius
}

#[cfg(test)]
mod tests {
  use super::*;

  fn center() -> AnnotationPoint {
    AnnotationPoint { x: 100.0, y: 100.0 }
  }

  #[test]
  fn the_disc_and_its_tail_are_both_picked() {
    let radius = 20.0;
    assert!(counter_distance((100.0, 100.0), center(), radius, 0.0) < 0.0);
    // Just past the edge on the tail's side is still the mark: the tail
    // reaches 1.85 radii out.
    assert!(counter_distance((125.0, 100.0), center(), radius, 0.0) < 0.0);
    // The same distance out on the other side is past the disc.
    assert!(counter_distance((75.0, 100.0), center(), radius, 0.0) > 0.0);
    // Turning the tail carries the picked region round with it.
    assert!(
      counter_distance(
        (100.0, 125.0),
        center(),
        radius,
        std::f64::consts::FRAC_PI_2
      ) < 0.0
    );
  }

  #[test]
  fn the_tail_ends_in_a_rounded_tip() {
    let radius = 20.0;
    let tip = counter_tail_tip(center(), radius, 0.0);
    assert_eq!((tip.x, tip.y), (100.0 + 20.0 * COUNTER_TAIL_REACH, 100.0));
    // The silhouette ends exactly at the tail's reach: the tip is the far
    // edge of a small circle rather than a point beyond one.
    assert!(counter_distance((tip.x, tip.y), center(), radius, 0.0).abs() < 1e-9);
    assert!(counter_distance((tip.x + 1.0, 100.0), center(), radius, 0.0) > 0.0);
    // Beside the tip the mark has already narrowed, and the narrowing is the
    // tangent between the two circles rather than a straight cut to a point.
    assert!(counter_distance((tip.x - 1.0, 110.0), center(), radius, 0.0) > 0.0);
    let tip_radius = radius * COUNTER_TIP_SHARE;
    assert!(
      counter_distance(
        (tip.x - tip_radius, 100.0 + tip_radius * 0.5),
        center(),
        radius,
        0.0
      ) < 0.0
    );
  }

  #[test]
  fn a_dragged_tail_takes_the_angle_of_the_drag() {
    let at = |x: f64, y: f64, snap: bool| {
      counter_tail_angle(center(), AnnotationPoint { x, y }, 0.0, snap)
    };
    assert!((at(100.0, 140.0, false) - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    // A drag onto the centre itself keeps the aim: there is no direction in
    // it to take one from.
    assert_eq!(counter_tail_angle(center(), center(), 1.25, false), 1.25);
  }

  #[test]
  fn a_snapped_tail_lands_on_the_absolute_eighths() {
    let at = |x: f64, y: f64| counter_tail_angle(center(), AnnotationPoint { x, y }, 0.0, true);
    // Just off east and just off the diagonals: each lands on the nearest
    // eighth of a turn, measured from east rather than from the drag.
    assert_eq!(at(180.0, 104.0), 0.0);
    assert!((at(150.0, 140.0) - COUNTER_SNAP_RADIANS).abs() < 1e-9);
    assert!((at(100.0, 150.0) - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    assert!((at(60.0, 145.0) - 3.0 * COUNTER_SNAP_RADIANS).abs() < 1e-9);
    // And above the disc it snaps to the turns on the other side.
    assert!((at(150.0, 60.0) + COUNTER_SNAP_RADIANS).abs() < 1e-9);
  }

  #[test]
  fn numbering_follows_the_counters_already_there() {
    let counter = |value: u32| {
      new_counter(
        format!("c{value}"),
        AnnotationPoint { x: 0.0, y: 0.0 },
        value,
        None,
        None,
      )
    };
    assert_eq!(next_counter_value(&[]), 1);
    assert_eq!(next_counter_value(&[counter(1), counter(2)]), 3);
  }
}
