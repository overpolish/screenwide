// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::edit::AnnotationTextEdit;
use super::geometry::{box_size, prepare_text, text_distance};
use super::metrics::{text_block, LINE_HEIGHT};
use super::model::{new_text, TextPointer};
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn pointer(along: (f64, f64), reach: (f64, f64)) -> TextPointer {
  TextPointer {
    along: point(along.0, along.1),
    reach: point(reach.0, reach.1),
  }
}

fn text_box(text: &str, pointer: TextPointer) -> Annotation {
  let mut annotation = new_text("t".to_owned(), point(100.0, 100.0), None);
  annotation.shape = AnnotationShape::Text {
    origin: point(100.0, 100.0),
    pointer,
    text: text.to_owned(),
  };
  annotation
}

fn held_pointer(annotation: &Annotation) -> TextPointer {
  let AnnotationShape::Text { pointer, .. } = &annotation.shape else {
    unreachable!()
  };
  *pointer
}

/// A box 80 by 25 of text at 20px, prepared whole.
fn prepared(
  pointer: TextPointer,
  reveal: AnnotationReveal,
) -> crate::editor::annotations::geometry::ArrowGeometry {
  prepare_text(
    [100.0, 100.0],
    pointer.encoded(),
    [80.0, 25.0],
    20.0,
    0,
    reveal,
  )
}

#[test]
fn the_box_grows_with_its_lines_and_its_longest_line() {
  let one = text_block("Save", 20.0);
  let wider = text_block("Save changes", 20.0);
  let two = text_block("Save\nchanges", 20.0);
  assert!(wider[0] > one[0], "{one:?} {wider:?}");
  assert_eq!(one[1], 20.0 * LINE_HEIGHT);
  assert_eq!(two[1], 2.0 * 20.0 * LINE_HEIGHT);
  // Two lines are as wide as the longer of them, not the two end to end.
  assert_eq!(two[0], text_block("changes", 20.0)[0]);
  // An empty box still has a line for its caret and room around it.
  let empty = box_size(text_block("", 20.0), 20.0);
  assert!(empty[0] > 0.0 && empty[1] > 20.0);
}

#[test]
fn a_pointer_leaves_the_edge_it_reaches_past_and_ends_at_its_tip() {
  // Three lines tall, so the base has room to slide level with the tip.
  let geometry = prepare_text(
    [100.0, 100.0],
    pointer((1.0, -0.2), (4.0, 0.0)).encoded(),
    [80.0, 75.0],
    20.0,
    0,
    AnnotationReveal::WHOLE,
  );
  // Four ems past the right edge, a fifth of the way up from the middle.
  let tip = geometry.c;
  assert!(
    (tip[0] - (geometry.b[0] + 80.0)).abs() < 1e-3,
    "{geometry:?}"
  );
  assert!(text_distance(tip, &geometry).abs() < 0.05);
  assert!(text_distance([tip[0] + 2.0, tip[1]], &geometry) > 0.0);
  // Its base sits inside the right edge, level with the tip.
  assert_eq!(geometry.end_head.a[0], geometry.b[0] - geometry.high);
  assert!((geometry.end_head.a[1] - tip[1]).abs() < 1e-3);
}

/// Drawn out past a corner at a shallow slope, a pointer used to swell into a
/// bump along the edge beside the one it left: its base grazed that edge.
#[test]
fn a_shallow_pointer_leaves_no_bulge_along_the_edge_beside_it() {
  let geometry = prepared(pointer((1.0, -1.0), (8.0, 1.2)), AnnotationReveal::WHOLE);
  assert!(geometry.high > 0.0);
  // A pixel above the top edge, a little way in from the corner the pointer
  // leaves by, is canvas.
  for step in 1..5 {
    let x = geometry.b[0] - 6.0 * step as f32;
    assert!(
      text_distance([x, geometry.a[1] - 1.0], &geometry) > 0.0,
      "a bulge above the top edge at {x}"
    );
  }
}

#[test]
fn a_tucked_pointer_is_not_drawn_but_keeps_its_grip_where_it_was_left() {
  let geometry = prepared(pointer((-0.5, 0.3), (0.0, 0.0)), AnnotationReveal::WHOLE);
  assert_eq!(geometry.high, 0.0);
  let centre = [
    (geometry.a[0] + geometry.b[0]) * 0.5,
    (geometry.a[1] + geometry.b[1]) * 0.5,
  ];
  let half = [
    (geometry.b[0] - geometry.a[0]) * 0.5,
    (geometry.b[1] - geometry.a[1]) * 0.5,
  ];
  assert!((geometry.c[0] - (centre[0] - 0.5 * half[0])).abs() < 1e-3);
  assert!((geometry.c[1] - (centre[1] + 0.3 * half[1])).abs() < 1e-3);
}

