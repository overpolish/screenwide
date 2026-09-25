// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A redaction's chrome: the layer selection's own box, with its eight grips
//! and its radius dot, around the chosen one. A hovered one wears the
//! compositor's halo instead. The twin of
//! `recording_preview_surface_macos+annotation_redact.m`.

use super::*;
use crate::editor::annotations::gesture::{BOX_HANDLES, RADIUS_HANDLE};
use crate::editor::annotations::redact::geometry::{prepare_redact, redact_distance};

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

/// The grip of a redaction under `point`, as the handle it reports.
pub(super) fn grip_at(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> Option<u32> {
  grips(image, item)
    .into_iter()
    .find(|((x, y), _)| (point.0 - x).abs() <= HANDLE_HIT && (point.1 - y).abs() <= HANDLE_HIT)
    .map(|(_, handle)| handle)
}

/// How far `point` is from a redaction's rounded box, in display points:
/// zero or less anywhere inside it, so a press anywhere on the box picks it.
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

/// The resize cursor a redaction's grip shows: its sides say which way, and
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

/// The chosen redaction's box in device pixels and its radius percentage,
/// for the chrome to draw the way the layer selection draws its own. `None`
/// when no redaction is chosen or the annotation tool has no say.
pub(crate) fn selected_box(state: &SurfaceState, scale: f64) -> Option<([f32; 4], f64)> {
  if state.annotation.mode == MODE_NONE {
    return None;
  }
  let item = selected_item(state)?;
  if item.shape_kind() != AnnotationKind::Redact {
    return None;
  }
  let rect = frame(image_frame(state)?, item);
  let (x, right) = window::scaled_edges(rect.x, rect.width, scale);
  let (y, bottom) = window::scaled_edges(rect.y, rect.height, scale);
  Some((
    [x as f32, y as f32, (right - x) as f32, (bottom - y) as f32],
    item.start_head,
  ))
}
