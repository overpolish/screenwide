// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::{magnify_distance, prepare_magnify};
use super::model::{lens_from_loupe, nearest_zoom, new_magnify, NEW_MAGNIFY_ZOOM};
use super::reveal::magnify_reveal_window;
use crate::geometry::ArrowGeometry;
use crate::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::reveal::AnnotationReveal;
use crate::{Annotation, AnnotationPoint, AnnotationShape};

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// A round lens 200 across at (100, 100), enlarging three times.
fn round(reveal: AnnotationReveal) -> ArrowGeometry {
  prepare_magnify([0.0, 0.0], [200.0, 200.0], 3.0, 50.0, 8.0, reveal)
}

#[test]
fn a_whole_lens_is_its_box_at_its_zoom() {
  let geometry = round(AnnotationReveal::WHOLE);
  assert_eq!(geometry.a, [100.0, 100.0]);
  assert_eq!(geometry.b, [100.0, 100.0]);
  // Rounded all the way, it is a circle.
  assert_eq!(geometry.rounding, 100.0);
  assert_eq!(geometry.high, 3.0);
  assert_eq!(geometry.low, 1.0);
  assert_eq!(geometry.head, 0);
}

#[test]
fn a_lens_picks_anywhere_on_it_and_nowhere_past_it() {
  let geometry = round(AnnotationReveal::WHOLE);
  assert!(magnify_distance([100.0, 100.0], &geometry) < 0.0);
  assert!(magnify_distance([190.0, 100.0], &geometry) < 0.0);
  // The square's corner lies outside the circle.
  assert!(magnify_distance([195.0, 195.0], &geometry) > 0.0);
}

#[test]
fn an_arriving_lens_settles_from_smaller_and_unmagnified() {
  let setting_off = round(magnify_reveal_window(0.0, 4_000.0, 0.0));
  assert_eq!(setting_off.a, [100.0, 100.0]);
  assert!((setting_off.b[0] - 60.0).abs() < 1e-4);
  assert_eq!(setting_off.high, 1.0);
  assert_eq!(setting_off.low, 0.0);
  let under_way = round(magnify_reveal_window(150.0, 4_000.0, 0.0));
  assert!(under_way.b[0] > 60.0 && under_way.b[0] < 100.0);
  assert!(under_way.high > 1.0 && under_way.high < 3.0);
  let arrived = round(magnify_reveal_window(1_000.0, 4_000.0, 0.0));
  assert_eq!(arrived.b, [100.0, 100.0]);
  assert_eq!(arrived.high, 3.0);
  // It leaves the way it came.
  let gone = round(magnify_reveal_window(4_000.0, 4_000.0, 0.0));
  assert_eq!(gone.high, 1.0);
}

fn lens() -> Annotation {
  let mut annotation = new_magnify("m".to_owned(), point(100.0, 100.0), None, None);
  annotation.shape = AnnotationShape::Magnify {
    start: point(50.0, 50.0),
    end: point(150.0, 150.0),
    zoom: 2.0,
  };
  annotation
}

fn dragged(handle: AnnotationHandle, from: AnnotationPoint, to: AnnotationPoint) -> Annotation {
  let mut annotation = lens();
  let origin = AnnotationDragOrigin::new(from, &annotation.shape);
  annotation.drag_grip(handle, to, &origin, false, None);
  annotation
}

#[test]
fn a_lens_is_moved_and_resized_as_a_box_and_keeps_its_zoom() {
  let moved = dragged(
    AnnotationHandle::Body,
    point(100.0, 100.0),
    point(130.0, 90.0),
  );
  assert_eq!(
    moved.shape,
    AnnotationShape::Magnify {
      start: point(80.0, 40.0),
      end: point(180.0, 140.0),
      zoom: 2.0,
    }
  );
  let resized = dragged(
    AnnotationHandle::Edges(2 | 8),
    point(150.0, 150.0),
    point(250.0, 210.0),
  );
  assert_eq!(
    resized.shape,
    AnnotationShape::Magnify {
      start: point(50.0, 50.0),
      end: point(250.0, 210.0),
      zoom: 2.0,
    }
  );
}

