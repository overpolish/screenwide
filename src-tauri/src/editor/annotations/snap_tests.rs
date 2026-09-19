// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::counter::new_counter;
use crate::editor::annotations::model::new_arrow;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn canvas(position: f64) -> AxisGuide {
  AxisGuide {
    position,
    object: false,
  }
}

fn object(position: f64) -> AxisGuide {
  AxisGuide {
    position,
    object: true,
  }
}

#[test]
fn the_nearest_guide_inside_the_threshold_wins() {
  let guides = [canvas(100.0), canvas(140.0), canvas(500.0)];
  assert_eq!(
    snap_axis(&[132.0], &guides, 10.0),
    Some((8.0, canvas(140.0)))
  );
  assert_eq!(
    snap_axis(&[107.0], &guides, 10.0),
    Some((-7.0, canvas(100.0)))
  );
}

#[test]
fn a_guide_outside_the_threshold_does_not_snap() {
  let guides = [canvas(100.0)];
  assert_eq!(
    snap_axis(&[110.0], &guides, 10.0),
    Some((-10.0, canvas(100.0)))
  );
  assert_eq!(snap_axis(&[110.001], &guides, 10.0), None);
  assert!(snap_axis(&[0.0], &[], 10.0).is_none());
  assert!(snap_axis(&[], &guides, 10.0).is_none());
}

#[test]
fn an_object_guide_beats_a_canvas_guide_at_the_same_distance() {
  // Either order of the candidates resolves the same way: the object wins on
  // the tie, and a nearer canvas line still beats a further object one.
  assert_eq!(
    snap_axis(&[100.0], &[canvas(96.0), object(104.0)], 8.0),
    Some((4.0, object(104.0)))
  );
  assert_eq!(
    snap_axis(&[100.0], &[object(104.0), canvas(96.0)], 8.0),
    Some((4.0, object(104.0)))
  );
  assert_eq!(
    snap_axis(&[100.0], &[canvas(98.0), object(104.0)], 8.0),
    Some((-2.0, canvas(98.0)))
  );
}

#[test]
fn an_edge_takes_an_object_guide_over_a_canvas_one_at_the_same_distance() {
  // A box offers two edges and a centre at once, so the tie can fall between
  // two different lines of the same shape. The object candidate still wins,
  // and the move it asks for is the one the whole box makes.
  let lines = [100.0, 130.0, 160.0];
  assert_eq!(
    snap_axis(&lines, &[canvas(96.0), object(134.0)], 8.0),
    Some((4.0, object(134.0)))
  );
  // A nearer canvas line beats the object one whichever line reaches it.
  assert_eq!(
    snap_axis(&lines, &[canvas(158.0), object(134.0)], 8.0),
    Some((-2.0, canvas(158.0)))
  );
}

#[test]
fn the_field_holds_the_canvas_lines_and_every_other_counters_disc() {
  let annotations = vec![
    new_counter("held".to_owned(), point(300.0, 300.0), 1, None, None),
    new_counter("other".to_owned(), point(700.0, 200.0), 2, None, None),
    new_arrow(
      "arrow".to_owned(),
      point(10.0, 10.0),
      point(20.0, 20.0),
      None,
    ),
  ];
  // Drawn 1000 output pixels across a 1000-pixel source, so the default
  // 56-pixel disc is 56 source pixels wide and its edges sit 28 out.
  let field = SnapField::new((1000, 500), &annotations, "held", 1000.0);
  // Inset is 2% of the shorter side, so both axes are inset by the same length.
  assert_eq!(
    field.guides_x,
    vec![
      canvas(10.0),
      canvas(500.0),
      canvas(990.0),
      object(672.0),
      object(700.0),
      object(728.0)
    ]
  );
  assert_eq!(
    field.guides_y,
    vec![
      canvas(10.0),
      canvas(250.0),
      canvas(490.0),
      object(172.0),
      object(200.0),
      object(228.0)
    ]
  );
  // The exact lists above are the whole rule: the dragged counter offers
  // neither a centre nor an edge, so it cannot pull itself back to where it
  // began, and an arrow is never an axis candidate.
  assert_eq!(
    field.boxes,
    vec![SnapBox {
      x: 672.0,
      y: 172.0,
      width: 56.0,
      height: 56.0
    }]
  );
}

#[test]
fn a_disc_is_a_point_when_the_pictures_drawn_width_is_unknown() {
  // Without a drawn width there is no way to turn output pixels into source
  // pixels, so a disc keeps only its centre rather than guessing a radius.
  let annotations = vec![new_counter(
    "other".to_owned(),
    point(700.0, 200.0),
    1,
    None,
    None,
  )];
  let field = SnapField::new((1000, 500), &annotations, "held", 0.0);
  assert_eq!(
    field.guides_x,
    vec![
      canvas(10.0),
      canvas(500.0),
      canvas(990.0),
      object(700.0),
      object(700.0),
      object(700.0)
    ]
  );
}

#[test]
fn the_threshold_converts_screen_points_into_source_pixels() {
  // A 1920-wide source drawn 960 points across: one point is two source pixels.
  assert_eq!(threshold_source_px(1920, 960.0), Some(16.0));
  assert_eq!(threshold_source_px(1920, 1920.0), Some(8.0));
  assert!(threshold_source_px(1920, 0.0).is_none());
  assert!(threshold_source_px(1920, f64::NAN).is_none());
  assert!(threshold_source_px(1920, -10.0).is_none());
}
