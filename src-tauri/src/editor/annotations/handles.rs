// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native handle conversion, independent of the owning workspace.

use super::bend::curve_midpoint;
use super::{Annotation, AnnotationPoint, AnnotationShape, AnnotationStyle};

/// A point in image-normalised space, in the source's pixels.
pub(crate) fn source_point(x: f64, y: f64, source: (u32, u32)) -> AnnotationPoint {
  AnnotationPoint {
    x: x * f64::from(source.0.max(1)),
    y: y * f64::from(source.1.max(1)),
  }
}

/// A point in the source's pixels, normalised over the image.
pub(crate) fn normalised_point(point: AnnotationPoint, source: (u32, u32)) -> (f64, f64) {
  (
    point.x / f64::from(source.0.max(1)),
    point.y / f64::from(source.1.max(1)),
  )
}

/// One arrow as the native chrome needs it: the three grips, normalised over
/// the source image - the two tips and the point of the curve at `t = 0.5`,
/// which is where the middle handle sits - and how far each head reaches back
/// from its tip, as a fraction of the image's drawn width.
///
/// The heads travel as a length rather than as a triangle because the native
/// side already has the tips and can take the aim from them; zero means that
/// end carries no head. The half-base is half the length, which is the
/// shader's four-to-two proportions. Eight doubles, matching the C
/// `ScreenwidePreviewAnnotation`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativeAnnotationHandles {
  pub(crate) start_x: f64,
  pub(crate) start_y: f64,
  pub(crate) middle_x: f64,
  pub(crate) middle_y: f64,
  pub(crate) end_x: f64,
  pub(crate) end_y: f64,
  pub(crate) start_head: f64,
  pub(crate) end_head: f64,
}

const _: () = assert!(std::mem::size_of::<NativeAnnotationHandles>() == 64);

/// How far a head reaches back from its tip, as a fraction of the image's
/// drawn width. The stroke is in output pixels and the image is drawn
/// `image_width` of them across, so the head's four widths become a share of
/// the picture the native side can place without knowing either number.
fn head_reach(style: &AnnotationStyle, wanted: bool, image_width: f64) -> f64 {
  if !wanted || image_width <= 0.0 || !image_width.is_finite() {
    return 0.0;
  }
  style.width.max(0.0) * HEAD_LENGTH_WIDTHS / image_width
}

/// The head's length as a multiple of the stroke width, matching the shader's
/// `annotation_head_length`.
const HEAD_LENGTH_WIDTHS: f64 = 4.0;

/// Every annotation's grips, in the order the layer stores them.
pub(crate) fn annotation_handles(
  annotations: &[Annotation],
  source: (u32, u32),
  image_width: f64,
) -> Vec<NativeAnnotationHandles> {
  annotations
    .iter()
    .map(|annotation| {
      let head = annotation.style.head;
      let AnnotationShape::Arrow {
        start,
        control,
        end,
      } = &annotation.shape;
      let (start_x, start_y) = normalised_point(*start, source);
      let (middle_x, middle_y) = normalised_point(curve_midpoint(*start, *control, *end), source);
      let (end_x, end_y) = normalised_point(*end, source);
      NativeAnnotationHandles {
        start_x,
        start_y,
        middle_x,
        middle_y,
        end_x,
        end_y,
        start_head: head_reach(
          &annotation.style,
          head == crate::editor::annotations::AnnotationHead::Both,
          image_width,
        ),
        end_head: head_reach(
          &annotation.style,
          head != crate::editor::annotations::AnnotationHead::None,
          image_width,
        ),
      }
    })
    .collect()
}
