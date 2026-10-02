// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Turning a layer's annotations into what the compositor draws this frame:
//! the flattening, the placement into canvas pixels, and the exposure
//! sampling for an annotation that moved since the shutter opened, so a
//! moving arrow smears along the path it actually travelled.

use super::{PreparedArrows, PreparedType, PreviewArrow, PreviewSample, MAX_EXPOSURE_SAMPLES};
use crate::editor::annotations::exposure::{annotation_travel, highlight_travel, magnify_travel};
use crate::editor::annotations::native::{native_annotations, NativeAnnotations};
use crate::editor::annotations::redact::native::RedactSource;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::text::geometry::HEAD_ALIGN_MASK;
use crate::editor::annotations::text::typing::TypingMarks;
use crate::editor::annotations::{Annotation, AnnotationKind};
use crate::screenshots::{output_placement, CapturedImage};

/// Which geometry each kind prepares from its placed points.
mod prepare_kind;
/// A layer's annotations resolved once and placed per draw.
mod source;

pub(crate) use source::SourceAnnotations;

/// Prepares the annotations for one composition, in canvas pixels, with those
/// under the camera first.
///
/// Returns the arrows and where the above-camera run starts, which is what the
/// shader's two passes are bounded by. Points arrive in the source's own pixels
/// and are placed through the same `output_placement` the picture is, so an
/// annotation stays glued to what it points at through a crop or a resize. A
/// size is in points and drawn at the capture's scale, straight into output
/// pixels - that is what keeps an annotation's weight on the canvas instead of
/// growing with the picture. `annotations` is the list this frame draws. A still passes the
/// document's own; a video export passes the annotations its timeline clips
/// resolve to at that frame, which is why the list is given rather than read
/// from `settings`. `picture` is a screenshot's own pixels, which its
/// redactions read their fills from; a video frame has none, and its
/// redactions take the fills held from their clips' frames.
pub(crate) fn prepared_arrows(
  annotations: &[Annotation],
  source: (u32, u32),
  settings: &crate::screenshots::ScreenshotOutputSettings,
  picture: Option<&CapturedImage>,
  // The halo this composition carries, if the hovered arrow belongs to it:
  // its place in this layer's list and the halo's width in canvas pixels.
  halo: Option<(usize, f32)>,
  // The box being typed into, if it is on this layer: its place in the list,
  // and the caret and selection it is drawn with.
  typing: Option<(usize, TypingMarks)>,
) -> Result<PreparedArrows, String> {
  if annotations.is_empty() || source.0 == 0 || source.1 == 0 {
    return Ok(PreparedArrows::default());
  }
  let placement = output_placement(source.0, source.1, settings)?;
  Ok(SourceAnnotations::new(annotations, source, settings, picture).placed(placement, halo, typing))
}

/// The same annotations in whatever pixels they are drawn in: `offset` and
/// `scale` carry a point there, and `size_scale` a point of size. The live
/// overlay passes the identity, its annotations arriving in its layer pixels.
pub(crate) fn placed_arrows(
  annotations: &[Annotation],
  offset: (f64, f64),
  scale: (f64, f64),
  size_scale: f64,
  halo: Option<(usize, f32)>,
  typing: Option<(usize, TypingMarks)>,
) -> PreparedArrows {
  if annotations.is_empty() {
    return PreparedArrows::default();
  }
  placed_native(
    native_annotations(annotations, RedactSource::None, size_scale),
    offset,
    scale,
    halo,
    typing,
  )
}

/// Flattened annotations placed into drawn pixels. The live overlay prepares
/// through this too, so both draw from one resolved list rather than from
/// two readings of the document.
fn placed_native(
  native: NativeAnnotations,
  offset: (f64, f64),
  scale: (f64, f64),
  halo: Option<(usize, f32)>,
  typing: Option<(usize, TypingMarks)>,
) -> PreparedArrows {
  let NativeAnnotations { items, data } = native;
  let annotations = &items;
  let place = |point: [f32; 2]| {
    [
      (offset.0 + f64::from(point[0]) * scale.0) as f32,
      (offset.1 + f64::from(point[1]) * scale.1) as f32,
    ]
  };
  let mut prepared = PreparedArrows {
    arrows: Vec::with_capacity(annotations.len()),
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
      let shape = |reveal: AnnotationReveal| {
        prepare_kind::prepared_geometry(annotation, [a, b, c], reveal, scale.0 as f32)
      };
      let geometry = shape(annotation.reveal);
      let mut arrow = PreviewArrow::new(
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
      // The points arrive placed already, so the axis scale travel is
      // measured in is the identity. A text box's travel reads its pointer
      // and its block as they are held, which are never placed, a
      // highlight's and a stroke's the sweep their records keep, and a
      // magnifier's its loupe size, placed here as `prepared_geometry` does.
      let identity = [1.0, 1.0];
      let (width, reveal) = (annotation.width, annotation.reveal);
      let travel = match annotation.shape_kind() {
        kind @ AnnotationKind::Text => annotation_travel(
          kind,
          a,
          annotation.p1,
          annotation.p2,
          identity,
          width,
          reveal,
        ),
        AnnotationKind::Highlight | AnnotationKind::Draw => {
          highlight_travel(a, b, identity, annotation.params[2], reveal)
        }
        AnnotationKind::Magnify => magnify_travel(
          [a, b, c],
          annotation.params[0] * scale.0 as f32,
          identity,
          reveal,
        ),
        kind => annotation_travel(kind, a, b, c, identity, width, reveal),
      };
      let count = exposure_sample_count(travel, reveal);
      if count == 0 {
        // A still annotation carries its reveal's opacity in its colour. The
        // shader only knows the colour, so it is folded in here.
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
          prepared.samples.push(PreviewSample::new(
            shape(sample),
            sample.opacity.clamp(0.0, 1.0),
          ));
        }
      }
      // A counter's number is set at its disc's radius and a text box's text
      // at its own type size, both as this frame draws them.
      let text = || {
        let start = annotation.data_offset as usize;
        data
          .text
          .get(start..start + annotation.data_count as usize)
          .map(|text| String::from_utf8_lossy(text).into_owned())
          .unwrap_or_default()
      };
      prepared.types.push(match annotation.shape_kind() {
        AnnotationKind::Counter => PreparedType {
          text: text(),
          size: arrow.geometry.rounding,
          style: 0,
          marks: None,
        },
        AnnotationKind::Text => PreparedType {
          text: text(),
          size: arrow.geometry.width,
          style: 1 + (annotation.head & HEAD_ALIGN_MASK),
          marks: typing
            .filter(|(typed, _)| *typed == index)
            .map(|(_, marks)| marks),
        },
        // Only a counter and a text box carry type.
        _ => PreparedType::default(),
      });
      prepared.arrows.push(arrow);
    }
    if above == 0 {
      prepared.below_camera = prepared.arrows.len() as u32;
    }
  }
  prepared.points = data.points;
  prepared.text = data.text;
  prepared
}

/// How many exposure samples an annotation that moved `travel` pixels this
/// frame needs: none when it has not moved, else enough that consecutive
/// samples are under a pixel apart, from eight to forty-eight.
fn exposure_sample_count(travel: f32, r: AnnotationReveal) -> u32 {
  if travel < 1.5 && (r.opacity - r.previous[3]).abs() < 0.01 {
    return 0;
  }
  ((travel / 0.75).ceil() + 1.0).clamp(8.0, MAX_EXPOSURE_SAMPLES as f32) as u32
}

#[cfg(test)]
mod tests;
