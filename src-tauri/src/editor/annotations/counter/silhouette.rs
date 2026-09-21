// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shape a counter is drawn and picked as: where its tail ends, and how
//! far a point falls outside the annotation.
//!
//! The silhouette is the disc unioned with the tail: the overlap of two
//! circles, one either side of the axis, each tangent to the disc and to the
//! little circle the tip is rounded to. Tangency at both ends is what makes
//! the outline one curve - it leaves the disc without a seam and arrives at
//! the point without a corner. Their centres sit a radius behind the disc's
//! own, which is what holds the sides out: set level with it they bow out into
//! a blunt teardrop, and straight sides would pinch in off the disc instead.
//! At a radius back the outline is `MapPinPlusInside`, the glyph the counter
//! tool is drawn with, to within a hundredth of the radius. Every drawing and
//! picking path - Metal, HLSL and this module - builds it that way.

use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::AnnotationPoint;

/// How far the tail reaches from the disc's centre, in radii, and how round
/// its tip is as a share of the disc's own radius.
///
/// The reach past the edge is what makes it a tail rather than a bump, and
/// keeping both proportional means one direction handle serves every size.
/// The two together are the shape of `MapPinPlusInside`, the glyph the counter
/// tool is drawn with, so the annotation and its icon read as one thing.
///
/// Placing and picking a counter's grip from its aim is the D3D11 backend's
/// work; the Metal one and the macOS chrome pick from the prepared record.
pub(crate) const COUNTER_TAIL_REACH: f64 = 1.5;
#[cfg(any(target_os = "windows", test))]
const COUNTER_TIP_SHARE: f64 = 0.125;

/// Where the tail's tip falls, in the space `center` and `radius` are in.
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

/// How far `point` falls outside one counter's drawn silhouette, in the space
/// the geometry is in. Zero anywhere the annotation is painted, which is what
/// picks it.
#[cfg(any(target_os = "windows", test))]
pub(crate) fn counter_distance(
  point: (f64, f64),
  center: AnnotationPoint,
  radius: f64,
  angle: f64,
) -> f64 {
  let local = (point.0 - center.x, point.1 - center.y);
  if radius <= 0.0 {
    return local.0.hypot(local.1);
  }
  silhouette_distance(
    local,
    (angle.cos(), angle.sin()),
    radius,
    radius * COUNTER_TIP_SHARE,
    radius * COUNTER_TAIL_REACH,
  )
}

/// The same measurement from a prepared record rather than from a centre and
/// an aim: the disc's centre is `a`, the tail's tip `b`, the radius
/// `rounding` and the radius the tip is rounded to `low`. This is what the
/// macOS chrome picks with, over the record the compositor draws.
///
/// The record is single precision and the picking is not, so each difference
/// is taken in the record's own precision before it is widened: the answer is
/// then the one the whole native side has always produced.
pub(crate) fn prepared_counter_distance(point: (f64, f64), prepared: &ArrowGeometry) -> f64 {
  let radius = f64::from(prepared.rounding);
  let local = (
    point.0 - f64::from(prepared.a[0]),
    point.1 - f64::from(prepared.a[1]),
  );
  if radius <= 0.0 {
    return local.0.hypot(local.1);
  }
  let tail = (
    f64::from(prepared.b[0] - prepared.a[0]),
    f64::from(prepared.b[1] - prepared.a[1]),
  );
  let reach = tail.0.hypot(tail.1);
  if reach <= 0.0 {
    return local.0.hypot(local.1) - radius;
  }
  silhouette_distance(
    local,
    (tail.0 / reach, tail.1 / reach),
    radius,
    f64::from(prepared.low),
    reach,
  )
}

/// How far a point offset from the disc's centre falls outside the
/// silhouette, measured along the tail's `axis` and across it.
fn silhouette_distance(
  local: (f64, f64),
  axis: (f64, f64),
  radius: f64,
  tip_radius: f64,
  reach: f64,
) -> f64 {
  let along = local.0 * axis.0 + local.1 * axis.1;
  let across = (local.0 * -axis.1 + local.1 * axis.0).abs();
  counter_silhouette_distance(across, along, radius, tip_radius, reach)
}

