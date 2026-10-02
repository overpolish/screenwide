// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The compositor's spotlights, through a real GPU dispatch: the shade the
//! canvas pass casts, and the blur the source pass adds.

use crate::editor::annotations::spotlight::geometry::{light, prepare_spotlight, shade};
use crate::editor::annotations::spotlight::model::{new_spotlight, SPOTLIGHT_DIM};
use crate::editor::annotations::{Annotation, AnnotationPoint};

const SIZE: (u32, u32) = (320, 200);

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn spotlight(
  start: AnnotationPoint,
  end: AnnotationPoint,
  softness: f64,
  blur: bool,
) -> Annotation {
  let mut annotation = new_spotlight("spotlight".to_owned(), [start, end], None);
  annotation.style.radius = 0.0;
  annotation.style.softness = softness;
  annotation.style.blur = blur;
  annotation
}

/// `source` composed as a screenshot the size of the canvas, with
/// `annotations` over it.
fn composed(source: Vec<u8>, annotations: Vec<Annotation>) -> Vec<u8> {
  let source = crate::screenshots::CapturedImage {
    rgba: source,
    width: SIZE.0,
    height: SIZE.1,
  };
  let mut settings = crate::screenshots::test_output_settings(SIZE.0, SIZE.1);
  settings.background_color = "#000000".to_owned();
  settings.background_type = "solid".to_owned();
  settings.crop_height = f64::from(SIZE.1);
  settings.crop_width = f64::from(SIZE.0);
  settings.crop_x = 0.0;
  settings.crop_y = 0.0;
  settings.image_width = f64::from(SIZE.0);
  settings.image_x = 0.0;
  settings.image_y = 0.0;
  settings.annotations = annotations;
  crate::screenshots::compose_output_layers(
    &source, &settings, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap()
  .rgba
}

fn grey(level: u8) -> Vec<u8> {
  [level, level, level, 255].repeat(SIZE.0 as usize * SIZE.1 as usize)
}

/// Black and white in alternate columns: sharp, it keeps both extremes;
/// blurred, it goes to the grey between them.
fn stripes() -> Vec<u8> {
  (0..SIZE.1)
    .flat_map(|_| (0..SIZE.0).flat_map(|x| if x % 2 == 0 { [0, 0, 0, 255] } else { [255; 4] }))
    .collect()
}

fn red(rgba: &[u8], x: u32, y: u32) -> u8 {
  rgba[((y * SIZE.0 + x) * 4) as usize]
}

/// Whether `drawn` is `expected` within the output's dither.
fn near(drawn: u8, expected: f32) -> bool {
  (f32::from(drawn) - expected).abs() <= 2.0
}

#[test]
fn the_shade_falls_outside_the_box_by_the_rust_rule() {
  let annotation = spotlight(point(100.0, 60.0), point(220.0, 140.0), 20.0, false);
  let image = composed(grey(200), vec![annotation]);
  // Twenty percent of eighty pixels fade past the right edge.
  let geometry = prepare_spotlight([100.0, 60.0], [220.0, 140.0], 0.0, 20.0);
  for x in (150..260).step_by(3) {
    let lit = light([x as f32 + 0.5, 100.5], &geometry, 1.0);
    let expected = 200.0 * (1.0 - SPOTLIGHT_DIM * shade([(1.0, lit)]));
    let drawn = red(&image, x, 100);
    assert!(
      near(drawn, expected),
      "x {x}: drawn {drawn}, expected {expected}"
    );
  }
  assert!(near(red(&image, 160, 100), 200.0));
  assert!(near(red(&image, 5, 5), 200.0 * (1.0 - SPOTLIGHT_DIM)));
}

#[test]
fn overlapping_spotlights_share_one_shade() {
  let first = spotlight(point(40.0, 40.0), point(140.0, 160.0), 0.0, false);
  let mut second = spotlight(point(120.0, 40.0), point(260.0, 160.0), 0.0, false);
  second.id = "second".to_owned();
  let image = composed(grey(200), vec![first, second]);
  // Where they overlap, lit; outside both, one shade and not two.
  assert!(near(red(&image, 130, 100), 200.0));
  let outside = red(&image, 300, 100);
  assert!(near(outside, 200.0 * (1.0 - SPOTLIGHT_DIM)), "{outside}");
}

#[test]
fn a_blurring_spotlight_softens_only_what_lies_outside_it() {
  let annotation = spotlight(point(100.0, 60.0), point(220.0, 140.0), 0.0, true);
  let image = composed(stripes(), vec![annotation]);
  // Inside the light the stripes are as sharp as they were.
  assert!(near(red(&image, 160, 100), 0.0));
  assert!(near(red(&image, 161, 100), 255.0));
  // Outside, neighbouring columns have run together towards the grey
  // between them, then been shaded.
  let (even, odd) = (
    f32::from(red(&image, 30, 30)),
    f32::from(red(&image, 31, 30)),
  );
  assert!((even - odd).abs() < 20.0, "{even} {odd}");
  let middle = 127.5 * (1.0 - SPOTLIGHT_DIM);
  assert!((even - middle).abs() < 20.0, "{even}");
}

#[test]
fn a_spotlight_that_does_not_blur_leaves_the_picture_sharp() {
  let annotation = spotlight(point(100.0, 60.0), point(220.0, 140.0), 0.0, false);
  let image = composed(stripes(), vec![annotation]);
  assert!(near(red(&image, 30, 30), 0.0));
  let odd = red(&image, 31, 30);
  assert!(near(odd, 255.0 * (1.0 - SPOTLIGHT_DIM)), "{odd}");
}
