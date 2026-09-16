// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Turning a layer's marks into what the compositor draws this frame.
//!
//! The twin of `screenwide_bind_annotations` in
//! `gpu_compositor_macos_annotations.h`: the same flattening, the same
//! placement into canvas pixels, and the same exposure sampling for a mark
//! that moved since the shutter opened, so a moving arrow smears along the
//! path it actually travelled on both backends.

use super::*;
use crate::editor::annotations::geometry::prepare_arrow;
use crate::editor::annotations::native::{native_annotations, NativeAnnotation};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::Annotation;
use crate::screenshots::output_placement;

/// Prepares the marks for one composition, in canvas pixels, with those
/// under the camera first.
///
/// Returns the arrows and where the above-camera run starts, which is what
/// the shader's two passes are bounded by. Points arrive in the source's own
/// pixels and are placed through the same `output_placement` the picture is,
/// so a mark stays glued to what it points at through a crop or a resize.
/// The stroke is already in output pixels - that is what keeps a mark's
/// weight on the canvas instead of growing with the picture - so it is used
/// as it comes.
/// `annotations` is the list this frame draws. A still passes the document's
/// own; a video export passes the marks its timeline clips resolve to at that
/// frame, which is why the list is given rather than read from `settings`.
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
  // The same flattening the Metal backend presents through, so both prepare
  // from one resolved list rather than from two readings of the document.
  let marks = native_annotations(annotations);
  let marks = &marks.items[..marks.count as usize];
  let placement = output_placement(source.0, source.1, settings)?;
  let scale_x = f64::from(placement.image_width) / f64::from(source.0);
  let scale_y = f64::from(placement.image_height) / f64::from(source.1);
  let canvas = |point: [f32; 2]| {
    [
      (placement.image_x + f64::from(point[0]) * scale_x) as f32,
      (placement.image_y + f64::from(point[1]) * scale_y) as f32,
    ]
  };
  let mut prepared = compositor::PreparedArrows {
    arrows: Vec::with_capacity(marks.len()),
    ..Default::default()
  };
  // Two passes rather than a sort: within each side the marks have to keep
  // the order their layer stores them in, so a later mark paints over an
  // earlier one.
  for above in [0, 1] {
    for (index, mark) in marks
      .iter()
      .enumerate()
      .filter(|(_, mark)| mark.above_camera == above)
    {
      let (a, b, c) = (canvas(mark.p0), canvas(mark.p1), canvas(mark.p2));
      let mut arrow = compositor::PreviewArrow::new(
        prepare_arrow(a, b, c, mark.width, mark.head, mark.reveal),
        mark.color,
        // The halo is preview chrome: `native_annotations` never sets it, so
        // nothing the export composes can carry one.
        halo
          .filter(|(hovered, _)| *hovered == index)
          .map_or(0.0, |(_, width)| width),
      );
      let count = exposure_sample_count(mark, a, b, c);
      if count == 0 {
        // A still mark carries its reveal's opacity in its colour. The shader
        // only knows the colour, so it is folded in here, exactly as the
        // Metal compositor's `draw->color[3] *= reveal.opacity` does.
        arrow.color[3] *= mark.reveal.opacity.clamp(0.0, 1.0);
      } else {
        // A moving mark is drawn at every sample between the shutter start
        // and now, each at the reveal it had then; the shader averages them.
        arrow.sample_first = prepared.samples.len() as u32;
        arrow.sample_count = count;
        let reveal = mark.reveal;
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
            prepare_arrow(a, b, c, mark.width, mark.head, sample),
            sample.opacity.clamp(0.0, 1.0),
          ));
        }
      }
      prepared.arrows.push(arrow);
    }
    if above == 0 {
      prepared.below_camera = prepared.arrows.len() as u32;
    }
  }
  Ok(prepared)
}

/// How many exposure samples a mark needs this frame: none when it has not
/// moved, else enough that consecutive samples are under a pixel apart, in
/// the same eight-to-forty-eight band the Metal compositor uses. The twin of
/// `screenwide_annotation_sample_count`.
fn exposure_sample_count(mark: &NativeAnnotation, a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> u32 {
  let r = mark.reveal;
  let length = (b[0] - a[0]).hypot(b[1] - a[1]) + (c[0] - b[0]).hypot(c[1] - b[1]);
  let mut travel = length
    * (r.low - r.previous[0])
      .abs()
      .max((r.high - r.previous[1]).abs());
  travel += mark.width * 4.0 * (r.scale - r.previous[2]).abs();
  if travel < 1.5 && (r.opacity - r.previous[3]).abs() < 0.01 {
    return 0;
  }
  ((travel / 0.75).ceil() + 1.0).clamp(8.0, compositor::MAX_EXPOSURE_SAMPLES as f32) as u32
}

#[cfg(test)]
#[path = "arrow_tests.rs"]
mod tests;
