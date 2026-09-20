// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Draw-ready counter geometry, the twin of `annotation_prepare_counter` in
//! `geometry.h`.

use crate::editor::annotations::geometry::{add, scale, ArrowGeometry};
use crate::editor::annotations::reveal::AnnotationReveal;

/// One counter prepared for drawing, read out of the slots an arrow fills
/// with its curve: `a` is the disc's centre and `b` the tail's tip,
/// `rounding` is the disc's radius, `low` the radius the tip is rounded to
/// and `width` the disc's diameter. The twin of
/// `annotation_prepare_counter` in `geometry.h`.
///
/// The silhouette is the disc unioned with the tail: the overlap of two
/// circles, one either side of the axis, each tangent to the disc and to the
/// little circle the tip is rounded to, with their centres a radius behind the
/// disc's own. That is the outline of `MapPinPlusInside`.
pub(crate) fn prepare_counter(
  center: [f32; 2],
  diameter: f32,
  angle: f32,
  reveal: AnnotationReveal,
) -> ArrowGeometry {
  let radius = diameter.max(0.0) * 0.5 * reveal.scale.clamp(0.0, 1.0);
  let mut result = ArrowGeometry {
    a: center,
    b: center,
    c: center,
    width: radius * 2.0,
    low: 0.0,
    high: 1.0,
    rounding: radius,
    ..Default::default()
  };
  if radius <= 0.0 {
    return result;
  }
  result.low = radius * COUNTER_TIP_SHARE;
  let reach = radius * COUNTER_TAIL_REACH;
  result.b = add(center, scale([angle.cos(), angle.sin()], reach));
  result
}

/// How far the tail reaches from the disc's centre, in radii, and how round
/// its tip is as a share of the radius. The twins of `COUNTER_TAIL_REACH`
/// and `COUNTER_TIP_SHARE` in `counter.rs`.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) const COUNTER_TAIL_REACH: f32 = 1.5;
const COUNTER_TIP_SHARE: f32 = 0.125;

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn a_counter_reads_its_disc_and_rounded_tail_back() {
    let counter = prepare_counter([100.0, 100.0], 40.0, 0.0, AnnotationReveal::WHOLE);
    assert_eq!(counter.a, [100.0, 100.0]);
    assert_eq!(counter.rounding, 20.0);
    assert_eq!(counter.width, 40.0);
    // The tail's tip is the point itself, 1.5 radii east of the centre, and
    // `low` is the radius it is rounded to.
    assert!((counter.low - 20.0 * 0.125).abs() < 1e-3, "{counter:?}");
    assert!(
      (counter.b[0] - (100.0 + 20.0 * 1.5)).abs() < 1e-3,
      "{counter:?}"
    );
    assert!((counter.b[1] - 100.0).abs() < 1e-3);
  }

  #[test]
  fn a_counter_arriving_is_prepared_smaller() {
    let reveal = AnnotationReveal {
      scale: 0.5,
      ..AnnotationReveal::WHOLE
    };
    let arriving = prepare_counter([100.0, 100.0], 40.0, 0.0, reveal);
    assert_eq!(arriving.rounding, 10.0);
    // The tail and its tip shrink with the disc rather than staying out at
    // full reach.
    assert!((arriving.low - 10.0 * 0.125).abs() < 1e-3);
    assert!((arriving.b[0] - (100.0 + 10.0 * 1.5)).abs() < 1e-3);
  }
}
