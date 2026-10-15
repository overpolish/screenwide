// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::prepare_focus;
use super::model::{default_focus_style, share};
use super::native::draw_points;

#[test]
fn a_focus_rounds_its_box_by_a_share_of_its_shorter_side() {
  let geometry = prepare_focus([200.0, 150.0], [100.0, 100.0], 20.0);
  // The box is 100 by 50, so a 20 percent radius is ten pixels.
  assert_eq!(geometry.rounding, 10.0);
  assert_eq!(geometry.width, 0.0);
}

#[test]
fn a_focus_reads_its_box_and_radius_through_the_native_slots() {
  let style = default_focus_style();
  let points = draw_points(
    crate::AnnotationPoint { x: 10.0, y: 20.0 },
    crate::AnnotationPoint { x: 110.0, y: 70.0 },
    &style,
  );
  assert_eq!(points[0], [10.0, 20.0]);
  assert_eq!(points[2], [110.0, 70.0]);
  // The corner radius rides in p1, the slot a spotlight keeps its softness.
  assert_eq!(points[1][0], style.radius as f32);
  assert_eq!(points[1][1], 0.0);
}

#[test]
fn a_fresh_focus_wears_the_middle_strength() {
  assert_eq!(default_focus_style().strength, 3.0);
}

#[test]
fn the_radius_share_is_held_between_nothing_and_half_the_side() {
  assert_eq!(share(0.0), 0.0);
  assert_eq!(share(50.0), 50.0);
  assert_eq!(share(80.0), 50.0);
  assert_eq!(share(f64::NAN), 0.0);
}
