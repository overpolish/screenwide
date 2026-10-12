// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

#[test]
fn a_drag_about_one_stroke_tall_is_one_stroke_centred_on_it() {
  let bands = strokes(point(10.0, 100.0), point(200.0, 110.0), 24.0);
  assert_eq!(
    bands,
    vec![HighlightBand {
      left: 10.0,
      top: 93.0,
      right: 200.0,
      bottom: 117.0
    }]
  );
}

#[test]
fn a_box_is_covered_edge_to_edge_by_overlapping_strokes() {
  // Drawn from its bottom-right, which lays the same strokes.
  let bands = strokes(point(200.0, 300.0), point(10.0, 100.0), 24.0);
  assert!(bands.len() > 1, "{bands:?}");
  assert_eq!(bands.first().map(|band| band.top), Some(100.0));
  assert_eq!(bands.last().map(|band| band.bottom), Some(300.0));
  for band in &bands {
    assert_eq!((band.left, band.right), (10.0, 200.0));
    assert!((band.bottom - band.top - 24.0).abs() < 1e-9, "{band:?}");
  }
  for pair in bands.windows(2) {
    // Every stroke reaches over the top of the next, and only just.
    let overlap = pair[0].bottom - pair[1].top;
    assert!(
      (24.0 * 0.08 - 1e-9..24.0 * 0.5).contains(&overlap),
      "{pair:?}"
    );
  }
}
