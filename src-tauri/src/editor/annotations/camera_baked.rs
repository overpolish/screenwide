// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A camera's annotations in One video, where the camera is drawn inside the
//! screen's composition rather than as a picture of its own.
//!
//! The screen's composition draws its annotations in two runs, the second
//! over the camera. The camera's annotations join that second run: each is
//! carried from the camera's pixels, through where the camera image sits on
//! the canvas, into the screen's pixels. They are not clipped to the camera's
//! frame, so an arrow can reach from the camera into the screen.
//!
//! A size is in points and drawn at its picture's capture scale, so on its
//! own a camera annotation would keep its weight however small the camera is
//! shown. Drawn with the camera, it shrinks with it instead, but never below
//! half its weight: a camera bubble is small, and its arrows have to stay
//! readable.

use super::{Annotation, AnnotationKind, AnnotationPoint};
use crate::editor::CameraOverlaySettings;
use crate::screenshots::ScreenshotOutputSettings;

/// How far a camera annotation's weight follows the camera down.
const SMALLEST_WEIGHT: f64 = 0.5;

/// `camera`'s annotations, in `camera_source` pixels, carried onto the
/// screen's composition to be drawn over the camera. `overlay` places the
/// camera image in a canvas `overlay_canvas` pixels in size; `screen` is the
/// screen's output and `screen_source` the pixels its annotations are in.
/// Only the kinds that mark the picture are carried: an effect reads or
/// changes its own picture's frames, which a baked camera does not have.
pub(crate) fn camera_annotations_on_screen(
  camera: &[Annotation],
  camera_source: (u32, u32),
  overlay: CameraOverlaySettings,
  overlay_canvas: (f64, f64),
  screen: &ScreenshotOutputSettings,
  screen_source: (u32, u32),
) -> Vec<Annotation> {
  if camera.is_empty()
    || camera_source.0 == 0
    || camera_source.1 == 0
    || screen_source.0 == 0
    || overlay_canvas.0 <= 0.0
    || overlay_canvas.1 <= 0.0
  {
    return Vec::new();
  }
  // The overlay is placed on the screen output it was set up against, which
  // a scaled export draws at another size.
  let horizontal = f64::from(screen.width) / overlay_canvas.0;
  let vertical = f64::from(screen.height) / overlay_canvas.1;
  let image_width = overlay.camera_width * horizontal;
  let image_height = image_width * f64::from(camera_source.1) / f64::from(camera_source.0);
  let left = overlay.camera_x * horizontal - image_width / 2.0;
  let top = overlay.camera_y * vertical - image_height / 2.0;
  let camera_scale = image_width / f64::from(camera_source.0);
  let screen_scale = screen.image_width / f64::from(screen_source.0);
  if !(camera_scale.is_finite()
    && camera_scale > 0.0
    && screen_scale.is_finite()
    && screen_scale > 0.0)
  {
    return Vec::new();
  }
  let carry = |point: AnnotationPoint| AnnotationPoint {
    x: (left + point.x * camera_scale - screen.image_x) / screen_scale,
    y: (top + point.y * camera_scale - screen.image_y) / screen_scale,
  };
  // A camera is captured one pixel to the point; the screen's points are its
  // own capture's, which the weight is handed over in.
  let weight = camera_scale.clamp(SMALLEST_WEIGHT, 1.0) / screen.size_scale();
  camera
    .iter()
    .filter(|annotation| marks_the_picture(annotation.shape.kind()))
    .map(|annotation| {
      let mut carried = annotation.clone();
      carried.shape = annotation.shape.mapped(carry);
      carried.style.width *= weight;
      carried.above_camera = true;
      carried
    })
    .collect()
}

fn marks_the_picture(kind: AnnotationKind) -> bool {
  matches!(
    kind,
    AnnotationKind::Arrow
      | AnnotationKind::Counter
      | AnnotationKind::Text
      | AnnotationKind::Shape
      | AnnotationKind::Draw
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::arrow::model::new_arrow;
  use crate::editor::annotations::AnnotationShape;

  fn point(x: f64, y: f64) -> AnnotationPoint {
    AnnotationPoint { x, y }
  }

  /// A 1,000 by 500 canvas whose screen picture, 2,000 source pixels wide, is
  /// drawn at half size from (0, 0), with a 400 by 300 camera image drawn
  /// 200 pixels wide centred on (800, 400).
  fn carried(camera_width: f64) -> Annotation {
    let mut screen = crate::screenshots::test_output_settings(1_000, 500);
    screen.image_x = 0.0;
    screen.image_y = 0.0;
    screen.image_width = 1_000.0;
    let overlay = CameraOverlaySettings {
      camera_x: 800.0,
      camera_y: 400.0,
      camera_width,
      frame_height: 150.0,
      frame_width: 200.0,
      frame_x: 700.0,
      frame_y: 325.0,
      radius_percent: 50.0,
    };
    let arrow = new_arrow("a".to_owned(), point(0.0, 0.0), point(400.0, 300.0), None);
    let mut carried = camera_annotations_on_screen(
      std::slice::from_ref(&arrow),
      (400, 300),
      overlay,
      (1_000.0, 500.0),
      &screen,
      (2_000, 1_000),
    );
    assert_eq!(carried.len(), 1);
    carried.remove(0)
  }

  #[test]
  fn a_camera_arrow_lands_where_the_camera_image_is_drawn() {
    let arrow = carried(200.0);
    // The camera image spans canvas (700, 325) to (900, 475); the screen
    // source is two of its pixels to one canvas pixel.
    let AnnotationShape::Arrow { start, end, .. } = arrow.shape else {
      unreachable!()
    };
    assert_eq!(start, point(1_400.0, 650.0));
    assert_eq!(end, point(1_800.0, 950.0));
    assert!(arrow.above_camera);
  }

  #[test]
  fn its_weight_follows_the_camera_down_to_half() {
    let full = new_arrow("a".to_owned(), point(0.0, 0.0), point(1.0, 1.0), None)
      .style
      .width;
    // Half a pixel to the camera's pixel: half the weight.
    assert_eq!(carried(200.0).style.width, full * 0.5);
    // A quarter: still half.
    assert_eq!(carried(100.0).style.width, full * 0.5);
    // Drawn larger than it was captured: its own weight.
    assert_eq!(carried(800.0).style.width, full);
  }
}
