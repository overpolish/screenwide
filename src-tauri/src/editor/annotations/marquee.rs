// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a marquee band chooses: every annotation whose box it touches.

use super::group::annotation_bounds;
use super::{Annotation, AnnotationPoint};

/// The ids of `annotations` the band between corners `from` and `to` touches,
/// in their stacking order. Only those `eligible` count: a pinned recording
/// annotation follows its content on its own and is never chosen with others.
/// `source_per_size` turns a style's size into source pixels.
pub(crate) fn swept_annotations(
  annotations: &[Annotation],
  (from, to): (AnnotationPoint, AnnotationPoint),
  source_per_size: f64,
  eligible: impl Fn(&Annotation) -> bool,
) -> Vec<String> {
  let (left, right) = (from.x.min(to.x), from.x.max(to.x));
  let (top, bottom) = (from.y.min(to.y), from.y.max(to.y));
  annotations
    .iter()
    .filter(|annotation| eligible(annotation))
    .filter(|annotation| {
      annotation_bounds(annotation, source_per_size).is_some_and(|bounds| {
        bounds.x <= right
          && left <= bounds.x + bounds.width
          && bounds.y <= bottom
          && top <= bounds.y + bounds.height
      })
    })
    .map(|annotation| annotation.id.clone())
    .collect()
}

/// The choice a band leaves: what it touched, or with `additive`, that added
/// to what was chosen already.
pub(crate) fn chosen_after_band(
  current: &[String],
  swept: Vec<String>,
  additive: bool,
) -> Vec<String> {
  if !additive {
    return swept;
  }
  let mut chosen = current.to_vec();
  for id in swept {
    if !chosen.contains(&id) {
      chosen.push(id);
    }
  }
  chosen
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::counter::new_counter;

  fn point(x: f64, y: f64) -> AnnotationPoint {
    AnnotationPoint { x, y }
  }

  fn counters() -> Vec<Annotation> {
    vec![
      new_counter("left".to_owned(), point(100.0, 100.0), 1, None, None),
      new_counter("right".to_owned(), point(900.0, 100.0), 2, None, None),
    ]
  }

  #[test]
  fn a_band_chooses_what_it_touches_whichever_way_it_was_drawn() {
    // The band reaches only the edge of the left disc, 28 pixels out.
    let band = (point(300.0, 300.0), point(120.0, 50.0));
    assert_eq!(
      swept_annotations(&counters(), band, 2.0, |_| true),
      vec!["left"]
    );
  }

  #[test]
  fn a_band_passes_over_what_may_not_be_chosen() {
    let band = (point(0.0, 0.0), point(1000.0, 200.0));
    let swept = swept_annotations(&counters(), band, 2.0, |annotation| annotation.id != "left");
    assert_eq!(swept, vec!["right"]);
  }

  #[test]
  fn an_additive_band_keeps_the_choice_and_adds_to_it() {
    let current = vec!["a".to_owned(), "b".to_owned()];
    let swept = vec!["b".to_owned(), "c".to_owned()];
    assert_eq!(
      chosen_after_band(&current, swept.clone(), true),
      vec!["a", "b", "c"]
    );
    assert_eq!(chosen_after_band(&current, swept, false), vec!["b", "c"]);
  }
}
