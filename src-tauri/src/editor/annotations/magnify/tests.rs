// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::{magnify_distance, magnify_part, prepare_magnify, MagnifyPart};
use super::gesture::{beside, drag, drag_new};
use super::model::{new_magnify, MAX_MAGNIFY_ZOOM, MIN_MAGNIFY_ZOOM};
use super::reveal::magnify_reveal_window;
use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// A round zoom area 100 across at (100, 100), enlarged twice over into a
/// loupe centred at (500, 300).
fn round(reveal: AnnotationReveal) -> ArrowGeometry {
  prepare_magnify(
    [50.0, 50.0],
    [500.0, 300.0],
    [150.0, 150.0],
    200.0,
    50.0,
    8.0,
    reveal,
  )
}

#[test]
fn a_whole_loupe_is_the_zoom_area_scaled_to_its_size() {
  let geometry = round(AnnotationReveal::WHOLE);
  assert_eq!(geometry.a, [500.0, 300.0]);
  assert_eq!(geometry.b, [100.0, 100.0]);
  assert_eq!(geometry.c, [100.0, 100.0]);
  assert_eq!(geometry.start_head.c, [50.0, 50.0]);
  // Rounded all the way, both are circles.
  assert_eq!(geometry.rounding, 100.0);
  assert_eq!(geometry.high, 50.0);
}

#[test]
fn one_line_runs_from_the_zoom_areas_edge_to_the_loupes() {
  let geometry = round(AnnotationReveal::WHOLE);
  assert_eq!(geometry.head, 1);
  let (from, to) = (geometry.start_head.a, geometry.start_head.b);
  let on = |at: [f32; 2], centre: [f32; 2], radius: f32| {
    ((at[0] - centre[0]).hypot(at[1] - centre[1]) - radius).abs() < 0.05
  };
  assert!(on(from, [100.0, 100.0], 50.0), "{from:?}");
  assert!(on(to, [500.0, 300.0], 100.0), "{to:?}");
  // Along the line between the two centres.
  let across = (to[0] - from[0]) * (300.0 - 100.0) - (to[1] - from[1]) * (500.0 - 100.0);
  assert!(across.abs() < 1.0);
}

#[test]
fn a_zoom_area_under_its_loupe_has_no_line() {
  let geometry = prepare_magnify(
    [80.0, 80.0],
    [100.0, 100.0],
    [120.0, 120.0],
    200.0,
    50.0,
    8.0,
    AnnotationReveal::WHOLE,
  );
  assert_eq!(geometry.head, 0);
}

#[test]
fn the_loupes_grip_sits_at_the_middle_of_its_right_side() {
  let geometry = round(AnnotationReveal::WHOLE);
  assert_eq!(geometry.end_head.c, [600.0, 300.0]);
}

#[test]
fn an_arriving_loupe_sets_off_as_its_zoom_area_and_grows_on_its_way() {
  let setting_off = round(magnify_reveal_window(0.0, 4_000.0, 0.0));
  assert_eq!(setting_off.a, setting_off.c);
  assert_eq!(setting_off.b, setting_off.start_head.c);
  assert_eq!(setting_off.low, 0.0);
  assert_eq!(setting_off.head, 0);
  let under_way = round(magnify_reveal_window(150.0, 4_000.0, 0.0));
  assert!(under_way.b[0] > 50.0 && under_way.b[0] < 100.0);
  assert!(under_way.a[0] > 100.0 && under_way.a[0] < 500.0);
  let arrived = round(magnify_reveal_window(1_000.0, 4_000.0, 0.0));
  assert_eq!(arrived.a, [500.0, 300.0]);
  assert_eq!(arrived.low, 1.0);
  // It leaves back into the zoom area.
  let gone = round(magnify_reveal_window(4_000.0, 4_000.0, 0.0));
  assert_eq!(gone.a, gone.c);
}

#[test]
fn the_loupe_picks_before_the_zoom_area_and_the_line_picks_nothing() {
  let geometry = round(AnnotationReveal::WHOLE);
  assert_eq!(
    magnify_part([500.0, 300.0], &geometry),
    Some(MagnifyPart::Loupe)
  );
  assert_eq!(
    magnify_part([110.0, 95.0], &geometry),
    Some(MagnifyPart::Area)
  );
  // Halfway along the line, between the two, is empty picture.
  assert_eq!(magnify_part([300.0, 200.0], &geometry), None);
  assert!(magnify_distance([300.0, 200.0], &geometry) > 0.0);
}

fn magnifier() -> Annotation {
  let mut annotation = new_magnify("m".to_owned(), point(100.0, 100.0), None);
  annotation.style.radius = 0.0;
  annotation.shape = AnnotationShape::Magnify {
    start: point(50.0, 50.0),
    end: point(150.0, 150.0),
    loupe: point(500.0, 300.0),
    size: 200.0,
  };
  annotation
}

