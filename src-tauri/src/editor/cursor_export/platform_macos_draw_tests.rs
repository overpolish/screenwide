// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The draw tool's strokes, through a real Metal dispatch: the line the
//! compositor paints is the one Rust fitted and picks by.

use crate::editor::annotations::freehand::geometry::freehand_distance;
use crate::editor::annotations::freehand::model::new_draw;
use crate::editor::annotations::freehand::path::fitted;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

const SIZE: (u32, u32) = (320, 200);

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// A stroke through `points`, red, eight points wide.
fn stroke(points: Vec<AnnotationPoint>, smooth: bool) -> Annotation {
  let mut annotation = new_draw("stroke".to_owned(), points[0], None);
  annotation.style.color = "#ff0000".to_owned();
  annotation.shape = AnnotationShape::Draw { points, smooth };
  annotation
}

/// A black picture the size of the canvas, with `annotations` over it.
fn composed(annotations: Vec<Annotation>) -> Vec<u8> {
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
  settings.annotations = annotations;
  crate::screenshots::compose_output_layers(
    &source, &settings, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap()
  .rgba
}

fn red(rgba: &[u8], x: u32, y: u32) -> u8 {
  rgba[((y * SIZE.0 + x) * 4) as usize]
}

/// An L drawn the way a hand draws one: a sample every two pixels along it.
fn corner() -> Vec<AnnotationPoint> {
  let across = (0..=80).map(|step| point(40.0 + f64::from(step) * 2.0, 40.0));
  let down = (1..=60).map(|step| point(200.0, 40.0 + f64::from(step) * 2.0));
  across.chain(down).collect()
}

#[test]
fn a_drawn_stroke_paints_the_line_it_is_picked_by() {
  let points = corner();
  let pixels = composed(vec![stroke(points.clone(), false)]);
  let chain = fitted(&points, false);
  let pen = crate::editor::annotations::freehand::model::NEW_DRAW_WIDTH as f32;
  // Every pixel well inside the fitted line is painted, and every pixel well
  // outside it is left black; only the antialiased edge lies between.
  for y in 0..SIZE.1 {
    for x in 0..SIZE.0 {
      let distance = freehand_distance([x as f32 + 0.5, y as f32 + 0.5], &chain, pen);
      let drawn = red(&pixels, x, y);
      if distance < -1.0 {
        assert!(drawn > 240, "{x}, {y} is inside the line but {drawn}");
      } else if distance > 1.0 {
        assert!(drawn < 16, "{x}, {y} is outside the line but {drawn}");
      }
    }
  }
  // A drawn L keeps its corner.
  assert!(red(&pixels, 200, 40) > 240);
}

#[test]
fn a_smoothed_stroke_cuts_its_corner() {
  let pixels = composed(vec![stroke(corner(), true)]);
  // The line runs round the inside of the corner rather than into it.
  assert!(red(&pixels, 200, 40) < 16);
  assert!(red(&pixels, 120, 40) > 240);
  assert!(red(&pixels, 200, 140) > 240);
}

#[test]
fn a_stroke_drawing_itself_in_paints_only_the_line_it_has_reached() {
  // Half of the L's 280 pixels: along its top, as far as 180 across.
  let mut drawing = stroke(corner(), false);
  drawing.reveal = crate::editor::annotations::reveal::AnnotationReveal {
    low: 0.0,
    high: 0.5,
    scale: 1.0,
    opacity: 1.0,
    previous: [0.0, 0.5, 1.0, 1.0],
  };
  let pixels = composed(vec![drawing]);
  assert!(red(&pixels, 60, 40) > 240, "the start is drawn");
  assert!(
    red(&pixels, 170, 40) > 240,
    "so is the line up to where it has reached"
  );
  assert!(red(&pixels, 192, 40) < 16, "but not past that");
  assert!(
    red(&pixels, 200, 120) < 16,
    "nor the leg it has yet to reach"
  );
}

#[test]
fn a_stroke_drawing_itself_in_smears_its_end_over_the_exposure() {
  // While the shutter was open the end ran from 112 pixels along the L to
  // 140, from 152 across to 180.
  let mut drawing = stroke(corner(), false);
  drawing.reveal = crate::editor::annotations::reveal::AnnotationReveal {
    low: 0.0,
    high: 0.5,
    scale: 1.0,
    opacity: 1.0,
    previous: [0.0, 0.4, 1.0, 1.0],
  };
  let pixels = composed(vec![drawing]);
  assert!(
    red(&pixels, 120, 40) > 240,
    "behind the whole sweep it is solid"
  );
  let smeared = red(&pixels, 170, 40);
  assert!(
    smeared > 30 && smeared < 220,
    "along the sweep it is part-covered, not {smeared}"
  );
  assert!(red(&pixels, 192, 40) < 16, "past the sweep it is clear");
}
