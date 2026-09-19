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

/// One annotation as the native chrome needs it, normalised over the source
/// image.
///
/// An arrow fills every slot: the three grips - the two tips and the point of
/// the curve at `t = 0.5`, which is where the middle handle sits - how far each
/// head reaches back from its tip, and the stroke's own width, both as a
/// fraction of the image's drawn width. The heads travel as a length rather
/// than as a triangle because the native side already has the tips and can take
/// the aim from them; zero means that end carries no head. The half-base is
/// half the length, which is the shader's four-to-two proportions. `width`
/// rides separately because a headless annotation still has a stroke to pick,
/// and picking is the drawn shape exactly.
///
/// A counter reads the same slots differently: every point is the disc's
/// centre, `start_head` is where its tail points in radians clockwise from
/// east, `end_head` is zero, and `width` is the disc's diameter as a share of
/// the drawn width. Its one grip - the tail's tip - is placed from those by the
/// native side, which works in isotropic display points. `kind` says which
/// reading applies. Layer identity, index and kind follow the nine geometry
/// doubles, matching the C `ScreenwidePreviewAnnotation`.
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
  pub(crate) width: f64,
  pub(crate) layer_id: i32,
  pub(crate) index: u32,
  /// Zero is an arrow, one a counter: the native `ScreenwideAnnotationKind`.
  pub(crate) kind: u32,
  pub(crate) padding: u32,
}

const _: () = assert!(std::mem::size_of::<NativeAnnotationHandles>() == 88);

/// The native kinds, matching the shapes the compositor draws.
pub(crate) const HANDLE_KIND_ARROW: u32 = 0;
pub(crate) const HANDLE_KIND_COUNTER: u32 = 1;

/// One equal gap the chrome draws a bar across, normalised over the source:
/// where it starts and ends along its own axis, and where it sits across it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativeGapSpan {
  pub(crate) from: f64,
  pub(crate) to: f64,
  pub(crate) cross: f64,
}

/// What the snap chrome draws, in the same image-normalised space the grips
/// use, matching the C `ScreenwideAnnotationSnap`. `flags` says which members
/// are live; a default value is what puts the chrome away.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativeAnnotationSnap {
  pub(crate) flags: u32,
  /// Whether each guide came from another annotation rather than the canvas,
  /// which is what picks the line's colour.
  pub(crate) guide_x_object: u32,
  pub(crate) guide_y_object: u32,
  pub(crate) padding: u32,
  pub(crate) guide_x: f64,
  pub(crate) guide_y: f64,
  pub(crate) anchor_x: f64,
  pub(crate) anchor_y: f64,
  pub(crate) box_x: f64,
  pub(crate) box_y: f64,
  pub(crate) box_width: f64,
  pub(crate) box_height: f64,
  /// The two equal gaps each axis landed on, in order along that axis.
  pub(crate) gap_x: [NativeGapSpan; 2],
  pub(crate) gap_y: [NativeGapSpan; 2],
}

const _: () = assert!(std::mem::size_of::<NativeAnnotationSnap>() == 176);

/// Which members of [`NativeAnnotationSnap`] are live, matching the C
/// `ScreenwideAnnotationSnapFlag`.
pub(crate) const SNAP_FLAG_GUIDE_X: u32 = 1;
pub(crate) const SNAP_FLAG_GUIDE_Y: u32 = 1 << 1;
pub(crate) const SNAP_FLAG_ANCHOR: u32 = 1 << 2;
pub(crate) const SNAP_FLAG_GAP_X: u32 = 1 << 3;
pub(crate) const SNAP_FLAG_GAP_Y: u32 = 1 << 4;