fn dragged(handle: AnnotationHandle, from: AnnotationPoint, to: AnnotationPoint) -> Annotation {
  let mut annotation = magnifier();
  let origin = AnnotationDragOrigin::new(from, &annotation.shape);
  drag(&mut annotation, handle, to, &origin, false, None);
  annotation
}

#[test]
fn the_loupes_inside_carries_the_loupe_and_leaves_the_zoom_area() {
  let moved = dragged(
    AnnotationHandle::Middle,
    point(500.0, 300.0),
    point(530.0, 290.0),
  );
  assert_eq!(
    moved.shape,
    AnnotationShape::Magnify {
      start: point(50.0, 50.0),
      end: point(150.0, 150.0),
      loupe: point(530.0, 290.0),
      size: 200.0,
    }
  );
}

#[test]
fn resizing_the_zoom_area_keeps_the_loupe_its_size() {
  let resized = dragged(
    AnnotationHandle::Edges(2 | 8),
    point(150.0, 150.0),
    point(110.0, 110.0),
  );
  assert_eq!(
    resized.shape,
    AnnotationShape::Magnify {
      start: point(50.0, 50.0),
      end: point(110.0, 110.0),
      loupe: point(500.0, 300.0),
      size: 200.0,
    }
  );
  // A zoom area grown past the loupe takes the loupe up to the least zoom.
  let grown = dragged(
    AnnotationHandle::Edges(2 | 8),
    point(150.0, 150.0),
    point(250.0, 250.0),
  );
  let AnnotationShape::Magnify { size, .. } = grown.shape else {
    unreachable!()
  };
  assert_eq!(size, 200.0 * MIN_MAGNIFY_ZOOM);
}

#[test]
fn the_loupes_grip_sizes_the_loupe_between_the_zooms_it_allows() {
  // The grip is the middle of the loupe's right side: dragged to 150 right
  // of its centre, whatever the height, the loupe is 300 across.
  let sized = dragged(
    AnnotationHandle::Tail,
    point(600.0, 300.0),
    point(650.0, 340.0),
  );
  let AnnotationShape::Magnify { size, .. } = sized.shape else {
    unreachable!()
  };
  assert!((size - 300.0).abs() < 1e-9, "{size}");
  let tiny = dragged(
    AnnotationHandle::Tail,
    point(600.0, 400.0),
    point(501.0, 301.0),
  );
  let huge = dragged(
    AnnotationHandle::Tail,
    point(600.0, 400.0),
    point(9_000.0, 9_000.0),
  );
  let size = |annotation: &Annotation| match annotation.shape {
    AnnotationShape::Magnify { size, .. } => size,
    _ => unreachable!(),
  };
  assert_eq!(size(&tiny), 100.0 * MIN_MAGNIFY_ZOOM);
  assert_eq!(size(&huge), 100.0 * MAX_MAGNIFY_ZOOM);
}

#[test]
fn a_fresh_zoom_area_is_pulled_out_and_its_loupe_set_beside_it() {
  let mut annotation = new_magnify("m".to_owned(), point(100.0, 100.0), None);
  let mut origin = AnnotationDragOrigin::new(point(100.0, 100.0), &annotation.shape);
  origin.source_size = (1_000, 1_000);
  drag_new(&mut annotation, point(140.0, 160.0), &origin, false, None);
  assert_eq!(
    annotation.shape,
    AnnotationShape::Magnify {
      start: point(100.0, 100.0),
      end: point(140.0, 160.0),
      loupe: point(120.0 + 20.0 * 3.5, 130.0),
      size: 120.0,
    }
  );
}

#[test]
fn a_loupe_with_no_room_to_the_right_goes_to_the_left() {
  // Beside a zoom area against the picture's right edge, the loupe would
  // spill out on the right, so it goes where it fits.
  let placed = beside(
    point(900.0, 400.0),
    point(980.0, 480.0),
    2.0,
    (1_000, 1_000),
  );
  assert_eq!(placed, point(940.0 - 40.0 * 3.5, 440.0));
  // A picture too small for any side takes the side that spills least.
  let cramped = beside(point(0.0, 0.0), point(80.0, 80.0), 2.0, (100, 100));
  assert!(cramped.x.is_finite() && cramped.y.is_finite());
}

/// The editor reaches a fresh magnifier's clip back by its arrival, and
/// TypeScript cannot read the constant, so it keeps its own copy. This is
/// what stops the two from drifting.
#[test]
fn the_editor_places_a_fresh_magnifier_by_its_arrival() {
  const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../src/features/editor/annotation-kinds.ts"
  ));
  let declared = SOURCE
    .lines()
    .find_map(|line| {
      line
        .trim()
        .strip_prefix("const ANNOTATION_MAGNIFY_DRAW_IN_MS = ")
    })
    .and_then(|value| value.trim_end_matches(';').parse::<f32>().ok());
  assert_eq!(declared, Some(super::reveal::MAGNIFY_REVEAL_IN_MS));
}
