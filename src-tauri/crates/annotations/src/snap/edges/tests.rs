// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn detected(x: u32, y: u32, width: u32, height: u32) -> ComponentBox {
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

/// Where a tip riding an element's edges ends up, with the axes it left free
/// still on the hand: the reading the arrow path takes.
fn landed(
  tip: AnnotationPoint,
  bounds: &[SnapBounds],
  threshold: f64,
) -> Option<(AnnotationPoint, SnapBounds)> {
  snap_edges(tip, bounds, threshold).map(|edge| {
    (
      AnnotationPoint {
        x: edge.x.unwrap_or(tip.x),
        y: edge.y.unwrap_or(tip.y),
      },
      edge.bounds,
    )
  })
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

#[test]
fn a_box_is_grown_by_its_padding_before_it_is_ridden() {
  // Half a percent of the shorter side: 4 pixels on an 800-high source.
  assert_eq!(element_padding((1200, 800)), 4.0);
  assert_eq!(
    SnapBounds::padded(detected(100, 200, 60, 40), 4.0),
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
  let rect = SnapBounds::padded(detected(400, 300, 200, 100), 0.0);
  let bounds = [rect];
  // Beside the left edge, half way down: x is pinned, y is untouched, so the
  // tip can be aimed anywhere along that side.
  assert_eq!(
    landed(point(406.0, 350.0), &bounds, 10.0),
    Some((point(400.0, 350.0), rect))
  );
  assert_eq!(
    landed(point(406.0, 380.0), &bounds, 10.0),
    Some((point(400.0, 380.0), rect))
  );
  // Beside the bottom edge, half way across: y is pinned instead.
  assert_eq!(
    landed(point(500.0, 396.0), &bounds, 10.0),
    Some((point(500.0, 400.0), rect))
  );
}

#[test]
fn the_axis_a_tip_still_follows_the_hand_in_is_reported_as_free() {
  // A free axis reads `None` rather than the tip's own position: a counter's
  // tail tip has to know whether the element moved it at all before it can
  // be weighed against the alignment guides, and a nought-length move that
  // looked like a win would beat every one of them.
  let bounds = [SnapBounds::padded(detected(400, 300, 200, 100), 0.0)];
  assert_eq!(
    snap_edges(point(406.0, 350.0), &bounds, 10.0),
    Some(EdgeSnap {
      x: Some(400.0),
      y: None,
      bounds: bounds[0]
    })
  );
  assert_eq!(
    snap_edges(point(406.0, 306.0), &bounds, 10.0),
    Some(EdgeSnap {
      x: Some(400.0),
      y: Some(300.0),
      bounds: bounds[0]
    })
  );
}

#[test]
fn a_tip_near_two_edges_lands_on_the_corner() {
  let rect = SnapBounds::padded(detected(400, 300, 200, 100), 0.0);
  let bounds = [rect];
  assert_eq!(
    landed(point(406.0, 306.0), &bounds, 10.0),
    Some((point(400.0, 300.0), rect))
  );
  // Diagonally outside the rectangle the corner is still in reach, which is
  // where a span limited to the edge itself would have left a dead spot.
  assert_eq!(
    landed(point(394.0, 294.0), &bounds, 10.0),
    Some((point(400.0, 300.0), rect))
  );
}

#[test]
fn an_edge_out_of_reach_or_out_of_span_does_not_snap() {
  let bounds = [SnapBounds::padded(detected(400, 300, 200, 100), 0.0)];
  // Far from every edge.
  assert!(snap_edges(point(500.0, 350.0), &bounds, 10.0).is_none());
  // In line with the left edge but well past the bottom of its span, so the
  // edge does not reach down the whole frame after the tip.
  assert!(snap_edges(point(400.0, 500.0), &bounds, 10.0).is_none());
  assert!(snap_edges(point(380.0, 350.0), &bounds, 10.0).is_none());
}

#[test]
fn the_nearest_edge_decides_which_element_the_tip_rides() {
  let near = SnapBounds::padded(detected(400, 300, 200, 100), 0.0);
  let far = SnapBounds::padded(detected(408, 300, 200, 100), 0.0);
  let bounds = [far, near];
  // Both left edges are in reach; the nearer one wins and travels with the
  // point so the chrome outlines the element actually being ridden.
  assert_eq!(
    landed(point(402.0, 350.0), &bounds, 10.0),
    Some((point(400.0, 350.0), near))
  );
  assert_eq!(
    landed(point(407.0, 350.0), &bounds, 10.0),
    Some((point(408.0, 350.0), far))
  );
}
