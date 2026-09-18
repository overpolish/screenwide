// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn a_straight_arrow_keeps_its_tip_and_trims_its_shaft() {
  let arrow = prepare_arrow(
    [0.0, 0.0],
    [100.0, 0.0],
    [200.0, 0.0],
    8.0,
    1,
    AnnotationReveal::WHOLE,
  );
  assert_eq!(arrow.head, 1);
  assert_eq!(arrow.width, 8.0);
  assert_eq!(arrow.low, 0.0);
  // The shaft stops one head length short of the tip.
  let join = bezier(arrow.a, arrow.b, arrow.c, arrow.high);
  assert!((join[0] - 168.0).abs() < 0.5, "{join:?}");
  // The visible tip stays on the end point: the inner apex sits behind it
  // by exactly the rounding.
  assert!((arrow.end_head.a[0] + arrow.rounding - 200.0).abs() < 0.05);
  assert_eq!(
    head_distance([199.0, 0.0], arrow.end_head, arrow.rounding),
    0.0
  );
  assert!(head_distance([230.0, 0.0], arrow.end_head, arrow.rounding) > 20.0);
}

#[test]
fn a_short_arrow_scales_down_and_coincident_handles_draw_a_dot() {
  let short = prepare_arrow(
    [0.0, 0.0],
    [5.0, 0.0],
    [10.0, 0.0],
    8.0,
    1,
    AnnotationReveal::WHOLE,
  );
  assert!(short.width < 8.0);
  let dot = prepare_arrow(
    [3.0, 3.0],
    [3.0, 3.0],
    [3.0, 3.0],
    8.0,
    2,
    AnnotationReveal::WHOLE,
  );
  assert_eq!(dot.head, 0);
  assert_eq!(dot.width, 8.0);
}

#[test]
fn shaft_distance_follows_the_bend() {
  let distance = shaft_distance([100.0, 50.0], [0.0, 0.0], [100.0, 100.0], [200.0, 0.0]);
  assert!(distance < 1.0, "{distance}");
  assert!(shaft_distance([100.0, 0.0], [0.0, 0.0], [100.0, 100.0], [200.0, 0.0]) > 40.0);
}

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
