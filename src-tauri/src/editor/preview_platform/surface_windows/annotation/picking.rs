// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a mark's picture is on screen, and what a press lands on.
//!
//! Marks are published in their layer's image-normalised space, so picking
//! first has to find that image inside the pane the layer is drawn in. The
//! twin of `recording_preview_annotation_geometry_macos.h` and the layer
//! lookup in `recording_preview_annotation_layers_macos.h`.

use super::*;
use crate::editor::annotations::counter::silhouette::counter_distance;
use crate::editor::annotations::geometry::{head_distance, prepare_arrow, shaft_distance};
use crate::editor::annotations::handles::HANDLE_KIND_COUNTER;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::AnnotationPoint;

/// A mark belongs to its image, independently of the selected layer. The twin
/// of `annotation_layer_selection`.
pub(super) fn layer_selection(state: &SurfaceState, layer: i32) -> Option<PreviewSelection> {
  if let Some(current) = state.selection {
    if layer < 0 || current.layer_id == layer as u32 {
      return Some(current);
    }
  }
  state
    .selection_targets
    .iter()
    .copied()
    .find(|target| layer >= 0 && target.layer_id == layer as u32)
}

/// The whole source image's rectangle on screen for one layer. Every
/// normalised handle is placed inside it.
fn layer_image_rect(state: &SurfaceState, layer: i32) -> Option<PreviewSurfaceRect> {
  let mut selection = layer_selection(state, layer)?;
  selection.x = selection.image_x;
  selection.y = selection.image_y;
  selection.width = selection.image_width;
  selection.height = selection.image_height;
  let rect = display_selection(state, selection)?;
  (rect.width > 0.0 && rect.height > 0.0).then_some(rect)
}

/// The image one arrow's grips and its halo are placed in. A mark belongs to
/// its own layer, which is not always the selected one: the keyboard
/// shortcut can hold the selection while the pointer rests on an arrow over
/// the screen.
pub(super) fn item_image_frame(state: &SurfaceState, index: i32) -> Option<PreviewSurfaceRect> {
  let layer = usize::try_from(index)
    .ok()
    .and_then(|index| state.annotation.handles.get(index))
    .map_or(-1, |item| item.layer_id);
  layer_image_rect(state, layer)
}

/// The image the selected arrow's grips are placed in.
fn image_frame(state: &SurfaceState) -> Option<PreviewSurfaceRect> {
  item_image_frame(state, state.annotation.selected)
}

pub(super) fn selected_item(state: &SurfaceState) -> Option<&NativeAnnotationHandles> {
  usize::try_from(state.annotation.selected)
    .ok()
    .and_then(|index| state.annotation.handles.get(index))
}

fn display_point(image: PreviewSurfaceRect, x: f64, y: f64) -> (f64, f64) {
  (image.x + image.width * x, image.y + image.height * y)
}

/// A point on screen, back in the layer's image-normalised space. This is
/// exactly what Rust above the facade turns into source pixels.
pub(super) fn normalised_point(state: &SurfaceState, point: (f64, f64)) -> Option<(f64, f64)> {
  let image = image_frame(state)?;
  Some((
    (point.0 - image.x) / image.width,
    (point.1 - image.y) / image.height,
  ))
}

