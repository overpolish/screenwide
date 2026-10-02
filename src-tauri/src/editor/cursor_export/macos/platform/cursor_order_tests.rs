// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the cursor sits among the annotations, through a real GPU
//! dispatch: over every mark, but lying on the picture as far as a spotlight
//! and a loupe are concerned.

use crate::editor::annotations::arrow::model::new_arrow;
use crate::editor::annotations::magnify::model::new_magnify;
use crate::editor::annotations::spotlight::model::{new_spotlight, SPOTLIGHT_DIM};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};
use crate::editor::cursor_effects::{GpuArtwork, GpuCursor};

/// Large enough that the spotlights' blur, a share of the shorter side, is
/// several pixels wide.
const SIZE: (u32, u32) = (1_000, 750);
/// The picture's grey, and the cursor's white square's top-left and side.
const GREY: u8 = 40;
const CURSOR: (f32, f32) = (500.0, 400.0);
const SIDE: u32 = 24;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// The grey picture composed the size of the canvas, with `annotations`
/// over it and a white square cursor at `at`.
fn composed(annotations: Vec<Annotation>, at: (f32, f32)) -> Vec<u8> {
  let source = crate::screenshots::CapturedImage {
    rgba: [GREY, GREY, GREY, 255].repeat(SIZE.0 as usize * SIZE.1 as usize),
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
  let cursor = GpuCursor {
    opacity: 1.0,
    blur_delta_x: 0.0,
    blur_delta_y: 0.0,
    height: SIDE as f32,
    hotspot_x: 0.0,
    hotspot_y: 0.0,
    rotation_radians: 0.0,
    scale: 1.0,
    style: 0,
    width: SIDE as f32,
    x: at.0,
    y: at.1,
    clip_at_video_edge: false,
  };
  let artwork = GpuArtwork {
    design_height: 0.0,
    design_width: 0.0,
    height: SIDE,
    origin_x: 0.0,
    origin_y: 0.0,
    pixels: [255; 4].repeat((SIDE * SIDE) as usize),
    clip_local_box: true,
    supersample: false,
    use_design: false,
    width: SIDE,
  };
  crate::screenshots::compose_output_layers(
    &source,
    &settings,
    0.0,
    false,
    Some((&cursor, &[artwork])),
    None,
    None,
    None,
    false,
    false,
  )
  .unwrap()
  .rgba
}

fn pixel(rgba: &[u8], x: u32, y: u32) -> [u8; 3] {
  let at = ((y * SIZE.0 + x) * 4) as usize;
  [rgba[at], rgba[at + 1], rgba[at + 2]]
}

/// A yellow arrow along the cursor's middle row, well past it both ways.
fn arrow() -> Annotation {
  new_arrow(
    "arrow".to_owned(),
    point(300.0, 412.0),
    point(800.0, 412.0),
    None,
  )
}

fn spotlight(start: AnnotationPoint, end: AnnotationPoint, blur: bool) -> Annotation {
  let mut annotation = new_spotlight("spotlight".to_owned(), [start, end], None);
  annotation.style.radius = 0.0;
  annotation.style.softness = 0.0;
  annotation.style.blur = blur;
  annotation
}

/// A clear loupe 160 across at `loupe`, enlarging twice the zoom area
/// centred at `area`.
fn magnifier(area: AnnotationPoint, loupe: AnnotationPoint) -> Annotation {
  let mut annotation = new_magnify("magnify".to_owned(), area, None);
  annotation.shape = AnnotationShape::Magnify {
    start: point(area.x - 40.0, area.y - 40.0),
    end: point(area.x + 40.0, area.y + 40.0),
    loupe,
    size: 160.0,
  };
  annotation.style.radius = 0.0;
  annotation.style.shadow = false;
  annotation
}

#[test]
fn the_cursor_goes_over_a_mark() {
  let drawn = composed(vec![arrow()], CURSOR);
  // The arrow's yellow has no blue; the cursor's white does.
  assert!(
    pixel(&drawn, 450, 412)[2] < 40,
    "{:?}",
    pixel(&drawn, 450, 412)
  );
  assert!(
    pixel(&drawn, 512, 412)[2] > 240,
    "{:?}",
    pixel(&drawn, 512, 412)
  );
}

#[test]
fn a_spotlights_shade_darkens_the_cursor_but_no_mark_above_it() {
  let dark = composed(
    vec![
      spotlight(point(50.0, 50.0), point(200.0, 200.0), false),
      arrow(),
    ],
    CURSOR,
  );
  let shaded = f32::from(pixel(&dark, 512, 412)[0]);
  let expected = 255.0 * (1.0 - SPOTLIGHT_DIM);
  assert!((shaded - expected).abs() < 3.0, "{shaded} for {expected}");
  assert!(
    pixel(&dark, 450, 412)[0] > 250,
    "{:?}",
    pixel(&dark, 450, 412)
  );
  // In the light it is the cursor's own white.
  let lit = composed(
    vec![spotlight(point(450.0, 350.0), point(600.0, 480.0), false)],
    CURSOR,
  );
  assert!(pixel(&lit, 512, 412)[0] > 250);
}

#[test]
fn a_blurring_spotlight_softens_the_cursor_outside_its_light() {
  // Three pixels left of the cursor's edge, the sharp cursor leaves the
  // shaded picture alone, and the blurred one spills onto it.
  let beside = |blur: bool, start: AnnotationPoint, end: AnnotationPoint| {
    pixel(
      &composed(vec![spotlight(start, end, blur)], CURSOR),
      497,
      412,
    )[0]
  };
  let far = (point(50.0, 50.0), point(200.0, 200.0));
  let sharp = beside(false, far.0, far.1);
  let soft = beside(true, far.0, far.1);
  assert!(soft > sharp + 10, "{soft} against {sharp}");
  // Where its light falls, it stays sharp.
  let around = (point(450.0, 350.0), point(600.0, 480.0));
  assert!(beside(true, around.0, around.1).abs_diff(GREY) <= 1);
}

#[test]
fn a_blurred_cursor_fades_smoothly_rather_than_in_copies() {
  let drawn = composed(
    vec![spotlight(point(50.0, 50.0), point(200.0, 200.0), true)],
    CURSOR,
  );
  // Across the cursor's left edge, from the shaded picture to its middle.
  let red: Vec<f32> = (480..512)
    .map(|x| f32::from(pixel(&drawn, x, 412)[0]))
    .collect();
  let rises: Vec<f32> = red.windows(2).map(|pair| pair[1] - pair[0]).collect();
  assert!(
    rises.iter().all(|&rise| rise >= -1.0),
    "it never falls back: {red:?}"
  );
  // A Gaussian edge rises fastest at one place, its rise growing to there
  // and shrinking after, give or take the rounding to eight bits; copies of a
  // sharp edge rise in steps, their rise swinging up and down.
  let steepest = (0..rises.len())
    .max_by(|&a, &b| rises[a].total_cmp(&rises[b]))
    .unwrap();
  let growing = rises[..=steepest]
    .windows(2)
    .all(|pair| pair[1] >= pair[0] - 1.5);
  let easing = rises[steepest..]
    .windows(2)
    .all(|pair| pair[1] <= pair[0] + 1.5);
  assert!(growing && easing, "one rise, not steps: {red:?}");
}

#[test]
fn a_loupe_shows_the_cursor_in_its_zoom_area_enlarged() {
  let drawn = composed(
    vec![magnifier(point(512.0, 412.0), point(800.0, 412.0))],
    CURSOR,
  );
  // The loupe's middle is the zoom area's middle, where the cursor is; twice
  // over, the cursor's 24 fill its middle 48.
  assert!(
    pixel(&drawn, 800, 412)[2] > 240,
    "{:?}",
    pixel(&drawn, 800, 412)
  );
  assert!(
    pixel(&drawn, 780, 412)[2] > 240,
    "{:?}",
    pixel(&drawn, 780, 412)
  );
}

#[test]
fn a_loupe_hides_the_cursor_under_it() {
  let drawn = composed(
    vec![magnifier(point(300.0, 412.0), point(800.0, 412.0))],
    (788.0, 400.0),
  );
  // Under the loupe is the enlarged grey of the zoom area, not the cursor.
  assert!(
    pixel(&drawn, 800, 412)[2] < 80,
    "{:?}",
    pixel(&drawn, 800, 412)
  );
  // Beside it, the same cursor shows.
  let bare = composed(vec![], (788.0, 400.0));
  assert!(pixel(&bare, 800, 412)[2] > 240);
}
