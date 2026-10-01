// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The same cases `src/features/editor/annotations/annotation-pace.test.ts` holds, so a
//! live annotation and an editor one keep one pace.

use super::*;
use crate::editor::annotations::arrow::model::new_arrow;
use crate::editor::annotations::outline::model::new_shape;
use crate::editor::annotations::text::model::TextPointer;

const FRAME: (u32, u32) = (1_920, 1_080);

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// A straight arrow running `share` of the picture's diagonal.
fn arrow(share: f64) -> Annotation {
  let diagonal = f64::from(FRAME.0).hypot(f64::from(FRAME.1));
  let x = share * diagonal * f64::from(FRAME.0) / diagonal;
  let y = share * diagonal * f64::from(FRAME.1) / diagonal;
  new_arrow("a".to_owned(), point(0.0, 0.0), point(x, y), None)
}

#[test]
fn a_longer_path_draws_for_longer_but_not_in_proportion_within_its_range() {
  assert_eq!(path_ms(&arrow(0.2), FRAME), Some(1_000.0));
  assert_eq!(path_ms(&arrow(0.45), FRAME), Some(1_500.0));
  assert_eq!(path_ms(&arrow(2.0), FRAME), Some(2_000.0));
  assert_eq!(path_ms(&arrow(0.02), FRAME), Some(600.0));
}

#[test]
fn a_shape_is_paced_round_its_outline() {
  let shape = new_shape(
    "s".to_owned(),
    [point(200.0, 200.0), point(1_200.0, 800.0)],
    1,
    None,
  );
  assert_eq!(path_ms(&shape, FRAME), Some(2_000.0));
}

#[test]
fn a_pointer_is_paced_by_its_reach_and_a_tucked_one_not_at_all() {
  let text = |reach: f64| {
    let mut text =
      crate::editor::annotations::text::new_text("t".to_owned(), point(0.0, 0.0), None, 0.0);
    if let AnnotationShape::Text { pointer, .. } = &mut text.shape {
      *pointer = TextPointer {
        along: point(0.0, 1.0),
        reach: point(0.0, reach),
      };
    }
    text
  };
  assert_eq!(path_ms(&text(12.0), FRAME), Some(560.0));
  assert_eq!(path_ms(&text(0.0), FRAME), None);
}