/// How far a point falls outside the silhouette, measured `along` the tail and
/// `across` it from the disc's centre, with `across` already folded to the
/// near side. The shaders carry the only other copies.
///
/// The side circles are solved rather than chosen: a radius behind the disc's
/// centre, tangent to the disc and tangent to the tip's own circle, leaves one
/// radius and one distance across the axis to find. Their overlap is measured
/// the way any lens is - to the near circle inside its span, to the shared
/// corner past it - and that corner is the tip's centre, so taking the tip's
/// radius off the whole thing rounds the point without moving it.
///
/// The overlap runs back behind the disc as well, and is wider than the disc
/// where it does, so it is cut at the plane where the circles touch the disc.
/// That cut is a chord of the disc, inside the union, and so never shows.
fn counter_silhouette_distance(
  across: f64,
  along: f64,
  radius: f64,
  tip_radius: f64,
  reach: f64,
) -> f64 {
  let disc = across.hypot(along) - radius;
  let gap = radius - tip_radius;
  if gap <= 0.0 {
    return disc;
  }
  // Where the tip's circle sits, and the side circles that reach it.
  let tip = reach - tip_radius;
  let side = (tip * tip + 2.0 * tip * radius + gap * gap) / (2.0 * gap);
  let apart = side + tip_radius - radius;
  let offset = apart * apart - radius * radius;
  if offset <= 0.0 {
    return disc;
  }
  let offset = offset.sqrt();
  let lens = if (along - tip) * offset > across * (tip + radius) {
    across.hypot(along - tip)
  } else {
    (across + offset).hypot(along + radius) - side
  };
  let touch = radius * radius / apart;
  disc.min((lens - tip_radius).max(touch - along))
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
    // Just past the edge on the tail's side is still the annotation: the tail
    // reaches 1.5 radii out.
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
  fn the_tail_curves_off_the_disc_to_a_rounded_point() {
    let radius = 20.0;
    let tip = counter_tail_tip(center(), radius, 0.0);
    assert_eq!((tip.x, tip.y), (100.0 + 20.0 * COUNTER_TAIL_REACH, 100.0));
    // The silhouette ends exactly at the tail's reach, and ends in an arc: a
    // tip radius back and half of one across is still inside the annotation.
    assert!(counter_distance((tip.x, tip.y), center(), radius, 0.0).abs() < 1e-9);
    assert!(counter_distance((tip.x + 1.0, 100.0), center(), radius, 0.0) > 0.0);
    let tip_radius = radius * COUNTER_TIP_SHARE;
    assert!(
      counter_distance(
        (tip.x - tip_radius, 100.0 + tip_radius * 0.5),
        center(),
        radius,
        0.0
      ) < 0.0
    );
    // Level with the centre the annotation is exactly as wide as the disc, and
    // no wider anywhere: the disc is the widest the silhouette ever gets.
    assert!(counter_distance((100.0, 120.0), center(), radius, 0.0).abs() < 1e-9);
    assert!(counter_distance((100.0, 121.0), center(), radius, 0.0) > 0.0);
    assert!(counter_distance((110.0, 120.4), center(), radius, 0.0) > 0.0);
    // Three quarters of a radius along the tail the sides still stand outside
    // the disc's own circle, which reaches 13.2 across there: they hold out
    // and then turn, rather than tapering straight at the tip.
    assert!(counter_distance((115.0, 114.0), center(), radius, 0.0) < 0.0);
    assert!((115.0f64 - 100.0).hypot(114.0 - 100.0) > radius);
    // And they are held out, not bowed out: sides that left the disc at its
    // widest point would still be 16.4 across here.
    assert!(counter_distance((115.0, 115.5), center(), radius, 0.0) > 0.0);
  }
}
