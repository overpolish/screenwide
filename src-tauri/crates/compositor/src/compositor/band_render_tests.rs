// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A screenshot drawn a band at a time comes out as it does drawn whole:
//! nothing that reads the picture or the canvas around a pixel leaves a seam
//! where one band meets the next.

use super::still::compose_layers_within;
use screenwide_annotations::arrow::model::new_arrow;
use screenwide_annotations::focus::model::new_focus;
use screenwide_annotations::highlight::model::{new_highlight, HighlightBand, HighlightTone};
use screenwide_annotations::magnify::model::new_magnify;
use screenwide_annotations::redact::new_redact;
use screenwide_annotations::spotlight::model::new_spotlight;
use screenwide_annotations::{Annotation, AnnotationPoint, AnnotationRedaction, AnnotationShape};
use screenwide_imaging::CapturedImage;
use screenwide_output::fixtures::test_output_settings;

const SOURCE: (u32, u32) = (300, 1200);
const CANVAS: (u32, u32) = (400, 1400);

/// A busy picture, so a seam in anything that reads it would show.
fn picture() -> CapturedImage {
  let mut rgba = Vec::with_capacity(SOURCE.0 as usize * SOURCE.1 as usize * 4);
  for y in 0..SOURCE.1 {
    for x in 0..SOURCE.0 {
      let stripe = if (x / 3 + y / 5) % 2 == 0 { 230 } else { 20 };
      rgba.extend_from_slice(&[stripe, (x * 7 + y) as u8, (y * 3) as u8, 255]);
    }
  }
  CapturedImage {
    width: SOURCE.0,
    height: SOURCE.1,
    rgba,
  }
}

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn redaction(
  id: &str,
  from: AnnotationPoint,
  to: AnnotationPoint,
  kind: AnnotationRedaction,
) -> Annotation {
  let mut annotation = new_redact(id.to_owned(), from, None);
  if let AnnotationShape::Redact { end, .. } = &mut annotation.shape {
    *end = to;
  }
  annotation.style.redaction = kind;
  annotation
}

fn lens(id: &str, from: AnnotationPoint, to: AnnotationPoint) -> Annotation {
  let mut annotation = new_magnify(id.to_owned(), from, None, Some(2.0));
  if let AnnotationShape::Magnify { end, .. } = &mut annotation.shape {
    *end = to;
  }
  annotation
}

/// The picture on a rounded, shadowed canvas under `annotations`, drawn in
/// bands no larger than `limit`. At 512 the bands meet at the canvas's rows
/// 512 and 1024, the picture's rows 412 and 924.
fn composed(annotations: Vec<Annotation>, limit: u32) -> CapturedImage {
  let mut settings = test_output_settings(CANVAS.0, CANVAS.1);
  (settings.image_x, settings.image_y, settings.image_width) = (50.0, 100.0, 300.0);
  (settings.crop_x, settings.crop_y) = (50.0, 100.0);
  (settings.crop_width, settings.crop_height) = (300.0, 1200.0);
  settings.drop_shadow = true;
  settings.radius_percent = 6.0;
  settings.background_radius_percent = 4.0;
  settings.annotations = annotations;
  let image = picture();
  compose_layers_within(&[(&image, settings)], Some(limit)).expect("the screenshot composes")
}

fn assert_same(whole: &CapturedImage, banded: &CapturedImage) {
  assert_eq!((whole.width, whole.height), (banded.width, banded.height));
  let worst = whole
    .rgba
    .iter()
    .zip(&banded.rgba)
    .enumerate()
    .map(|(index, (a, b))| (a.abs_diff(*b), index / 4))
    .max()
    .expect("pixels");
  let (x, y) = (worst.1 as u32 % whole.width, worst.1 as u32 / whole.width);
  assert!(worst.0 <= 2, "off by {} at {x}, {y}", worst.0);
}

#[test]
#[ignore = "requires a GPU adapter"]
fn redactions_across_band_edges_draw_as_they_do_whole() {
  let annotations = vec![
    redaction(
      "blur",
      point(20.0, 250.0),
      point(280.0, 700.0),
      AnnotationRedaction::Blur,
    ),
    redaction(
      "classic",
      point(40.0, 380.0),
      point(260.0, 520.0),
      AnnotationRedaction::PixelateClassic,
    ),
    redaction(
      "secure",
      point(30.0, 800.0),
      point(200.0, 1000.0),
      AnnotationRedaction::Pixelate,
    ),
  ];
  let whole = composed(annotations.clone(), 16_384);
  let banded = composed(annotations, 512);
  assert_same(&whole, &banded);
}

#[test]
#[ignore = "requires a GPU adapter"]
fn a_lens_across_a_band_edge_draws_as_it_does_whole() {
  let annotations = vec![lens("lens", point(60.0, 330.0), point(240.0, 520.0))];
  let whole = composed(annotations.clone(), 16_384);
  let banded = composed(annotations, 512);
  assert_same(&whole, &banded);
}

#[test]
#[ignore = "requires a GPU adapter"]
fn a_lens_taller_than_the_device_draws_as_it_does_whole() {
  // A thousand rows of lens on a device that takes 512: each band reads only
  // what its part of the lens shows.
  let annotations = vec![lens("lens", point(40.0, 100.0), point(260.0, 1100.0))];
  let whole = composed(annotations.clone(), 16_384);
  let banded = composed(annotations, 512);
  assert_same(&whole, &banded);
}

#[test]
#[ignore = "requires a GPU adapter"]
fn a_highlight_across_a_band_edge_draws_as_it_does_whole() {
  // A page was read under it, so it recolours the picture's ink, which it
  // reads across the whole line: a tall one, read far past the band edge.
  let mut highlight = new_highlight("highlight".to_owned(), point(30.0, 412.0), None, 1.0);
  if let AnnotationShape::Highlight {
    end, bands, tone, ..
  } = &mut highlight.shape
  {
    *end = point(270.0, 412.0);
    *bands = vec![HighlightBand {
      left: 30.0,
      top: 300.0,
      right: 270.0,
      bottom: 520.0,
    }];
    *tone = HighlightTone {
      surface: 0.9,
      ink: 0.1,
    };
  }
  let annotations = vec![highlight];
  let whole = composed(annotations.clone(), 16_384);
  let banded = composed(annotations, 512);
  assert_same(&whole, &banded);
}

#[test]
#[ignore = "requires a GPU adapter"]
fn a_blurring_spotlight_and_a_focus_across_band_edges_draw_as_they_do_whole() {
  // The arrow lies under the spotlight's shade, which blurs it through the
  // blur layers.
  let mut spotlight = new_spotlight(
    "spotlight".to_owned(),
    [point(120.0, 300.0), point(250.0, 380.0)],
    None,
  );
  spotlight.style.blur = true;
  let annotations = vec![
    new_arrow(
      "arrow".to_owned(),
      point(20.0, 380.0),
      point(280.0, 460.0),
      None,
    ),
    spotlight,
    new_focus(
      "focus".to_owned(),
      [point(60.0, 700.0), point(240.0, 900.0)],
      None,
    ),
  ];
  let whole = composed(annotations.clone(), 16_384);
  let banded = composed(annotations, 512);
  assert_same(&whole, &banded);
}
