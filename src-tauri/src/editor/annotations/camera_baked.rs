// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A camera's annotations in One video, where the camera is drawn inside the
//! screen's composition rather than as a picture of its own.
//!
//! A redaction or a highlight changes the camera's own pixels, so the camera
//! is composed on its own first, with those drawn into it and cut at its
//! edge, and the screen's composition draws that picture where the raw frame
//! went. Everything else on the camera joins the screen's second run, over
//! the camera: it is carried from the camera's pixels, through where the
//! camera image sits on the canvas, into the screen's pixels, and is never
//! clipped to the camera's frame, so an arrow can reach from the camera into
//! the screen, and a magnifier's loupe can sit beside the camera while it
//! shows the camera enlarged. A redaction is applied to the picture before
//! any mark is drawn, on the screen as here, so a mark is never under one
//! whatever their order; a highlight drawn into the camera's picture is under
//! the camera's marks the same way.
//!
//! A size is in points and drawn at its picture's capture scale, so on its
//! own a camera annotation would keep its weight however small the camera is
//! shown. Drawn over the camera, it shrinks with it instead, but never below
//! half its weight: a camera bubble is small, and its arrows have to stay
//! readable. Drawn into the camera's picture it is part of the picture,
//! sized in the camera's own pixels as the camera's own file has it: a
//! redaction's blocks and blur hide the same detail however large the camera
//! is shown, as the screen's do.

use super::{Annotation, AnnotationKind, AnnotationPoint};
use crate::editor::CameraOverlaySettings;
use crate::screenshots::ScreenshotOutputSettings;

/// How far a camera annotation's weight follows the camera down.
const SMALLEST_WEIGHT: f64 = 0.5;

/// Where one of a baked camera's annotations is drawn: into the camera's own
/// picture, or over it on the screen's composition. Each counts its place in
/// its own list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BakedPlace {
  Within(usize),
  Over(usize),
}

/// A baked camera's annotations, split by where they are drawn.
#[derive(Debug, Default)]
pub(crate) struct BakedCameraAnnotations {
  /// Drawn into the camera's picture, in its own pixels, as they were.
  pub(crate) within: Vec<Annotation>,
  /// Carried onto the screen's composition, over the camera.
  pub(crate) over: Vec<Annotation>,
}

/// Where each of `camera` is drawn, in order.
pub(crate) fn baked_places(camera: &[Annotation]) -> Vec<BakedPlace> {
  let (mut within, mut over) = (0, 0);
  let next = |list: &mut usize| {
    *list += 1;
    *list - 1
  };
  camera
    .iter()
    .map(|annotation| match annotation.shape.kind() {
      AnnotationKind::Redact | AnnotationKind::Highlight => BakedPlace::Within(next(&mut within)),
      _ => BakedPlace::Over(next(&mut over)),
    })
    .collect()
}