#[test]
fn a_box_arrives_before_its_pointer_draws_out() {
  let out = pointer((0.0, 1.0), (0.0, 3.0));
  let grown = |high: f32| {
    prepared(
      out,
      AnnotationReveal {
        high,
        ..AnnotationReveal::WHOLE
      },
    )
  };
  assert_eq!(grown(0.0).high, 0.0);
  // Half drawn out, the tip is half way from the edge to where it will be.
  let whole = grown(1.0);
  let half = grown(0.5);
  let edge = whole.b[1];
  assert!((half.end_head.b[1] + half.low - (edge + (whole.c[1] - edge) * 0.5)).abs() < 0.5);
}

#[test]
fn a_box_arriving_grows_about_its_centre() {
  let whole = prepared(TextPointer::default(), AnnotationReveal::WHOLE);
  let half = prepared(
    TextPointer::default(),
    AnnotationReveal {
      scale: 0.5,
      ..AnnotationReveal::WHOLE
    },
  );
  let centre = |g: &crate::editor::annotations::geometry::ArrowGeometry| {
    [(g.a[0] + g.b[0]) * 0.5, (g.a[1] + g.b[1]) * 0.5]
  };
  let (settled, growing) = (centre(&whole), centre(&half));
  assert!((settled[0] - growing[0]).abs() < 1e-3 && (settled[1] - growing[1]).abs() < 1e-3);
  assert!(((half.b[0] - half.a[0]) * 2.0 - (whole.b[0] - whole.a[0])).abs() < 1e-3);
  assert_eq!(half.width, 10.0);
}

/// Pulled just past the box a pointer is the one short tooltip point every
/// box has; pulled well away it stretches to where it was pulled; pushed back
/// in it tucks away on the nearest edge, clear of the corners.
#[test]
fn the_pointers_grip_tucks_points_and_stretches() {
  let mut annotation = text_box("Hello", TextPointer::default());
  let [width, height] = box_size(text_block("Hello", 28.0), 28.0);
  let mut origin = AnnotationDragOrigin::new(point(0.0, 0.0), &annotation.shape);
  origin.source_per_output = 1.0;
  let right = 100.0 + width;
  let middle = 100.0 + height / 2.0;
  let mut drag = |to: AnnotationPoint| {
    annotation.drag_grip(AnnotationHandle::Tail, to, &origin, false, None);
    held_pointer(&annotation)
  };
  let short = drag(point(right + 10.0, middle));
  assert_eq!(short.along, point(1.0, 0.0));
  assert!((short.reach.x - 0.55).abs() < 1e-9, "{short:?}");
  let a_little_further = drag(point(right + 30.0, middle));
  assert_eq!(a_little_further, short);
  let stretched = drag(point(right + 280.0, middle));
  assert!((stretched.reach.x - 10.0).abs() < 1e-9, "{stretched:?}");
  // Just inside the left edge, it tucks onto that edge where it was let go.
  let tucked = drag(point(110.0, middle + 2.0));
  assert!(!tucked.is_drawn());
  assert_eq!(tucked.along.x, -1.0);
  assert!((tucked.along.y - 4.0 / height).abs() < 1e-9, "{tucked:?}");
  // Let go in the middle of the box, it goes to the nearer, bottom edge.
  let middle_x = 100.0 + width / 2.0;
  let centred = drag(point(middle_x + 5.0, middle + 1.0));
  assert_eq!(centred.along.y, 1.0);
  // Let go in a corner, nearer its left edge, it stays on the straight part
  // of that edge.
  let cornered = drag(point(101.0, 103.0));
  let half_height = height / 2.0;
  let corner = (0.4 * 28.0_f64).min(half_height);
  assert_eq!(cornered.along.x, -1.0);
  assert!(
    (cornered.along.y + (1.0 - corner / half_height)).abs() < 1e-6,
    "{cornered:?}"
  );
}

