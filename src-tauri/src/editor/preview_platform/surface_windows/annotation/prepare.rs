// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Turning a layer's annotations into what the compositor draws this frame.
//!
//! The twin of `screenwide_bind_annotations` in
//! `gpu_compositor_macos_annotations.h`: the same flattening, the same
//! placement into canvas pixels, and the same exposure sampling for an
//! annotation that moved since the shutter opened, so a moving arrow smears
//! along the path it actually travelled on both backends.

use super::*;
use crate::editor::annotations::arrow::geometry::prepare_arrow;
use crate::editor::annotations::counter::geometry::prepare_counter;
use crate::editor::annotations::exposure::annotation_travel;
use crate::editor::annotations::native::{native_annotations, NativeAnnotation};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::Annotation;
use crate::screenshots::output_placement;

/// Prepares the annotations for one composition, in canvas pixels, with those
/// under the camera first.
///
/// Returns the arrows and where the above-camera run starts, which is what the
/// shader's two passes are bounded by. Points arrive in the source's own pixels
/// and are placed through the same `output_placement` the picture is, so an
/// annotation stays glued to what it points at through a crop or a resize. The
/// stroke is already in output pixels - that is what keeps an annotation's
/// weight on the canvas instead of growing with the picture - so it is used as
/// it comes. `annotations` is the list this frame draws. A still passes the
/// document's own; a video export passes the annotations its timeline clips
/// resolve to at that frame, which is why the list is given rather than read
/// from `settings`.
pub(crate) fn prepared_arrows(
  annotations: &[Annotation],
  source: (u32, u32),
  settings: &ScreenshotOutputSettings,
  // The halo this composition carries, if the hovered arrow belongs to it:
  // its place in this layer's list and the halo's width in canvas pixels.
  halo: Option<(usize, f32)>,
) -> Result<compositor::PreparedArrows, String> {
  if annotations.is_empty() || source.0 == 0 || source.1 == 0 {
    return Ok(compositor::PreparedArrows::default());
  }
  let placement = output_placement(source.0, source.1, settings)?;
  Ok(placed_arrows(
    annotations,
    (placement.image_x, placement.image_y),
    (
      f64::from(placement.image_width) / f64::from(source.0),
      f64::from(placement.image_height) / f64::from(source.1),
    ),
    halo,
  ))
}

