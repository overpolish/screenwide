// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::counter::new_counter;
use crate::editor::annotations::model::new_arrow;
use crate::ruler::analysis::ComponentBox;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn bounds(x: u32, y: u32, width: u32, height: u32) -> ComponentBox {
  ComponentBox {
    x,
    y,
    width,
    height,
  }
}

/// A rectangle already in `SnapBounds` form, for testing the merge rule
/// without going through the detector.
fn rect(x: f64, y: f64, width: f64, height: f64) -> SnapBounds {
  SnapBounds {
    x,
    y,
    width,
    height,
  }
}

#[test]
fn touching_fragments_collapse_into_one_rectangle() {
  // Source 800 high: fragment limit 24, cluster limit 48, padding 4. Four
  // 3-px dashes on a 4-px pitch have padded rectangles that overlap, so they
  // fold into one rectangle the width of the glyph.
  let dashes = vec![
    rect(96.0, 196.0, 11.0, 11.0),
    rect(100.0, 196.0, 11.0, 11.0),
    rect(104.0, 196.0, 11.0, 11.0),
    rect(108.0, 196.0, 11.0, 11.0),
  ];
  let merged = merge_fragments(dashes, (1200, 800));
  assert_eq!(merged, vec![rect(96.0, 196.0, 23.0, 11.0)]);
}

#[test]
fn a_run_longer_than_the_cluster_cap_splits() {
  // Fifteen dashes would need a 67-px rectangle, over the 48-px cap, so the
  // greedy growth yields two clusters and never a rectangle wider than the
  // cap. This is the guarantee that keeps a row of icons from chaining into
  // one undifferentiated box.
  let dashes: Vec<SnapBounds> = (0..15)
    .map(|i| rect(96.0 + i as f64 * 4.0, 196.0, 11.0, 11.0))
    .collect();
  let merged = merge_fragments(dashes, (1200, 800));
  assert!(merged.len() > 1);
  for r in &merged {
    assert!(r.width <= 48.0, "cluster {r:?} wider than the cap");
  }
}

#[test]
fn an_element_above_fragment_size_passes_through_untouched() {
  // A 68-px padded rectangle (a button) is not a fragment, so a fragment that
  // overlaps it merges only with other fragments and the button keeps its
  // exact bounds rather than absorbing the dash.
  let button = rect(96.0, 196.0, 68.0, 68.0);
  let dash = rect(120.0, 220.0, 11.0, 11.0);
  let merged = merge_fragments(vec![button, dash], (1200, 800));
  assert!(merged.contains(&button));
  assert!(merged.contains(&dash));
  assert_eq!(merged.len(), 2);
}

#[test]
fn fragments_out_of_reach_stay_separate() {
  // Two dashes whose padded rectangles do not touch are two elements, not one.
  let merged = merge_fragments(
    vec![
      rect(96.0, 196.0, 11.0, 11.0),
      rect(120.0, 196.0, 11.0, 11.0),
    ],
    (1200, 800),
  );
  assert_eq!(merged.len(), 2);
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
  assert_eq!(snap_axis(132.0, &guides, 10.0), Some(canvas(140.0)));
  assert_eq!(snap_axis(107.0, &guides, 10.0), Some(canvas(100.0)));
}

#[test]
fn a_guide_outside_the_threshold_does_not_snap() {
  let guides = [canvas(100.0)];
  assert_eq!(snap_axis(110.0, &guides, 10.0), Some(canvas(100.0)));
  assert_eq!(snap_axis(110.001, &guides, 10.0), None);
  assert!(snap_axis(0.0, &[], 10.0).is_none());
}

#[test]
fn an_object_guide_beats_a_canvas_guide_at_the_same_distance() {
  // Either order of the candidates resolves the same way: the object wins on
  // the tie, and a nearer canvas line still beats a further object one.
  assert_eq!(
    snap_axis(100.0, &[canvas(96.0), object(104.0)], 8.0),
    Some(object(104.0))
  );
  assert_eq!(
    snap_axis(100.0, &[object(104.0), canvas(96.0)], 8.0),
    Some(object(104.0))
  );
  assert_eq!(
    snap_axis(100.0, &[canvas(98.0), object(104.0)], 8.0),
    Some(canvas(98.0))
  );
}

