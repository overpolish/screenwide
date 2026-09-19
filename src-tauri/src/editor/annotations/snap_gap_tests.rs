// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Equal spacing through the whole edit path, and the bars it publishes.

use super::counter::new_counter;
use super::edit::AnnotationEdit;
use super::gesture::{AnnotationGestureTarget, AnnotationHandle, NewAnnotationKind};
use super::handles::{annotation_snap, SNAP_FLAG_GAP_X, SNAP_FLAG_GAP_Y};
use super::snap::{GapSpan, SnapField, SnapModifiers, SnapRequest, SnapResult};
use super::{Annotation, AnnotationPoint, AnnotationShape};

const SOURCE: (u32, u32) = (1920, 1080);
/// Drawn one output pixel per source pixel, so the default 56-pixel disc is
/// 56 source pixels across and its edges sit 28 either side of its centre.
const IMAGE_WIDTH: f64 = 1920.0;
const THRESHOLD: f64 = 16.0;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn positional() -> SnapModifiers {
  SnapModifiers::from_bits(0b10)
}

fn counter(id: &str, x: f64, y: f64) -> Annotation {
  new_counter(id.to_owned(), point(x, y), 1, None, None)
}

fn centre(annotation: &Annotation) -> AnnotationPoint {
  let AnnotationShape::Counter { center, .. } = annotation.shape else {
    unreachable!()
  };
  center
}

fn span(from: f64, to: f64, cross: f64) -> GapSpan {
  GapSpan { from, to, cross }
}

/// Drags the counter at index 0 - which stands where the press landed - to
/// `to`, with the positional modifier held.
fn moved_to(
  annotations: &mut Vec<Annotation>,
  field: &SnapField,
  to: AnnotationPoint,
) -> SnapResult {
  let from = centre(&annotations[0]);
  let edit = AnnotationEdit::begin(
    annotations,
    AnnotationGestureTarget::Existing {
      index: 0,
      handle: AnnotationHandle::Body,
    },
    from,
    None,
    NewAnnotationKind::Counter,
    None,
  )
  .unwrap();
  edit.update(
    annotations,
    to,
    positional(),
    Some(SnapRequest {
      field,
      threshold: THRESHOLD,
    }),
  )
}

#[test]
fn a_third_counter_takes_the_gap_the_first_two_have() {
  // Two discs 144 pixels apart down one row. The third arrives five pixels
  // short of the same gap past the second, so it slides out to match it.
  let mut annotations = vec![
    counter("moved", 100.0, 500.0),
    counter("first", 200.0, 500.0),
    counter("second", 400.0, 500.0),
  ];
  let field = SnapField::new(SOURCE, &annotations, "moved", IMAGE_WIDTH);
  let result = moved_to(&mut annotations, &field, point(595.0, 500.0));
  assert_eq!(centre(&annotations[0]).x, 600.0);
  // The bars run between the pair and between the pair and the newcomer,
  // both 144 long, and lie along the row the three discs share.
  assert_eq!(
    result.gap_x.map(|gap| gap.spans),
    Some([span(228.0, 372.0, 500.0), span(428.0, 572.0, 500.0)])
  );
  assert!(result.guide_x.is_none());
  // The row itself is an alignment, not a gap: both chromes show at once.
  assert!(result.guide_y.is_some_and(|guide| guide.object));
  assert!(result.gap_y.is_none());
}

#[test]
fn a_counter_between_two_others_centres_itself() {
  let mut annotations = vec![
    counter("moved", 100.0, 500.0),
    counter("first", 200.0, 500.0),
    counter("second", 800.0, 500.0),
  ];
  let field = SnapField::new(SOURCE, &annotations, "moved", IMAGE_WIDTH);
  let result = moved_to(&mut annotations, &field, point(505.0, 500.0));
  assert_eq!(centre(&annotations[0]).x, 500.0);
  assert_eq!(
    result.gap_x.map(|gap| gap.spans),
    Some([span(228.0, 472.0, 500.0), span(528.0, 772.0, 500.0)])
  );
}

#[test]
fn a_pair_that_shares_no_row_with_the_moving_counter_offers_no_gap() {
  // The same two discs, but far up the picture: nothing about their spacing
  // says anything about where a counter down here belongs.
  let mut annotations = vec![
    counter("moved", 100.0, 700.0),
    counter("first", 200.0, 100.0),
    counter("second", 400.0, 100.0),
  ];
  let field = SnapField::new(SOURCE, &annotations, "moved", IMAGE_WIDTH);
  let result = moved_to(&mut annotations, &field, point(595.0, 700.0));
  assert_eq!(centre(&annotations[0]), point(595.0, 700.0));
  assert_eq!(result, SnapResult::default());
}

#[test]
fn an_alignment_beats_a_gap_asking_for_the_same_move() {
  // The gap wants the counter five pixels further right; so does another
  // counter's left edge. Lining up with something the hand placed reads
  // better than matching a rhythm, so the guide wins and the bars stay away.
  let mut annotations = vec![
    counter("moved", 100.0, 500.0),
    counter("first", 200.0, 500.0),
    counter("second", 400.0, 500.0),
    // Out of the row, so it offers its lines but never a gap.
    counter("aligned", 600.0, 100.0),
  ];
  let field = SnapField::new(SOURCE, &annotations, "moved", IMAGE_WIDTH);
  let result = moved_to(&mut annotations, &field, point(595.0, 500.0));
  assert_eq!(centre(&annotations[0]).x, 600.0);
  assert_eq!(
    result.guide_x.map(|guide| (guide.position, guide.object)),
    Some((572.0, true))
  );
  assert!(result.gap_x.is_none());
}

#[test]
fn the_bars_are_published_along_their_own_axis_and_across_the_other() {
  // A gap in x measures along the width and its bars lie at a height; the
  // chrome would draw them in the wrong place if either were normalised over
  // the other side of the picture.
  let mut annotations = vec![
    counter("moved", 100.0, 500.0),
    counter("first", 200.0, 500.0),
    counter("second", 400.0, 500.0),
  ];
  let field = SnapField::new(SOURCE, &annotations, "moved", IMAGE_WIDTH);
  let result = moved_to(&mut annotations, &field, point(595.0, 500.0));
  let snap = annotation_snap(&result, SOURCE);
  assert_eq!(snap.flags & SNAP_FLAG_GAP_X, SNAP_FLAG_GAP_X);
  assert_eq!(snap.flags & SNAP_FLAG_GAP_Y, 0);
  assert_eq!(snap.gap_x[0].from, 228.0 / 1920.0);
  assert_eq!(snap.gap_x[0].to, 372.0 / 1920.0);
  assert_eq!(snap.gap_x[0].cross, 500.0 / 1080.0);
  assert_eq!(snap.gap_x[1].from, 428.0 / 1920.0);
  assert_eq!(snap.gap_x[1].to, 572.0 / 1920.0);
}
