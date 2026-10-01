// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::model::{new_highlight, HighlightBand, HighlightTone};
use super::picture::HighlightPicture;
use crate::editor::annotations::edit::AnnotationEdit;
use crate::editor::annotations::gesture::{
  AnnotationDragOrigin, AnnotationGestureTarget, AnnotationHandle,
};
use crate::editor::annotations::snap::SnapModifiers;
use crate::editor::annotations::{Annotation, AnnotationKind, AnnotationPoint, AnnotationShape};
use crate::screenshots::CapturedImage;
use std::sync::Arc;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn bands(annotation: &Annotation) -> Vec<HighlightBand> {
  let AnnotationShape::Highlight { bands, .. } = &annotation.shape else {
    unreachable!()
  };
  bands.clone()
}

#[test]
fn a_highlight_round_trips_through_its_document() {
  let mut annotation = new_highlight("h".to_owned(), point(10.0, 20.0), None, 1.0);
  annotation.style.hand_drawn = true;
  if let AnnotationShape::Highlight { end, tone, .. } = &mut annotation.shape {
    *end = point(80.0, 20.0);
    *tone = HighlightTone {
      surface: 0.1,
      ink: 0.9,
    };
  }
  let json = serde_json::to_string(&annotation).unwrap();
  assert!(json.contains("\"kind\":\"highlight\""), "{json}");
  assert!(json.contains("\"handDrawn\":true"), "{json}");
  assert_eq!(
    serde_json::from_str::<Annotation>(&json).unwrap(),
    annotation
  );
}

#[test]
fn a_document_without_bands_or_a_tone_still_reads() {
  let annotation: Annotation = serde_json::from_str(
    r##"{"id":"h","shape":{"kind":"highlight","start":{"x":0,"y":0},
      "end":{"x":10,"y":0}},"style":{"color":"#ffcc00","width":24}}"##,
  )
  .unwrap();
  assert!(annotation.shape.placed());
  assert!(!annotation.style.hand_drawn);
}

#[test]
fn drawing_one_out_without_a_picture_runs_a_band_along_the_drag() {
  let mut annotation = new_highlight("h".to_owned(), point(10.0, 50.0), None, 1.0);
  let origin = AnnotationDragOrigin::new(point(10.0, 50.0), &annotation.shape);
  annotation.drag_new(point(90.0, 52.0), &origin, false, None);
  assert_eq!(
    bands(&annotation),
    vec![HighlightBand {
      left: 10.0,
      top: 44.0,
      right: 90.0,
      bottom: 56.0
    }]
  );
}

#[test]
fn moving_an_end_without_a_picture_keeps_the_lines_it_covers() {
  let mut annotation = new_highlight("h".to_owned(), point(0.0, 0.0), None, 1.0);
  let lines = vec![
    HighlightBand {
      left: 40.0,
      top: 0.0,
      right: 200.0,
      bottom: 20.0,
    },
    HighlightBand {
      left: 10.0,
      top: 30.0,
      right: 120.0,
      bottom: 50.0,
    },
  ];
  if let AnnotationShape::Highlight { bands, .. } = &mut annotation.shape {
    *bands = lines.clone();
  }
  let origin = AnnotationDragOrigin::new(point(120.0, 40.0), &annotation.shape);
  annotation.drag_grip(
    AnnotationHandle::End,
    point(150.0, 90.0),
    &origin,
    false,
    None,
  );
  let moved = bands(&annotation);
  assert_eq!(moved[0], lines[0]);
  assert_eq!(moved[1].right, 150.0);
  assert_eq!((moved[1].top, moved[1].bottom), (30.0, 50.0));
}

#[test]
fn the_body_carries_every_band_with_it() {
  let mut annotation = new_highlight("h".to_owned(), point(10.0, 10.0), None, 1.0);
  let origin_shape = annotation.shape.clone();
  let origin = AnnotationDragOrigin::new(point(10.0, 10.0), &origin_shape);
  annotation.drag_grip(
    AnnotationHandle::Body,
    point(15.0, 30.0),
    &origin,
    false,
    None,
  );
  let AnnotationShape::Highlight {
    start, end, bands, ..
  } = &annotation.shape
  else {
    unreachable!()
  };
  assert_eq!((*start, *end), (point(15.0, 30.0), point(15.0, 30.0)));
  assert_eq!((bands[0].top, bands[0].bottom), (24.0, 36.0));
}

/// Two lines of dark bars on a light page, 200 by 100, each 16 pixels tall.
fn two_lines() -> Arc<CapturedImage> {
  let (width, height) = (200_u32, 100_u32);
  let mut rgba = Vec::with_capacity((width * height * 4) as usize);
  for y in 0..height {
    for x in 0..width {
      let inked = [(20, 36), (50, 66)]
        .iter()
        .any(|(top, bottom)| (*top..*bottom).contains(&y))
        && (10..190).contains(&x)
        && x % 20 < 14;
      rgba.extend_from_slice(if inked {
        &[20, 20, 20, 255]
      } else {
        &[250, 250, 250, 255]
      });
    }
  }
  Arc::new(CapturedImage {
    rgba,
    width,
    height,
  })
}

