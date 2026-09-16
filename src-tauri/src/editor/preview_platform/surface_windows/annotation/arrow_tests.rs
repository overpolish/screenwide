// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Placing source-space marks on the canvas, and the camera ordering the
//! shader's two passes rely on.

use super::*;
use crate::editor::annotations::{
  Annotation, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};

fn arrow(above_camera: bool, start: (f64, f64), end: (f64, f64)) -> Annotation {
  Annotation {
    above_camera,
    animated: false,
    id: format!("{}-{}", start.0, end.0),
    reveal: Default::default(),
    shape: AnnotationShape::Arrow {
      start: AnnotationPoint {
        x: start.0,
        y: start.1,
      },
      control: AnnotationPoint {
        x: (start.0 + end.0) / 2.0,
        y: (start.1 + end.1) / 2.0,
      },
      end: AnnotationPoint { x: end.0, y: end.1 },
    },
    style: AnnotationStyle {
      color: "#ff0000".to_owned(),
      head: AnnotationHead::End,
      width: 8.0,
    },
  }
}

#[test]
fn the_halo_lands_only_on_the_arrow_it_belongs_to() {
  let mut settings = crate::screenshots::test_output_settings(640, 360);
  settings.annotations = vec![
    arrow(false, (10.0, 10.0), (20.0, 20.0)),
    arrow(false, (30.0, 30.0), (40.0, 40.0)),
  ];
  let prepared =
    prepared_arrows(&settings.annotations, (320, 180), &settings, Some((1, 6.0))).unwrap();
  assert_eq!(
    prepared.arrows[0].hover, 0.0,
    "an unhovered arrow wears no halo"
  );
  assert_eq!(prepared.arrows[1].hover, 6.0);
}

#[test]
fn no_hover_leaves_every_arrow_bare() {
  let mut settings = crate::screenshots::test_output_settings(640, 360);
  settings.annotations = vec![arrow(false, (10.0, 10.0), (20.0, 20.0))];
  let prepared = prepared_arrows(&settings.annotations, (320, 180), &settings, None).unwrap();
  assert_eq!(prepared.arrows[0].hover, 0.0);
}

#[test]
fn a_still_mark_prepares_no_exposure_samples() {
  let mut settings = crate::screenshots::test_output_settings(640, 360);
  settings.annotations = vec![arrow(false, (10.0, 10.0), (300.0, 170.0))];
  let prepared = prepared_arrows(&settings.annotations, (320, 180), &settings, None).unwrap();
  assert!(prepared.samples.is_empty());
  assert_eq!(prepared.arrows[0].sample_count, 0);
}

#[test]
fn a_mark_that_moved_this_frame_is_averaged_over_its_exposure() {
  let mut settings = crate::screenshots::test_output_settings(640, 360);
  let mut mark = arrow(false, (10.0, 10.0), (300.0, 170.0));
  mark.animated = true;
  // The shutter opened with the mark half drawn and closes with it whole: a
  // long way for the head to have travelled in one frame.
  mark.reveal = crate::editor::annotations::reveal::AnnotationReveal {
    low: 0.0,
    high: 1.0,
    scale: 1.0,
    opacity: 1.0,
    previous: [0.0, 0.5, 1.0, 1.0],
  };
  settings.annotations = vec![mark];
  let prepared = prepared_arrows(&settings.annotations, (320, 180), &settings, None).unwrap();
  let arrow = prepared.arrows[0];
  assert!(arrow.sample_count >= 8, "{}", arrow.sample_count);
  assert!(arrow.sample_count <= 48);
  assert_eq!(prepared.samples.len(), arrow.sample_count as usize);
  // The samples walk from the shutter start to now, so the first is drawn
  // shorter than the last.
  let first = prepared.samples[0].geometry;
  let last = prepared.samples[prepared.samples.len() - 1].geometry;
  assert!(first.high < last.high, "{} vs {}", first.high, last.high);
  // A moving mark leaves its opacity to its samples rather than its colour.
  assert_eq!(arrow.color[3], 1.0);
}

#[test]
fn no_marks_prepare_no_arrows() {
  let settings = crate::screenshots::test_output_settings(640, 360);
  let prepared = prepared_arrows(&settings.annotations, (640, 360), &settings, None).unwrap();
  assert!(prepared.arrows.is_empty());
  assert_eq!(prepared.below_camera, 0);
}

#[test]
fn a_mark_is_placed_on_the_picture_not_on_the_canvas() {
  let mut settings = crate::screenshots::test_output_settings(640, 360);
  // A mark on the source's own corner has to land on the corner of the drawn
  // picture, which the canvas insets and scales.
  settings.annotations = vec![arrow(false, (0.0, 0.0), (320.0, 180.0))];
  let placement =
    crate::screenshots::output_placement(320, 180, &settings).expect("a valid placement");
  let prepared = prepared_arrows(&settings.annotations, (320, 180), &settings, None).unwrap();
  assert_eq!(prepared.arrows.len(), 1);
  assert!(
    (prepared.arrows[0].geometry.ax - placement.image_x as f32).abs() < 0.01,
    "the tail sits on the picture's left edge, not the canvas's: {} vs {}",
    prepared.arrows[0].geometry.ax,
    placement.image_x
  );
  assert!(
    (prepared.arrows[0].geometry.ay - placement.image_y as f32).abs() < 0.01,
    "and on its top edge"
  );
}

#[test]
fn the_stroke_keeps_its_weight_in_canvas_pixels() {
  let mut settings = crate::screenshots::test_output_settings(640, 360);
  settings.annotations = vec![arrow(false, (10.0, 10.0), (300.0, 170.0))];
  let prepared = prepared_arrows(&settings.annotations, (320, 180), &settings, None).unwrap();
  // The picture is scaled to the canvas, but the stroke is given in output
  // pixels, so it must not be scaled with the picture.
  assert_eq!(prepared.arrows[0].geometry.width, 8.0);
}

#[test]
fn marks_under_the_camera_are_prepared_ahead_of_those_above_it() {
  let mut settings = crate::screenshots::test_output_settings(640, 360);
  settings.annotations = vec![
    arrow(true, (10.0, 10.0), (20.0, 20.0)),
    arrow(false, (30.0, 30.0), (40.0, 40.0)),
    arrow(true, (50.0, 50.0), (60.0, 60.0)),
  ];
  let prepared = prepared_arrows(&settings.annotations, (320, 180), &settings, None).unwrap();
  assert_eq!(prepared.arrows.len(), 3);
  // One mark sits under the camera, so the above-camera run starts at 1.
  assert_eq!(prepared.below_camera, 1);
}

#[test]
fn marks_on_one_side_keep_the_order_their_layer_stores_them_in() {
  let mut settings = crate::screenshots::test_output_settings(640, 360);
  // Two marks above the camera: the later one has to stay later, so it
  // paints over the earlier one.
  settings.annotations = vec![
    arrow(true, (10.0, 10.0), (20.0, 20.0)),
    arrow(true, (100.0, 100.0), (200.0, 150.0)),
  ];
  let prepared = prepared_arrows(&settings.annotations, (320, 180), &settings, None).unwrap();
  assert_eq!(prepared.below_camera, 0);
  assert!(
    prepared.arrows[0].geometry.ax < prepared.arrows[1].geometry.ax,
    "the first stored mark must be prepared first"
  );
}
