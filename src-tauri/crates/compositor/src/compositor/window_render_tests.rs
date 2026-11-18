// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A picture drawn from a window of it, as the editor draws one larger than
//! the device takes, lands where the whole picture does, its redactions with
//! it, whether the window holds the picture at its own size or scaled down.

use super::render_test_helpers::{compositor, gpu, target_size, view};
use super::*;
use screenwide_annotations::redact::new_redact;
use screenwide_annotations::{Annotation, AnnotationPoint, AnnotationRedaction, AnnotationShape};
use screenwide_imaging::CapturedImage;
use screenwide_output::fixtures::test_output_settings;

const SOURCE: (u32, u32) = (400, 1600);
const VIEW: (u32, u32) = (300, 400);

/// Soft bands, which a scaled-down window still holds nearly as they are,
/// with a sharp mark every 100 rows that would show any shift.
fn picture() -> CapturedImage {
  let mut rgba = Vec::with_capacity(SOURCE.0 as usize * SOURCE.1 as usize * 4);
  for y in 0..SOURCE.1 {
    for x in 0..SOURCE.0 {
      let mark = y % 100 < 8 && x < 200;
      let shade = if mark { 0 } else { 128 + (y / 4 % 100) as u8 };
      rgba.extend_from_slice(&[shade, (x / 2) as u8, 200, 255]);
    }
  }
  CapturedImage {
    width: SOURCE.0,
    height: SOURCE.1,
    rgba,
  }
}

fn redaction(from: (f64, f64), to: (f64, f64), kind: AnnotationRedaction) -> Annotation {
  let point = |(x, y)| AnnotationPoint { x, y };
  let mut annotation = new_redact(format!("{kind:?}"), point(from), None);
  if let AnnotationShape::Redact { end, .. } = &mut annotation.shape {
    *end = point(to);
  }
  annotation.style.redaction = kind;
  annotation
}

/// The picture from its row 600 drawn into the view at `zoom` view pixels
/// to a picture pixel, from the whole picture or from `window`.
fn drawn(window: Option<([u32; 4], u32)>, zoom: f32) -> Vec<u8> {
  let compositor = compositor();
  let image = picture();
  let mut settings = test_output_settings(SOURCE.0, SOURCE.1);
  (settings.image_x, settings.image_y, settings.image_width) = (0.0, 0.0, f64::from(SOURCE.0));
  (settings.crop_x, settings.crop_y) = (0.0, 0.0);
  (settings.crop_width, settings.crop_height) = (f64::from(SOURCE.0), f64::from(SOURCE.1));
  settings.annotations = vec![
    redaction((40.0, 650.0), (360.0, 760.0), AnnotationRedaction::Blur),
    redaction(
      (40.0, 800.0),
      (360.0, 900.0),
      AnnotationRedaction::PixelateClassic,
    ),
  ];
  let prepared = crate::annotation_gpu::prepared_arrows(
    &settings.annotations,
    SOURCE,
    &settings,
    Some(&image),
    None,
    None,
  )
  .expect("the annotations prepare");
  let (source, source_window) = match window {
    Some((rect, scale)) => (
      compositor
        .screenshot_window(&image, rect, scale)
        .expect("the window uploads"),
      Some(SourceWindow {
        origin: (rect[0], rect[1]),
        scale,
      }),
    ),
    None => (
      compositor
        .screenshot_source(&image)
        .expect("the picture uploads"),
      None,
    ),
  };
  let output = target_size(VIEW);
  compositor
    .draw_layer(
      &view(&output),
      &source,
      &settings,
      &CanvasGeometry::of(SOURCE, &settings).expect("the canvas places"),
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: false,
        seconds: 0.0,
      },
      None,
      None,
      &prepared,
      LayerDraw {
        placement: Some(LayerPlacement {
          rect: [
            0.0,
            -600.0 * zoom,
            SOURCE.0 as f32 * zoom,
            SOURCE.1 as f32 * zoom,
          ],
          target: VIEW,
        }),
        band: None,
        source_window,
        clear: true,
        redaction_slot: 0,
        retained_source: false,
      },
    )
    .expect("the canvas draws");
  gpu().read_texture(&output).expect("the view reads back")
}

/// The largest difference between two views, and how many channels differ
/// by more than `tolerance`.
fn compare(a: &[u8], b: &[u8], tolerance: u8) -> (u8, usize) {
  let differences = a.iter().zip(b).map(|(a, b)| a.abs_diff(*b));
  let worst = differences.clone().max().unwrap_or(0);
  (
    worst,
    differences
      .filter(|&difference| difference > tolerance)
      .count(),
  )
}

#[test]
#[ignore = "requires a GPU adapter"]
fn a_window_at_the_pictures_own_size_draws_as_the_whole_picture() {
  let whole = drawn(None, 0.75);
  // The view shows the picture's rows 600 to 1134.
  let window = drawn(Some(([0, 500, 400, 1200], 1)), 0.75);
  let (worst, _) = compare(&whole, &window, 0);
  assert!(worst <= 1, "off by {worst}");
}

#[test]
#[ignore = "requires a GPU adapter"]
fn a_scaled_down_window_draws_the_picture_and_its_redactions_in_place() {
  // Half the view's pixels to a picture pixel: the window holds two picture
  // pixels to a texel, no coarser than the view shows.
  let whole = drawn(None, 0.5);
  // The view shows the picture's rows 600 to 1400.
  let window = drawn(Some(([0, 400, 400, 1500], 2)), 0.5);
  let (worst, _) = compare(&whole, &window, 0);
  // A texel that averages two of the picture's pixels reads a little apart
  // from a filtered read between them; a shifted picture or redaction would
  // be far apart.
  assert!(worst <= 12, "off by {worst}");
}