#[test]
fn a_highlight_drawn_out_over_a_picture_covers_the_lines_it_crosses() {
  let image = two_lines();
  let picture = HighlightPicture::new(image, (200, 100)).map(Arc::new);
  let mut annotations = Vec::new();
  let mut edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::New,
    point(30.0, 28.0),
    None,
    Some(AnnotationKind::Highlight),
    None,
    1.0,
  )
  .unwrap();
  assert!(edit.wants_picture(&annotations));
  edit.set_picture(picture);
  edit.update(
    &mut annotations,
    point(120.0, 58.0),
    SnapModifiers::default(),
    None,
  );
  let covered = bands(&annotations[0]);
  assert_eq!(covered.len(), 2, "{covered:?}");
  assert!(
    covered[0].top < 20.0 && covered[0].bottom > 36.0,
    "{covered:?}"
  );
  assert!(
    covered[1].top < 50.0 && covered[1].bottom > 66.0,
    "{covered:?}"
  );
}

/// A fresh highlight drawn over the two lines from `from` to `to`, laid by
/// hand over a box when `manual`.
fn drawn_over_two_lines(from: AnnotationPoint, to: AnnotationPoint, manual: bool) -> Annotation {
  let picture = HighlightPicture::new(two_lines(), (200, 100)).map(Arc::new);
  let mut style = super::model::default_highlight_style();
  style.manual = manual;
  let mut annotations = Vec::new();
  let mut edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::New,
    from,
    Some(&style),
    Some(AnnotationKind::Highlight),
    None,
    1.0,
  )
  .unwrap();
  edit.set_picture(picture);
  edit.update(&mut annotations, to, SnapModifiers::default(), None);
  annotations.remove(0)
}

#[test]
fn a_box_laid_by_hand_covers_what_it_spans_and_reads_the_page_under_it() {
  let annotation = drawn_over_two_lines(point(30.0, 10.0), point(150.0, 90.0), true);
  // The box as dragged, not the lines under it: strokes from its top to its
  // bottom, each as wide as the box.
  let strokes = bands(&annotation);
  assert!(strokes.len() > 2, "{strokes:?}");
  assert_eq!(strokes.first().map(|band| band.top), Some(10.0));
  assert_eq!(strokes.last().map(|band| band.bottom), Some(90.0));
  assert!(strokes
    .iter()
    .all(|band| (band.left, band.right) == (30.0, 150.0)));
  // Dark bars on a light page, read so the box can recolour them.
  let AnnotationShape::Highlight { tone, .. } = annotation.shape else {
    unreachable!()
  };
  assert!(tone.surface > 0.9 && tone.ink < 0.2, "{tone:?}");
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[test]
fn a_tinting_highlight_hands_the_compositor_the_unread_tone_and_keeps_its_own() {
  let mut annotation = drawn_over_two_lines(point(30.0, 20.0), point(150.0, 60.0), false);
  let AnnotationShape::Highlight { tone: read, .. } = annotation.shape.clone() else {
    unreachable!()
  };
  assert_ne!(read, HighlightTone::UNREAD);
  let recolours = annotation.shape.draw_points(&annotation.style)[2];
  assert_eq!(recolours, [read.surface as f32, read.ink as f32]);
  annotation.style.tint = true;
  assert_eq!(
    annotation.shape.draw_points(&annotation.style)[2],
    [0.5, 0.5]
  );
  // Switched back, it recolours from the page it read, without reading again.
  annotation.style.tint = false;
  assert_eq!(
    annotation.shape.draw_points(&annotation.style)[2],
    recolours
  );
}

#[test]
fn a_hand_laid_box_moves_the_corner_its_grip_is_on() {
  // Drawn up and to the left, so the press was its bottom-right corner.
  let mut annotation = drawn_over_two_lines(point(150.0, 90.0), point(30.0, 10.0), true);
  let origin = AnnotationDragOrigin::new(point(30.0, 10.0), &annotation.shape);
  // The start grip sits on the box's top-left, which is the corner it moves.
  annotation.drag_grip(
    AnnotationHandle::Start,
    point(20.0, 4.0),
    &origin,
    false,
    None,
  );
  let strokes = bands(&annotation);
  assert_eq!(strokes.first().map(|band| band.top), Some(4.0));
  assert_eq!(strokes.last().map(|band| band.bottom), Some(90.0));
  assert!(strokes
    .iter()
    .all(|band| (band.left, band.right) == (20.0, 150.0)));
}

/// Where a highlight's end grip sits down the picture: half way down its last
/// band, where the chrome draws it.
fn end_grip(annotation: &Annotation) -> f64 {
  let last = *bands(annotation).last().unwrap();
  (last.top + last.bottom) * 0.5
}

#[test]
fn a_hand_laid_box_keeps_its_grip_under_the_hand() {
  // One stroke to start with, then dragged down into a box: the grip stays
  // with the hand the whole way, across the change from one to several.
  let mut annotation = drawn_over_two_lines(point(30.0, 50.0), point(150.0, 54.0), true);
  let grip = end_grip(&annotation);
  let origin = AnnotationDragOrigin::new(point(150.0, grip), &annotation.shape);
  for y in [
    grip + 4.0,
    grip + 10.0,
    grip + 30.0,
    grip + 60.0,
    grip - 2.0,
  ] {
    annotation.drag_grip(AnnotationHandle::End, point(170.0, y), &origin, false, None);
    assert!(
      (end_grip(&annotation) - y).abs() < 1e-9,
      "{y} {:?}",
      bands(&annotation)
    );
    assert_eq!(bands(&annotation)[0].right, 170.0);
  }
}