/// The same annotations in whatever pixels they are drawn in: `offset` and
/// `scale` carry a point from the source's own pixels into them. The still
/// passes its output placement; the live overlay passes the identity, because
/// its annotations arrive in the display's layer pixels already.
pub(crate) fn placed_arrows(
  annotations: &[Annotation],
  offset: (f64, f64),
  scale: (f64, f64),
  halo: Option<(usize, f32)>,
) -> compositor::PreparedArrows {
  if annotations.is_empty() {
    return compositor::PreparedArrows::default();
  }
  // The same flattening the Metal backend presents through, so both prepare
  // from one resolved list rather than from two readings of the document.
  let native = native_annotations(annotations);
  let annotations = &native.items[..native.count as usize];
  let place = |point: [f32; 2]| {
    [
      (offset.0 + f64::from(point[0]) * scale.0) as f32,
      (offset.1 + f64::from(point[1]) * scale.1) as f32,
    ]
  };
  let mut prepared = compositor::PreparedArrows {
    arrows: Vec::with_capacity(annotations.len()),
    points: native.data.points[..native.data.point_count as usize].to_vec(),
    text: native.data.text[..native.data.text_len as usize].to_vec(),
    ..Default::default()
  };
  // keep the order their layer stores them in, so a later annotation paints
  // over an earlier one.
  for above in [0, 1] {
    for (index, annotation) in annotations
      .iter()
      .enumerate()
      .filter(|(_, annotation)| annotation.above_camera == above)
    {
      let (a, b, c) = (
        place(annotation.p0),
        place(annotation.p1),
        place(annotation.p2),
      );
      // A counter keeps its centre in `p0` and its aim in `p1[0]`, so only
      // the centre is placed: the disc's diameter is in output pixels, as an
      // arrow's stroke is, and an angle is the same angle in either space.
      let geometry = match annotation.shape_kind() {
        AnnotationKind::Counter => {
          prepare_counter(a, annotation.width, annotation.p1[0], annotation.reveal)
        }
        AnnotationKind::Arrow => prepare_arrow(
          a,
          b,
          c,
          annotation.width,
          annotation.head,
          annotation.reveal,
        ),
      };
      let mut arrow = compositor::PreviewArrow::new(
        geometry,
        annotation.color,
        halo
          .filter(|(hovered, _)| *hovered == index)
          .map_or(0.0, |(_, width)| width),
        annotation.kind,
      );
      arrow.flags = annotation.flags;
      arrow.params = annotation.params;
      arrow.data_offset = annotation.data_offset;
      arrow.data_count = annotation.data_count;
      let count = exposure_sample_count(annotation, a, b, c);
      if count == 0 {
        // A still annotation carries its reveal's opacity in its colour. The
        // shader only knows the colour, so it is folded in here, exactly as the
        // Metal compositor's `draw->color[3] *= reveal.opacity` does.
        arrow.color[3] *= annotation.reveal.opacity.clamp(0.0, 1.0);
      } else {
        // A moving annotation is drawn at every sample between the shutter
        // start and now, each at the reveal it had then; the shader averages
        // them.
        arrow.sample_first = prepared.samples.len() as u32;
        arrow.sample_count = count;
        let reveal = annotation.reveal;
        for tap in 0..count {
          let t = (tap as f32 + 0.5) / count as f32;
          let lerp = |from: f32, to: f32| from + (to - from) * t;
          let sample = AnnotationReveal {
            low: lerp(reveal.previous[0], reveal.low),
            high: lerp(reveal.previous[1], reveal.high),
            scale: lerp(reveal.previous[2], reveal.scale),
            opacity: lerp(reveal.previous[3], reveal.opacity),
            previous: reveal.previous,
          };
          prepared.samples.push(compositor::PreviewSample::new(
            match annotation.shape_kind() {
              AnnotationKind::Arrow => {
                prepare_arrow(a, b, c, annotation.width, annotation.head, sample)
              }
              AnnotationKind::Counter => {
                prepare_counter(a, annotation.width, annotation.p1[0], sample)
              }
            },
            sample.opacity.clamp(0.0, 1.0),
          ));
        }
      }
      prepared.counters.push(match annotation.shape_kind() {
        AnnotationKind::Counter => (
          String::from_utf8_lossy(
            &native.data.text[annotation.data_offset as usize
              ..annotation.data_offset as usize + annotation.data_count as usize],
          )
          .into_owned(),
          arrow.geometry.rounding,
        ),
        AnnotationKind::Arrow => (String::new(), 0.0),
      });
      prepared.arrows.push(arrow);
    }
    if above == 0 {
      prepared.below_camera = prepared.arrows.len() as u32;
    }
  }
  prepared
}

/// How many exposure samples an annotation needs this frame: none when it has
/// not moved, else enough that consecutive samples are under a pixel apart, in
/// the same eight-to-forty-eight band the Metal compositor uses. The twin of
/// `screenwide_annotation_sample_count`.
fn exposure_sample_count(
  annotation: &NativeAnnotation,
  a: [f32; 2],
  b: [f32; 2],
  c: [f32; 2],
) -> u32 {
  let r = annotation.reveal;
  // The points arrive placed already, so the axis scale travel is measured
  // in is the identity.
  let travel = annotation_travel(
    annotation.shape_kind(),
    a,
    b,
    c,
    [1.0, 1.0],
    annotation.width,
    r,
  );
  if travel < 1.5 && (r.opacity - r.previous[3]).abs() < 0.01 {
    return 0;
  }
  ((travel / 0.75).ceil() + 1.0).clamp(8.0, compositor::MAX_EXPOSURE_SAMPLES as f32) as u32
}

#[cfg(test)]
#[path = "arrow_tests.rs"]
mod tests;
