// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The compositor's shape annotations, through a real Metal dispatch.

use crate::editor::annotations::outline::geometry::{prepare_shape, shape_distance};
use crate::editor::annotations::outline::model::new_shape;
use crate::editor::annotations::outline::native::hand;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

const SIZE: (u32, u32) = (480, 320);
const PEN: f64 = 8.0;

/// A shape from `start` to `end` on a black canvas, in pure red so a channel
/// test reads as coverage.
fn shape(
  start: AnnotationPoint,
  end: AnnotationPoint,
  radius: f64,
  hand_drawn: bool,
) -> Annotation {
  let mut annotation = new_shape("shape".to_owned(), [start, end], 0x5eed_1234, None);
  annotation.style.color = "#ff0000".to_owned();
  annotation.style.hand_drawn = hand_drawn;
  annotation.style.radius = radius;
  annotation.style.width = PEN;
  annotation
}

fn composed(annotation: Annotation) -> crate::screenshots::CapturedImage {
  let source = crate::screenshots::CapturedImage {
    rgba: [0, 0, 0, 255].repeat(SIZE.0 as usize * SIZE.1 as usize),
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
  settings.annotations = vec![annotation];
  let rgba = crate::screenshots::compose_output_layers(
    &source, &settings, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap();
  crate::screenshots::CapturedImage {
    rgba: rgba.rgba,
    width: SIZE.0,
    height: SIZE.1,
  }
}

fn red(image: &crate::screenshots::CapturedImage, x: u32, y: u32) -> u8 {
  image.rgba[((y * image.width + x) * 4) as usize]
}

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// Every pixel clear of the stroke's edge is lit exactly where the Rust
/// distance - which picks the shape - says the stroke is, so the shader draws
/// what a press lands on.
fn assert_drawn_as_picked(annotation: &Annotation) {
  let AnnotationShape::Shape { start, end, seed } = annotation.shape else {
    unreachable!()
  };
  let geometry = prepare_shape(
    [start.x as f32, start.y as f32],
    [end.x as f32, end.y as f32],
    annotation.style.radius as f32,
    hand(annotation.style.hand_drawn, seed),
    annotation.style.width as f32,
    AnnotationReveal::WHOLE,
  );
  let image = composed(annotation.clone());
  let mut lit = 0;
  for y in 0..SIZE.1 {
    for x in 0..SIZE.0 {
      let distance = shape_distance([x as f32 + 0.5, y as f32 + 0.5], &geometry);
      let value = red(&image, x, y);
      if distance < -1.0 {
        assert!(
          value > 250,
          "({x}, {y}) is {distance} inside but reads {value}"
        );
        lit += 1;
      } else if distance > 1.0 {
        assert!(
          value < 5,
          "({x}, {y}) is {distance} outside but reads {value}"
        );
      }
    }
  }
  assert!(lit > 0, "nothing was drawn");
}

#[test]
fn a_clean_shape_is_drawn_as_it_is_picked() {
  assert_drawn_as_picked(&shape(point(100.0, 80.0), point(380.0, 240.0), 0.0, false));
  assert_drawn_as_picked(&shape(point(100.0, 80.0), point(380.0, 240.0), 20.0, false));
}

#[test]
fn a_hand_drawn_shape_is_drawn_as_it_is_picked() {
  for seed in [1, 7, 0x5eed_1234] {
    for (start, end, radius) in [
      (point(100.0, 80.0), point(380.0, 240.0), 0.0),
      (point(100.0, 80.0), point(380.0, 240.0), 20.0),
      (point(140.0, 60.0), point(340.0, 260.0), 50.0),
    ] {
      let mut annotation = shape(start, end, radius, true);
      annotation.shape = AnnotationShape::Shape { start, end, seed };
      assert_drawn_as_picked(&annotation);
    }
  }
}

/// Rounding a square all the way leaves its corners dark and its circle lit.
#[test]
fn a_square_rounded_all_the_way_is_drawn_as_a_circle() {
  let image = composed(shape(point(140.0, 60.0), point(340.0, 260.0), 50.0, false));
  assert!(red(&image, 142, 62) < 5);
  let diagonal = 100.0 * std::f64::consts::FRAC_1_SQRT_2;
  assert!(red(&image, (240.0 - diagonal) as u32, (160.0 - diagonal) as u32) > 250);
  assert!(red(&image, 240, 160) < 5);
}
