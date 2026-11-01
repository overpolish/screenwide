// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The grips an annotation is held by, and which one a press lands on, read
//! where they show: through the lenses over the annotation, if any.

use super::*;
use screenwide_annotations::highlight::geometry::HighlightFlow;
use screenwide_annotations::text::geometry::HEAD_TAIL;
use screenwide_annotations::AnnotationPoint;

/// The grip of one annotation under `point`, in display points, and the view
/// a drag of it reads back through. A box is seen whole through `whole`, so
/// its frame keeps its shape; every disc grip shows where `seen` places it.
/// An image's frame and its turning rings stay where the image lies, over
/// any lens: they are one gizmo, turned with the picture. Pure so the hit
/// box can be tested without a composition device.
pub(in super::super) fn grip_seen(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
  whole: LensView,
  seen: impl Fn((f64, f64)) -> ((f64, f64), LensView),
) -> Option<(u32, LensView)> {
  let kind = item.shape_kind();
  if kind == AnnotationKind::Image {
    return super::super::image_chrome::grip_at(image, item, point)
      .map(|handle| (handle, LensView::IDENTITY));
  }
  if super::super::redact_chrome::is_box(kind) {
    return super::super::redact_chrome::grip_at(image, item, point, whole)
      .map(|handle| (handle, whole));
  }
  item_grips(image, item)
    .into_iter()
    .enumerate()
    .find_map(|(found, grip)| {
      let (shown, view) = seen(grip);
      near_grip(point, shown).then(|| (grip_handle(kind, found as u32), view))
    })
}

/// Whether `point` lies on the grip drawn at `grip`.
fn near_grip(point: (f64, f64), grip: (f64, f64)) -> bool {
  (point.0 - grip.0).abs() <= HANDLE_HIT && (point.1 - grip.1).abs() <= HANDLE_HIT
}

/// The handle the grip `found` of an annotation of `kind` reports, in the
/// order [`item_grips`] lists them.
fn grip_handle(kind: AnnotationKind, found: u32) -> u32 {
  match kind {
    // A counter and a text box have one grip - the tip of the tail or the
    // pointer - rather than an arrow's three, so the grip it reports is the
    // tail rather than the start.
    AnnotationKind::Counter | AnnotationKind::Text => HANDLE_TAIL,
    // A highlight's two grips are the selection's start and end.
    AnnotationKind::Highlight if found == 0 => 0,
    AnnotationKind::Highlight => 2,
    AnnotationKind::Arrow
    | AnnotationKind::Redact
    | AnnotationKind::Shape
    | AnnotationKind::Spotlight
    | AnnotationKind::Focus
    | AnnotationKind::Draw
    | AnnotationKind::Magnify
    | AnnotationKind::Image => found,
  }
}

/// A highlight's flow of bands in display points, from the grips it was
/// published with.
pub(in super::super) fn highlight_flow(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
) -> HighlightFlow {
  screenwide_annotations::highlight::handles::flow(item, |x, y| {
    let (x, y) = display_point(image, x, y);
    [x as f32, y as f32]
  })
}

/// The grips one annotation shows as discs, in display points: an arrow's
/// three, and the tip of a counter's tail or of a text box's pointer where
/// its look draws one. A redaction's, a shape's, a spotlight's and a lens's
/// are the selection box's own, which `redact_chrome` draws and hits, and an
/// image's frame is its own, which `image_chrome` does. The tail is placed
/// here rather than sent because a normalised offset is a different length
/// in each axis on a picture that is not square, while display points are
/// isotropic.
pub(in super::super) fn item_grips(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
) -> Vec<(f64, f64)> {
  let centre = display_point(image, item.start_x, item.start_y);
  match item.shape_kind() {
    AnnotationKind::Counter => {
      let radius = item.width * image.width / 2.0;
      let tip = screenwide_annotations::counter::silhouette::counter_tail_tip(
        AnnotationPoint {
          x: centre.0,
          y: centre.1,
        },
        radius,
        item.start_head,
      );
      vec![(tip.x, tip.y)]
    }
    // The pointer's tip, drawn out of the box or tucked into it where it was
    // left; none where the look draws no pointer.
    AnnotationKind::Text if item.start_head as u32 & HEAD_TAIL == 0 => Vec::new(),
    AnnotationKind::Text => {
      let geometry = text_geometry(image, item);
      vec![(f64::from(geometry.c[0]), f64::from(geometry.c[1]))]
    }
    // A box's and an image's grips are its frame's, which their chrome draws
    // itself.
    AnnotationKind::Redact
    | AnnotationKind::Shape
    | AnnotationKind::Spotlight
    | AnnotationKind::Focus
    | AnnotationKind::Draw
    | AnnotationKind::Magnify
    | AnnotationKind::Image => Vec::new(),
    AnnotationKind::Highlight => {
      let flow = highlight_flow(image, item);
      [flow.start_grip(), flow.end_grip()]
        .map(|[x, y]| (f64::from(x), f64::from(y)))
        .to_vec()
    }
    AnnotationKind::Arrow => vec![
      centre,
      display_point(image, item.middle_x, item.middle_y),
      display_point(image, item.end_x, item.end_y),
    ],
  }
}