#[test]
fn a_box_is_grown_by_its_padding_before_it_is_ridden() {
  // Half a percent of the shorter side: 4 pixels on an 800-high source.
  assert_eq!(element_padding((1200, 800)), 4.0);
  assert_eq!(
    SnapBounds::padded(bounds(100, 200, 60, 40), 4.0),
    SnapBounds {
      x: 96.0,
      y: 196.0,
      width: 68.0,
      height: 48.0
    }
  );
}

#[test]
fn a_tip_pins_one_axis_to_an_edge_and_keeps_following_the_hand() {
  let rect = SnapBounds::padded(bounds(400, 300, 200, 100), 0.0);
  let bounds = [rect];
  // Beside the left edge, half way down: x is pinned, y is untouched, so the
  // tip can be aimed anywhere along that side.
  assert_eq!(
    snap_edges(point(406.0, 350.0), &bounds, 10.0),
    Some((point(400.0, 350.0), rect))
  );
  assert_eq!(
    snap_edges(point(406.0, 380.0), &bounds, 10.0),
    Some((point(400.0, 380.0), rect))
  );
  // Beside the bottom edge, half way across: y is pinned instead.
  assert_eq!(
    snap_edges(point(500.0, 396.0), &bounds, 10.0),
    Some((point(500.0, 400.0), rect))
  );
}

#[test]
fn a_tip_near_two_edges_lands_on_the_corner() {
  let rect = SnapBounds::padded(bounds(400, 300, 200, 100), 0.0);
  let bounds = [rect];
  assert_eq!(
    snap_edges(point(406.0, 306.0), &bounds, 10.0),
    Some((point(400.0, 300.0), rect))
  );
  // Diagonally outside the rectangle the corner is still in reach, which is
  // where a span limited to the edge itself would have left a dead spot.
  assert_eq!(
    snap_edges(point(394.0, 294.0), &bounds, 10.0),
    Some((point(400.0, 300.0), rect))
  );
}

#[test]
fn an_edge_out_of_reach_or_out_of_span_does_not_snap() {
  let bounds = [SnapBounds::padded(bounds(400, 300, 200, 100), 0.0)];
  // Far from every edge.
  assert!(snap_edges(point(500.0, 350.0), &bounds, 10.0).is_none());
  // In line with the left edge but well past the bottom of its span, so the
  // edge does not reach down the whole frame after the tip.
  assert!(snap_edges(point(400.0, 500.0), &bounds, 10.0).is_none());
  assert!(snap_edges(point(380.0, 350.0), &bounds, 10.0).is_none());
}

#[test]
fn the_nearest_edge_decides_which_element_the_tip_rides() {
  let near = SnapBounds::padded(bounds(400, 300, 200, 100), 0.0);
  let far = SnapBounds::padded(bounds(408, 300, 200, 100), 0.0);
  let bounds = [far, near];
  // Both left edges are in reach; the nearer one wins and travels with the
  // point so the chrome outlines the element actually being ridden.
  assert_eq!(
    snap_edges(point(402.0, 350.0), &bounds, 10.0),
    Some((point(400.0, 350.0), near))
  );
  assert_eq!(
    snap_edges(point(407.0, 350.0), &bounds, 10.0),
    Some((point(408.0, 350.0), far))
  );
}

#[test]
fn the_field_holds_the_canvas_lines_and_every_other_counter() {
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
  let field = SnapField::new((1000, 500), &annotations, "held");
  // Inset is 2% of the shorter side, so both axes are inset by the same length.
  assert_eq!(
    field.guides_x,
    vec![canvas(10.0), canvas(500.0), canvas(990.0), object(700.0)]
  );
  assert_eq!(
    field.guides_y,
    vec![canvas(10.0), canvas(250.0), canvas(490.0), object(200.0)]
  );
  // The exact lists above are the whole rule: the dragged counter is absent,
  // so it cannot snap back to where it began, and an arrow is never an axis
  // candidate.
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