/// `camera`'s annotations, in `camera_source` pixels, split between the
/// camera's own picture and the screen's composition over it. `overlay`
/// places the camera image in a canvas `overlay_canvas` pixels in size;
/// `screen` is the screen's output and `screen_source` the pixels its
/// annotations are in.
pub(crate) fn baked_camera_annotations(
  camera: &[Annotation],
  camera_source: (u32, u32),
  overlay: CameraOverlaySettings,
  overlay_canvas: (f64, f64),
  screen: &ScreenshotOutputSettings,
  screen_source: (u32, u32),
) -> BakedCameraAnnotations {
  if camera.is_empty()
    || camera_source.0 == 0
    || camera_source.1 == 0
    || screen_source.0 == 0
    || overlay_canvas.0 <= 0.0
    || overlay_canvas.1 <= 0.0
  {
    return BakedCameraAnnotations::default();
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
    return BakedCameraAnnotations::default();
  }
  let carry = |point: AnnotationPoint| AnnotationPoint {
    x: (left + point.x * camera_scale - screen.image_x) / screen_scale,
    y: (top + point.y * camera_scale - screen.image_y) / screen_scale,
  };
  // A camera is captured one pixel to the point; the screen's points are its
  // own capture's, which the weight is handed over in. The weight is taken at
  // the size the camera was set up at, and an export drawn at another size
  // scales it with the rest of the picture.
  let weight = (camera_scale / horizontal).clamp(SMALLEST_WEIGHT, 1.0) * horizontal;
  let mut split = BakedCameraAnnotations::default();
  for (annotation, place) in camera.iter().zip(baked_places(camera)) {
    match place {
      BakedPlace::Within(_) => split.within.push(annotation.clone()),
      BakedPlace::Over(_) => {
        let mut carried = annotation.clone();
        carried.shape = annotation.shape.mapped(carry);
        carried.style.width *= weight / screen.size_scale();
        carried.above_camera = true;
        split.over.push(carried);
      }
    }
  }
  split
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::arrow::model::new_arrow;
  use crate::editor::annotations::redact::model::new_redact;
  use crate::editor::annotations::spotlight::model::new_spotlight;
  use crate::editor::annotations::AnnotationShape;

  fn point(x: f64, y: f64) -> AnnotationPoint {
    AnnotationPoint { x, y }
  }

  /// A 1,000 by 500 canvas whose screen picture, 2,000 source pixels wide, is
  /// drawn at half size from (0, 0), with a 400 by 300 camera image drawn
  /// `camera_width` pixels wide centred on (800, 400).
  fn split(camera: &[Annotation], camera_width: f64) -> BakedCameraAnnotations {
    split_on(camera, camera_width, (1_000.0, 500.0))
  }

  /// `split`, with the camera set up on a canvas `overlay_canvas` in size.
  fn split_on(
    camera: &[Annotation],
    camera_width: f64,
    overlay_canvas: (f64, f64),
  ) -> BakedCameraAnnotations {
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
    baked_camera_annotations(
      camera,
      (400, 300),
      overlay,
      overlay_canvas,
      &screen,
      (2_000, 1_000),
    )
  }

  fn arrow(id: &str) -> Annotation {
    new_arrow(id.to_owned(), point(0.0, 0.0), point(400.0, 300.0), None)
  }

  fn carried(camera_width: f64) -> Annotation {
    let mut over = split(&[arrow("a")], camera_width).over;
    assert_eq!(over.len(), 1);
    over.remove(0)
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
    let full = arrow("a").style.width;
    // Half a pixel to the camera's pixel: half the weight.
    assert_eq!(carried(200.0).style.width, full * 0.5);
    // A quarter: still half.
    assert_eq!(carried(100.0).style.width, full * 0.5);
    // Drawn larger than it was captured: its own weight.
    assert_eq!(carried(800.0).style.width, full);
  }

  /// Set up at 0.8 of its pixels on a canvas twice the size this export is
  /// drawn at: the weight it was set up with, then halved with the picture,
  /// rather than the half weight a camera shown at 0.4 would keep.
  #[test]
  fn an_export_drawn_smaller_scales_the_weight_it_was_set_up_with() {
    let full = arrow("a").style.width;
    let over = split_on(&[arrow("a")], 320.0, (2_000.0, 1_000.0)).over;
    assert!((over[0].style.width - full * 0.4).abs() < 1e-9);
  }

  /// Only the redaction is drawn into the camera's picture; the arrows on
  /// either side of it, the spotlight and the magnifier are all carried over
  /// the camera, never cut at its edge.
  #[test]
  fn only_what_changes_the_camera_s_pixels_is_drawn_into_it() {
    let redaction = new_redact("r".to_owned(), point(0.0, 0.0), None);
    let spotlight = new_spotlight("s".to_owned(), [point(0.0, 0.0), point(10.0, 10.0)], None);
    let mut magnifier = arrow("m");
    magnifier.shape = AnnotationShape::Magnify {
      start: point(0.0, 0.0),
      end: point(10.0, 10.0),
      loupe: point(50.0, 50.0),
      size: 40.0,
    };
    let camera = [
      arrow("under"),
      spotlight,
      redaction,
      arrow("over"),
      magnifier,
    ];
    assert_eq!(
      baked_places(&camera),
      vec![
        BakedPlace::Over(0),
        BakedPlace::Over(1),
        BakedPlace::Within(0),
        BakedPlace::Over(2),
        BakedPlace::Over(3),
      ]
    );
    let ids = |list: &[Annotation]| list.iter().map(|a| a.id.clone()).collect::<Vec<_>>();
    let split = split(&camera, 200.0);
    assert_eq!(ids(&split.within), ["r"]);
    assert_eq!(ids(&split.over), ["under", "s", "over", "m"]);
  }
}