/// Zoomed in, the pointer follows the hand from just past the standard
/// length; zoomed out, a short pull still lands on the standard point.
#[test]
fn the_pointer_stretches_sooner_the_further_the_view_is_zoomed_in() {
  let [width, height] = box_size(text_block("Hello", 28.0), 28.0);
  let right = 100.0 + width;
  let middle = 100.0 + height / 2.0;
  let pulled = |source_per_point: f64, past: f64| {
    let mut annotation = text_box("Hello", TextPointer::default());
    let mut origin = AnnotationDragOrigin::new(point(0.0, 0.0), &annotation.shape);
    origin.source_per_output = 1.0;
    origin.source_per_point = source_per_point;
    annotation.drag_grip(
      AnnotationHandle::Tail,
      point(right + past, middle),
      &origin,
      false,
      None,
    );
    held_pointer(&annotation).reach.x
  };
  // One em is 28 source pixels. At 4x zoom, 24 points is 6 pixels: under the
  // standard reach, so a pull of 0.6 em is drawn at 0.6 em.
  assert!((pulled(0.25, 0.6 * 28.0) - 0.6).abs() < 1e-9);
  // At 1x, the same pull is inside the 24 pixel stretch point.
  assert!((pulled(1.0, 0.6 * 28.0) - 0.55).abs() < 1e-9);
  assert!((pulled(1.0, 30.0) - 30.0 / 28.0).abs() < 1e-9);
  // Zoomed out to a quarter, 24 points is 96 pixels.
  assert!((pulled(4.0, 90.0) - 0.55).abs() < 1e-9);
}

#[test]
fn moving_or_retyping_the_box_carries_its_pointer_with_it() {
  let held = pointer((1.0, 0.0), (3.0, 0.0));
  let mut annotation = text_box("Hello", held);
  let origin = AnnotationDragOrigin::new(point(120.0, 110.0), &annotation.shape);
  annotation.drag_grip(
    AnnotationHandle::Body,
    point(150.0, 90.0),
    &origin,
    false,
    None,
  );
  let AnnotationShape::Text { origin: corner, .. } = &annotation.shape else {
    unreachable!()
  };
  assert_eq!(*corner, point(130.0, 80.0));
  assert_eq!(held_pointer(&annotation), held);
  // Held against the box, the tip is three ems past whichever right edge the
  // box has: a longer text moves it out with the edge.
  let short = prepare_text(
    [0.0, 0.0],
    held.encoded(),
    [40.0, 25.0],
    20.0,
    0,
    AnnotationReveal::WHOLE,
  );
  let long = prepare_text(
    [0.0, 0.0],
    held.encoded(),
    [90.0, 25.0],
    20.0,
    0,
    AnnotationReveal::WHOLE,
  );
  assert!((short.c[0] - short.b[0] - 60.0).abs() < 1e-3);
  assert!((long.c[0] - long.b[0] - 60.0).abs() < 1e-3);
}

#[test]
fn a_box_left_with_nothing_to_read_is_removed() {
  let mut annotations = vec![text_box("", TextPointer::default())];
  let mut edit = AnnotationTextEdit::begin("t".to_owned(), String::new());
  assert!(edit.take(" \n ", 1));
  assert!(!edit.finish(&mut annotations));
  assert!(annotations.is_empty());
}

/// The document trails the typing by a round trip, so a list it sends back
/// mid-typing - dressed differently from the panel, say - takes the text typed
/// so far rather than the text it last heard of.
#[test]
fn the_text_typed_so_far_is_laid_over_a_list_the_document_sends_back() {
  let mut edit = AnnotationTextEdit::begin("t".to_owned(), "Old".to_owned());
  assert!(edit.take("New\r\nline", 2));
  // A change that arrives after a newer one is dropped.
  assert!(!edit.take("Ne", 1));
  let mut echoed = vec![text_box("Ne", TextPointer::default())];
  echoed[0].style.width = 48.0;
  assert!(edit.apply(&mut echoed));
  assert!(matches!(
    &echoed[0].shape,
    AnnotationShape::Text { text, .. } if text == "New\nline"
  ));
  assert_eq!(echoed[0].style.width, 48.0);
  assert!(edit.finish(&mut echoed));
  assert_eq!(echoed.len(), 1);
}

#[test]
fn a_stored_box_reads_back_with_its_pointer_tucked_in_when_it_has_none() {
  let annotation: Annotation = serde_json::from_str(
    r##"{"id":"a","shape":{"kind":"text","origin":{"x":1,"y":2},"text":"Hi"},
      "style":{"color":"#fff000","width":28}}"##,
  )
  .unwrap();
  assert_eq!(held_pointer(&annotation), TextPointer::default());
  let json = serde_json::to_string(&annotation).unwrap();
  assert!(json.contains("\"align\":\"left\""), "{json}");
  assert!(json.contains("\"pointer\":{\"along\""), "{json}");
  assert_eq!(
    serde_json::from_str::<Annotation>(&json).unwrap(),
    annotation
  );
}
