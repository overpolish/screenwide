// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A box's chrome - a redaction's, a shape's, a spotlight's or a lens's: the
//! layer selection's own box, with its eight grips, around the chosen one,
//! drawn where it shows through the lenses over it. A hovered one wears the
//! compositor's halo instead. An image wears a frame of its own, turned with
//! it: `image_chrome`.

use super::*;
use screenwide_annotations::gesture::{BOX_HANDLES, MODE_MARQUEE, MODE_SELECT};
use screenwide_annotations::magnify::lens::LensView;
use screenwide_annotations::outline::geometry::{prepare_shape, shape_distance};
use screenwide_annotations::redact::geometry::{prepare_redact, redact_distance};
use screenwide_annotations::reveal::AnnotationReveal;

/// Whether `kind` is held by an upright box and its eight grips.
pub(super) fn is_box(kind: AnnotationKind) -> bool {
  matches!(
    kind,
    AnnotationKind::Redact
      | AnnotationKind::Shape
      | AnnotationKind::Spotlight
      | AnnotationKind::Focus
      | AnnotationKind::Draw
      | AnnotationKind::Magnify
  )
}

/// The box on screen, in display points, from the normalised corners Rust
/// published, where `view` shows it.
fn frame(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  view: LensView,
) -> PreviewSurfaceRect {
  let [left, top] = view.seen([
    image.x + image.width * item.start_x,
    image.y + image.height * item.start_y,
  ]);
  let [right, bottom] = view.seen([
    image.x + image.width * item.end_x,
    image.y + image.height * item.end_y,
  ]);
  PreviewSurfaceRect {
    x: left,
    y: top,
    width: right - left,
    height: bottom - top,
  }
}

/// Every grip in display points with the handle it reports: the eight of
/// the box, clockwise from its top-left corner as the selection chrome draws
/// them, each carrying the sides it moves - 1 left, 2 right, 4 top, 8
/// bottom.
fn grips(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  view: LensView,
) -> [((f64, f64), u32); 8] {
  const BOX: [(usize, usize, u32); 8] = [
    (0, 0, 1 | 4),
    (1, 0, 4),
    (2, 0, 2 | 4),
    (2, 1, 2),
    (2, 2, 2 | 8),
    (1, 2, 8),
    (0, 2, 1 | 8),
    (0, 1, 1),
  ];
  let frame = frame(image, item, view);
  let xs = [frame.x, frame.x + frame.width / 2.0, frame.x + frame.width];
  let ys = [
    frame.y,
    frame.y + frame.height / 2.0,
    frame.y + frame.height,
  ];
  BOX.map(|(x, y, edges)| ((xs[x], ys[y]), BOX_HANDLES + edges))
}

/// The grip of a box under `point`, as the handle it reports, read where
/// `view` shows the box.
pub(super) fn grip_at(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
  view: LensView,
) -> Option<u32> {
  let near =
    |(x, y): (f64, f64)| (point.0 - x).abs() <= HANDLE_HIT && (point.1 - y).abs() <= HANDLE_HIT;
  grips(image, item, view)
    .into_iter()
    .find(|(grip, _)| near(*grip))
    .map(|(_, handle)| handle)
}

/// How far `point` is from a redaction's or a spotlight's rounded box, in
/// display points: zero or less anywhere inside it, so a press anywhere on
/// the box picks it.
pub(super) fn distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  let frame = frame(image, item, LensView::IDENTITY);
  let geometry = prepare_redact(
    [frame.x as f32, frame.y as f32],
    [
      (frame.x + frame.width) as f32,
      (frame.y + frame.height) as f32,
    ],
    item.start_head as f32,
  );
  redact_distance([point.0 as f32, point.1 as f32], &geometry)
}

/// A shape prepared in display points, from the grips it was published with.
/// Its radius and its stroke's hand ride in the head slots, never placed.
fn shape_geometry(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
) -> screenwide_annotations::geometry::ArrowGeometry {
  let frame = frame(image, item, LensView::IDENTITY);
  prepare_shape(
    [frame.x as f32, frame.y as f32],
    [
      (frame.x + frame.width) as f32,
      (frame.y + frame.height) as f32,
    ],
    item.start_head as f32,
    item.end_head as f32,
    (item.width * image.width) as f32,
    AnnotationReveal::WHOLE,
  )
}

/// How far `point` is from a shape's drawn stroke, in display points: zero on
/// its edge, so a press on the line picks it and one inside the box does not.
pub(super) fn shape_stroke_distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  shape_distance(
    [point.0 as f32, point.1 as f32],
    &shape_geometry(image, item),
  )
}

/// How far `point` is from a shape's stroke or the box it outlines, in
/// display points: what picks a shape from anywhere inside it.
pub(super) fn shape_body_distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  screenwide_annotations::outline::geometry::shape_body_distance(
    [point.0 as f32, point.1 as f32],
    &shape_geometry(image, item),
  )
}

/// Whether the annotation at `index` is picked by its inside as well as its
/// line: a shape or a stroke once it is chosen, or while the select tool is
/// in hand. It can be carried from anywhere in it, while any other tool still
/// draws inside one not in hand. The twin of the rule in
/// `annotation_shaft_at_point`.
pub(super) fn grabs_inside(
  state: &SurfaceState,
  index: usize,
  item: &NativeAnnotationHandles,
) -> bool {
  matches!(
    item.shape_kind(),
    AnnotationKind::Shape | AnnotationKind::Draw
  ) && (matches!(state.annotation.mode, MODE_SELECT | MODE_MARQUEE)
    || state.annotation.selected == index as i32)
}

/// The resize cursor a box's grip shows: its sides say which way.
pub(super) fn grip_cursor(handle: u32) -> Option<CursorKind> {
  let edges = handle
    .checked_sub(BOX_HANDLES)
    .filter(|edges| *edges < 16)?;
  Some(match edges {
    1 | 2 => CursorKind::ResizeHorizontal,
    4 | 8 => CursorKind::ResizeVertical,
    5 | 10 => CursorKind::ResizeNwse,
    _ => CursorKind::ResizeNesw,
  })
}

/// The chosen redaction's, shape's, spotlight's, stroke's or lens's box in
/// device pixels, where it shows through the lenses over it, for the chrome
/// to draw the way the layer selection draws its own. `None` when no box is
/// chosen or the annotation tool has no say.
pub(crate) fn selected_box(state: &SurfaceState, scale: f64) -> Option<[f32; 4]> {
  if state.annotation.mode == MODE_NONE {
    return None;
  }
  let index = usize::try_from(state.annotation.selected).ok()?;
  let item = selected_item(state)?;
  if !is_box(item.shape_kind()) {
    return None;
  }
  let image = image_frame(state)?;
  let view = super::magnify_chrome::whole_view(state, index, image, item);
  let rect = frame(image, item, view);
  let (x, right) = pixels::scaled_edges(rect.x, rect.width, scale);
  let (y, bottom) = pixels::scaled_edges(rect.y, rect.height, scale);
  Some([x as f32, y as f32, (right - x) as f32, (bottom - y) as f32])
}