/// The grip of one arrow under `point`, in display points. Pure so the hit
/// box can be tested without a composition device.
fn grip_at_point(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> Option<u32> {
  let grips = item_grips(image, item);
  let found = grips
    .iter()
    .position(|grip| {
      (point.0 - grip.0).abs() <= HANDLE_HIT && (point.1 - grip.1).abs() <= HANDLE_HIT
    })
    .map(|index| index as u32)?;
  // A counter has one grip - the tip of its tail - rather than an arrow's
  // three, so the grip it reports is the tail rather than the start.
  Some(if item.kind == HANDLE_KIND_COUNTER {
    HANDLE_TAIL
  } else {
    found
  })
}

/// The grips one mark shows, in display points: an arrow's three, or the tip
/// of a counter's tail. The tail is placed here rather than sent because a
/// normalised offset is a different length in each axis on a picture that is
/// not square, while display points are isotropic.
fn item_grips(image: PreviewSurfaceRect, item: &NativeAnnotationHandles) -> Vec<(f64, f64)> {
  let centre = display_point(image, item.start_x, item.start_y);
  if item.kind == HANDLE_KIND_COUNTER {
    let radius = item.width * image.width / 2.0;
    let tip = crate::editor::annotations::counter::silhouette::counter_tail_tip(
      AnnotationPoint {
        x: centre.0,
        y: centre.1,
      },
      radius,
      item.start_head,
    );
    return vec![(tip.x, tip.y)];
  }
  vec![
    centre,
    display_point(image, item.middle_x, item.middle_y),
    display_point(image, item.end_x, item.end_y),
  ]
}

/// The chosen arrow's grip under `point`.
pub(super) fn handle_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<u32> {
  let item = *selected_item(state)?;
  grip_at_point(image_frame(state)?, &item, point)
}

/// The chosen arrow's three grips, in device pixels, for the chrome to draw.
/// Empty when no arrow is chosen or the arrow tool has no say, which is what
/// leaves the layer's own chrome standing.
pub(crate) fn selected_grips(state: &SurfaceState, scale: f64) -> Vec<[f32; 2]> {
  if state.annotation.mode == MODE_NONE {
    return Vec::new();
  }
  let Some(item) = selected_item(state).copied() else {
    return Vec::new();
  };
  let Some(image) = image_frame(state) else {
    return Vec::new();
  };
  item_grips(image, &item)
    .into_iter()
    .map(|(x, y)| [(x * scale) as f32, (y * scale) as f32])
    .collect()
}

/// Whether the arrow chrome is what is on screen. The arrow tool always
/// draws its own chrome; the select tool only once it is holding an arrow, so
/// an ordinary layer selection is untouched. The twin of
/// `annotation_owns_chrome`.
pub(crate) fn owns_chrome(state: &SurfaceState) -> bool {
  state.annotation.mode == MODE_ARROW
    || state.annotation.mode == MODE_COUNTER
    || (state.annotation.mode != MODE_NONE && state.annotation.selected >= 0)
}

/// The cursor the arrow tool asks for over `point`, or `None` when the
/// choice belongs to the layer underneath. The twin of `annotation_cursor`.
pub(crate) fn cursor_for(state: &SurfaceState, point: (f64, f64)) -> Option<editor::CursorKind> {
  if state.annotation.mode == MODE_NONE {
    return None;
  }
  if handle_at_point(state, point).is_some() || shaft_at_point(state, point).is_some() {
    return Some(editor::CursorKind::Arrow);
  }
  (state.annotation.mode == MODE_ARROW || state.annotation.mode == MODE_COUNTER)
    .then_some(editor::CursorKind::Crosshair)
}

/// How far a point is from one arrow's drawn shape, in display points: its
/// shaft, and the heads on it. Zero anywhere the arrow is actually painted,
/// because the tolerance is measured from the stroke's edge rather than its
/// centreline. A press on a head is a press on the arrow - it is the part of
/// it the hand aims at. A mark half-way through drawing itself in is still
/// picked by the whole of what it will be.
fn arrow_distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  let start = display_point(image, item.start_x, item.start_y);
  if item.kind == HANDLE_KIND_COUNTER {
    return counter_distance(
      point,
      AnnotationPoint {
        x: start.0,
        y: start.1,
      },
      item.width * image.width / 2.0,
      item.start_head,
    ) as f32;
  }
  let middle = display_point(image, item.middle_x, item.middle_y);
  let end = display_point(image, item.end_x, item.end_y);
  // The control point behind a reported middle handle, so the shaft can be
  // sampled without solving the curve again.
  let a = [start.0 as f32, start.1 as f32];
  let b = [
    (2.0 * middle.0 - (start.0 + end.0) / 2.0) as f32,
    (2.0 * middle.1 - (start.1 + end.1) / 2.0) as f32,
  ];
  let c = [end.0 as f32, end.1 as f32];
  let probe = [point.0 as f32, point.1 as f32];
  // The stroke rides on the mark rather than being read back off its heads,
  // so a headless arrow is picked over the width it shows too.
  let width = (item.width * image.width) as f32;
  let heads = if item.start_head > 0.0 {
    2
  } else if item.end_head > 0.0 {
    1
  } else {
    0
  };
  let geometry = prepare_arrow(a, b, c, width, heads, AnnotationReveal::WHOLE);
  let mut best = (shaft_distance(probe, a, b, c) - geometry.width * 0.5).max(0.0);
  if geometry.head != 0 {
    best = best.min(head_distance(probe, geometry.end_head, geometry.rounding));
  }
  if geometry.head == 2 {
    best = best.min(head_distance(probe, geometry.start_head, geometry.rounding));
  }
  best
}

/// The topmost arrow whose drawn shape `point` lands on. There is no
/// tolerance around it: the arrow is picked, and haloed, exactly where it is
/// painted, which is what keeps the halo off the space beside a mark.
pub(super) fn shaft_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<usize> {
  state
    .annotation
    .handles
    .iter()
    .enumerate()
    .rev()
    .find(|(_, item)| {
      layer_image_rect(state, item.layer_id)
        .is_some_and(|image| arrow_distance(image, item, point) <= 0.0)
    })
    .map(|(index, _)| index)
}

#[cfg(test)]
#[path = "picking_tests.rs"]
mod tests;
