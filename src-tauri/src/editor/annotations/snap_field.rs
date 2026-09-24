// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What one gesture can land on, gathered when it begins.

use super::{AnchorBoxes, Annotation, AnnotationPoint, Axis, AxisGuide, SnapBox};
use std::sync::Arc;

/// How far the canvas's inset guides sit from each edge, as a share of the
/// source's shorter side. The layer engines' own inset.
const CANVAS_INSET: f64 = 0.02;

/// Everything one gesture can snap to, built once when it begins.
///
/// The annotation being edited is excluded when the field is built: a counter
/// that could see its own disc would never leave the place it started.
pub(crate) struct SnapField {
  pub(crate) guides_x: Vec<AxisGuide>,
  pub(crate) guides_y: Vec<AxisGuide>,
  /// Every other annotation's alignment rectangle, which is also what a gap
  /// is measured between.
  pub(crate) boxes: Vec<SnapBox>,
  /// Source pixels per output pixel, which is what turns a counter's disc
  /// diameter into a radius. Zero where the picture's drawn width is not
  /// known, which leaves every disc a point and aligns centres - the rule
  /// this engine began with - rather than boxes of the wrong size.
  source_per_output: f64,
  /// The frame's detected elements, which arrive on a blocking thread and may
  /// land part way through the gesture.
  pub(crate) anchors: Option<Arc<AnchorBoxes>>,
}

impl SnapField {
  /// The canvas's own lines and every other counter's disc, in the source's
  /// pixels. `edited` is the annotation the gesture holds and `image_width`
  /// how wide this pane's picture is drawn, in output pixels - the space a
  /// style's width is measured in.
  pub(crate) fn new(
    source: (u32, u32),
    annotations: &[Annotation],
    edited: &str,
    image_width: f64,
  ) -> Self {
    let width = f64::from(source.0.max(1));
    let height = f64::from(source.1.max(1));
    let inset = width.min(height) * CANVAS_INSET;
    let canvas = |extent: f64| {
      [inset, extent / 2.0, extent - inset].map(|position| AxisGuide {
        position,
        object: false,
      })
    };
    let source_per_output = if image_width.is_finite() && image_width > 0.0 {
      width / image_width
    } else {
      0.0
    };
    let boxes: Vec<SnapBox> = annotations
      .iter()
      .filter(|annotation| annotation.id != edited)
      .filter_map(|annotation| {
        annotation
          .shape
          .field_box(annotation.style.width, source_per_output)
      })
      .collect();
    let mut field = Self {
      guides_x: canvas(width).to_vec(),
      guides_y: canvas(height).to_vec(),
      boxes,
      source_per_output,
      anchors: None,
    };
    // Both edges and the centre, so a disc can line up with the side of
    // another one as readily as with the middle of it.
    for bounds in &field.boxes {
      let object = |position| AxisGuide {
        position,
        object: true,
      };
      field.guides_x.extend(bounds.lines(Axis::X).map(object));
      field.guides_y.extend(bounds.lines(Axis::Y).map(object));
    }
    field
  }

  /// A counter's disc radius in source pixels. `width` is the disc's
  /// diameter in output pixels, which is what a style carries.
  pub(crate) fn radius(&self, width: f64) -> f64 {
    disc_radius(width, self.source_per_output)
  }

  /// A counter's disc in source pixels.
  pub(crate) fn disc(&self, center: AnnotationPoint, width: f64) -> SnapBox {
    SnapBox::disc(center, self.radius(width))
  }

  /// Source pixels per output pixel: what turns a size a style carries into
  /// the space the field is in.
  pub(crate) fn source_per_output(&self) -> f64 {
    self.source_per_output
  }
}

pub(crate) fn disc_radius(width: f64, source_per_output: f64) -> f64 {
  width.max(0.0) / 2.0 * source_per_output
}
