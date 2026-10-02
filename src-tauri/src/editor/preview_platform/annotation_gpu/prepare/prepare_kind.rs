// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which geometry each kind of annotation prepares, from its record's points
//! placed into drawn pixels. The twin of `screenwide_prepare_annotation`.

use super::*;
use crate::editor::annotations::arrow::geometry::prepare_arrow;
use crate::editor::annotations::counter::geometry::prepare_counter;
use crate::editor::annotations::freehand::geometry::prepare_freehand;
use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::highlight::geometry::prepare_highlight;
use crate::editor::annotations::magnify::geometry::prepare_magnify;
use crate::editor::annotations::native::NativeAnnotation;
use crate::editor::annotations::outline::geometry::prepare_shape;
use crate::editor::annotations::redact::geometry::prepare_redact;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::spotlight::geometry::prepare_spotlight;
use crate::editor::annotations::text::geometry::prepare_text;

/// `annotation` prepared at `reveal`, its points `p0`, `p1` and `p2` placed at
/// `a`, `b` and `c` by a placement that stretches a length `scale` times.
///
/// A counter keeps its centre in `p0` and its aim in `p1[0]`, so only the
/// centre is placed: the disc's diameter is in output pixels, as an arrow's
/// stroke is, and an angle is the same angle in either space. A text box
/// keeps its pointer, held against the box, in `p1` and its text block's
/// size, in output pixels, in `p2`; neither is placed. A shape's and a
/// spotlight's `p1` is never placed either. A magnifier places all three -
/// its zoom area's corners and its loupe's centre - and reads its loupe's
/// size out of `params`, stretched by `scale`, and its rounding beside it.
pub(super) fn prepared_geometry(
  annotation: &NativeAnnotation,
  [a, b, c]: [[f32; 2]; 3],
  reveal: AnnotationReveal,
  scale: f32,
) -> ArrowGeometry {
  match annotation.shape_kind() {
    AnnotationKind::Counter => prepare_counter(a, annotation.width, annotation.p1[0], reveal),
    AnnotationKind::Text => prepare_text(
      a,
      annotation.p1,
      annotation.p2,
      annotation.width,
      annotation.head,
      reveal,
    ),
    AnnotationKind::Arrow => prepare_arrow(a, b, c, annotation.width, annotation.head, reveal),
    AnnotationKind::Redact => prepare_redact(a, c, annotation.width),
    // A highlight is placed by where the source's origin and its pixel
    // (1, 1) land, and keeps its tone in `p2`, which is never placed.
    AnnotationKind::Highlight => prepare_highlight(a, b, annotation.p2, reveal),
    AnnotationKind::Shape => prepare_shape(
      a,
      c,
      annotation.p1[0],
      annotation.p1[1],
      annotation.width,
      reveal,
    ),
    AnnotationKind::Spotlight => prepare_spotlight(a, c, annotation.p1[0], annotation.p1[1]),
    // A stroke is placed by where its box's corner, that corner moved a
    // source pixel, and its far corner land.
    AnnotationKind::Draw => prepare_freehand(a, b, c, annotation.width, reveal),
    AnnotationKind::Magnify => prepare_magnify(
      a,
      b,
      c,
      annotation.params[0] * scale,
      annotation.params[1],
      annotation.width,
      reveal,
    ),
  }
}
