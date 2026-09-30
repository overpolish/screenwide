// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A box's chrome - a redaction's, a shape's, a spotlight's or a magnifier's
//! zoom area: the layer selection's own box, with its eight grips and its radius
//! dot, around the chosen one. A hovered one wears the compositor's halo
//! instead. The twin of `recording_preview_surface_macos+annotation_redact.m`.

use super::*;
use crate::editor::annotations::gesture::{BOX_HANDLES, MODE_MARQUEE, MODE_SELECT, RADIUS_HANDLE};
use crate::editor::annotations::outline::geometry::{prepare_shape, shape_distance};
use crate::editor::annotations::redact::geometry::{prepare_redact, redact_distance};
use crate::editor::annotations::reveal::AnnotationReveal;

/// Whether `kind` is held by a box and its eight grips.
pub(super) fn is_box(kind: AnnotationKind) -> bool {
  matches!(
    kind,
    AnnotationKind::Redact
      | AnnotationKind::Shape
      | AnnotationKind::Spotlight
      | AnnotationKind::Draw
      | AnnotationKind::Magnify
  )
}

/// Whether a box of `kind` has a radius dot: a stroke has no corners to round.
fn has_radius(kind: AnnotationKind) -> bool {
  kind != AnnotationKind::Draw
}

/// The box on screen, in display points, from the normalised corners Rust
/// published.
fn frame(image: PreviewSurfaceRect, item: &NativeAnnotationHandles) -> PreviewSurfaceRect {
  let left = image.x + image.width * item.start_x;
  let top = image.y + image.height * item.start_y;
  PreviewSurfaceRect {
    x: left,
    y: top,
    width: image.x + image.width * item.end_x - left,
    height: image.y + image.height * item.end_y - top,
  }
}

/// Every grip in display points with the handle it reports: the eight of
/// the box, clockwise from its top-left corner as the selection chrome draws
/// them, each carrying the sides it moves - 1 left, 2 right, 4 top, 8
/// bottom - and then the radius dot, whose percentage rides in
/// `start_head`. The box's grips come first, so on a box too small to keep
/// them apart a corner wins over the dot.
fn grips(image: PreviewSurfaceRect, item: &NativeAnnotationHandles) -> [((f64, f64), u32); 9] {
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
  let frame = frame(image, item);
  let xs = [frame.x, frame.x + frame.width / 2.0, frame.x + frame.width];
  let ys = [
    frame.y,
    frame.y + frame.height / 2.0,
    frame.y + frame.height,
  ];
  let mut grips = [((0.0, 0.0), RADIUS_HANDLE); 9];
  for (slot, (x, y, edges)) in BOX.into_iter().enumerate() {
    grips[slot] = ((xs[x], ys[y]), BOX_HANDLES + edges);
  }
  grips[8].0 = radius_point(frame, item.start_head);
  grips
}

/// The grip of a box under `point`, as the handle it reports. A magnifier's
/// loupe grip comes after its zoom area's.
pub(super) fn grip_at(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> Option<u32> {
  let near =
    |(x, y): (f64, f64)| (point.0 - x).abs() <= HANDLE_HIT && (point.1 - y).abs() <= HANDLE_HIT;
  grips(image, item)
    .into_iter()
    .filter(|(_, handle)| *handle != RADIUS_HANDLE || has_radius(item.shape_kind()))
    .find(|(grip, _)| near(*grip))
    .map(|(_, handle)| handle)
    .or_else(|| {
      (item.shape_kind() == AnnotationKind::Magnify
        && near(super::magnify_chrome::loupe_grip(image, item)))
      .then_some(HANDLE_TAIL)
    })
}

/// How far `point` is from a redaction's or a spotlight's rounded box, in
/// display points: zero or less anywhere inside it, so a press anywhere on
/// the box picks it.
pub(super) fn distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  let frame = frame(image, item);
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
) -> crate::editor::annotations::geometry::ArrowGeometry {
  let frame = frame(image, item);
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
  crate::editor::annotations::outline::geometry::shape_body_distance(
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

/// The resize cursor a box's grip shows: its sides say which way, and
/// the radius dot drags diagonally as the layer selection's does.
pub(super) fn grip_cursor(handle: u32) -> Option<editor::CursorKind> {
  if handle == RADIUS_HANDLE {
    return Some(editor::CursorKind::ResizeNwse);
  }
  let edges = handle
    .checked_sub(BOX_HANDLES)
    .filter(|edges| *edges < 16)?;
  Some(match edges {
    1 | 2 => editor::CursorKind::ResizeHorizontal,
    4 | 8 => editor::CursorKind::ResizeVertical,
    5 | 10 => editor::CursorKind::ResizeNwse,
    _ => editor::CursorKind::ResizeNesw,
  })
}

/// The chosen redaction's, shape's, spotlight's or stroke's box in device
/// pixels and its radius percentage, `None` for a stroke's, which has no
/// radius dot, for the chrome to draw the way the layer selection draws its
/// own. `None` when no box is chosen or the annotation tool has no say.
pub(crate) fn selected_box(state: &SurfaceState, scale: f64) -> Option<([f32; 4], Option<f64>)> {
  if state.annotation.mode == MODE_NONE {
    return None;
  }
  let item = selected_item(state)?;
  if !is_box(item.shape_kind()) {
    return None;
  }
  let rect = frame(image_frame(state)?, item);
  let (x, right) = window::scaled_edges(rect.x, rect.width, scale);
  let (y, bottom) = window::scaled_edges(rect.y, rect.height, scale);
  Some((
    [x as f32, y as f32, (right - x) as f32, (bottom - y) as f32],
    has_radius(item.shape_kind()).then_some(item.start_head),
  ))
}
