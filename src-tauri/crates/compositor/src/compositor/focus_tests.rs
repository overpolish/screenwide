// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A focus softens everything under it outside its box - the screen, the
//! background and the annotations before it - and leaves the annotations
//! after it sharp.

use super::render_test_helpers::{compositor, gpu, target, view};
use super::*;
use crate::annotation_gpu::prepared_arrows;
use screenwide_annotations::focus::model::new_focus;
use screenwide_annotations::outline::model::new_shape;
use screenwide_annotations::{Annotation, AnnotationPoint};
use screenwide_imaging::CapturedImage;
use screenwide_output::fixtures::test_output_settings;

/// A black 400 by 300 screen in the middle of a white canvas, one canvas
/// pixel to a source pixel.
fn settings() -> ScreenshotOutputSettings {
  let mut settings = test_output_settings(1920, 1080);
  settings.background_color = "#ffffff".to_owned();
  settings.drop_shadow = false;
  (settings.image_width, settings.crop_width) = (400.0, 400.0);
  settings.crop_height = 300.0;
  (settings.image_x, settings.crop_x) = (760.0, 760.0);
  (settings.image_y, settings.crop_y) = (390.0, 390.0);
  settings
}

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// A focus over the screen's top-left corner, well clear of what is measured.
fn focus() -> Annotation {
  new_focus(
    "focus".to_owned(),
    [point(0.0, 0.0), point(50.0, 50.0)],
    None,
  )
}

/// An outline whose left side crosses canvas row 630 at about x 1010.
fn outline() -> Annotation {
  new_shape(
    "outline".to_owned(),
    [point(250.0, 200.0), point(380.0, 280.0)],
    1,
    None,
  )
}

/// The canvas drawn with `annotations` over the screen, read back.
fn drawn(annotations: &[Annotation]) -> Vec<u8> {
  drawn_with_camera(annotations, None, None)
}

/// The same, with a camera clear of the screen at canvas (1400, 100), 200
/// pixels square, white down its left half and black down its right, drawn
/// in front of the screen or behind it as `camera_on_top` says, and `halo`
/// the hovered annotation and its halo's width.
fn drawn_with_camera(
  annotations: &[Annotation],
  camera_on_top: Option<bool>,
  halo: Option<(usize, f32)>,
) -> Vec<u8> {
  let compositor = compositor();
  let source = compositor
    .screenshot_source(&CapturedImage {
      width: 400,
      height: 300,
      rgba: [0, 0, 0, 255].repeat(400 * 300),
    })
    .expect("the source uploads");
  let camera = compositor
    .screenshot_source(&CapturedImage {
      width: 200,
      height: 200,
      rgba: (0..200 * 200)
        .flat_map(|index| match index % 200 < 100 {
          true => [255, 255, 255, 255],
          false => [0, 0, 0, 255],
        })
        .collect(),
    })
    .expect("the camera uploads");
  let geometry = BakeGeometry {
    crop_height: 200,
    crop_width: 200,
    crop_x: 0,
    crop_y: 0,
    frame_height: 200,
    frame_width: 200,
    frame_x: 1400,
    frame_y: 100,
    output_height: 1080,
    output_width: 1920,
    radius: 0,
  };
  let settings = settings();
  let prepared = prepared_arrows(annotations, (400, 300), &settings, None, halo, None)
    .expect("the annotations prepare");
  let output = target();
  compositor
    .draw_with_camera(
      &view(&output),
      &source,
      &settings,
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: false,
        seconds: 0.0,
      },
      camera_on_top.map(|on_top| (&camera, geometry, false, on_top)),
      None,
      &prepared,
    )
    .expect("the canvas draws");
  gpu().read_texture(&output).expect("the target reads back")
}

/// The steepest step in brightness between neighbouring pixels of row `y`
/// from `x0` to `x1`: high across a sharp edge, low across a softened one.
fn steepest(pixels: &[u8], y: u32, (x0, x1): (u32, u32)) -> u32 {
  let brightness = |x: u32| {
    let at = (y * 1920 + x) as usize * 4;
    pixels[at..at + 3]
      .iter()
      .map(|v| u32::from(*v))
      .sum::<u32>()
      / 3
  };
  (x0..x1)
    .map(|x| brightness(x).abs_diff(brightness(x + 1)))
    .max()
    .unwrap_or(0)
}

/// The screen's right edge against the background, at canvas x 1160.
const SCREEN_EDGE: (u32, (u32, u32)) = (540, (1140, 1180));
/// The outline's left side, at canvas x 1010.
const OUTLINE_EDGE: (u32, (u32, u32)) = (630, (990, 1030));
/// The camera's seam between its halves, at canvas x 1500.
const CAMERA_SEAM: (u32, (u32, u32)) = (200, (1480, 1520));

#[test]
fn without_a_focus_every_edge_stays_sharp() {
  let pixels = drawn(&[outline()]);
  let (y, span) = SCREEN_EDGE;
  assert!(
    steepest(&pixels, y, span) > 150,
    "the screen's edge is sharp"
  );
  let (y, span) = OUTLINE_EDGE;
  assert!(steepest(&pixels, y, span) > 100, "the outline is sharp");
}

#[test]
fn a_focus_softens_the_screen_against_the_background_and_what_lies_under_it() {
  let pixels = drawn(&[outline(), focus()]);
  let (y, span) = SCREEN_EDGE;
  let screen = steepest(&pixels, y, span);
  assert!(screen < 60, "the screen's edge softens: {screen}");
  let (y, span) = OUTLINE_EDGE;
  let outline = steepest(&pixels, y, span);
  assert!(
    outline < 60,
    "the outline under the focus softens: {outline}"
  );
}