#[test]
fn a_fresh_lens_is_pulled_out_as_a_box_at_the_zoom_chosen_last() {
  let mut annotation = new_magnify("m".to_owned(), point(100.0, 100.0), None, Some(3.0));
  let origin = AnnotationDragOrigin::new(point(100.0, 100.0), &annotation.shape);
  annotation.drag_new(point(140.0, 160.0), &origin, false, None);
  assert_eq!(
    annotation.shape,
    AnnotationShape::Magnify {
      start: point(100.0, 100.0),
      end: point(140.0, 160.0),
      zoom: 3.0,
    }
  );
  let fresh = new_magnify("m".to_owned(), point(0.0, 0.0), None, None);
  assert!(matches!(
    fresh.shape,
    AnnotationShape::Magnify { zoom, .. } if zoom == NEW_MAGNIFY_ZOOM
  ));
}

#[test]
fn a_zoom_snaps_to_the_nearest_look_as_a_ratio() {
  assert_eq!(nearest_zoom(1.25), 1.5);
  assert_eq!(nearest_zoom(1.8), 2.0);
  // 2.5 is a quarter more than two and a sixth less than three.
  assert_eq!(nearest_zoom(2.5), 3.0);
  assert_eq!(nearest_zoom(8.0), 4.0);
  assert_eq!(nearest_zoom(f64::NAN), NEW_MAGNIFY_ZOOM);
}

#[test]
fn an_old_magnifier_becomes_a_lens_over_its_zoom_area_as_big_as_its_loupe() {
  // A 100 by 50 zoom area at (150, 125), enlarged into a loupe 250 across.
  let (start, end, zoom) = lens_from_loupe(point(100.0, 100.0), point(200.0, 150.0), 250.0);
  assert_eq!(start, point(150.0 - 125.0, 125.0 - 62.5));
  assert_eq!(end, point(150.0 + 125.0, 125.0 + 62.5));
  // 2.5 times, nearest three.
  assert_eq!(zoom, 3.0);
}

#[test]
fn an_old_document_loads_its_magnifier_as_a_lens() {
  let stored = r##"{"id":"m","animated":false,
    "shape":{"kind":"magnify","start":{"x":100,"y":100},"end":{"x":200,"y":200},
      "loupe":{"x":600,"y":400},"size":200},
    "style":{"color":"#ffffff","width":8,"radius":25,"shadow":true}}"##;
  let annotation: Annotation = serde_json::from_str(stored).unwrap();
  assert_eq!(
    annotation.shape,
    AnnotationShape::Magnify {
      start: point(50.0, 50.0),
      end: point(250.0, 250.0),
      zoom: 2.0,
    }
  );
  // Its corners and its reveal timing are the annotation's own.
  assert_eq!(annotation.style.radius, 25.0);
  assert!(!annotation.animated);
  // Written back, it is a lens, and reads back the same.
  let json = serde_json::to_string(&annotation).unwrap();
  assert!(!json.contains("loupe"), "{json}");
  assert_eq!(
    serde_json::from_str::<Annotation>(&json).unwrap(),
    annotation
  );
}

/// The editor reaches a fresh lens's clip back by its arrival, and
/// TypeScript cannot read the constant, so it keeps its own copy. This is
/// what stops the two from drifting.
#[test]
fn the_editor_places_a_fresh_lens_by_its_arrival() {
  const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../src/features/editor/annotations/annotation-draw-in.ts"
  ));
  let declared = SOURCE
    .lines()
    .find_map(|line| {
      line
        .trim()
        .strip_prefix("export const ANNOTATION_MAGNIFY_DRAW_IN_MS = ")
    })
    .and_then(|value| value.trim_end_matches(';').parse::<f32>().ok());
  assert_eq!(declared, Some(super::reveal::MAGNIFY_REVEAL_IN_MS));
}

/// The editor offers the same zooms the model snaps to.
#[test]
fn the_editor_offers_the_zooms_the_model_knows() {
  const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../src/features/editor/annotations/annotation-shape-parsers.ts"
  ));
  let declared = SOURCE.lines().find_map(|line| {
    line
      .trim()
      .strip_prefix("export const LENS_ZOOMS = [")
      .and_then(|rest| rest.strip_suffix("] as const;"))
  });
  let zooms: Option<Vec<f64>> = declared.map(|list| {
    list
      .split(',')
      .filter_map(|value| value.trim().parse().ok())
      .collect()
  });
  assert_eq!(zooms, Some(super::model::MAGNIFY_ZOOMS.to_vec()));
}
