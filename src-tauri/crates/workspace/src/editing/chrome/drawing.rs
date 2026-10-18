// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The selection chrome over a viewport `viewport_size` physical pixels: the
/// size it is drawn at, its vertices and the constants they all share.
#[allow(clippy::too_many_arguments)]
pub(crate) fn chrome(
  viewport_size: (u32, u32),
  frame: Option<[f32; 4]>,
  radius_point: Option<[f32; 2]>,
  crop_image: Option<[f32; 4]>,
  // The selected layer's corner radius, as a percentage of the crop
  // rectangle's shorter side, so the shade rounds with the result.
  crop_radius_percent: f64,
  guides: Option<(Option<f32>, Option<f32>, bool, bool)>,
  magnifier_box: Option<[f32; 4]>,
  // `Some` when the annotation chrome owns the screen. The layer's own
  // chrome stands down for as long as it does, matching
  // `annotation_owns_chrome`; `None` leaves it up.
  annotation: Option<&AnnotationChrome>,
  // The objects picked in the camera while they are, drawn in place of
  // every other chrome.
  objects: Option<&crate::editing::object_picking::ObjectChrome>,
  // A turned layer's frame and rings, and the turn its annotation chrome
  // is carried through.
  turned: &crate::editing::turn::TurnChrome,
  scale: f64,
  light: bool,
) -> ((u32, u32), Vec<Vertex>, RenderConstants) {
  let size = (viewport_size.0.max(2), viewport_size.1.max(2));
  let scale = scale.max(0.1);
  let view = Size {
    width: f64::from(size.0) / scale,
    height: f64::from(size.1) / scale,
  };
  let mut constants = RenderConstants::new(light);
  if let Some(box_rect) = magnifier_box.filter(|rect| rect[2] > 0.0) {
    constants.magnifier_box = box_rect;
    // The workspace draws the loupe's pixels below this transparent chrome;
    // the chrome only owns the cutout here.
    constants.magnifier_flags[1] = 1;
  }
  let mut vertices = Vec::with_capacity(96);
  // The arrow chrome draws only its own grips: the layer's selection and
  // crop chrome stand down for as long as it has the pointer.
  if let Some(objects) = objects {
    crate::editing::object_picking::add_objects(
      &mut vertices,
      &mut constants,
      view,
      objects,
      scale,
    );
  } else if let Some(annotation) = annotation {
    // A turned layer's annotations are turned with it, and their grips too.
    match turned.annotations {
      Some(forward) => {
        super::annotation::add_turned(&mut vertices, view, annotation, forward, scale)
      }
      None => super::annotation::add_upright(&mut vertices, view, annotation, scale),
    }
  } else if let Some(frame) = frame {
    let logical_frame = logical_rect(frame, scale);
    if let Some(image) = crop_image.filter(|rect| rect[2] >= 0.0) {
      osc_gpu::add_crop(
        &mut vertices,
        view,
        logical_frame,
        logical_rect(image, scale),
        scale,
        crop_radius_percent,
      );
    } else if let Some(frame) = turned.frame {
      let [a, b, c, d, dot] = frame.map(|[x, y]| Point {
        x: f64::from(x) / scale,
        y: f64::from(y) / scale,
      });
      osc_gpu::add_turned_selection(&mut vertices, view, [a, b, c, d], Some(dot), scale);
    } else {
      let radius_percent = radius_point
        .filter(|point| point[0].is_finite() && point[1].is_finite())
        .map(|point| {
          let shortest = logical_frame
            .size
            .width
            .min(logical_frame.size.height)
            .max(1.0);
          (((f64::from(point[0] - frame[0]) / scale - 10.0) / (shortest * 0.55)) * 100.0)
            .clamp(0.0, 50.0)
        });
      osc_gpu::add_selection(
        &mut vertices,
        view,
        logical_frame,
        scale,
        radius_percent.unwrap_or_default(),
        radius_percent.is_some(),
      );
    }
  }
  if let Some(rings) = turned.rings.as_ref().filter(|_| objects.is_none()) {
    constants.turn_rings = rings.rows;
    osc_gpu::add_quad(
      &mut vertices,
      view,
      logical_rect(rings.quad, scale),
      crate::editing::turn::RING_KIND,
    );
  }
  // A turned layer's annotation guides run across it, and are carried with
  // it; the layer's own guides are the canvas's, which never turns.
  let guides = guides.filter(|_| objects.is_none());
  match (guides, turned.annotations.filter(|_| annotation.is_some())) {
    (Some(guides), Some(forward)) => {
      super::annotation::add_turned_guides(&mut vertices, view, guides, forward, scale);
    }
    (Some((x, y, x_object, y_object)), None) => {
      let half = 0.5 / scale;
      if let Some(x) = x {
        let x = f64::from(x) / scale;
        osc_gpu::add_pixel_aligned_quad(
          &mut vertices,
          view,
          Rect::from_xywh(x - half, 0.0, half * 2.0, view.height),
          scale,
          if x_object { 5 } else { 4 },
        );
      }
      if let Some(y) = y {
        let y = f64::from(y) / scale;
        osc_gpu::add_pixel_aligned_quad(
          &mut vertices,
          view,
          Rect::from_xywh(0.0, y - half, view.width, half * 2.0),
          scale,
          if y_object { 5 } else { 4 },
        );
      }
    }
    (None, _) => {}
  }
  (size, vertices, constants)
}