#[test]
fn what_lies_over_the_focus_stays_sharp() {
  let pixels = drawn(&[focus(), outline()]);
  let (y, span) = OUTLINE_EDGE;
  let outline = steepest(&pixels, y, span);
  assert!(
    outline > 100,
    "the outline over the focus stays sharp: {outline}"
  );
  let (y, span) = SCREEN_EDGE;
  let screen = steepest(&pixels, y, span);
  assert!(screen < 60, "the screen under it still softens: {screen}");
}

#[test]
fn a_camera_in_front_of_the_screen_stays_sharp_under_a_focus_on_it() {
  let pixels = drawn_with_camera(&[focus()], Some(true), None);
  let (y, span) = CAMERA_SEAM;
  let camera = steepest(&pixels, y, span);
  assert!(camera > 150, "the camera in front stays sharp: {camera}");
  let (y, span) = SCREEN_EDGE;
  let screen = steepest(&pixels, y, span);
  assert!(screen < 60, "the screen still softens: {screen}");
}

#[test]
fn a_camera_behind_the_screen_softens_as_a_spotlight_would_darken_it() {
  let pixels = drawn_with_camera(&[focus()], Some(false), None);
  let (y, span) = CAMERA_SEAM;
  let camera = steepest(&pixels, y, span);
  assert!(
    camera < 60,
    "the camera behind the screen softens: {camera}"
  );
}

/// The brightness at canvas `(x, y)`.
fn brightness_at(pixels: &[u8], (x, y): (u32, u32)) -> u32 {
  let at = (y * 1920 + x) as usize * 4;
  pixels[at..at + 3]
    .iter()
    .map(|value| u32::from(*value))
    .sum::<u32>()
    / 3
}

#[test]
fn a_hovered_focus_wears_a_halo_round_its_box_over_the_blur() {
  // A box over the black screen's middle: canvas (910, 490) to (1010, 590).
  let focus = new_focus(
    "focus".to_owned(),
    [point(150.0, 100.0), point(250.0, 200.0)],
    None,
  );
  // Just outside the box's right edge, inside the halo's band.
  let beside = (1012, 540);
  let plain = brightness_at(&drawn(std::slice::from_ref(&focus)), beside);
  assert!(plain < 20, "no halo while not hovered: {plain}");
  let hovered = drawn_with_camera(&[focus], None, Some((0, 6.0)));
  let halo = brightness_at(&hovered, beside);
  assert!(halo > 80, "the hovered focus wears a light halo: {halo}");
}

#[test]
fn the_screen_edge_blurs_into_the_background_without_a_seam() {
  // A box 150 pixels in from the screen's right edge, where the screen is
  // only a little soft and the background beyond it is wholly blurred.
  let near = new_focus(
    "focus".to_owned(),
    [point(150.0, 100.0), point(250.0, 200.0)],
    None,
  );
  let pixels = drawn(&[near]);
  // From inside the black screen out into the white background. A blur
  // whose width jumps at the edge rises steeply, stalls where the width
  // changes and rises again: a hard line. One that carries on from the
  // screen's blur rises once, its steps growing and then shrinking.
  let row: Vec<u32> = (1100..1260)
    .map(|x| brightness_at(&pixels, (x, 540)))
    .collect();
  assert!(
    row[0] < 60 && row[row.len() - 1] > 200,
    "the span crosses the edge: {row:?}"
  );
  let steps: Vec<i64> = row
    .windows(2)
    .map(|pair| i64::from(pair[1]) - i64::from(pair[0]))
    .collect();
  let peak = (0..steps.len()).max_by_key(|&at| steps[at]).unwrap_or(0);
  let rising = steps[..=peak].windows(2).all(|pair| pair[1] >= pair[0] - 2);
  let falling = steps[peak..].windows(2).all(|pair| pair[1] <= pair[0] + 2);
  assert!(
    rising && falling && steps.iter().all(|&step| step >= -2),
    "the edge rises in more than one stretch: {row:?}"
  );
}

#[test]
fn the_canvas_keeps_its_rounded_corners_sharp_under_a_focus() {
  let compositor = compositor();
  let source = compositor
    .screenshot_source(&CapturedImage {
      width: 400,
      height: 300,
      rgba: [0, 0, 0, 255].repeat(400 * 300),
    })
    .expect("the source uploads");
  // Corners rounded by 108 pixels: the curve crosses the canvas's diagonal
  // about 31.6 pixels in from its corner.
  let mut settings = settings();
  settings.background_radius_percent = 10.0;
  let prepared = prepared_arrows(&[focus()], (400, 300), &settings, None, None, None)
    .expect("the annotations prepare");
  let output = target();
  compositor
    .draw_with_camera(
      &view(&output),
      &source,
      &settings,
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: false,
        seconds: 0.0,
      },
      None,
      None,
      &prepared,
    )
    .expect("the canvas draws");
  let pixels = gpu().read_texture(&output).expect("the target reads back");
  let alpha: Vec<u8> = (0..64)
    .map(|at: usize| pixels[(at * 1920 + at) * 4 + 3])
    .collect();
  let steepest = alpha
    .windows(2)
    .map(|pair| pair[1].abs_diff(pair[0]))
    .max()
    .unwrap_or(0);
  assert!(
    alpha[20] == 0 && alpha[44] == 255 && steepest > 120,
    "the corner is cut cleanly, not blurred: {alpha:?}"
  );
}