/// One sample's snap as the native chrome needs it. The engine works in source
/// pixels and the chrome places everything inside the picture on screen, so
/// every length is normalised over the source here, once, exactly as
/// [`normalised_point`] does.
pub(crate) fn annotation_snap(
  result: &super::snap::SnapResult,
  source: (u32, u32),
) -> NativeAnnotationSnap {
  let width = f64::from(source.0.max(1));
  let height = f64::from(source.1.max(1));
  let mut snap = NativeAnnotationSnap::default();
  if let Some(guide) = result.guide_x {
    snap.flags |= SNAP_FLAG_GUIDE_X;
    snap.guide_x = guide.position / width;
    snap.guide_x_object = u32::from(guide.object);
  }
  if let Some(guide) = result.guide_y {
    snap.flags |= SNAP_FLAG_GUIDE_Y;
    snap.guide_y = guide.position / height;
    snap.guide_y_object = u32::from(guide.object);
  }
  if let Some(anchor) = result.anchor {
    snap.flags |= SNAP_FLAG_ANCHOR;
    let (x, y) = normalised_point(anchor.point, source);
    snap.anchor_x = x;
    snap.anchor_y = y;
    snap.box_x = anchor.bounds.x / width;
    snap.box_y = anchor.bounds.y / height;
    snap.box_width = anchor.bounds.width / width;
    snap.box_height = anchor.bounds.height / height;
  }
  if let Some(gap) = result.gap_x {
    snap.flags |= SNAP_FLAG_GAP_X;
    snap.gap_x = gap_spans(gap.spans, width, height);
  }
  if let Some(gap) = result.gap_y {
    snap.flags |= SNAP_FLAG_GAP_Y;
    // A gap in y runs down the picture and its bars lie across it, so the
    // extents normalise over the height and the bars' own places over the
    // width.
    snap.gap_y = gap_spans(gap.spans, height, width);
  }
  snap
}

/// One axis's two gaps as the chrome needs them: the extents over `along`,
/// the bars' own places over `across`.
fn gap_spans(spans: [super::snap::GapSpan; 2], along: f64, across: f64) -> [NativeGapSpan; 2] {
  spans.map(|span| NativeGapSpan {
    from: span.from / along,
    to: span.to / along,
    cross: span.cross / across,
  })
}

/// The stroke's width as a fraction of the image's drawn width. The stroke is
/// in output pixels and the image is drawn `image_width` of them across, so
/// the width becomes a share of the picture the native side can place without
/// knowing either number.
fn stroke_width(style: &AnnotationStyle, image_width: f64) -> f64 {
  if image_width <= 0.0 || !image_width.is_finite() {
    return 0.0;
  }
  style.width.max(0.0) / image_width
}

/// How far a head reaches back from its tip, in the same fraction.
fn head_reach(style: &AnnotationStyle, wanted: bool, image_width: f64) -> f64 {
  if !wanted {
    return 0.0;
  }
  stroke_width(style, image_width) * HEAD_LENGTH_WIDTHS
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
    .enumerate()
    .map(|(index, annotation)| match &annotation.shape {
      AnnotationShape::Arrow {
        start,
        control,
        end,
      } => {
        let head = annotation.style.head;
        let (start_x, start_y) = normalised_point(*start, source);
        let (middle_x, middle_y) = normalised_point(curve_midpoint(*start, *control, *end), source);
        let (end_x, end_y) = normalised_point(*end, source);
        NativeAnnotationHandles {
          layer_id: -1,
          index: index as u32,
          kind: HANDLE_KIND_ARROW,
          padding: 0,
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
          width: stroke_width(&annotation.style, image_width),
        }
      }
      AnnotationShape::Counter {
        center,
        angle,
        value: _,
      } => {
        // The tail's tip is not sent as a point. A normalised point is a
        // share of the image in each axis, and those shares differ on a
        // picture that is not square, so a circular offset does not survive
        // the trip. The angle does, and the native side - which works in
        // isotropic display points - places the tip from it and the disc.
        let (start_x, start_y) = normalised_point(*center, source);
        NativeAnnotationHandles {
          layer_id: -1,
          index: index as u32,
          kind: HANDLE_KIND_COUNTER,
          padding: 0,
          start_x,
          start_y,
          middle_x: start_x,
          middle_y: start_y,
          end_x: start_x,
          end_y: start_y,
          start_head: *angle,
          end_head: 0.0,
          width: stroke_width(&annotation.style, image_width),
        }
      }
    })
    .collect()
}
